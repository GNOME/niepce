/*
 * niepce - npc_fwk/toolkit/widgets/dock.rs
 *
 * Copyright (C) 2023-2025 Hubert Figuière
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
use gtk4::subclass::prelude::*;

glib::wrapper! {
    pub struct Dock(
        ObjectSubclass<imp::Dock>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Dock {
    pub fn new() -> Dock {
        glib::Object::new()
    }

    pub fn vbox(&self) -> &gtk4::Box {
        &self.imp().vbox
    }
}

impl Default for Dock {
    fn default() -> Dock {
        Dock::new()
    }
}

mod imp {
    use crate::glib;
    use crate::gtk4;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;

    #[derive(Default)]
    pub struct Dock {
        scrolled: gtk4::ScrolledWindow,
        pub(super) vbox: gtk4::Box,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Dock {
        const NAME: &'static str = "Dock";
        type Type = super::Dock;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for Dock {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().append(&self.scrolled);
            self.vbox.set_orientation(gtk4::Orientation::Vertical);
            self.scrolled.set_child(Some(&self.vbox));
            self.scrolled
                .set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Always);
        }
    }

    impl BoxImpl for Dock {}
    impl WidgetImpl for Dock {}
}
