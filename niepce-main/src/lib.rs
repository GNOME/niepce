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
extern crate libadwaita as adw;

pub mod config;
pub mod modules;
pub mod niepce;
mod notification_center;

use std::sync::Once;

// Initialize the resource as we can use the C trick,
// we inline and load them.
pub fn init_resources() -> Result<(), glib::Error> {
    // load the gresource binary at build time and include/link it into the final
    // binary.
    // The assumption here is that it's built within the build system.
    let res_bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_DIR"),
        "/../src/niepce/npc-resources.gresource"
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
        ffi::init();

        gtk4::init().unwrap();
        adw::init().unwrap();
        npc_fwk::init();

        init_resources().expect("Couldn't load resources");
    });
}

pub use notification_center::NotificationCenter;

// cxx bindings
use niepce::ui::niepce_application::{action_about, action_preferences};
use niepce::ui::niepce_window::{niepce_window_new, NiepceWindowWrapper};

#[cxx::bridge(namespace = "npc")]
pub mod ffi {
    #[namespace = ""]
    unsafe extern "C++" {
        type GMenu;
        type GtkApplication;
        type GtkBox;
        type GtkDrawingArea;
        type GtkPopoverMenu;
        type GtkWidget;
        type GtkWindow;
    }

    #[namespace = "eng"]
    extern "C++" {
        include!("fwk/cxx_prelude.hpp");
    }

    extern "Rust" {
        type NiepceWindowWrapper;

        unsafe fn niepce_window_new(app: *mut GtkApplication) -> Box<NiepceWindowWrapper>;
        fn on_ready(&self);
        fn on_open_catalog(&self);
        fn widget(&self) -> *mut GtkWidget;
        fn window(&self) -> *mut GtkWindow;
        fn menu(&self) -> *mut GMenu;
    }

    #[namespace = "Gio"]
    unsafe extern "C++" {
        include!(<giomm/init.h>);

        fn init();
    }

    #[namespace = "ui"]
    unsafe extern "C++" {
        include!("niepce/ui/niepceapplication.hpp");
        type NiepceApplication;

        fn niepce_application_create() -> SharedPtr<NiepceApplication>;
        fn main(&self);
    }

    extern "Rust" {
        unsafe fn action_about(parent: *mut GtkWindow);
        unsafe fn action_preferences(parent: *mut GtkWindow);
    }
}
