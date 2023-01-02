/*
 * niepce - fwk/lib.rs
 *
 * Copyright (C) 2017-2023 Hubert Figuière
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

extern crate libadwaita as adw;

#[macro_use]
pub mod base;
pub mod toolkit;
pub mod utils;

pub use self::base::fractions::{fraction_to_decimal, parse_fraction};
pub use self::base::propertybag::PropertyBag;
pub use self::base::propertyvalue::PropertyValue;
pub use self::base::PropertySet;
pub use self::utils::exempi::{gps_coord_from_xmp, ExempiManager, NsDef, XmpMeta};

pub use self::base::date::{xmp_date_from, Date, Time};

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

use gdk_pixbuf_sys::GdkPixbuf;
use glib::translate::*;

use crate::base::rgbcolour::RgbColour;
use crate::toolkit::cxx::*;
use crate::toolkit::thumbnail::Thumbnail;
use crate::toolkit::widgets::cxx::*;
use crate::toolkit::widgets::MetadataWidget;
use crate::toolkit::{Configuration, UndoCommand, UndoHistory, UndoTransaction};
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

pub fn thumbnail_for_file(path: &str, w: i32, h: i32, orientation: i32) -> Box<Thumbnail> {
    Box::new(Thumbnail::thumbnail_file(path, w, h, orientation))
}

/// Create a %Thumbnail from a %GdkPixbuf
///
/// The resulting object must be freed by %fwk_toolkit_thumbnail_delete
///
/// # Safety
/// Dereference the pointer
unsafe fn thumbnail_from_pixbuf(pixbuf: *mut ffi::GdkPixbuf) -> Box<Thumbnail> {
    let pixbuf: Option<gdk_pixbuf::Pixbuf> = from_glib_none(pixbuf as *mut GdkPixbuf);
    Box::new(Thumbnail::from(pixbuf))
}

fn thumbnail_to_pixbuf(self_: &Thumbnail) -> *mut ffi::GdkPixbuf {
    let pixbuf: *mut GdkPixbuf = self_.make_pixbuf().to_glib_full();
    pixbuf as *mut ffi::GdkPixbuf
}

/// Get the files in directory dir with extension ext
/// `ext` has no dot
pub fn file_list_get_files_from_directory_with_ext(dir: &str, ext: String) -> Box<FileList> {
    Box::new(FileList::get_files_from_directory(dir, move |file| {
        if let Some(file_ext) = file.extension() {
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

pub fn metadata_widget_new(title: &str) -> Box<MetadataWidget> {
    Box::new(MetadataWidget::new(title))
}

#[cxx::bridge(namespace = "fwk")]
pub mod ffi {
    // Gtk types
    #[namespace = ""]
    extern "C++" {
        type GdkPixbuf;
        type GtkWidget;
    }

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
        unsafe fn thumbnail_from_pixbuf(pixbuf: *mut GdkPixbuf) -> Box<Thumbnail>;
        #[cxx_name = "Thumbnail_to_pixbuf"]
        fn thumbnail_to_pixbuf(self_: &Thumbnail) -> *mut GdkPixbuf;
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

        fn is_empty(&self) -> bool;
        fn is_string(&self) -> bool;
        #[cxx_name = "get_string"]
        fn string_unchecked(&self) -> &str;
    }

    extern "C++" {
        include!("fwk/cxx_widgets_bindings.hpp");

        type WrappedPropertyBag = crate::toolkit::widgets::WrappedPropertyBag;
        type MetadataSectionFormat = crate::toolkit::widgets::MetadataSectionFormat;
    }

    extern "Rust" {
        type MetadataWidget;

        fn gobj(&self) -> *mut GtkWidget;
        #[cxx_name = "MetadataWidget_new"]
        fn metadata_widget_new(title: &str) -> Box<MetadataWidget>;
        #[cxx_name = "set_data_format"]
        fn set_data_format_(&self, fmt: &MetadataSectionFormat);
        #[cxx_name = "set_data_source"]
        fn set_data_source_wrapped(&self, properties: &WrappedPropertyBag);
        fn set_data_source_none(&self);
        fn wrapped_property_bag_clone(bag: &WrappedPropertyBag) -> *mut WrappedPropertyBag;
        unsafe fn wrapped_property_bag_drop(bag: *mut WrappedPropertyBag);
    }

    unsafe extern "C++" {
        include!("fwk/toolkit/undo.hpp");
        type UndoListener;

        fn call(&self);
    }

    unsafe extern "C++" {
        type UndoFnInt;

        fn call(&self, v: i64);
    }

    unsafe extern "C++" {
        type RedoFnInt;

        fn call(&self) -> i64;
    }

    unsafe extern "C++" {
        type UndoFnVoid;

        fn call(&self);
    }

    unsafe extern "C++" {
        type RedoFnVoid;

        fn call(&self);
    }

    extern "Rust" {
        type UndoCommand;

        #[cxx_name = "UndoCommand_new"]
        pub fn undo_command_new(
            redo_fn: UniquePtr<RedoFnVoid>,
            undo_fn: UniquePtr<UndoFnVoid>,
        ) -> Box<UndoCommand>;
        #[cxx_name = "UndoCommand_new_int"]
        pub fn undo_command_new_int(
            redo_fn: UniquePtr<RedoFnInt>,
            undo_fn: UniquePtr<UndoFnInt>,
        ) -> Box<UndoCommand>;
    }

    extern "Rust" {
        type UndoTransaction;

        #[cxx_name = "UndoTransaction_new"]
        fn undo_transaction_new(name: &str) -> Box<UndoTransaction>;
        #[cxx_name = "add"]
        fn add_(&mut self, command: Box<UndoCommand>);
        fn is_empty(&self) -> bool;
        fn execute(&self);
    }

    extern "Rust" {
        type UndoHistory;

        #[cxx_name = "UndoHistory_new"]
        fn undo_history_new() -> Box<UndoHistory>;
        #[cxx_name = "add"]
        fn add_(&mut self, transaction: Box<UndoTransaction>);
        fn has_redo(&self) -> bool;
        fn next_redo(&self) -> String;
        fn redo(&mut self);
        fn has_undo(&self) -> bool;
        fn next_undo(&self) -> String;
        fn undo(&mut self);

        fn add_listener(&self, listener: UniquePtr<UndoListener>);
    }

    unsafe extern "C++" {
        include!("fwk/toolkit/application.hpp");
        type Application;

        fn Application_app() -> SharedPtr<Application>;
        fn config(&self) -> &SharedPtr<SharedConfiguration>;
        fn undo_history(&self) -> &UndoHistory;
        fn begin_undo(&self, undo: Box<UndoTransaction>);
    }
}
