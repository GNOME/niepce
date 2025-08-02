/*
 * niepce - niepce/ui/dialogs/import/thumb_item_row.rs
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

use std::cell::RefMut;

use glib::subclass::prelude::*;
use npc_fwk::{glib, gtk4};

use super::ThumbItem;
use npc_fwk::toolkit::ListViewRow;

glib::wrapper! {
    /// Item in the workspace
    pub struct ThumbItemRow(
        ObjectSubclass<imp::ThumbItemRow>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl ThumbItemRow {
    pub fn new() -> Self {
        glib::Object::builder::<Self>()
            .property("spacing", 2)
            .property("orientation", gtk4::Orientation::Vertical)
            .build()
    }
}

impl ListViewRow<ThumbItem> for ThumbItemRow {
    fn bind(&self, thumb_item: &ThumbItem, _tree_list_row: Option<&gtk4::TreeListRow>) {
        self.bind_to_prop("filename", thumb_item, "name");
        self.bind_to_prop("image", thumb_item, "pixbuf");
    }

    fn bindings_mut(&self) -> RefMut<'_, Vec<glib::Binding>> {
        self.imp().bindings.borrow_mut()
    }
}

impl Default for ThumbItemRow {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::Properties;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use npc_fwk::{gdk4, glib, gtk4};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::ThumbItemRow)]
    pub struct ThumbItemRow {
        #[property(set = |row: &&Self, p: Option<&gdk4::Paintable>| row.image.set_paintable(p), type = gdk4::Paintable, nullable)]
        pub(super) image: gtk4::Image,
        #[property(set = |row: &&Self, n| row.filename.set_label(n), type = String)]
        pub(super) filename: gtk4::Label,
        pub(super) bindings: RefCell<Vec<glib::Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ThumbItemRow {
        const NAME: &'static str = "ThumbItemRow";
        type Type = super::ThumbItemRow;
        type ParentType = gtk4::Box;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ThumbItemRow {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().append(&self.image);
            self.obj().append(&self.filename);
            self.image.set_size_request(100, 100);
            self.image.set_pixel_size(100);
            // Adwaita class
            self.filename.add_css_class("caption");
        }
    }

    impl WidgetImpl for ThumbItemRow {}
    impl BoxImpl for ThumbItemRow {}
}
