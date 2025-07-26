/*
 * niepce - fwk/toolkit/widgets/toolbox_item.rs
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

use crate::glib;
use crate::gtk4;
use glib::subclass::prelude::*;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

glib::wrapper! {
    /// A `ToolboxItem` is just a box with a `gtk4::Expander`
    /// The content is the child, which is actually the expander child
    /// The label is the `gtk4::Expander` label.
    pub struct ToolboxItem(
        ObjectSubclass<imp::ToolboxItem>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl ToolboxItem {
    pub fn new(label: &str) -> ToolboxItem {
        let obj = glib::Object::builder::<Self>()
            .property("spacing", 0)
            .property("orientation", gtk4::Orientation::Vertical)
            .build();
        obj.imp().expander.set_label(Some(label));
        obj
    }

    pub fn set_title(&self, title: &str) {
        self.imp().expander.set_label(Some(title));
    }
}

pub trait ToolboxItemExt {
    /// Set the child (of the `gtk4::Expander`).
    fn set_child(&self, child: Option<&impl IsA<gtk4::Widget>>);
}

impl ToolboxItemExt for ToolboxItem {
    fn set_child(&self, child: Option<&impl IsA<gtk4::Widget>>) {
        self.imp().expander.set_child(child);
    }
}

pub trait ToolboxItemImpl: BoxImpl {}

unsafe impl<T: ToolboxItemImpl> IsSubclassable<T> for ToolboxItem {}

mod imp {
    use crate::glib;
    use crate::gtk4;
    use glib::subclass::prelude::*;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;

    pub struct ToolboxItem {
        pub(super) expander: gtk4::Expander,
    }

    impl ObjectImpl for ToolboxItem {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().append(&self.expander);
            self.expander.set_expanded(true);
            self.expander.set_use_markup(true);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ToolboxItem {
        const NAME: &'static str = "NpcToolboxItem";
        type Type = super::ToolboxItem;
        type ParentType = gtk4::Box;

        fn new() -> Self {
            Self {
                expander: gtk4::Expander::new(None),
            }
        }
    }

    impl BoxImpl for ToolboxItem {}
    impl WidgetImpl for ToolboxItem {}
}
