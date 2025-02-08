/*
 * niepce - niepce/ui/dialogs/import/dest_folders/dest_folder.rs
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

use std::cell::Ref;
use std::path::PathBuf;

use glib::subclass::prelude::*;
use npc_fwk::{gio, glib};

use npc_engine::catalog::LibraryId;
use npc_fwk::base::PathTreeItem;
use npc_fwk::toolkit::tree_view_model::TreeViewItem;

/// Type of folder
#[derive(Clone, Copy, Default)]
pub enum FolderType {
    /// The folder exists in the catalog.
    #[default]
    Existing,
    /// The folder will be created by the import.
    New,
}

glib::wrapper! {
    pub struct DestFolder(ObjectSubclass<imp::DestFolder>);
}

impl PathTreeItem for DestFolder {
    type Id = LibraryId;

    fn path(&self) -> String {
        self.dest().to_string_lossy().into()
    }
    fn id(&self) -> Self::Id {
        self.imp().id.get()
    }
}

impl TreeViewItem for DestFolder {
    fn children(&self) -> Option<gio::ListStore> {
        Some(self.imp().children.clone())
    }
}

impl DestFolder {
    pub fn new(
        id: <Self as PathTreeItem>::Id,
        folder_type: FolderType,
        name: String,
        folder: PathBuf,
    ) -> DestFolder {
        let dest_folder = glib::Object::new::<DestFolder>();

        let imp = dest_folder.imp();
        imp.id.set(id);
        imp.folder_type.set(folder_type);
        imp.name.replace(name);
        imp.dest.replace(folder);
        dest_folder
    }

    pub fn folder_type(&self) -> FolderType {
        self.imp().folder_type.get()
    }

    pub fn dest(&self) -> Ref<'_, PathBuf> {
        self.imp().dest.borrow()
    }
}

mod imp {
    use std::cell::{Cell, RefCell};
    use std::path::PathBuf;

    use glib::prelude::*;
    use glib::subclass::prelude::*;
    use npc_fwk::base::PathTreeItem;
    use npc_fwk::{gio, glib};

    use super::FolderType;

    #[derive(glib::Properties)]
    #[properties(wrapper_type = super::DestFolder)]
    pub struct DestFolder {
        pub(super) id: Cell<<super::DestFolder as PathTreeItem>::Id>,
        pub(super) folder_type: Cell<FolderType>,
        pub(super) dest: RefCell<PathBuf>,
        #[property(get, default_value = "")]
        pub(super) name: RefCell<String>,
        pub(super) children: gio::ListStore,
    }

    impl Default for DestFolder {
        fn default() -> Self {
            let children = gio::ListStore::new::<super::DestFolder>();
            DestFolder {
                children,
                id: Cell::default(),
                folder_type: Cell::default(),
                dest: RefCell::default(),
                name: RefCell::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DestFolder {
        const NAME: &'static str = "DestFolder";
        type Type = super::DestFolder;
        type ParentType = glib::Object;
    }

    #[glib::derived_properties]
    impl ObjectImpl for DestFolder {}
}
