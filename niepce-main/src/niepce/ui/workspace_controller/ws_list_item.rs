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

use glib::prelude::*;
use glib::subclass::prelude::*;
use npc_fwk::{gio, glib};

use npc_engine::catalog;
use npc_fwk::err_out;

use super::TreeItemType;
use super::ws_list_model::WorkspaceList;

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

        let imp = item.imp();
        imp.data.replace(imp::ItemData {
            icon: icon.clone(),
            id,
            type_,
        });
        imp.label.replace(label.to_string());
        imp.count.set(0);

        item
    }

    /// Replace the values. This is mostly for placeholders
    pub(super) fn replace_values(&self, icon: &gio::Icon, label: &str, type_: TreeItemType) {
        let id = self.imp().data.borrow().id;
        let new_data = imp::ItemData {
            icon: icon.clone(),
            id,
            type_,
        };
        let imp = self.imp();
        imp.label.replace(label.to_string());
        imp.data.replace(new_data);
    }

    pub(super) fn id(&self) -> catalog::LibraryId {
        self.imp().data.borrow().id
    }

    // `type_` would hide the `glib::Object::type_()` method.
    pub(super) fn tree_item_type(&self) -> TreeItemType {
        self.imp().data.borrow().type_
    }

    pub(super) fn set_count(&self, count: CountUpdate) {
        match count {
            CountUpdate::Set(count) => self.imp().count.set(count),
            CountUpdate::Change(count) => {
                let imp = self.imp();
                let count = imp.count.get() + count;
                imp.count.set(count);
            }
        }
        self.notify("count");
    }

    pub fn icon(&self) -> gio::Icon {
        self.imp().data.borrow().icon.clone()
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
    /// Return its position. If it already exist (same id) replace it.
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
    use std::cell::{Cell, OnceCell, RefCell};

    use gio::subclass::prelude::*;
    use glib::prelude::*;
    use npc_fwk::{gio, glib};

    use super::super::TreeItemType;
    use super::super::ws_list_model::WorkspaceList;
    use npc_engine::catalog;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::Item)]
    pub struct Item {
        pub(super) data: RefCell<ItemData>,
        #[property(get, set, default_value = "")]
        pub(super) label: RefCell<String>,
        #[property(get, default_value = 0)]
        pub(super) count: Cell<i32>,
        pub(super) children: OnceCell<WorkspaceList>,
    }

    pub(super) struct ItemData {
        pub(super) icon: gio::Icon,
        pub(super) id: catalog::LibraryId,
        pub(super) type_: TreeItemType,
    }

    impl Default for ItemData {
        fn default() -> Self {
            Self {
                icon: gio::ThemedIcon::new("dialog-error-symbolic").upcast(),
                id: 0,
                type_: TreeItemType::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Item {
        const NAME: &'static str = "WorkspaceListItem";
        type Type = super::Item;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Item {}
}
