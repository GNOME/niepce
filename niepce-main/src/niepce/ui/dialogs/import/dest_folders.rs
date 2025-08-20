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

use npc_engine::catalog::libfolder::FolderVirtualType;
use npc_engine::catalog::{LibFolder, LibraryId};
use npc_engine::importer::{self, DatePathFormat, Importer};
use npc_engine::libraryclient::{ClientInterface, LibraryClient};
use npc_fwk::toolkit::{
    self, Controller, ControllerImplCell, ListViewRow, TreeViewFactory, TreeViewItem,
    TreeViewModel, UiController,
};
use npc_fwk::{Date, base::PathTreeItem, trace_out};

use dest_folder::{DestFolder, FolderId, FolderType};

pub struct DestFile {
    date: Option<Date>,
    dest: PathBuf,
}

impl DestFile {
    pub fn new(date: Option<Date>) -> DestFile {
        DestFile {
            date,
            dest: PathBuf::default(),
        }
    }

    pub fn sort(&mut self, base: &Path, format: DatePathFormat) {
        self.dest = Importer::dest_dir_for_date(base, self.date.as_ref(), format)
    }
}

pub enum DestFoldersIn {
    /// A preview was received. Currently only the date matters.
    PreviewReceived(Option<Date>),
    /// The sorting changed.
    SortingChanged(DatePathFormat),
    /// Root folders have been load (from the library client).
    RootFoldersLoaded(Vec<LibFolder>),
    /// Folders have been load (from the library client).
    FoldersLoaded(Vec<LibFolder>),
    /// Selected Folder change to id.
    SelectionChanged(u32),
    /// Select the item at path
    SelectPath(Option<PathBuf>),
    /// Remove all the sources.
    Clear,
    /// A destination dir for a file has been received
    DestDirFile(PathBuf),
}

pub enum DestFoldersOut {
    SelectedFolder(DestFolder),
    DeselectAll,
}

pub struct DestFolders {
    imp_: ControllerImplCell<DestFoldersIn, DestFoldersOut>,
    tree_model: Rc<TreeViewModel<DestFolder>>,
    listview: gtk4::ListView,
    dest_files: RefCell<Vec<DestFile>>,
    copy_mode: Cell<bool>,
    sorting: Cell<DatePathFormat>,
    /// The base directory for import.
    base: RefCell<PathBuf>,
    client: std::sync::Weak<LibraryClient>,
    /// Counter for temp IDs.
    temp_id: Cell<LibraryId>,
    /// The created temporary folders.
    temp_folders: RefCell<Vec<<DestFolder as PathTreeItem>::Id>>,
}

impl Controller for DestFolders {
    type InMsg = DestFoldersIn;
    type OutMsg = DestFoldersOut;

    npc_fwk::controller_imp_imp!(imp_);

