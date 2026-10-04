/*
 * niepce - fwk/toolkit/widgets/choice.rs
 *
 * Copyright (C) 2026 Hubert Figuière
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

//! A widget to show choices (in a DropDown).

use std::rc::Rc;

use crate::adw;
use crate::glib;
use crate::gtk4;
use adw::subclass::prelude::*;

use crate::toolkit::ComboModel;

glib::wrapper! {
    pub struct Choice(
        ObjectSubclass<imp::Choice>)
        @extends adw::Bin, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for Choice {
    fn default() -> Self {
        Self::new()
    }
}

impl Choice {
    pub fn new() -> Choice {
        glib::Object::new()
    }

    pub fn selected(&self) -> u32 {
        self.imp().dropdown.selected()
    }

    pub fn set_selected(&self, selected: u32) {
        self.imp().dropdown.set_selected(selected);
    }

    /// Bind a `ComboModel` to the `Choice` widget.
    pub fn bind_model<T, F>(&self, combo_model: Rc<ComboModel<T>>, f: F)
    where
        T: Clone + std::cmp::PartialEq + 'static,
        F: Fn(&T) + 'static,
    {
        combo_model.bind(&self.imp().dropdown, f);
    }
}

mod imp {
    use crate::adw;
    use crate::glib;
    use crate::gtk4;
    use adw::prelude::*;
    use adw::subclass::prelude::*;

    #[derive(Default)]
    pub struct Choice {
        pub(super) dropdown: gtk4::DropDown,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Choice {
        const NAME: &'static str = "Choice";
        type Type = super::Choice;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for Choice {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().set_child(Some(&self.dropdown));
        }
    }

    impl BinImpl for Choice {}
    impl WidgetImpl for Choice {}
}
