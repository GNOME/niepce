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

pub mod libraryclient;
pub mod niepce;

use std::sync::Once;

fn niepce_init() {
    static START: Once = Once::new();

    START.call_once(|| {
        gtk4::init().unwrap();
        npc_fwk::init();
    });
}

use crate::niepce::ui::metadata_pane_controller::get_format;
use npc_fwk::toolkit;

#[cxx::bridge(namespace = "npc")]
mod ffi {
    extern "Rust" {
        fn niepce_init();
    }

    #[namespace = "fwk"]
    extern "C++" {
        include!("fwk/cxx_widgets_bindings.hpp");

        type MetadataSectionFormat = crate::toolkit::widgets::MetadataSectionFormat;
    }

    extern "Rust" {
        fn get_format() -> &'static [MetadataSectionFormat];
    }
}
