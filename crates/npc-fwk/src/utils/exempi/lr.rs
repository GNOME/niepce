/*
 * niepce - fwk/utils/exempi/lr.rs
 *
 * Copyright (C) 2026 Hubert Figuière
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

//! Lightroom™ specific support for XMP

use base64::prelude::*;
use exempi2::XmpString;
use lrcat::lron::Value;

use super::XmpKeyword;

fn decode_kw_entry(value: &Value) -> Option<XmpKeyword> {
    value.as_dict().and_then(|entries| {
        let mut path: Option<&Value> = None;
        let mut primary: Option<&Value> = None;
        for entry in entries {
            if let Some(entry) = entry.as_pair() {
                match entry.key.as_str() {
                    // we ignore flat
                    "path" => path = Some(&entry.value),
                    "primary" => primary = Some(&entry.value),
                    _ => {}
                }
            }
        }
        if let Some(path) = path.as_ref().and_then(|path| path.as_dict()) {
            let path = path
                .iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect();
            Some(XmpKeyword::Hier(path))
        } else {
            primary
                .and_then(|primary| primary.as_str().map(String::from))
                .map(XmpKeyword::Flat)
        }
    })
}

pub(super) fn decode_old_hierarchical_kw(prop: &str, k: XmpString) -> Option<Vec<XmpKeyword>> {
    BASE64_STANDARD
        .decode(k.to_string())
        .map(|hkw| String::from_utf8_lossy(&hkw).to_string())
        .ok()
        .and_then(|hkw| {
            if let Ok(Value::Pair(hier_kw)) = Value::from_string(hkw.as_str()) {
                if hier_kw.key == prop {
                    hier_kw
                        .value
                        .as_dict()
                        .map(|dict| dict.iter().filter_map(decode_kw_entry).collect())
                } else {
                    None
                }
            } else {
                None
            }
        })
}
