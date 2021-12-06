/*
 * niepce - fwk/lib.rs
 *
 * Copyright (C) 2017-2020 Hubert Figuière
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

extern crate chrono;
extern crate exempi2;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gdk_pixbuf_sys;
extern crate gio;
extern crate gio_sys;
extern crate glib;
extern crate glib_sys;
extern crate gtk;
extern crate gtk_sys;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate multimap;
extern crate once_cell;

#[macro_use]
pub mod base;
pub mod capi;
pub mod toolkit;
pub mod utils;

pub use self::base::fractions::fraction_to_decimal;
pub use self::base::propertybag::PropertyBag;
pub use self::base::propertyvalue::PropertyValue;
pub use self::base::PropertySet;
pub use self::utils::exempi::{gps_coord_from_xmp, ExempiManager, NsDef, XmpMeta};

pub use self::base::date::*;

pub use self::toolkit::mimetype::MimeType;

use libc::c_char;
use std::f64;
use std::ffi::CStr;

/// Convert a gps coord (in string format) to a decimal (floating point)
///
/// # Safety
/// Dereference the pointer.
#[no_mangle]
pub unsafe extern "C" fn fwk_gps_coord_from_xmp(cvalue: *const c_char) -> f64 {
    let value = CStr::from_ptr(cvalue);
    if let Ok(svalue) = value.to_str() {
        if let Some(coord) = gps_coord_from_xmp(svalue) {
            return coord;
        }
    }
    f64::NAN
}

/// Convert a fraction (in string format) to a decimal (floating point)
///
/// # Safety
/// Dereference the pointer.
#[no_mangle]
pub unsafe extern "C" fn fwk_fraction_to_decimal(cvalue: *const c_char) -> f64 {
    let value = CStr::from_ptr(cvalue);
    if let Ok(svalue) = value.to_str() {
        if let Some(dec) = fraction_to_decimal(svalue) {
            return dec;
        }
    }
    f64::NAN
}

///
/// Init funtion because rexiv2 need one.
///
/// Make sure to call it after gtk::init()
///
pub fn init() {
    rexiv2::initialize().expect("Unable to initialize rexiv2");
}
