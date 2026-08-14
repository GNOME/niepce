/*
 * niepce - engine/metadata/xmp.rs
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

//! Niepce specific XMP

use npc_fwk::utils::xmp::NsDef;
use npc_fwk::{XmpManager, XmpMeta};

pub const NIEPCE_XMP_NAMESPACE: &str = "http://xmlns.figuiere.net/ns/niepce/1.0";
pub const NIEPCE_XMP_NS_PREFIX: &str = "niepce";

/// Darktable XMP namespace
const DARKTABLE_NAMESPACE: &str = "http://darktable.sf.net/";
/// Darktable XMP "defualt" prefix.
const DARKTABLE_NS_PREFIX: &str = "darktable";

pub fn xmp_manager(namespaces: Option<Vec<NsDef>>) -> XmpManager {
    let mut namespaces = namespaces.unwrap_or_default();
    namespaces.extend([
        NsDef {
            ns: NIEPCE_XMP_NAMESPACE,
            prefix: NIEPCE_XMP_NS_PREFIX,
        },
        NsDef {
            ns: DARKTABLE_NAMESPACE,
            prefix: DARKTABLE_NS_PREFIX,
        },
    ]);
    XmpManager::new(Some(namespaces))
}

/// Trait to extend XmpMeta api specific to that scope.
pub trait NpcXmp {
    fn flag(&self) -> Option<i32>;
}

impl NpcXmp for XmpMeta {
    fn flag(&self) -> Option<i32> {
        let mut flags: exempi2::PropFlags = exempi2::PropFlags::empty();
        self.xmp
            .get_property_i32(NIEPCE_XMP_NAMESPACE, "Flag", &mut flags)
            .ok()
    }
}
