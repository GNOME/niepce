/*
 * niepce - niepce/ui/workspace_controller/ws_list_item.rs
 *
 * Copyright (C) 2022 Hubert Figuière
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

use npc_engine::db;
use npc_fwk::dbg_out;

use super::ws_list_model::WorkspaceList;
use super::TreeItemType;

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
        glib::Object::new(&[])
    }

    pub(super) fn with_values(
        icon: &gio::Icon,
        label: &str,
        id: db::LibraryId,
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

    pub(super) fn id(&self) -> db::LibraryId {
        self.imp().data.borrow().id
    }

    // `type_` would hide the `glib::Object::type_()` method.
    pub(super) fn tree_item_type(&self) -> TreeItemType {
        self.imp().data.borrow().type_
    }

    pub fn count(&self) -> i32 {
        self.imp().data.borrow().count
    }

    pub fn set_count(&self, count: i32) {
        self.imp().data.borrow_mut().count = count
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
        dbg_out!("children called");
        self.imp().children.get()
    }

    pub fn create_children(&self) -> Option<&WorkspaceList> {
        match self.tree_item_type() {
            TreeItemType::Trash | TreeItemType::Album => return None,
            _ => {}
        }
        Some(self.imp().children.get_or_init(WorkspaceList::new))
    }
}

mod imp {
    use std::cell::RefCell;

    use gio::subclass::prelude::*;
    use glib::prelude::*;
    use once_cell::unsync::OnceCell;

    use super::super::ws_list_model::WorkspaceList;
    use super::super::TreeItemType;
    use npc_engine::db;

    #[derive(Default)]
    pub struct Item {
        pub(super) data: RefCell<ItemData>,
        pub(super) children: OnceCell<WorkspaceList>,
    }

    pub(super) struct ItemData {
        pub(super) icon: gio::Icon,
        pub(super) id: db::LibraryId,
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
