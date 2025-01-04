/*
 * niepce - libraryclient/uidataprovider.rs
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

use std::cell::RefCell;

use crate::catalog::{Label, LibraryId};
use npc_fwk::base::rgbcolour::RgbColour;

/// UIDataProvider provide data for the library for the UI
/// Currently handle the `Label`
pub struct UIDataProvider {
    labels: RefCell<Vec<Label>>,
}

impl Default for UIDataProvider {
    fn default() -> UIDataProvider {
        UIDataProvider {
            labels: RefCell::new(Vec::default()),
        }
    }
}

impl UIDataProvider {
    pub fn add_label(&self, label: &Label) {
        self.labels.borrow_mut().push(label.clone());
    }

    pub fn update_label(&self, label: &Label) {
        let id = label.id();
        for l in &mut self.labels.borrow_mut().iter_mut() {
            if l.id() == id {
                *l = label.clone();
                break;
            }
        }
    }

    pub fn delete_label(&self, label: LibraryId) {
        let mut labels = self.labels.borrow_mut();
        if let Some(idx) = labels.iter().position(|l| l.id() == label) {
            labels.remove(idx);
        }
    }

    pub fn colour_for_label(&self, id: LibraryId) -> RgbColour {
        if let Some(label) = self.labels.borrow().iter().find(|l| l.id() == id) {
            label.colour().clone()
        } else {
            RgbColour::default()
        }
    }

    pub fn label_count(&self) -> usize {
        self.labels.borrow().len()
    }

    pub fn label_at(&self, idx: usize) -> Label {
        self.labels.borrow()[idx].clone()
    }
}
