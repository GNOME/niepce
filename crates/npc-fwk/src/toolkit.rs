/*
 * niepce - crates/npc-fwk/src/toolkit.rs
 *
 * Copyright (C) 2020-2025 Hubert Figuière
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
mod combo_model;
mod configuration;
pub mod confirm;
#[macro_use]
mod controller;
mod gphoto;
pub mod gtk_utils;
pub mod heif;
mod image_bitmap;
mod list_view;
mod map_controller;
pub mod mimetype;
pub mod movieutils;
pub mod request;
pub mod thumbnail;
pub mod tree_view_model;
mod uicontroller;
mod undo;
pub mod widgets;
mod window_controller;

pub use app_controller::AppController;
pub use channels::{Receiver, Sender, channel, send_async_any, send_async_local};
pub use combo_model::ComboModel;
pub use controller::{Controller, ControllerImpl, ControllerImplCell};
pub use gphoto::{GpCamera, GpDevice, GpDeviceList};
pub use image_bitmap::ImageBitmap;
pub use list_view::ListViewRow;
pub use map_controller::MapController;
pub use thumbnail::Thumbnail;
pub use tree_view_model::{TreeViewFactory, TreeViewItem, TreeViewModel};
pub use uicontroller::{DialogController, UiController, WindowSize};
pub use undo::do_command as undo_do_command;
pub use undo::{RedoFn, Storage, UndoCommand, UndoFn, UndoHistory, UndoTransaction};
pub use window_controller::{WindowController, create_redo_action, create_undo_action};

pub use configuration::{ConfigBackend, Configuration};

use crate::glib;

pub fn thread_context() -> glib::MainContext {
    glib::MainContext::thread_default().unwrap_or_else(|| {
        let ctx = glib::MainContext::new();
        on_err_out!(ctx.with_thread_default(|| true));
        ctx
    })
}
