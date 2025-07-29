/*
 * niepce - niepce/ui/dialogs/import/thumb_item.rs
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

use glib::subclass::prelude::*;
use npc_fwk::glib;

use npc_engine::importer::ImportedFile;
use npc_fwk::Date;

glib::wrapper! {
    /// Item in the workspace
    pub struct ThumbItem(
        ObjectSubclass<imp::ThumbItem>);
}

impl ThumbItem {
    pub fn new(imported_file: &dyn ImportedFile) -> Self {
        let obj: Self = glib::Object::builder()
            .property("name", imported_file.name().to_string())
            .build();
        obj.imp().data.replace(Some(imp::ItemData { date: None }));

        obj
    }

    pub fn set_date(&self, date: Option<Date>) {
        if let Some(ref mut data) = *self.imp().data.borrow_mut() {
            data.date = date;
        }
    }
}

mod imp {
    use std::cell::RefCell;

    use gio::subclass::prelude::*;
    use glib::Properties;
    use glib::prelude::*;
    use npc_fwk::{gdk4, gio, glib};

    use npc_fwk::Date;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::ThumbItem)]
    pub struct ThumbItem {
        pub(super) data: RefCell<Option<ItemData>>,
        #[property(get, set)]
        pub(super) name: RefCell<String>,
        #[property(get, set, nullable)]
        pub(super) pixbuf: RefCell<Option<gdk4::Paintable>>,
    }

    pub(super) struct ItemData {
        pub(super) date: Option<Date>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ThumbItem {
        const NAME: &'static str = "ThumbItem";
        type Type = super::ThumbItem;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ThumbItem {}
}
