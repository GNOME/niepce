/*
 * niepce - niepce/ui/workspace_controller/ws_list_model.rs
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
use gtk4::prelude::*;
use thiserror::Error;

use super::ws_list_item::Item;
use npc_engine::db;

#[derive(Error, Debug)]
/// Errors from the list model
pub enum Error {
    /// Trying to append an item with invalid Id.
    #[error("Invalid ID")]
    InvalidId,
    /// Item wasn't found
    #[error("Not found")]
    NotFound,
}

glib::wrapper! {
    /// List model for the workspace
    pub struct WorkspaceList(
    ObjectSubclass<imp::WorkspaceList>)
        @implements gio::ListModel;
}

impl Default for WorkspaceList {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceList {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Append item. Returns an error if the id is invalid.
    pub fn append(&self, item: &Item) -> Result<(), Error> {
        // this scopes the borrow_mut() as items_changed() will call n_items()
        if item.id() == 0 {
            return Err(Error::InvalidId);
        }
        let mut removed = 0_u32;
        let pos = {
            let mut items = self.imp().items.borrow_mut();
            let mut pos = items.len();
            if items.push((item.id(), item.clone())).is_some() {
                // If the item is not found, then it's a bug.
                pos = items.index_of(&item.id()).unwrap();
                removed = 1;
            }

            pos as u32
        };
        self.items_changed(pos, removed, 1);
        Ok(())
    }

    /// Remove the item at `position`. Return it if found or an error.
    pub fn remove(&self, position: u32) -> Result<Item, Error> {
        if let Some(item) = self.imp().items.borrow_mut().remove_at(position as usize) {
            self.items_changed(position, 1, 0);
            Ok(item)
        } else {
            Err(Error::NotFound)
        }
    }

    /// Remove the item with `id`. Return it if found or an error.
    pub fn remove_by_id(&self, id: &db::LibraryId) -> Result<Item, Error> {
        let items = &self.imp().items;
        let index = items.borrow().index_of(id).ok_or(Error::NotFound)?;
        let item = items.borrow_mut().remove_at(index).ok_or(Error::NotFound)?;

        self.items_changed(index as u32, 1, 0);
        Ok(item)
    }

    /// Get item with `id`
    pub fn item_by_id(&self, id: &db::LibraryId) -> Option<Item> {
        self.imp().items.borrow().get(id).cloned()
    }

    /// Return the position of item with `id`.
    /// Currently tied to `IndexedMap::index_of` which is slow.
    pub fn pos_by_id(&self, id: &db::LibraryId) -> Option<u32> {
        self.imp().items.borrow().index_of(id).map(|idx| idx as u32)
    }
}

mod imp {
    use std::cell::RefCell;

    use gio::subclass::prelude::*;
    use glib::prelude::*;

    use npc_engine::db;
    use npc_fwk::base::IndexedMap;
    use npc_fwk::err_out;

    #[derive(Default)]
    pub struct WorkspaceList {
        /// The ordered items `id`
        pub(super) items: RefCell<IndexedMap<db::LibraryId, super::Item>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WorkspaceList {
        const NAME: &'static str = "WorkspaceList";
        type Type = super::WorkspaceList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for WorkspaceList {}

    impl ListModelImpl for WorkspaceList {
        fn item_type(&self) -> glib::Type {
            super::Item::static_type()
        }

        fn n_items(&self) -> u32 {
            self.items.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            let position: usize = position as usize;
            if position >= self.items.borrow().len() {
                err_out!(
                    "Item out of range {} > {}",
                    position,
                    self.items.borrow().len()
                );
                return None;
            }
            Some(
                self.items.borrow()[position]
                    .upcast_ref::<glib::Object>()
                    .clone(),
            )
        }
    }
}

#[cfg(test)]
mod test {
    use gio::prelude::*;

    use super::super::ws_list_item::Item;
    use super::super::TreeItemType;
    use super::WorkspaceList;

    #[test]
    fn test_ws_list_model() {
        let wl = WorkspaceList::new();

        assert_eq!(wl.n_items(), 0);

        // Adding an item with a 0 id is an error.
        assert!(wl.append(&Item::new()).is_err());
        assert_eq!(wl.n_items(), 0);

        let r = wl.append(&Item::with_values(
            &gio::ThemedIcon::new("dialog-error-symbolic").upcast(),
            "",
            1,
            TreeItemType::Folder,
        ));
        assert!(r.is_ok());
        assert_eq!(wl.n_items(), 1);
    }
}
