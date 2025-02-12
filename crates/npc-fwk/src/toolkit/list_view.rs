/*
 * niepce - fwk/toolkit/list_view.rs
 *
 * Copyright (C) 2025 Hubert Figuière
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

//! ListView utilities.

use std::cell::RefMut;

use glib::prelude::*;

use crate::glib;

/// Trait to implement gtk list view rows for a list item.
/// Helps with bind and unbind, the latter being automatic.
///
/// Implementors need to have a field of type
/// `RefCell<Vec<glib::Binding>>` and return it in `binding_mut()`.
pub trait ListViewRow<I>: ObjectType
where
    I: IsA<glib::Object>,
{
    /// Bind an item to the row. It this is not a treeview, pass `None`
    /// as a tree_list_row.
    fn bind(&self, item: &I, tree_list_row: Option<&gtk4::TreeListRow>);

    /// Bind property `item_prop` of `item` to property `target_prop`
    /// of self.  And save it for auto unbind.
    ///
    /// The binding goes item.item_prop -> self.target
    fn bind_to_prop(&self, target_prop: &str, item: &I, item_prop: &str) {
        self.bind_to(self, target_prop, item, item_prop)
    }

    /// Bind property `item_prop` of `item` to property `target_prop`
    /// of target.  And save it for auto unbind.
    ///
    /// The binding goes item.item_prop -> target.target_prop
    fn bind_to<T: ObjectType>(&self, target: &T, target_prop: &str, item: &I, item_prop: &str) {
        let binding = item
            .bind_property(item_prop, target, target_prop)
            .sync_create()
            .build();
        self.save_binding(binding);
    }

    /// Close this from `unbind`. The default implementation does it.
    fn clear_bindings(&self) {
        let mut bindings = self.bindings_mut();
        bindings.iter().for_each(glib::Binding::unbind);
        bindings.clear();
    }

    /// Unbind the item from the row. The default implementation just
    /// unbind all the saved property bindings.
    fn unbind(&self) {
        self.clear_bindings();
    }

    /// Save the binding for automatic unbind.
    fn save_binding(&self, binding: glib::Binding) {
        self.bindings_mut().push(binding);
    }

    /// Return the stored bindings.
    ///
    /// Example implementation:
    /// ```ignore
    /// self.imp().bindings.borrow_mut()
    /// ```
    fn bindings_mut(&self) -> RefMut<'_, Vec<glib::Binding>>;
}
