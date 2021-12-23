/*
 * niepce - lib.rs
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

extern crate cairo;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gdk_pixbuf_sys;
extern crate gettextrs;
extern crate gio;
extern crate gio_sys;
#[macro_use]
extern crate glib;
extern crate gtk;
extern crate gtk_sys;
extern crate libc;
extern crate once_cell;
#[macro_use]
extern crate num_derive;

// internal crates
#[macro_use]
extern crate npc_fwk;
extern crate npc_engine;

pub mod niepce;

use std::sync::Once;

/// Call this to initialize npc-fwk the gtk-rs bindings
#[no_mangle]
pub extern "C" fn niepce_init() {
    static START: Once = Once::new();

    START.call_once(|| {
        gtk::init().unwrap();
        npc_fwk::init();
    });
}
