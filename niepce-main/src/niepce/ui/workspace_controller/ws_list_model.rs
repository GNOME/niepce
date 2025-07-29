/*
 * niepce - niepce/ui/workspace_controller/ws_list_model.rs
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

use glib::subclass::prelude::*;
use gtk4::prelude::*;
use npc_fwk::{gio, glib, gtk4};
use thiserror::Error;

use super::ws_list_item::{CountUpdate, Item};
use npc_engine::catalog;

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

pub(super) struct ItemPos {
    pub model: WorkspaceList,
    pub pos: u32,
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
    /// Replace the item if it exists. (same id).
    /// Return the index of the item (pos).
    pub fn append(&self, item: Item) -> Result<u32, Error> {
        // this scopes the borrow_mut() as items_changed() will call n_items()
        if item.id() == 0 {
            return Err(Error::InvalidId);
        }
        let mut removed = 0_u32;
        let pos = {
            let mut items = self.imp().items.borrow_mut();
            let mut pos = items.len();
            let id = item.id();
            if items.push((id, item)).is_some() {
                // The item already exist. Get its position.
                // If the item is not found, then it's a bug.
                pos = items.index_of(&id).unwrap();
                removed = 1;
            }

            pos as u32
        };
        self.items_changed(pos, removed, 1);
        Ok(pos)
    }

    /// Remove the item at `position`. Return it if found or an error.
    pub fn remove(&self, position: u32) -> Result<Item, Error> {
        let r = self
            .imp()
            .items
            .borrow_mut()
            .remove_at(position as usize)
            .ok_or(Error::NotFound);
        if r.is_ok() {
            self.items_changed(position, 1, 0);
        }

        r
    }

    /// Remove the item with `id`. Return it if found or an error.
    pub fn remove_by_id(&self, id: catalog::LibraryId) -> Result<Item, Error> {
        let items = &self.imp().items;
        if items.borrow().contains_key(&id) {
            let index = items.borrow().index_of(&id).ok_or(Error::NotFound)?;
            self.remove(index as u32)
        } else {
            items
                .borrow()
                .iter()
                .find_map(|(_, item)| {
                    item.children()
                        .and_then(|children| children.remove_by_id(id).ok())
                })
                .ok_or(Error::NotFound)
        }
    }

    /// Get item with `id`
    /// Will recurse into the tree.
    pub fn item_by_id(&self, id: catalog::LibraryId) -> Option<Item> {
        let items = self.imp().items.borrow();
        items.get(&id).cloned().or_else(|| {
            items
                .iter()
                .find_map(|(_, item)| item.children().and_then(|children| children.item_by_id(id)))
        })
    }

    /// Return the position of item with `id`.
    /// Currently tied to `IndexedMap::index_of` which is slow.
    pub(super) fn pos_by_id(&self, id: catalog::LibraryId) -> Option<ItemPos> {
        let items = self.imp().items.borrow();
        items
            .index_of(&id)
            .map(|idx| ItemPos {
                model: self.clone(),
                pos: idx as u32,
            })
            .or_else(|| {
                items.iter().find_map(|(_, item)| {
                    item.children().and_then(|children| children.pos_by_id(id))
                })
            })
    }

    pub(super) fn set_count_by_id(&self, id: catalog::LibraryId, count: CountUpdate) {
        if let Some(item_pos) = self.pos_by_id(id) {
            if let Some(item) = item_pos
                .model
                .item(item_pos.pos)
                .and_then(|item| item.downcast::<Item>().ok())
            {
                item.set_count(count);
                item_pos.model.items_changed(item_pos.pos, 0, 0);
            }
        }
    }

    pub(super) fn rename_by_id(&self, id: catalog::LibraryId, name: &str) {
        if let Some(item_pos) = self.pos_by_id(id) {
            if let Some(item) = item_pos
                .model
                .item(item_pos.pos)
                .and_then(|item| item.downcast::<Item>().ok())
            {
                item.set_label(name);
                item_pos.model.items_changed(item_pos.pos, 0, 0);
            }
        }
    }
}

mod imp {
    use std::cell::RefCell;

    use gio::subclass::prelude::*;
    use glib::prelude::*;
    use npc_fwk::{gio, glib};

    use npc_engine::catalog;
    use npc_fwk::base::IndexedMap;
    use npc_fwk::err_out;

    #[derive(Default)]
    pub struct WorkspaceList {
        /// The ordered items `id`
        pub(super) items: RefCell<IndexedMap<catalog::LibraryId, super::Item>>,
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
    use npc_fwk::gio;

    use super::super::TreeItemType;
    use super::super::ws_list_item::Item;
    use super::WorkspaceList;

    #[test]
    fn test_ws_list_model() {
        let wl = WorkspaceList::new();

        assert_eq!(wl.n_items(), 0);

        // Adding an item with a 0 id is an error.
        assert!(wl.append(Item::new()).is_err());
        assert_eq!(wl.n_items(), 0);

        let r = wl.append(Item::with_values(
            &gio::ThemedIcon::new("dialog-error-symbolic").upcast(),
            "Top",
            1,
            TreeItemType::Folder,
        ));
        assert!(r.is_ok());
        assert_eq!(wl.n_items(), 1);

        let item = wl.item_by_id(1);
        assert!(item.is_some());

        let top_item = item.unwrap();
        let child = Item::with_values(
            &gio::ThemedIcon::new("dialog-error-symbolic").upcast(),
            "Child1",
            2,
            TreeItemType::Folder,
        );
        top_item.add_item(child);
        let child = Item::with_values(
            &gio::ThemedIcon::new("dialog-error-symbolic").upcast(),
            "Child2",
            3,
            TreeItemType::Folder,
        );
        let idx3 = top_item.add_item(child);
        assert!(idx3.is_some());

        // We have 2 children
        assert_eq!(top_item.children().unwrap().n_items(), 2);

        let item = wl.item_by_id(1);
        assert!(item.is_some());
        let item = item.unwrap();
        assert_eq!(item.label(), "Top");
        let item = wl.item_by_id(2);
        assert!(item.is_some());
        let item = item.unwrap();
        assert_eq!(item.label(), "Child1");
        let item = wl.item_by_id(3);
        assert!(item.is_some());
        let item = item.unwrap();
        assert_eq!(item.label(), "Child2");

        let child = Item::with_values(
            &gio::ThemedIcon::new("dialog-error-symbolic").upcast(),
            "Child3",
            3,
            TreeItemType::Folder,
        );
        let r = top_item.add_item(child);
        assert_eq!(r, idx3);
        // Item 3 is now "Child3"
        let item = wl.item_by_id(3);
        assert!(item.is_some());
        let item = item.unwrap();
        assert_eq!(item.label(), "Child3");

        // We still have 2 children
        assert_eq!(top_item.children().unwrap().n_items(), 2);

        let r = wl.remove_by_id(2);
        assert!(r.is_ok());

        // We have 1 child
        assert_eq!(top_item.children().unwrap().n_items(), 1);
        let item = wl.item_by_id(2);
        assert!(item.is_none());
    }
}
