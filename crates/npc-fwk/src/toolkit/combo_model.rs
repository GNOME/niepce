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

use std::cell::RefCell;
use std::rc::Rc;

use crate::glib;
use crate::gtk4;
use gtk4::prelude::*;

/// A "model" to bind T values to a `gtk4::StringList`.
#[derive(Default)]
pub struct ComboModel<T> {
    model: gtk4::StringList,
    map: RefCell<Vec<T>>,
}

impl<T: Clone + std::cmp::PartialEq + 'static> ComboModel<T> {
    /// New empty model.
    pub fn new() -> Rc<ComboModel<T>> {
        Rc::new(ComboModel {
            model: gtk4::StringList::new(&[]),
            map: RefCell::default(),
        })
    }

    /// New model with map.
    pub fn with_map(map: &[(&str, T)]) -> Rc<ComboModel<T>> {
        Rc::new(ComboModel {
            model: gtk4::StringList::new(&map.iter().map(|v| v.0).collect::<Vec<&str>>()),
            map: RefCell::new(map.iter().map(|v| v.1.clone()).collect()),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.map.borrow().is_empty()
    }

    /// Push a value.
    pub fn push(&self, key: &str, value: T) {
        self.map.borrow_mut().push(value);
        self.model.append(key);
    }

    /// Remove value at index.
    pub fn remove(&self, index: usize) {
        self.map.borrow_mut().remove(index);
        self.model.remove(index as u32);
    }

    /// Get value at index.
    pub fn value(&self, index: usize) -> T {
        self.map.borrow()[index].clone()
    }

    pub fn index_of(&self, value: &T) -> Option<usize> {
        self.map.borrow().iter().position(|v| v == value)
    }

    /// Bind the DropDown selection change to the `callback`
    /// and set the model.
    pub fn bind<F: Fn(&T) + 'static>(
        self: &Rc<Self>,
        dropdown: &impl IsA<gtk4::DropDown>,
        callback: F,
    ) {
        let dropdown = dropdown.as_ref();
        dropdown.set_model(Some(&self.model));
        dropdown.connect_selected_item_notify(glib::clone!(
            #[strong(rename_to = model)]
            self,
            move |dropdown| {
                let value = model.value(dropdown.selected() as usize);
                callback(&value);
            }
        ));
    }
}
