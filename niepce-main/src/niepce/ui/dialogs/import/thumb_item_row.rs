/*
 * niepce - niepce/ui/dialogs/import/thumb_item_row.rs
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

use glib::subclass::prelude::*;

glib::wrapper! {
    /// Item in the workspace
    pub struct ThumbItemRow(
        ObjectSubclass<imp::ThumbItemRow>)
        @extends gtk4::Box, gtk4::Widget;
}

impl ThumbItemRow {
    pub fn new() -> Self {
        glib::Object::builder::<Self>()
            .property("spacing", 2)
            .property("orientation", &gtk4::Orientation::Vertical)
            .build()
    }

    pub fn set_label(&self, label: &str) {
        self.imp().filename.set_label(label);
    }

    pub fn set_image(&self, image: &gdk_pixbuf::Pixbuf) {
        self.imp().image.set_from_pixbuf(Some(image));
    }
}

impl Default for ThumbItemRow {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;

    #[derive(Default)]
    pub struct ThumbItemRow {
        pub(super) image: gtk4::Image,
        pub(super) filename: gtk4::Label,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ThumbItemRow {
        const NAME: &'static str = "ThumbItemRow";
        type Type = super::ThumbItemRow;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for ThumbItemRow {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().append(&self.image);
            self.obj().append(&self.filename);
            self.image.set_size_request(100, 100);
            // Adwaita class
            self.filename.add_css_class("caption");
        }
    }

    impl WidgetImpl for ThumbItemRow {}
    impl BoxImpl for ThumbItemRow {}
}
