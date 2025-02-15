/*
 * niepce - niepce/ui/dialogs/import/dest_folders.rs
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

mod dest_folder;

use std::cell::{Cell, RefCell, RefMut};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use glib::subclass::prelude::*;
use gtk4::prelude::*;
use npc_fwk::{glib, gtk4};

use npc_engine::catalog::LibFolder;
use npc_engine::catalog::libfolder::FolderVirtualType;
use npc_engine::importer::{DatePathFormat, Importer};
use npc_engine::libraryclient::{ClientInterface, LibraryClient};
use npc_fwk::toolkit::{
    Controller, ControllerImplCell, ListViewRow, TreeViewFactory, TreeViewItem, TreeViewModel,
    UiController,
};
use npc_fwk::{Date, dbg_out};

use dest_folder::{DestFolder, FolderType};

pub struct DestFile {
    source: String,
    date: Option<Date>,
    dest: PathBuf,
}

impl DestFile {
    pub fn new(source: String, date: Option<Date>) -> DestFile {
        DestFile {
            source,
            date,
            dest: PathBuf::default(),
        }
    }

    pub fn sort(&mut self, base: &Path, format: DatePathFormat) {
        if let Some(ref date) = self.date {
            self.dest = Importer::dest_dir_for_date(base, date, format)
        }
    }
}

pub enum DestFoldersIn {
    /// A preview was received.
    PreviewReceived(String, Option<Date>),
    /// The sorting changed.
    SortingChanged(DatePathFormat),
    /// Root folders have been load (from the library client).
    RootFoldersLoaded(Vec<LibFolder>),
    /// Folders have been load (from the library client).
    FoldersLoaded(Vec<LibFolder>),
    /// Selected Folder change to id.
    SelectionChanged(u32),
    /// Remove all the sources.
    Clear,
}

pub enum DestFoldersOut {
    SelectedFolder(u32),
    DeselectAll,
}

pub struct DestFolders {
    imp_: ControllerImplCell<DestFoldersIn, DestFoldersOut>,
    tree_model: Rc<TreeViewModel<DestFolder>>,
    listview: gtk4::ListView,
    dest_files: RefCell<Vec<DestFile>>,
    sorting: Cell<DatePathFormat>,
    /// The base directory for import.
    base: RefCell<PathBuf>,
    client: std::sync::Weak<LibraryClient>,
}

impl Controller for DestFolders {
    type InMsg = DestFoldersIn;
    type OutMsg = DestFoldersOut;

    npc_fwk::controller_imp_imp!(imp_);

    fn dispatch(&self, e: DestFoldersIn) {
        use DestFoldersIn::*;
        match e {
            RootFoldersLoaded(folders) => {
                dbg_out!("Received {:?} root folders", folders.len());
                self.populate_root_folders(&folders);
            }
            FoldersLoaded(folders) => self.populate_folders(&folders),
            PreviewReceived(source, date) => self.received_source(source, date),
            SortingChanged(format) => self.sorting_changed(format),
            SelectionChanged(idx) => self.selection_changed(idx),
            Clear => self.clear_source(),
        }
    }
}

impl UiController for DestFolders {
    fn widget(&self) -> &gtk4::Widget {
        self.listview.upcast_ref()
    }
}

impl TreeViewFactory<DestFolder> for DestFolders {
    type Widget = DestFolderRow;

    fn setup(&self) -> Self::Widget {
        DestFolderRow::new()
    }
}

impl DestFolders {
    pub fn new(client: Arc<LibraryClient>, listview: gtk4::ListView) -> Rc<Self> {
        let tree_model = TreeViewModel::<DestFolder>::new();

        let ctrl = Rc::new(DestFolders {
            imp_: ControllerImplCell::default(),
            listview,
            tree_model,
            dest_files: RefCell::default(),
            sorting: Cell::default(),
            base: RefCell::new(PathBuf::from("~/Pictures")),
            client: Arc::downgrade(&client),
        });

        ctrl.tree_model.bind(&ctrl.listview, &ctrl);

        <Self as Controller>::start(&ctrl);

        if let Some(selection_model) = ctrl.listview.model() {
            let sender = ctrl.sender();
            selection_model.connect_selection_changed(move |model, _, _| {
                let idx = model
                    .downcast_ref::<gtk4::SingleSelection>()
                    .unwrap()
                    .selected();
                npc_fwk::send_async_any!(DestFoldersIn::SelectionChanged(idx), sender);
            });
        }
        let sender = ctrl.sender();
        client.get_root_folders(Box::new(move |list| {
            npc_fwk::send_async_any!(DestFoldersIn::RootFoldersLoaded(list), sender);
        }));

        ctrl
    }

    fn clear_source(&self) {
        self.dest_files.borrow_mut().clear();
    }

    fn sort(&self, base: &Path, sorting: DatePathFormat) {
        for dest_file in self.dest_files.borrow_mut().iter_mut() {
            dest_file.sort(base, sorting);
            dbg_out!("DestFolders: Resorted {:?}", &dest_file.dest);
        }
    }

    fn selection_changed(&self, idx: u32) {
        if let Some(folder) = self.tree_model.item(idx) {
            let dest = folder.dest();
            self.base.replace(dest.to_path_buf());
            self.sort(&dest, self.sorting.get());
            self.emit(DestFoldersOut::SelectedFolder(idx));
        } else {
            self.emit(DestFoldersOut::DeselectAll);
        }
    }

    fn sorting_changed(&self, sorting: DatePathFormat) {
        self.sorting.set(sorting);
        self.sort(&self.base.borrow(), sorting)
    }

    /// Copy mode change the way the listview behave when copying.
    /// When not copying the list can be clicked.
    pub fn set_copy_mode(&self, copy: bool) {
        self.listview.set_can_target(copy);
    }

    fn received_source(&self, source: String, date: Option<Date>) {
        let mut dest_file = DestFile::new(source, date);
        dest_file.sort(&self.base.borrow(), self.sorting.get());
        dbg_out!("DestFolders: Added {:?}", &dest_file.dest);
        self.dest_files.borrow_mut().push(dest_file);
    }

    fn populate_any_folders(&self, folders: &[LibFolder], root: bool) {
        folders
            .iter()
            .filter(|folder| folder.virtual_type() == FolderVirtualType::None)
            .filter_map(|folder| {
                folder.path().map(|path| {
                    DestFolder::new(
                        folder.id(),
                        FolderType::Existing,
                        folder.name().to_string(),
                        path.into(),
                    )
                })
            })
            .for_each(|item| {
                if root {
                    self.tree_model.append_root(&item);
                } else {
                    self.tree_model.append(&item);
                }
            });
    }

    /// Populate the folders.
    fn populate_folders(&self, folders: &[LibFolder]) {
        self.populate_any_folders(folders, false);
    }

    /// Populate the root folders.
    fn populate_root_folders(&self, folders: &[LibFolder]) {
        self.populate_any_folders(folders, true);

        let sender = self.sender();
        if let Some(client) = self.client.upgrade() {
            client.get_all_folders(Some(Box::new(move |list| {
                npc_fwk::send_async_any!(DestFoldersIn::FoldersLoaded(list), sender);
            })));
        }

        // XXX Select the save base. Or the first root if none or not found.
        self.selection_changed(0);
    }

    /// Return the `DestFolder` as the idx as per the list model.
    pub fn folder_at(&self, idx: u32) -> Option<DestFolder> {
        self.tree_model.item(idx).clone()
    }
}

glib::wrapper! {
    pub struct DestFolderRow(ObjectSubclass<imp::DestFolderRow>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for DestFolderRow {
    fn default() -> Self {
        Self::new()
    }
}

impl DestFolderRow {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_type(&self, type_: FolderType) {
        self.imp().icon.set_icon_name(match type_ {
            FolderType::Existing => Some("folder-symbolic"),
            FolderType::New => Some("folder-new-symbolic"),
        });
    }

    pub fn set_label(&self, label: &str) {
        self.imp().label.set_label(label);
    }
}

impl ListViewRow<DestFolder> for DestFolderRow {
    fn bind(&self, item: &DestFolder, tree_list_row: Option<&gtk4::TreeListRow>) {
        self.bind_to(&self.imp().label, "label", item, "name");
        self.set_type(item.folder_type());
        let expander = &self.imp().expander;
        expander.set_list_row(tree_list_row);
        item.set_autohide_expander(expander);
    }

    fn unbind(&self) {
        let expander = &self.imp().expander;
        expander.set_hide_expander(false);
        expander.set_list_row(None);
        self.clear_bindings();
    }

    fn bindings_mut(&self) -> RefMut<'_, Vec<glib::Binding>> {
        self.imp().bindings.borrow_mut()
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use npc_fwk::{glib, gtk4};

    #[derive(Default)]
    pub struct DestFolderRow {
        pub(super) expander: gtk4::TreeExpander,
        pub(super) icon: gtk4::Image,
        pub(super) label: gtk4::Label,
        pub(super) bindings: RefCell<Vec<glib::Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DestFolderRow {
        const NAME: &'static str = "DestFolderRow";
        type ParentType = gtk4::Box;
        type Type = super::DestFolderRow;
    }

    impl ObjectImpl for DestFolderRow {
        fn constructed(&self) {
            self.parent_constructed();

            let box_ = &self.obj();
            self.expander.set_hexpand(true);
            self.expander.set_indent_for_icon(true);
            box_.append(&self.expander);
            let inner = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
            self.expander.set_child(Some(&inner));
            inner.set_hexpand(true);
            inner.set_vexpand(true);
            self.icon.set_margin_start(4);
            self.icon.set_margin_end(4);
            inner.append(&self.icon);
            self.label.set_hexpand(true);
            self.label.set_xalign(0.0);
            inner.append(&self.label);
        }
    }

    impl WidgetImpl for DestFolderRow {}
    impl BoxImpl for DestFolderRow {}
}
