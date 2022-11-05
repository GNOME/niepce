/*
 * niepce - niepce/ui/workspace_controller.rs
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

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Weak};

use gettextrs::gettext;
use glib::translate::*;
use glib::Cast;
use gtk4::gio;
use gtk4::glib;
use gtk4::glib::Type;
use gtk4::prelude::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use once_cell::unsync::OnceCell;

use crate::import::ImportRequest;
use crate::libraryclient::{ClientInterface, LibraryClient};
use crate::LibraryClientWrapper;
use npc_engine::db;
use npc_engine::library::notification::LibNotification;
use npc_fwk::base::Signal;
use npc_fwk::toolkit::{self, Controller, ControllerImpl, UiController};
use npc_fwk::{dbg_out, err_out, on_err_out};

#[derive(Debug, FromPrimitive, PartialEq)]
#[repr(i32)]
enum TreeItemType {
    Folders,
    Projects,
    Keywords,
    Folder,
    Project,
    Keyword,
}

#[repr(i32)]
/// Columns indices
enum Columns {
    Icon = 0,
    Id = 1,
    Label = 2,
    Type = 3,
    Count = 4,
    CountN = 5,
}

enum Event {
    ButtonPress(f64, f64),
    SelectionChanged,
    RowExpanded(gtk4::TreeIter, gtk4::TreePath),
    RowCollapsed(gtk4::TreeIter, gtk4::TreePath),
    NewFolder,
    DeleteFolder,
    /// Initiate the import
    Import,
    /// Sent after the import is initiated
    PerformImport(Box<ImportRequest>),
}

pub struct ImportDialogArgument {
    dialog: cxx::SharedPtr<crate::ffi::ImportDialog>,
    tx: glib::Sender<Event>,
}

pub struct WorkspaceController {
    imp_: RefCell<ControllerImpl>,
    tx: glib::Sender<Event>,
    cfg: Rc<toolkit::Configuration>,
    widgets: OnceCell<Widgets>,
    client: Weak<LibraryClient>,
    action_group: OnceCell<gio::ActionGroup>,
    pub selection_changed: Signal<()>,

    icon_trash: gio::Icon,
    icon_roll: gio::Icon,
    icon_folder: gio::Icon,
    icon_project: gio::Icon,
    icon_keyword: gio::Icon,
}

struct Widgets {
    widget_: gtk4::Widget,
    treestore: gtk4::TreeStore,
    librarytree: gtk4::TreeView,
    context_menu: gtk4::PopoverMenu,

    folderidmap: RefCell<HashMap<db::LibraryId, gtk4::TreeIter>>,
    keywordsidmap: RefCell<HashMap<db::LibraryId, gtk4::TreeIter>>,
    folder_node: gtk4::TreeIter,
    project_node: gtk4::TreeIter,
    keywords_node: gtk4::TreeIter,
    cfg: Rc<toolkit::Configuration>,
}

impl Widgets {
    fn add_folder_item(&self, folder: &db::LibFolder, icon: &gio::Icon) -> gtk4::TreeIter {
        let was_empty = self
            .treestore
            .iter_children(Some(&self.folder_node))
            .is_none();
        let iter = WorkspaceController::add_item(
            &self.treestore,
            Some(&self.folder_node),
            icon,
            folder.name(),
            folder.id(),
            TreeItemType::Folder,
        );
        if was_empty {
            self.expand_from_cfg("workspace_folders_expanded", &self.folder_node);
        }
        self.folderidmap.borrow_mut().insert(folder.id(), iter);

        iter
    }

    fn remove_folder_item(&self, id: db::LibraryId) {
        if let Some(iter) = self.folderidmap.borrow().get(&id) {
            self.treestore.remove(iter);
            self.folderidmap.borrow_mut().remove(&id);
        }
    }

    fn add_keyword_item(&self, keyword: &db::Keyword, icon: &gio::Icon) {
        let was_empty = self
            .treestore
            .iter_children(Some(&self.keywords_node))
            .is_none();
        let iter = WorkspaceController::add_item(
            &self.treestore,
            Some(&self.keywords_node),
            icon,
            keyword.keyword(),
            keyword.id(),
            TreeItemType::Keyword,
        );
        if was_empty {
            self.expand_from_cfg("workspace_keywords_expanded", &self.keywords_node);
        }
        self.keywordsidmap.borrow_mut().insert(keyword.id(), iter);
    }

    fn expand_from_cfg(&self, key: &str, node: &gtk4::TreeIter) {
        let expanded = self.cfg.value(key, "true");
        dbg_out!("expand from cfg {} - {}", key, &expanded);
        if expanded == "true" {
            self.librarytree
                .expand_row(&self.treestore.path(node), false);
        }
    }

    fn increase_count(&self, at: &gtk4::TreeIter, count: i32) {
        let new_count: i32 = self.treestore.get::<i32>(at, Columns::CountN as i32) + count as i32;
        let count_string = new_count.to_string();
        self.treestore.set(
            at,
            &[
                (Columns::CountN as u32, &new_count),
                (Columns::Count as u32, &count_string),
            ],
        );
    }

    fn set_count(&self, at: &gtk4::TreeIter, count: i32) {
        let count_string = count.to_string();
        self.treestore.set(
            at,
            &[
                (Columns::CountN as u32, &count),
                (Columns::Count as u32, &count_string),
            ],
        );
    }

    fn expand_row(&self, at: &gtk4::TreeIter, open_all: bool) -> bool {
        let path = self.treestore.path(at);
        self.librarytree.expand_row(&path, open_all)
    }
}

impl Controller for WorkspaceController {
    fn imp(&self) -> Ref<'_, ControllerImpl> {
        self.imp_.borrow()
    }

    fn imp_mut(&self) -> RefMut<'_, ControllerImpl> {
        self.imp_.borrow_mut()
    }

    fn on_ready(&self) {}
}

impl UiController for WorkspaceController {
    fn widget(&self) -> &gtk4::Widget {
        &self
            .widgets
            .get_or_init(|| {
                let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

                let treestore = gtk4::TreeStore::new(&[
                    // icon
                    gio::Icon::static_type(),
                    // id
                    Type::I64,
                    // label
                    Type::STRING,
                    // type
                    Type::I32,
                    // count (string)
                    Type::STRING,
                    // count (int)
                    Type::I32,
                ]);
                let librarytree = gtk4::TreeView::with_model(&treestore);
                librarytree.set_activate_on_single_click(true);

                let folder_node = Self::add_item(
                    &treestore,
                    None,
                    &self.icon_folder,
                    &gettext("Pictures"),
                    0,
                    TreeItemType::Folders,
                );
                let project_node = Self::add_item(
                    &treestore,
                    None,
                    &self.icon_project,
                    &gettext("Projects"),
                    0,
                    TreeItemType::Projects,
                );
                let keywords_node = Self::add_item(
                    &treestore,
                    None,
                    &self.icon_keyword,
                    &gettext("Keywords"),
                    0,
                    TreeItemType::Keywords,
                );
                librarytree.set_headers_visible(false);

                librarytree.insert_column_with_attributes(
                    -1,
                    "",
                    &gtk4::CellRendererPixbuf::new(),
                    &[("gicon", Columns::Icon as i32)],
                );
                librarytree.insert_column_with_attributes(
                    -1,
                    "",
                    &gtk4::CellRendererText::new(),
                    &[("text", Columns::Label as i32)],
                );
                librarytree.insert_column_with_attributes(
                    -1,
                    "",
                    &gtk4::CellRendererText::new(),
                    &[("text", Columns::Count as i32)],
                );

                let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
                // header.set_margin(4);
                let label = gtk4::Label::with_mnemonic(&gettext("_Workspace"));
                label.set_mnemonic_widget(Some(&librarytree));
                label.set_hexpand(true);
                header.append(&label);

                let add_btn = gtk4::builders::MenuButtonBuilder::new()
                    .direction(gtk4::ArrowType::None)
                    .icon_name("view-more-symbolic")
                    .build();

                // Menu
                let menu = gio::Menu::new();
                let section = gio::Menu::new();
                menu.append_section(None, &section);
                section.append(Some(&gettext("New Folder...")), Some("workspace.NewFolder"));
                section.append(
                    Some(&gettext("New Project...")),
                    Some("workspace.NewProject"),
                );
                section.append(
                    Some(&gettext("Delete Folder")),
                    Some("workspace.DeleteFolder"),
                );

                let section = gio::Menu::new();
                menu.append_section(None, &section);
                section.append(Some(&gettext("Import...")), Some("workspace.Import"));

                add_btn.set_menu_model(Some(&menu));

                let context_menu = gtk4::builders::PopoverMenuBuilder::new()
                    .menu_model(&menu)
                    .has_arrow(false)
                    .build();
                context_menu.set_parent(&librarytree);
                librarytree.connect_unrealize(glib::clone!(@strong context_menu => move |_| {
                    context_menu.unparent();
                }));
                header.append(&add_btn);
                main_box.append(&header);

                let scrolled = gtk4::ScrolledWindow::new();
                librarytree.set_vexpand(true);
                scrolled.set_child(Some(&librarytree));
                main_box.append(&scrolled);

                // connect signals
                librarytree.selection().connect_changed(
                    glib::clone!(@strong self.tx as tx => move |_| {
                        on_err_out!(tx.send(Event::SelectionChanged));
                    }),
                );
                librarytree.connect_row_expanded(
                    glib::clone!(@strong self.tx as tx => move |_, iter, path| {
                        on_err_out!(tx.send(Event::RowExpanded(*iter, path.clone())));
                    }),
                );
                librarytree.connect_row_collapsed(
                    glib::clone!(@strong self.tx as tx => move |_, iter, path| {
                        on_err_out!(tx.send(Event::RowCollapsed(*iter, path.clone())));
                    }),
                );
                let gesture = gtk4::GestureClick::new();
                gesture.set_button(3);
                librarytree.add_controller(&gesture);
                gesture.connect_pressed(glib::clone!(@strong self.tx as tx => move |_, _, x, y| {
                    on_err_out!(tx.send(Event::ButtonPress(x, y)));
                }));

                Widgets {
                    widget_: main_box.upcast(),
                    librarytree,
                    treestore,
                    context_menu,
                    folderidmap: RefCell::new(HashMap::new()),
                    keywordsidmap: RefCell::new(HashMap::new()),
                    project_node,
                    folder_node,
                    keywords_node,
                    cfg: self.cfg.clone(),
                }
            })
            .widget_
    }

    fn actions(&self) -> Option<(&str, &gio::ActionGroup)> {
        Some((
            "workspace",
            (self.action_group.get_or_init(|| {
                let group = gio::SimpleActionGroup::new();
                let tx = self.tx.clone();

                action!(group, "NewProject", move |_, _| {});
                action!(
                    group,
                    "NewFolder",
                    glib::clone!(@strong tx => move |_, _| {
                        on_err_out!(tx.send(Event::NewFolder));
                    })
                );
                action!(
                    group,
                    "Import",
                    glib::clone!(@strong tx => move |_, _| {
                        on_err_out!(tx.send(Event::Import));
                    })
                );
                action!(
                    group,
                    "DeleteFolder",
                    glib::clone!(@strong tx => move |_, _| {
                        on_err_out!(tx.send(Event::DeleteFolder));
                    })
                );

                group.upcast()
            })),
        ))
    }
}

impl WorkspaceController {
    pub fn new(
        cfg: Rc<toolkit::Configuration>,
        client: &LibraryClientWrapper,
    ) -> Rc<WorkspaceController> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let ctrl = Rc::new(WorkspaceController {
            imp_: RefCell::new(ControllerImpl::default()),
            tx,

            cfg,
            widgets: OnceCell::new(),
            action_group: OnceCell::new(),
            selection_changed: Signal::default(),
            client: Arc::downgrade(&client.client()),
            icon_folder: gio::ThemedIcon::new("folder-symbolic").upcast(),
            icon_trash: gio::ThemedIcon::new("user-trash").upcast(),
            icon_roll: gio::ThemedIcon::new("emblem-photos").upcast(),
            icon_project: gio::ThemedIcon::new("file-cabinet-symbolic").upcast(),
            icon_keyword: gio::ThemedIcon::new("tag-symbolic").upcast(),
        });

        rx.attach(
            None,
            glib::clone!(@strong ctrl => move |e| {
                ctrl.dispatch(e);
                glib::Continue(true)
            }),
        );

        ctrl
    }

    /// Initiate the initial loading.
    pub fn startup(&self) {
        if let Some(client) = self.client.upgrade() {
            client.get_all_folders();
            client.get_all_keywords();
        } else {
            err_out!("coudln't get client");
        }
    }

    fn dispatch(&self, e: Event) {
        use Event::*;

        match e {
            ButtonPress(x, y) => self.button_press_event(x, y),
            SelectionChanged => self.on_libtree_selection(),
            RowExpanded(iter, _) => self.row_expanded_collapsed(iter, true),
            RowCollapsed(iter, _) => self.row_expanded_collapsed(iter, false),
            NewFolder => self.action_new_folder(),
            DeleteFolder => self.action_delete_folder(),
            Import => self.action_import(),
            PerformImport(request) => self.perform_file_import(request.as_ref()),
        }
    }

    fn button_press_event(&self, x: f64, y: f64) {
        if let Some(widgets) = self.widgets.get() {
            if widgets.librarytree.selection().count_selected_rows() != 0 {
                widgets
                    .context_menu
                    .set_pointing_to(Some(&gdk4::Rectangle::new(x as i32, y as i32, 1, 1)));
                widgets.context_menu.popup();
            }
        }
    }

    fn on_libtree_selection(&self) {
        if let Some(widgets) = self.widgets.get() {
            if let Some((model, iter)) = widgets.librarytree.selection().selected() {
                let type_: i32 = model.get(&iter, Columns::Type as i32);
                let id: i64 = model.get(&iter, Columns::Id as i32);
                dbg_out!("selected type {}, id {}", type_, id);
                if let Some(type_) = FromPrimitive::from_i32(type_) {
                    if let Some(client) = self.client.upgrade() {
                        match type_ {
                            TreeItemType::Folder => client.query_folder_content(id),
                            TreeItemType::Keyword => client.query_keyword_content(id),
                            _ => {
                                dbg_out!("Something selected of type {:?}", type_);
                            }
                        }
                    }
                }
            }
            // XXX
            // disable DeleteFolder of type != folder
            self.selection_changed.emit(());
        }
    }

    fn row_expanded_collapsed(&self, iter: gtk4::TreeIter, expanded: bool) {
        if let Some(widgets) = self.widgets.get() {
            let value = widgets.treestore.get_value(&iter, Columns::Type as i32);
            if let Ok(n) = value.get() {
                FromPrimitive::from_i32(n)
                    .and_then(|v| match v {
                        TreeItemType::Folders => Some("workspace_folders_expanded"),
                        TreeItemType::Projects => Some("workspace_projects_expanded"),
                        TreeItemType::Keywords => Some("workspace_keywords_expanded"),
                        x => {
                            err_out!("Incorrect node type {}", x as i32);
                            None
                        }
                    })
                    .map(|key| self.cfg.set_value(key, &expanded.to_string()))
                    .or_else(|| {
                        err_out!("Invalid node type {}", n);
                        None
                    });
            }
        }
    }

    fn action_new_folder(&self) {
        if let Some(client) = self.client.upgrade() {
            let window = self
                .widget()
                .ancestor(gtk4::Window::static_type())
                .and_then(|w| w.downcast::<gtk4::Window>().ok());
            super::dialogs::requestnewfolder::request(client, window.as_ref());
        }
    }

    fn action_delete_folder(&self) {
        if let Some(id) = self.selected_folder_id() {
            let window = self
                .widget()
                .ancestor(gtk4::Window::static_type())
                .and_then(|w| w.downcast::<gtk4::Window>().ok());
            let dialog = super::dialogs::confirm::request(
                &gettext("Delete selected folder?"),
                window.as_ref(),
            );
            dialog.connect_response(
                glib::clone!(@strong dialog, @strong self.client as client => move |_, response| {
                if response == gtk4::ResponseType::Yes {
                    if let Some(client) = client.upgrade() {
                    client.delete_folder(id);
                    }
                }
                dialog.destroy();
                }),
            );
            dialog.show();
        }
    }

    fn perform_file_import(&self, request: &ImportRequest) {
        let app = npc_fwk::ffi::Application_app();
        let cfg = &app.config().cfg; // XXX change to getLibraryConfig()
                                     // as the last import should be part of the library not the application.

        // import
        // XXX change the API to provide more details.
        let source = request.source();
        if source.is_empty() {
            return;
        }
        // XXX this should be a different config key
        // specific to the importer.
        cfg.set_value("last_import_location", source);

        let importer = request.importer();
        if let Some(client) = self
            .client
            .upgrade()
            .as_ref()
            .map(LibraryClientWrapper::wrap)
        {
            let dest_dir = request.dest_dir();
            importer.do_import(
                source,
                dest_dir,
                move |client, path, files, manage| -> bool {
                    client.import_files(path.to_string(), files.0.clone(), manage);
                    // XXX the libraryclient function returns void
                    true
                },
                &client,
            );
        }
    }

    fn action_import(&self) {
        let import_dialog = crate::ffi::import_dialog_new();
        let arg = Box::new(ImportDialogArgument {
            dialog: import_dialog.clone(),
            tx: self.tx.clone(),
        });
        let parent: *mut gtk4_sys::GtkWindow = if let Some(parent) = self
            .widget()
            .root()
            .and_then(|root| root.downcast_ref::<gtk4::Window>().cloned())
        {
            parent.to_glib_none().0
        } else {
            err_out!("parent not found");
            std::ptr::null_mut()
        };
        unsafe {
            import_dialog.run_modal(
                parent as *mut crate::ffi::GtkWindow,
                |arg, response| {
                    dbg_out!("import dialog response: {}", response);
                    let request = arg.dialog.import_request();
                    arg.dialog.close();
                    if response == 0 {
                        on_err_out!(arg.tx.send(Event::PerformImport(request)));
                    }
                },
                Box::into_raw(arg),
            );
        }
    }

    fn selected_folder_id(&self) -> Option<db::LibraryId> {
        self.widgets.get().and_then(|widgets| {
            let selection = widgets.librarytree.selection();
            selection.selected().and_then(|selected| {
                let t: Option<TreeItemType> =
                    FromPrimitive::from_i32(selected.0.get(&selected.1, Columns::Type as i32));
                if t != Some(TreeItemType::Folder) {
                    None
                } else {
                    Some(selected.0.get(&selected.1, Columns::Id as i32))
                }
            })
        })
    }

    fn add_folder_item(&self, folder: &db::LibFolder) {
        if let Some(widgets) = self.widgets.get() {
            let icon = if folder.virtual_type() == db::libfolder::FolderVirtualType::TRASH {
                if let Some(client) = self.client.upgrade() {
                    client.set_trash_id(folder.id());
                }
                &self.icon_trash
            } else {
                &self.icon_roll
            };

            let iter = widgets.add_folder_item(folder, icon);
            if folder.expanded() {
                widgets.expand_row(&iter, false);
            }
            if let Some(client) = self.client.upgrade() {
                client.count_folder(folder.id());
            }
        } else {
            err_out!("couldn't get widgets");
        }
    }

    fn remove_folder_item(&self, id: db::LibraryId) {
        if let Some(widgets) = self.widgets.get() {
            widgets.remove_folder_item(id);
        } else {
            err_out!("couldn't get widgets");
        }
    }

    fn add_keyword_item(&self, keyword: &db::Keyword) {
        if let Some(widgets) = self.widgets.get() {
            widgets.add_keyword_item(keyword, &self.icon_keyword);
            if let Some(client) = self.client.upgrade() {
                client.count_keyword(keyword.id());
            }
        } else {
            err_out!("couldn't get widgets");
        }
    }

    fn add_item(
        treestore: &gtk4::TreeStore,
        subtree: Option<&gtk4::TreeIter>,
        icon: &gio::Icon,
        label: &str,
        id: db::LibraryId,
        type_: TreeItemType,
    ) -> gtk4::TreeIter {
        let iter = treestore.append(subtree);
        treestore.set(
            &iter,
            &[
                (Columns::Icon as u32, icon),
                (Columns::Id as u32, &id),
                (Columns::Label as u32, &label),
                (Columns::Type as u32, &(type_ as i32)),
                (Columns::Count as u32, &"--"),
                (Columns::CountN as u32, &0),
            ],
        );
        iter
    }

    pub fn on_lib_notification(&self, ln: &LibNotification) {
        dbg_out!("notification for workspace {:?}", ln);
        match ln {
            LibNotification::AddedFolder(f) => self.add_folder_item(f),
            LibNotification::FolderDeleted(id) => self.remove_folder_item(*id),
            LibNotification::AddedKeyword(k) => self.add_keyword_item(k),
            LibNotification::FolderCounted(count) | LibNotification::KeywordCounted(count) => {
                dbg_out!("count for container {} is {}", count.id, count.count);
                let iter = match ln {
                    LibNotification::FolderCounted(count) => self
                        .widgets
                        .get()
                        .and_then(|w| w.folderidmap.borrow().get(&count.id).cloned()),
                    LibNotification::KeywordCounted(count) => self
                        .widgets
                        .get()
                        .and_then(|w| w.keywordsidmap.borrow().get(&count.id).cloned()),
                    _ => unreachable!(),
                };
                if let Some(iter) = iter {
                    if let Some(widgets) = self.widgets.get() {
                        widgets.set_count(&iter, count.count as i32);
                    } else {
                        err_out!("No widget");
                    }
                } else {
                    err_out!("Iter not found");
                }
            }
            LibNotification::FolderCountChanged(count)
            | LibNotification::KeywordCountChanged(count) => {
                dbg_out!("count change for container {} is {}", count.id, count.count);
                let iter = match ln {
                    LibNotification::FolderCountChanged(count) => self
                        .widgets
                        .get()
                        .and_then(|w| w.folderidmap.borrow().get(&count.id).cloned()),
                    LibNotification::KeywordCountChanged(count) => self
                        .widgets
                        .get()
                        .and_then(|w| w.keywordsidmap.borrow().get(&count.id).cloned()),
                    _ => unreachable!(),
                };
                if let Some(iter) = iter {
                    if let Some(widgets) = self.widgets.get() {
                        widgets.increase_count(&iter, count.count as i32);
                    }
                }
            }
            _ => {}
        }
    }
}
