/*
 * niepce - crates/npc-fwk/src/toolkit/mod.rs
 *
 * Copyright (C) 2020-2021 Hubert Figuière
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

pub mod clickable_cell_renderer;
pub mod gdk_utils;
pub mod mimetype;
pub mod movieutils;
pub mod thumbnail;
pub mod widgets;

pub type Sender<T> = async_channel::Sender<T>;

/// Wrapper type for the channel tuple to get passed down to the unsafe C++ code.
pub struct PortableChannel<T>(pub Sender<T>);

pub fn thread_context() -> glib::MainContext {
    glib::MainContext::thread_default().unwrap_or_else(|| {
        let ctx = glib::MainContext::new();
        ctx.push_thread_default();
        ctx
    })
}
