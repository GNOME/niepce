/*
 * niepce - niepce/ui/image_list_item.rs
 *
 * Copyright (C) 2022-2025 Hubert Figuière
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

use std::cell::{Cell, RefCell};

use gdk4::subclass::prelude::*;
use glib::Properties;
use glib::prelude::*;
use npc_fwk::{gdk4, glib};

use npc_engine::catalog::libfile::{FileStatus, LibFile};

glib::wrapper! {
    /// The is the list item as stored in the `gio::ListModel`.
    pub struct ImageListItem(
        ObjectSubclass<ImageListItemPriv>);
}

impl ImageListItem {
    pub fn new(
        thumbnail: Option<gdk4::Paintable>,
        file: Option<LibFile>,
        file_status: FileStatus,
    ) -> Self {
        glib::Object::builder()
            .property("thumbnail", thumbnail)
            .property("file", file)
            .property("file-status", file_status)
            .build()
    }
}

#[derive(Default, Properties)]
#[properties(wrapper_type = ImageListItem)]
pub struct ImageListItemPriv {
    /// The thumbnail to display.
    #[property(get, set, nullable)]
    pub thumbnail: RefCell<Option<gdk4::Paintable>>,
    /// The corresponding file.
    #[property(get, set, nullable)]
    pub file: RefCell<Option<LibFile>>,
    /// The file status.
    #[property(get, set, name = "file-status", builder(FileStatus::default()))]
    pub file_status: Cell<FileStatus>,
}

#[glib::object_subclass]
impl ObjectSubclass for ImageListItemPriv {
    const NAME: &'static str = "ImageListItem";
    type Type = ImageListItem;
    type ParentType = glib::Object;
}

#[glib::derived_properties]
impl ObjectImpl for ImageListItemPriv {}
