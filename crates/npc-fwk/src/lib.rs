/*
 * niepce - fwk/lib.rs
 *
 * Copyright (C) 2017-2025 Hubert Figuière
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

pub use adw;
pub use adw::gdk as gdk4;
pub use adw::gio;
pub use adw::glib;
pub use adw::gtk as gtk4;
pub use gdk4::cairo;
pub use gdk4::gdk_pixbuf;
pub use gtk4::graphene;

#[macro_use]
pub mod base;
pub mod toolkit;
pub mod utils;

pub use base::PropertySet;
pub use base::date::{Date, DateExt, Time};
pub use base::fractions::{fraction_to_decimal, parse_fraction};
pub use base::propertybag::PropertyBag;
pub use base::propertyvalue::PropertyValue;
pub use toolkit::mimetype::MimeType;
pub use utils::exempi::{ExempiManager, NsDef, XmpMeta, gps_coord_from_xmp};

///
/// Init funtion because rexiv2 need one.
///
/// Make sure to call it after gtk::init()
///
pub fn init() {
    rexiv2::initialize().expect("Unable to initialise rexiv2");
    gstreamer::init().expect("Unable to initialise gstreamer");
}
