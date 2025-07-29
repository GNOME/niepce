/*
 * niepce - crates/npc-fwk/src/toolkit/assistant.rs
 *
 * Copyright (C) 2022-2023 Hubert Figuière
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

//!
//! Utilities for handling assistant.
//! Currently Gtk specific

use crate::gtk4;

use gtk4::Widget;
use gtk4::prelude::*;
use num_traits::{FromPrimitive, ToPrimitive};

/// Set the page index into the widget
pub fn set_page_index<T: ToPrimitive>(page: &Widget, idx: T) {
    unsafe {
        page.set_data("page-index", idx.to_i32().unwrap());
    }
}

/// Get the page index from the widget if there is one
pub fn get_page_index<T: FromPrimitive>(page: &Widget) -> Option<T> {
    unsafe { page.data::<i32>("page-index") }.and_then(|qd| T::from_i32(unsafe { *qd.as_ptr() }))
}
