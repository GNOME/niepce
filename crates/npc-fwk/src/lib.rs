/*
 * niepce - fwk/lib.rs
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

#[macro_use]
pub mod base;
pub mod toolkit;
pub mod utils;

pub use self::base::fractions::fraction_to_decimal;
pub use self::base::propertybag::PropertyBag;
pub use self::base::propertyvalue::PropertyValue;
pub use self::base::PropertySet;
pub use self::utils::exempi::{gps_coord_from_xmp, ExempiManager, NsDef, XmpMeta};

pub use self::base::date::*;

pub use self::toolkit::mimetype::MimeType;

use std::f64;

///
/// Init funtion because rexiv2 need one.
///
/// Make sure to call it after gtk::init()
///
pub fn init() {
    rexiv2::initialize().expect("Unable to initialize rexiv2");
}

// C++ bridge

use std::ffi::c_char;

use gdk_pixbuf_sys::GdkPixbuf;
use glib::translate::*;

use self::base::rgbcolour::RgbColour;
use crate::base::date::Date;
use crate::base::propertyvalue::{
    property_value_new_date, property_value_new_int, property_value_new_str,
    property_value_new_string_array,
};
use crate::toolkit::thumbnail::Thumbnail;
use crate::toolkit::Configuration;
use crate::utils::files::FileList;

fn make_config_path(file: &str) -> String {
    Configuration::make_config_path(file)
        .to_string_lossy()
        .into()
}

fn configuration_new(file: &str) -> cxx::SharedPtr<ffi::SharedConfiguration> {
    cxx::SharedPtr::new(ffi::SharedConfiguration {
        cfg: Box::new(Configuration::from_file(file)),
    })
}

fn exempi_manager_new() -> Box<ExempiManager> {
    Box::new(ExempiManager::new(None))
}

fn rgbcolour_new(r: u16, g: u16, b: u16) -> Box<RgbColour> {
    Box::new(RgbColour::new(r, g, b))
}

fn rgbcolour_to_string(r: u16, g: u16, b: u16) -> String {
    let colour = RgbColour::new(r, g, b);
    colour.to_string()
}

pub fn gps_coord_from_xmp_(value: &str) -> f64 {
    gps_coord_from_xmp(value).unwrap_or(f64::NAN)
}

pub fn fraction_to_decimal_(value: &str) -> f64 {
    fraction_to_decimal(value).unwrap_or(f64::NAN)
}

pub fn thumbnail_for_file(path: &str, w: i32, h: i32, orientation: i32) -> Box<Thumbnail> {
    Box::new(Thumbnail::thumbnail_file(path, w, h, orientation))
}

/// Create a %Thumbnail from a %GdkPixbuf
///
/// The resulting object must be freed by %fwk_toolkit_thumbnail_delete
///
/// # Safety
/// Dereference the pointer
unsafe fn thumbnail_from_pixbuf(pixbuf: *mut c_char) -> Box<Thumbnail> {
    let pixbuf: Option<gdk_pixbuf::Pixbuf> = from_glib_none(pixbuf as *mut GdkPixbuf);
    Box::new(Thumbnail::from(pixbuf))
}

fn thumbnail_to_pixbuf(self_: &Thumbnail) -> *mut c_char {
    let pixbuf: *mut GdkPixbuf = self_.make_pixbuf().to_glib_full();
    pixbuf as *mut c_char
}

/// Get the files in directory dir with extension ext
/// `ext` has no dot
pub fn file_list_get_files_from_directory_with_ext(dir: &str, ext: String) -> Box<FileList> {
    Box::new(FileList::get_files_from_directory(dir, move |file| {
        if let Some(file_ext) = file.name().extension() {
            if file_ext.to_string_lossy().to_lowercase() == ext {
                return true;
            }
        }
        false
    }))
}

/// Get all the files in directory dir.
pub fn file_list_get_files_from_directory(dir: &str) -> Box<FileList> {
    Box::new(FileList::get_files_from_directory(dir, |_| true))
}

/// Get all the files in directory dir.
pub fn file_list_get_media_files_from_directory(dir: &str) -> Box<FileList> {
    Box::new(FileList::get_files_from_directory(
        dir,
        FileList::file_is_media,
    ))
}

pub fn file_list_new() -> Box<FileList> {
    Box::new(FileList::default())
}

#[cxx::bridge(namespace = "fwk")]
mod ffi {
    struct SharedConfiguration {
        cfg: Box<Configuration>,
    }

    extern "Rust" {
        type Configuration;

        #[cxx_name = "Configuration_new"]
        fn configuration_new(file: &str) -> SharedPtr<SharedConfiguration>;
        #[cxx_name = "Configuration_make_config_path"]
        fn make_config_path(file: &str) -> String;
        #[cxx_name = "hasKey"]
        fn has(&self, key: &str) -> bool;
        #[cxx_name = "getValue"]
        fn value(&self, key: &str, def: &str) -> String;
        #[cxx_name = "setValue"]
        fn set_value(&self, key: &str, value: &str);
    }

    extern "Rust" {
        type ExempiManager;

        #[cxx_name = "ExempiManager_new"]
        fn exempi_manager_new() -> Box<ExempiManager>;
    }

    extern "C++" {
        include!("fwk/cxx_colour_bindings.hpp");

        type RgbColour = crate::base::rgbcolour::RgbColour;
    }

    extern "Rust" {
        #[cxx_name = "RgbColour_new"]
        fn rgbcolour_new(r: u16, g: u16, b: u16) -> Box<RgbColour>;

        fn rgbcolour_to_string(r: u16, g: u16, b: u16) -> String;
    }

    #[namespace = "fwk"]
    extern "Rust" {
        #[cxx_name = "gps_coord_from_xmp"]
        fn gps_coord_from_xmp_(value: &str) -> f64;
        #[cxx_name = "fraction_to_decimal"]
        fn fraction_to_decimal_(value: &str) -> f64;
    }

    extern "Rust" {
        type Date;

        fn to_string(&self) -> String;
    }

    impl Box<Date> {}

    extern "Rust" {
        type Thumbnail;

        #[cxx_name = "Thumbnail_for_file"]
        fn thumbnail_for_file(path: &str, w: i32, h: i32, orientation: i32) -> Box<Thumbnail>;
        #[cxx_name = "Thumbnail_from_pixbuf"]
        unsafe fn thumbnail_from_pixbuf(pixbuf: *mut c_char) -> Box<Thumbnail>;
        #[cxx_name = "Thumbnail_to_pixbuf"]
        fn thumbnail_to_pixbuf(self_: &Thumbnail) -> *mut c_char;
    }

    extern "Rust" {
        type FileList;

        #[cxx_name = "FileList_get_files_from_directory"]
        fn file_list_get_files_from_directory(dir: &str) -> Box<FileList>;
        #[cxx_name = "FileList_get_files_from_directory_with_ext"]
        fn file_list_get_files_from_directory_with_ext(dir: &str, ext: String) -> Box<FileList>;
        #[cxx_name = "FileList_get_media_files_from_directory"]
        fn file_list_get_media_files_from_directory(dir: &str) -> Box<FileList>;
        #[cxx_name = "FileList_new"]
        fn file_list_new() -> Box<FileList>;
        fn size(&self) -> usize;
        fn at(&self, idx: usize) -> String;
        fn push_back(&mut self, value: &str);
    }

    extern "Rust" {
        type PropertyValue;

        fn property_value_new_str(v: &str) -> Box<PropertyValue>;
        fn property_value_new_int(v: i32) -> Box<PropertyValue>;
        fn property_value_new_date(v: &Date) -> Box<PropertyValue>;
        fn property_value_new_string_array() -> Box<PropertyValue>;

        fn is_empty(&self) -> bool;
        fn is_integer(&self) -> bool;
        fn is_date(&self) -> bool;
        fn is_string(&self) -> bool;
        #[cxx_name = "get_integer"]
        fn integer_unchecked(&self) -> i32;
        #[cxx_name = "get_date"]
        fn date_unchecked(&self) -> Box<Date>;
        #[cxx_name = "get_string"]
        fn string_unchecked(&self) -> &str;
        #[cxx_name = "add_string"]
        fn add_string_unchecked(&mut self, string: &str);
        #[cxx_name = "get_string_array"]
        fn string_array_unchecked(&self) -> &[String];
    }
}
