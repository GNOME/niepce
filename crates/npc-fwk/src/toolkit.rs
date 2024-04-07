/*
 * niepce - crates/npc-fwk/src/toolkit.rs
 *
 * Copyright (C) 2020-2024 Hubert Figuière
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

mod app_controller;
pub mod assistant;
pub mod channels;
mod configuration;
pub mod confirm;
#[macro_use]
mod controller;
pub mod gdk_utils;
mod gphoto;
pub mod gtk_utils;
pub mod heif;
mod image_bitmap;
mod map_controller;
pub mod mimetype;
pub mod movieutils;
pub mod request;
pub mod thumbnail;
mod uicontroller;
mod undo;
pub mod widgets;
mod window_controller;

pub use app_controller::{AppController, AppControllerSingleton};
pub use channels::{channel, send_async_any, send_async_local, Receiver, Sender};
pub use controller::{Controller, ControllerImpl, ControllerImplCell};
pub use gphoto::{GpCamera, GpDevice, GpDeviceList};
pub use image_bitmap::ImageBitmap;
pub use map_controller::MapController;
pub use thumbnail::Thumbnail;
pub use uicontroller::{DialogController, UiController};
pub use undo::do_command as undo_do_command;
pub use undo::{Storage, UndoCommand, UndoHistory, UndoTransaction};
pub use window_controller::{create_redo_action, create_undo_action, WindowController};

pub use configuration::Configuration;

pub fn thread_context() -> glib::MainContext {
    glib::MainContext::thread_default().unwrap_or_else(|| {
        let ctx = glib::MainContext::new();
        on_err_out!(ctx.with_thread_default(|| true));
        ctx
    })
}