    fn dispatch(&self, e: DestFoldersIn) {
        use DestFoldersIn::*;
        match e {
            RootFoldersLoaded(folders) => {
                trace_out!("Received {:?} root folders", folders.len());
                self.populate_root_folders(&folders);
            }
            FoldersLoaded(folders) => self.populate_folders(&folders),
            PreviewReceived(date) => self.received_source(date),
            SortingChanged(format) => self.sorting_changed(format),
            SelectionChanged(idx) => self.selection_changed(idx),
            SelectPath(path) => self.handle_select_path(path.as_deref()),
            Clear => self.clear_source(),
            DestDirFile(path) => self.add_dest_dir_file(&path),
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
    pub fn new(
        client: Arc<LibraryClient>,
        listview: gtk4::ListView,
        cfg: &Rc<toolkit::Configuration>,
    ) -> Rc<Self> {
        let tree_model = TreeViewModel::<DestFolder>::new();

        let base = cfg
            .value_opt("base_import_dest_dir")
            .map(PathBuf::from)
            .unwrap_or_else(importer::default_import_destdir);

        let ctrl = Rc::new(DestFolders {
            imp_: ControllerImplCell::default(),
            listview,
            tree_model,
            copy_mode: Cell::default(),
            dest_files: RefCell::default(),
            sorting: Cell::default(),
            base: RefCell::new(base),
            client: Arc::downgrade(&client),
            temp_id: Cell::new(0),
            temp_folders: RefCell::default(),
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

    fn add_dest_dir_file(&self, dest_dir: &Path) {
        dest_dir
            .to_str()
            .and_then(|path| self.tree_model.item_index_for_path(path))
            .or_else(|| {
                let base = self.base.borrow();
                let leftovers = dest_dir.strip_prefix(&*base).ok()?;
                let mut current = base.clone();
                for folder in leftovers.components() {
                    current.push(folder);
                    self.add_temp_folder(&current);
                }
                None
            });
    }

    fn sort(&self, base: &Path, sorting: DatePathFormat) {
        for dest_file in self.dest_files.borrow_mut().iter_mut() {
            dest_file.sort(base, sorting);
            // trace_out!("DestFolders: Resorted {:?}", &dest_file.dest);
            self.add_dest_dir_file(&dest_file.dest);
        }
    }

    fn selection_changed(&self, idx: u32) {
        if idx != gtk4::INVALID_LIST_POSITION
            && let Some(folder) = self.tree_model.item(idx)
        {
            let dest = folder.dest();
            self.base.replace(dest.to_path_buf());
            // This invalidate the index.
            if self.copy_mode.get() {
                self.remove_temp_folders();
            }
            self.sort(&dest, self.sorting.get());
            self.emit(DestFoldersOut::SelectedFolder(folder.clone()));
        } else {
            trace_out!("Deselect all");
            self.emit(DestFoldersOut::DeselectAll);
        }
    }

    /// Handle the select path message.
    fn handle_select_path(&self, path: Option<&Path>) {
        trace_out!("handle select path {path:?}");
        let index = self.select_path(path);
        if let Some(path) = path {
            if index.is_none() {
                // Couldn't find path.
                trace_out!("add temp folder {path:?}");
                self.add_temp_folder(path);
                self.select_path(Some(path));
            }
        }
    }

    /// Select the folder by path and return index if found.
    fn select_path(&self, path: Option<&Path>) -> Option<u32> {
        path.and_then(|path| path.to_str())
            .and_then(|path| {
                self.tree_model.item_index_for_path(path).inspect(|&index| {
                    self.listview
                        .scroll_to(index, gtk4::ListScrollFlags::SELECT, None);
                })
            })
            .or_else(|| {
                self.tree_model.unselect_all();
                None
            })
    }

    fn sorting_changed(&self, sorting: DatePathFormat) {
        self.remove_temp_folders();
        self.sorting.set(sorting);
        self.sort(&self.base.borrow(), sorting)
    }

    /// Copy mode change the way the listview behave when copying.
    /// When not copying the list can be clicked.
    pub fn set_copy_mode(&self, copy: bool) {
        self.copy_mode.set(copy);
        self.listview.set_can_target(copy);
    }

    fn received_source(&self, date: Option<Date>) {
        let mut dest_file = DestFile::new(date);
        dest_file.sort(&self.base.borrow(), self.sorting.get());
        self.dest_files.borrow_mut().push(dest_file);
    }

    fn populate_any_folders(&self, folders: &[LibFolder], root: bool) {
        folders
            .iter()
            .filter(|folder| folder.virtual_type() == FolderVirtualType::None)
            .filter_map(|folder| {
                folder.path().map(|path| {
                    DestFolder::new(
                        FolderId::Existing(folder.id()),
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

        self.select_path(Some(&self.base.borrow()));
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
    }

    fn new_temp_id(&self) -> LibraryId {
        let id = self.temp_id.get() + 1;
        self.temp_id.set(id);
        id
    }

    /// Add a temp folder if it doesn't exist. Return `Some(Id)` if it
    /// was added, `None` if it already existed.
    fn add_temp_folder(&self, path: &Path) -> Option<FolderId> {
        if self
            .tree_model
            .item_for_path(&path.to_string_lossy())
            .is_some()
        {
            return None;
        }
        let id = self.new_temp_id();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let dest_folder = DestFolder::new(FolderId::New(id), name, path.into());
        self.tree_model.append(&dest_folder);
        self.temp_folders.borrow_mut().push(FolderId::New(id));
        Some(FolderId::New(id))
    }

    /// Remove the existing temp folders.
    fn remove_temp_folders(&self) {
        self.temp_folders
            .borrow()
            .iter()
            .for_each(|id| self.tree_model.remove(id));
        self.temp_folders.borrow_mut().clear();
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
