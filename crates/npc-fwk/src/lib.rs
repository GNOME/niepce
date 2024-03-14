/*
 * niepce - fwk/lib.rs
 *
 * Copyright (C) 2017-2024 Hubert Figuière
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

pub use self::base::date::{Date, Time};

pub use self::toolkit::mimetype::MimeType;

///
/// Init funtion because rexiv2 need one.
///
/// Make sure to call it after gtk::init()
///
pub fn init() {
    rexiv2::initialize().expect("Unable to initialize rexiv2");
}
