/*
 * niepce - niepce/ui/image_list_item.rs
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

use std::cell::RefCell;

use gdk4::subclass::prelude::*;

use npc_engine::db::libfile::{FileStatus, LibFile};

#[derive(Default)]
struct ImageListItemData {
    /// The thumbnail to display.
    pub thumbnail: Option<gdk4::Paintable>,
    /// The corresponding file.
    pub file: Option<LibFile>,
    /// The film strip thumbnail.
    // XXX do we need this?
    pub strip_thumbnail: Option<gdk4::Paintable>,
    /// The file status.
    pub file_status: FileStatus,
}

glib::wrapper! {
    /// The is the list item as stored in the `gio::ListModel`.
    pub struct ImageListItem(
        ObjectSubclass<ImageListItemPriv>);
}

impl ImageListItem {
    pub fn new(
        thumbnail: Option<gdk4::Paintable>,
        file: Option<LibFile>,
        strip_thumbnail: Option<gdk4::Paintable>,
        file_status: FileStatus,
    ) -> Self {
        let obj: Self = glib::Object::new();

        // This is suboptimal
        obj.imp().data.replace(ImageListItemData {
            thumbnail,
            file,
            strip_thumbnail,
            file_status,
        });
        obj
    }

    pub fn thumbnail(&self) -> Option<gdk4::Paintable> {
        self.imp().data.borrow().thumbnail.clone()
    }

    pub fn set_thumbnail(&self, thumbnail: Option<gdk4::Paintable>) {
        self.imp().data.borrow_mut().thumbnail = thumbnail;
    }

    pub fn file(&self) -> Option<LibFile> {
        // We unwrap here. It's an error to be None.
        self.imp().data.borrow().file.clone()
    }

    pub fn set_file(&self, file: Option<LibFile>) {
        self.imp().data.borrow_mut().file = file;
    }

    pub fn status(&self) -> FileStatus {
        self.imp().data.borrow().file_status
    }

    pub fn set_status(&self, status: FileStatus) {
        self.imp().data.borrow_mut().file_status = status;
    }

    pub fn set_strip_thumbnail(&self, strip_thumbnail: Option<gdk4::Paintable>) {
        self.imp().data.borrow_mut().strip_thumbnail = strip_thumbnail;
    }
}

#[derive(Default)]
pub struct ImageListItemPriv {
    data: RefCell<ImageListItemData>,
}

#[glib::object_subclass]
impl ObjectSubclass for ImageListItemPriv {
    const NAME: &'static str = "ImageListItem";
    type Type = ImageListItem;
    type ParentType = glib::Object;
}

impl ObjectImpl for ImageListItemPriv {}
