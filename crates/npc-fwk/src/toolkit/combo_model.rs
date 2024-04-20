/*
 * niepce - npc_fwk/toolkit/combo_model.rs
 *
 * Copyright (C) 2024 Hubert Figuière
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

use std::rc::Rc;

use gtk4::prelude::*;

/// A "model" to bind T values to a `gtk4::StringList`.
pub struct ComboModel<T> {
    model: gtk4::StringList,
    map: Vec<T>,
}

impl<T: Clone + 'static> ComboModel<T> {
    /// New empty model.
    pub fn new() -> Rc<ComboModel<T>> {
        Rc::new(ComboModel {
            model: gtk4::StringList::new(&[]),
            map: vec![],
        })
    }

    /// New model with map.
    pub fn with_map(map: &[(&str, T)]) -> Rc<ComboModel<T>> {
        Rc::new(ComboModel {
            model: gtk4::StringList::new(&map.iter().map(|v| v.0).collect::<Vec<&str>>()),
            map: map.iter().map(|v| v.1.clone()).collect(),
        })
    }

    /// Push a value.
    pub fn push(&mut self, key: &str, value: T) {
        self.model.append(key);
        self.map.push(value);
    }

    /// Remove value at index.
    pub fn remove(&mut self, index: usize) {
        self.model.remove(index as u32);
        self.map.remove(index);
    }

    /// Get value at index.
    fn value(&self, index: usize) -> &T {
        &self.map[index]
    }

    /// Bind the DropDown selection change to the `callback`
    /// and set the model.
    pub fn bind<F: Fn(&T) + 'static>(
        self: Rc<Self>,
        dropdown: &impl IsA<gtk4::DropDown>,
        callback: F,
    ) {
        let dropdown = dropdown.as_ref();
        dropdown.set_model(Some(&self.model));
        dropdown.connect_selected_item_notify(
            glib::clone!(@strong self as model => move |dropdown| {
                let value = model.value(dropdown.selected() as usize);
                callback(value);
            }),
        );
    }
}
