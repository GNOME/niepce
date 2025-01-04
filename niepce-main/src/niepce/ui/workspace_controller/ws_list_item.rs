/*
 * niepce - niepce/ui/workspace_controller/ws_list_item.rs
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
use npc_fwk::{gio, glib};

use npc_engine::catalog;
use npc_fwk::err_out;

use super::ws_list_model::WorkspaceList;
use super::TreeItemType;

/// Count update operation
pub(super) enum CountUpdate {
    /// Set absolute value.
    Set(i32),
    /// Change value but offset.
    Change(i32),
}

glib::wrapper! {
    /// Item in the workspace
    pub struct Item(
        ObjectSubclass<imp::Item>);
}

impl Default for Item {
    fn default() -> Item {
        Self::new()
    }
}

impl Item {
    pub(super) fn new() -> Item {
        glib::Object::new()
    }

    pub(super) fn with_values(
        icon: &gio::Icon,
        label: &str,
        id: catalog::LibraryId,
        type_: TreeItemType,
    ) -> Item {
        let item = Self::new();

        item.imp().data.replace(imp::ItemData {
            icon: icon.clone(),
            label: label.to_string(),
            id,
            type_,
            count: 0,
        });

        item
    }

    /// Replace the values. This is mostly for placeholders
    pub(super) fn replace_values(&self, icon: &gio::Icon, label: &str, type_: TreeItemType) {
        let id = self.imp().data.borrow().id;
        let count = self.imp().data.borrow().count;
        let new_data = imp::ItemData {
            icon: icon.clone(),
            label: label.to_string(),
            id,
            type_,
            count,
        };
        self.imp().data.replace(new_data);
    }

    pub(super) fn id(&self) -> catalog::LibraryId {
        self.imp().data.borrow().id
    }

    // `type_` would hide the `glib::Object::type_()` method.
    pub(super) fn tree_item_type(&self) -> TreeItemType {
        self.imp().data.borrow().type_
    }

    pub fn count(&self) -> i32 {
        self.imp().data.borrow().count
    }

    pub(super) fn set_count(&self, count: CountUpdate) {
        match count {
            CountUpdate::Set(count) => self.imp().data.borrow_mut().count = count,
            CountUpdate::Change(count) => self.imp().data.borrow_mut().count += count,
        }
    }

    pub fn icon(&self) -> gio::Icon {
        self.imp().data.borrow().icon.clone()
    }

    pub fn label(&self) -> String {
        self.imp().data.borrow().label.to_string()
    }

    pub fn set_label(&self, label: &str) {
        self.imp().data.borrow_mut().label = label.to_string()
    }

    pub fn children(&self) -> Option<&WorkspaceList> {
        self.imp().children.get()
    }

    pub fn create_children(&self) -> Option<&WorkspaceList> {
        match self.tree_item_type() {
            TreeItemType::Trash | TreeItemType::Album => return None,
            _ => {}
        }
        Some(self.imp().children.get_or_init(WorkspaceList::new))
    }

    /// Add item to the children.
    /// Return its position. If it already exist (same id) repace it.
    pub fn add_item(&self, item: Item) -> Option<u32> {
        self.create_children().and_then(|children| {
            children.append(item).ok().or_else(|| {
                err_out!("Coudln't add item");
                None
            })
        })
    }
}

mod imp {
    use std::cell::RefCell;

    use gio::subclass::prelude::*;
    use glib::prelude::*;
    use npc_fwk::{gio, glib};
    use once_cell::unsync::OnceCell;

    use super::super::ws_list_model::WorkspaceList;
    use super::super::TreeItemType;
    use npc_engine::catalog;

    #[derive(Default)]
    pub struct Item {
        pub(super) data: RefCell<ItemData>,
        pub(super) children: OnceCell<WorkspaceList>,
    }

    pub(super) struct ItemData {
        pub(super) icon: gio::Icon,
        pub(super) id: catalog::LibraryId,
        pub(super) label: String,
        pub(super) type_: TreeItemType,
        pub(super) count: i32,
    }

    impl Default for ItemData {
        fn default() -> Self {
            Self {
                icon: gio::ThemedIcon::new("dialog-error-symbolic").upcast(),
                id: 0,
                label: String::default(),
                type_: TreeItemType::default(),
                count: 0,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Item {
        const NAME: &'static str = "WorkspaceListItem";
        type Type = super::Item;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Item {}
}
