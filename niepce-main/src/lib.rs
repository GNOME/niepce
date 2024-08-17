/*
 * niepce - lib.rs
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

#[macro_use]
extern crate gtk_macros;

pub mod config;
pub mod modules;
pub mod niepce;
mod notification_center;

use std::sync::Once;

use npc_fwk::{adw, gio, glib, gtk4};

pub use niepce::ui::niepce_application::NiepceApplication;

// Initialize the resource as we can use the C trick,
// we inline and load them.
pub fn init_resources() -> Result<(), glib::Error> {
    // load the gresource binary at build time and include/link it into the final
    // binary.
    // The assumption here is that it's built within the build system.
    let res_bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_DIR"),
        "/../niepce-main/src/npc-resources.gresource"
    ));

    // Create Resource it will live as long the value lives.
    let gbytes = glib::Bytes::from_static(res_bytes.as_ref());
    let resource = gio::Resource::from_data(&gbytes)?;

    // Register the resource so it won't be dropped and will continue to live in
    // memory.
    gio::resources_register(&resource);
    Ok(())
}

pub fn niepce_init() {
    static START: Once = Once::new();

    START.call_once(|| {
        gtk4::init().unwrap();
        adw::init().unwrap();
        npc_fwk::init();

        init_resources().expect("Couldn't load resources");
    });
}

pub use notification_center::NotificationCenter;
