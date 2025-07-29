/*
 * niepce - modules/darkroom/dr_item.rs
 *
 * Copyright (C) 2024-2025 Hubert Figuière
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
use gtk4::prelude::*;
use npc_fwk::{glib, gtk4};

use npc_fwk::toolkit::widgets::ToolboxItem;

glib::wrapper! {
    /// A `DrItem` is just a box with a `ToolboxItem`
    pub struct DrItem(
        ObjectSubclass<imp::DrItem>)
        @extends ToolboxItem, gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl DrItem {
    pub fn new(label: &str) -> DrItem {
        let obj = glib::Object::builder::<Self>()
            .property("spacing", 0)
            .property("orientation", gtk4::Orientation::Vertical)
            .build();
        obj.upcast_ref::<ToolboxItem>().set_title(label);
        obj
    }

    pub fn add_widget(&self, label: &str, widget: &impl IsA<gtk4::Widget>) {
        let label = gtk4::Label::new(Some(label));
        label.set_xalign(0.0);
        let imp = self.imp();
        imp.vbox.append(&label);
        imp.vbox.append(widget);
    }
}

mod imp {
    use glib::subclass::prelude::*;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use npc_fwk::{glib, gtk4};

    use npc_fwk::toolkit::widgets::ToolboxItem;
    use npc_fwk::toolkit::widgets::prelude::*;

    pub struct DrItem {
        pub(super) vbox: gtk4::Box,
    }

    impl ObjectImpl for DrItem {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj()
                .upcast_ref::<ToolboxItem>()
                .set_child(Some(&self.vbox));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DrItem {
        const NAME: &'static str = "NpcDrItem";
        type Type = super::DrItem;
        type ParentType = ToolboxItem;

        fn new() -> Self {
            Self {
                vbox: gtk4::Box::new(gtk4::Orientation::Vertical, 0),
            }
        }
    }

    impl ToolboxItemImpl for DrItem {}
    impl BoxImpl for DrItem {}
    impl WidgetImpl for DrItem {}
}
