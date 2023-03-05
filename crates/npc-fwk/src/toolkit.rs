/*
 * niepce - crates/npc-fwk/src/toolkit.rs
 *
 * Copyright (C) 2020-2023 Hubert Figuière
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

pub mod assistant;
mod configuration;
pub mod confirm;
#[macro_use]
mod controller;
pub mod gdk_utils;
mod gphoto;
pub mod gtk_utils;
mod image_bitmap;
pub mod mimetype;
pub mod movieutils;
pub mod request;
pub mod thumbnail;
mod uicontroller;
mod undo;
pub mod widgets;
mod window_controller;

/// Module to re-export cxx only.
pub mod cxx {
    pub use super::undo::{
        undo_command_new, undo_command_new_int, undo_history_new, undo_transaction_new,
    };
}

pub use controller::{new_controller, to_controller, Controller, ControllerImpl};
pub use gphoto::{GpCamera, GpDevice, GpDeviceList};
pub use image_bitmap::ImageBitmap;
pub use thumbnail::Thumbnail;
pub use uicontroller::UiController;
pub use undo::do_command as undo_do_command;
pub use undo::{Storage, UndoCommand, UndoHistory, UndoTransaction};
pub use window_controller::{create_redo_action, create_undo_action, WindowController};

pub use configuration::Configuration;

pub type Sender<T> = async_channel::Sender<T>;

/// Wrapper type for the channel tuple to get passed down to the unsafe C++ code.
pub struct PortableChannel<T>(pub Sender<T>);

pub fn thread_context() -> glib::MainContext {
    glib::MainContext::thread_default().unwrap_or_else(|| {
        let ctx = glib::MainContext::new();
        on_err_out!(ctx.with_thread_default(|| true));
        ctx
    })
}
