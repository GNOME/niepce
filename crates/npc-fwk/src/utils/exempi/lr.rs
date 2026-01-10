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
use lrcat::lron::{Object, Value};

use super::XmpKeyword;

fn decode_kw_entry(value: Object) -> Option<XmpKeyword> {
    match value {
        Object::Dict(entries) => {
            let mut path: Option<Value> = None;
            let mut primary: Option<Value> = None;
            for entry in entries {
                if let Object::Pair(entry) = entry {
                    match entry.key.as_str() {
                        // we ignore flat
                        "path" => path = Some(entry.value),
                        "primary" => primary = Some(entry.value),
                        _ => {}
                    }
                }
            }
            if let Some(Value::Dict(path)) = path {
                let path = path
                    .into_iter()
                    .filter_map(|v| match v {
                        Object::Str(s) => Some(s),
                        _ => None,
                    })
                    .collect();
                Some(XmpKeyword::Hier(path))
            } else if let Some(Value::Str(primary)) = primary {
                Some(XmpKeyword::Flat(primary))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(super) fn decode_old_hierarchical_kw(prop: &str, k: XmpString) -> Option<Vec<XmpKeyword>> {
    BASE64_STANDARD
        .decode(k.to_string())
        .map(|hkw| String::from_utf8_lossy(&hkw).to_string())
        .ok()
        .and_then(|hkw| {
            if let Ok(Object::Pair(hier_kw)) = Object::from_string(hkw.as_str()) {
                if hier_kw.key == prop {
                    match hier_kw.value {
                        Value::Dict(dict) => {
                            Some(dict.into_iter().filter_map(decode_kw_entry).collect())
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
}
