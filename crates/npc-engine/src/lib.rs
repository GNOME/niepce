/*
 * niepce - engine/mod.rs
 *
 * Copyright (C) 2017-2022 Hubert Figuière
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

pub mod db;
pub mod library;

use std::ptr;

use npc_fwk::base::PropertyIndex;
use npc_fwk::PropertyValue;

use db::{NiepceProperties, NiepcePropertyIdx};

type NiepcePropertySet = npc_fwk::PropertySet<db::NiepceProperties>;
type NiepcePropertyBag = npc_fwk::PropertyBag<db::NiepceProperties>;

#[no_mangle]
pub extern "C" fn eng_property_set_new() -> *mut NiepcePropertySet {
    Box::into_raw(Box::new(NiepcePropertySet::new()))
}

/// Delete a %PropertySet
///
/// # Safety
/// Dereference the pointer.
#[no_mangle]
pub unsafe extern "C" fn eng_property_set_delete(set: *mut NiepcePropertySet) {
    drop(Box::from_raw(set));
}

#[no_mangle]
pub extern "C" fn eng_property_set_add(set: &mut NiepcePropertySet, v: NiepcePropertyIdx) {
    set.insert(NiepceProperties::Index(v));
}

#[no_mangle]
pub extern "C" fn eng_property_bag_new() -> *mut NiepcePropertyBag {
    Box::into_raw(Box::new(NiepcePropertyBag::new()))
}

/// Delete the %PropertyBag object
///
/// # Safety
/// Dereference the raw pointer.
#[no_mangle]
pub unsafe extern "C" fn eng_property_bag_delete(bag: *mut NiepcePropertyBag) {
    drop(Box::from_raw(bag));
}

#[no_mangle]
pub extern "C" fn eng_property_bag_is_empty(b: &NiepcePropertyBag) -> bool {
    b.is_empty()
}

#[no_mangle]
pub extern "C" fn eng_property_bag_len(b: &NiepcePropertyBag) -> usize {
    b.len()
}

#[no_mangle]
pub extern "C" fn eng_property_bag_key_by_index(
    b: &NiepcePropertyBag,
    idx: usize,
) -> PropertyIndex {
    b.bag[idx].into()
}

#[no_mangle]
pub extern "C" fn eng_property_bag_value(
    b: &NiepcePropertyBag,
    key: PropertyIndex,
) -> *mut PropertyValue {
    let key: db::NiepceProperties = key.into();
    if b.map.contains_key(&key) {
        let value = Box::new(b.map[&key].clone());
        Box::into_raw(value)
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn eng_property_bag_set_value(
    b: &mut NiepcePropertyBag,
    key: PropertyIndex,
    v: &PropertyValue,
) -> bool {
    b.set_value(key.into(), v.clone())
}

use crate::db::{Keyword, Label, LibFile};

#[cxx::bridge(namespace = "eng")]
mod ffi {
    #[namespace = "fwk"]
    extern "C++" {
        include!("fwk/cxx_colour_bindings.hpp");

        type RgbColour = npc_fwk::base::rgbcolour::RgbColour;
    }

    // This enum is only here for the purpose of binding generation.
    #[repr(i32)]
    /// A general type of the LibFile, cxx bindings version.
    pub enum FileType {
        /// Don't know
        #[allow(dead_code)]
        Unknown = 0,
        /// Camera Raw
        #[allow(dead_code)]
        Raw = 1,
        /// Bundle of RAW + processed. Don't assume JPEG.
        #[allow(dead_code)]
        RawJpeg = 2,
        /// Processed Image
        #[allow(dead_code)]
        Image = 3,
        /// Video
        #[allow(dead_code)]
        Video = 4,
    }

    extern "Rust" {
        type Keyword;

        fn id(&self) -> i64;
        fn keyword(&self) -> &str;
    }

    impl Box<Keyword> {}

    extern "Rust" {
        type Label;

        fn colour(&self) -> &RgbColour;
        fn label(&self) -> &str;
        fn id(&self) -> i64;
        fn clone_boxed(&self) -> Box<Label>;
    }

    extern "Rust" {
        type LibFile;

        #[cxx_name = "path"]
        fn path_str(&self) -> String;
        fn id(&self) -> i64;
        fn folder_id(&self) -> i64;
        fn orientation(&self) -> i32;
        #[cxx_name = "file_type"]
        // The type is `FileType`.
        fn file_type_int(&self) -> i32;
    }

    impl Box<LibFile> {}
}
