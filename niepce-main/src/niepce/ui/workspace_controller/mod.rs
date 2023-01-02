/*
 * niepce - niepce/ui/workspace_controller/mod.rs
 *
 * Copyright (C) 2021-2023 Hubert Figuière
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

mod ws_item_row;
mod ws_list_item;
mod ws_list_model;

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
use std::sync::{Arc, Weak};

use adw::prelude::*;
use gettextrs::gettext as i18n;
use glib::Cast;
use gtk4::gio;
use gtk4::glib;
use num_derive::FromPrimitive;
use once_cell::unsync::OnceCell;

use super::ContentView;
use crate::import::ImportRequest;
use npc_engine::db;
use npc_engine::library::notification::LibNotification;
use npc_engine::libraryclient::{ClientInterface, LibraryClient, LibraryClientWrapper};
use npc_fwk::base::Signal;
use npc_fwk::toolkit::{self, Controller, ControllerImpl, UiController};
use npc_fwk::{dbg_out, err_out, on_err_out};
use ws_item_row::WsItemRow;
use ws_list_item::Item;
use ws_list_model::WorkspaceList;

#[derive(Clone, Copy, Debug, Default, FromPrimitive, PartialEq)]
#[repr(i32)]
enum TreeItemType {
    #[default]
    None,
    Folders,
    Projects,
    Keywords,
    Albums,
    Trash,
    Folder,
    Project,
    Keyword,
    Album,
}

enum Event {
    ButtonPress(f64, f64),
    SelectionChanged,
    RowExpanded(u32),
    RowCollapsed(u32),
    NewFolder,
    NewAlbum,
    /// Delete the current item.
    DeleteItem,
    /// Rename the current item.
    RenameItem,
    /// Initiate the import.
    Import,
    /// Sent after the import is initiated
    PerformImport(ImportRequest),
    /// Import a library
    ImportLibrary,
    /// `LibFile`s dropped onto workspace. (target, type, source)
    DropLibFile(db::LibraryId, TreeItemType, Vec<db::LibraryId>),
}

pub struct WorkspaceController {
    imp_: RefCell<ControllerImpl>,
    tx: glib::Sender<Event>,
    cfg: Rc<toolkit::Configuration>,
    widgets: OnceCell<Widgets>,
    client: Weak<LibraryClient>,
    action_group: OnceCell<gio::ActionGroup>,
    pub selection_changed: Signal<ContentView>,

    icon_trash: gio::Icon,
    icon_roll: gio::Icon,
    icon_folder: gio::Icon,
    // icon_project: gio::Icon,
    icon_keyword: gio::Icon,
    icon_album: gio::Icon,
}

struct Widgets {
    widget_: gtk4::Widget,
    treemodel: gtk4::TreeListModel,
    librarytree: gtk4::ListView,
    context_menu: gtk4::PopoverMenu,

    // position of the nodes in the rootstore
    folders_node: gtk4::TreeListRow,
    // Projects are not implemented yet.
    // project_node: gtk4::TreeListRow,
    keywords_node: gtk4::TreeListRow,
    albums_node: gtk4::TreeListRow,
    cfg: Rc<toolkit::Configuration>,
}

impl Widgets {
    fn add_folder_item(&self, folder: &db::LibFolder, icon: &gio::Icon) -> Option<u32> {
        let was_empty = self
            .folders_node
            .children()
            .map(|children| children.n_items() == 0)
            .unwrap_or(true);
        let tree_item_type = if folder.virtual_type() == db::libfolder::FolderVirtualType::Trash {
            TreeItemType::Trash
        } else {
            TreeItemType::Folder
        };
        WorkspaceController::add_item(
            &self.folders_node,
            icon,
            folder.name(),
            folder.id(),
            tree_item_type,
        )
        .map(|pos| {
            if was_empty {
                self.expand_from_cfg("workspace_folders_expanded", &self.folders_node);
            }

            pos
        })
    }

    fn remove_folder_item(&self, id: db::LibraryId) {
        if let Some(store) = self
            .folders_node
            .children()
            .and_then(|store| store.downcast::<WorkspaceList>().ok())
        {
            if let Err(err) = store.remove_by_id(&id) {
                err_out!("Couldn't remove folder item {}: {:?}", id, err);
            }
        }
    }

    fn add_keyword_item(&self, keyword: &db::Keyword, icon: &gio::Icon) {
        let was_empty = self
            .keywords_node
            .children()
            .map(|children| children.n_items() == 0)
            .unwrap_or(true);
        if WorkspaceController::add_item(
            &self.keywords_node,
            icon,
            keyword.keyword(),
            keyword.id(),
            TreeItemType::Keyword,
        )
        .is_some()
            && was_empty
        {
            self.expand_from_cfg("workspace_keywords_expanded", &self.keywords_node);
        }
    }

    fn add_album_item(&self, album: &db::Album, icon: &gio::Icon) {
        let was_empty = self
            .albums_node
            .children()
            .map(|children| children.n_items() == 0)
            .unwrap_or(true);
        if WorkspaceController::add_item(
            &self.albums_node,
            icon,
            album.name(),
            album.id(),
            TreeItemType::Album,
        )
        .is_some()
            && was_empty
        {
            self.expand_from_cfg("workspace_albums_expanded", &self.albums_node);
        }
    }

    fn remove_album_item(&self, id: db::LibraryId) {
        if let Some(store) = self
            .albums_node
            .children()
            .and_then(|children| children.downcast::<WorkspaceList>().ok())
        {
            if let Err(err) = store.remove_by_id(&id) {
                err_out!("Couldn't remove album item {}: {:?}", id, err);
            }
        }
    }

    fn expand_from_cfg(&self, key: &str, row: &gtk4::TreeListRow) {
        let expanded = self.cfg.value(key, "true");
        dbg_out!("expand from cfg {} - {}", key, &expanded);
        row.set_expanded(expanded == "true");
    }

    fn find_item_index(
        &self,
        tree_item_type: TreeItemType,
        id: db::LibraryId,
    ) -> Option<(WorkspaceList, u32)> {
        match tree_item_type {
            TreeItemType::Folders => self.folders_node.children(),
            TreeItemType::Keywords => self.keywords_node.children(),
            TreeItemType::Albums => self.albums_node.children(),
            _ => {
                err_out!("find_item_index: Incorrect node type {:?}", tree_item_type);
                None
            }
        }
        .and_then(|children| children.downcast::<WorkspaceList>().ok())
        .and_then(|store| store.pos_by_id(&id).map(|pos| (store, pos)))
    }

    fn increase_count(&self, tree_item_type: TreeItemType, id: db::LibraryId, count: i32) {
        if let Some(item_index) = self.find_item_index(tree_item_type, id) {
            let store = item_index.0;
            if let Some(item) = store
                .item(item_index.1)
                .and_then(|item| item.downcast::<Item>().ok())
            {
                item.set_count(count + item.count());
                store.items_changed(item_index.1, 0, 0);
            }
        }
    }

    fn set_count(&self, tree_item_type: TreeItemType, id: db::LibraryId, count: i32) {
        if let Some(item_index) = self.find_item_index(tree_item_type, id) {
            let store = item_index.0;
            if let Some(item) = store
                .item(item_index.1)
                .and_then(|item| item.downcast::<Item>().ok())
            {
                item.set_count(count);
                store.items_changed(item_index.1, 0, 0);
            }
        }
    }

    /// Change the label of an item in the list
    fn rename_item(&self, tree_item_type: TreeItemType, id: db::LibraryId, name: &str) {
        if let Some(item_index) = self.find_item_index(tree_item_type, id) {
            let store = item_index.0;
            if let Some(item) = store
                .item(item_index.1)
                .and_then(|item| item.downcast::<Item>().ok())
            {
                item.set_label(name);
                store.items_changed(item_index.1, 0, 0);
            }
        }
    }

    fn expand_row(&self, at: u32) {
        if let Some(row) = self.treemodel.row(at) {
            row.set_expanded(true);
        }
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

                let rootstore = gio::ListStore::new(Item::static_type());
                let treemodel = gtk4::TreeListModel::new(&rootstore, false, true, |item| {
                    Some(item.downcast_ref::<Item>()?.create_children()?.upcast_ref::<gio::ListModel>().clone())
                });
                let selection_model = gtk4::SingleSelection::new(Some(&treemodel));

                let factory = gtk4::SignalListItemFactory::new();
                factory.connect_setup(glib::clone!(@strong self.tx as tx => move |_, item| {
                    let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
                    let item_row = WsItemRow::new(tx.clone());
                    item.set_child(Some(&item_row));
                }));
                factory.connect_bind(glib::clone!(@strong self.tx as tx => move |_, list_item| {
                    let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
                    let ws_item_row = list_item.child().unwrap().downcast_ref::<WsItemRow>().unwrap().clone();
                    if let Some(item) = list_item.item() {
                        let tree_list_row = item.downcast_ref::<gtk4::TreeListRow>().expect("to be a TreeListRow");
                        if let Some(item) = tree_list_row.item() {
                            match item.downcast_ref::<Item>().unwrap().tree_item_type() {
                                TreeItemType::Folders |
                                TreeItemType::Albums |
                                TreeItemType::Keywords => {
                                    // We connect the expanded notify signal only
                                    // for these top level tree item.
                                    tree_list_row.connect_expanded_notify(glib::clone!(@strong tx => move |tree_list_row| {
                                        let expanded = tree_list_row.is_expanded();
                                        let pos = tree_list_row.position();
                                        if expanded {
                                            on_err_out!(tx.send(Event::RowExpanded(pos)));
                                        } else {
                                            on_err_out!(tx.send(Event::RowCollapsed(pos)));
                                        }
                                    }));
                                }
                            _ => {}
                            };
                            let ws_item = item.downcast::<Item>().expect("is an item");
                            ws_item_row.bind(&ws_item, tree_list_row);
                        }
                    }
                }));
                factory.connect_unbind(move |_, list_item| {
                    let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
                    let ws_item_row = list_item.child().unwrap().downcast_ref::<WsItemRow>().unwrap().clone();
                    ws_item_row.unbind();
                });
                let librarytree = gtk4::ListView::new(Some(&selection_model), Some(&factory));
                librarytree.set_single_click_activate(false);

                let folders_node = WorkspaceController::add_toplevel_item(
                    &treemodel,
                    &self.icon_folder,
                    &i18n("Pictures"),
                    TreeItemType::Folders,
                );
                // Projects are not implemented yet
                // let project_node = Self::add_toplevel_item(
                //     &treestore,
                //     &self.icon_project,
                //     &i18n("Projects"),
                //     TreeItemType::Projects,
                // );
                let albums_node = WorkspaceController::add_toplevel_item(
                    &treemodel,
                    &self.icon_album,
                    &i18n("Albums"),
                    TreeItemType::Albums,
                );
                let keywords_node = WorkspaceController::add_toplevel_item(
                    &treemodel,
                    &self.icon_keyword,
                    &i18n("Keywords"),
                    TreeItemType::Keywords,
                );

                let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
                // header.set_margin(4);
                let label = gtk4::Label::with_mnemonic(&i18n("_Workspace"));
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
                section.append(Some(&i18n("New Folder…")), Some("workspace.NewFolder"));
                section.append(Some(&i18n("New Album…")), Some("workspace.NewAlbum"));
                // section.append(
                //     Some(&i18n("New Project…")),
                //     Some("workspace.NewProject"),
                // );
                section.append(Some(&i18n("Rename…")), Some("workspace.RenameItem"));
                section.append(Some(&i18n("Delete")), Some("workspace.DeleteItem"));

                let section = gio::Menu::new();
                menu.append_section(None, &section);
                section.append(Some(&i18n("Import…")), Some("workspace.Import"));
                section.append(
                    Some(&i18n("Import Library…")),
                    Some("workspace.ImportLibrary"),
                );

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
                if let Some(model) = librarytree.model() {
                    model.connect_selection_changed(
                        glib::clone!(@strong self.tx as tx => move |_, _, _| {
                            on_err_out!(tx.send(Event::SelectionChanged));
                        }),
                    );
                }
                let gesture = gtk4::GestureClick::new();
                gesture.set_button(3);
                librarytree.add_controller(&gesture);
                gesture.connect_pressed(glib::clone!(@strong self.tx as tx => move |_, _, x, y| {
                    on_err_out!(tx.send(Event::ButtonPress(x, y)));
                }));

                Widgets {
                    widget_: main_box.upcast(),
                    librarytree,
                    treemodel,
                    context_menu,
                    // project_node,
                    folders_node: folders_node.unwrap(),
                    albums_node: albums_node.unwrap(),
                    keywords_node: keywords_node.unwrap(),
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
                    "NewAlbum",
                    glib::clone!(@strong tx => move |_, _| {
                        on_err_out!(tx.send(Event::NewAlbum));
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
                    "ImportLibrary",
                    glib::clone!(@strong tx => move |_, _| {
                        on_err_out!(tx.send(Event::ImportLibrary));
                    })
                );
                action!(
                    group,
                    "RenameItem",
                    glib::clone!(@strong tx => move |_, _| {
                        on_err_out!(tx.send(Event::RenameItem));
                    })
                );
                action!(
                    group,
                    "DeleteItem",
                    glib::clone!(@strong tx => move |_, _| {
                        on_err_out!(tx.send(Event::DeleteItem));
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
            // icon_project: gio::ThemedIcon::new("file-cabinet-symbolic").upcast(),
            icon_keyword: gio::ThemedIcon::new("tag-symbolic").upcast(),
            icon_album: gio::ThemedIcon::new("open-book-symbolic").upcast(),
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
            client.get_all_albums();
        } else {
            err_out!("coudln't get client");
        }
    }

    fn dispatch(&self, e: Event) {
        use Event::*;

        match e {
            ButtonPress(x, y) => self.button_press_event(x, y),
            SelectionChanged => self.on_libtree_selection(),
            RowExpanded(pos) => self.row_expanded_collapsed(pos, true),
            RowCollapsed(pos) => self.row_expanded_collapsed(pos, false),
            NewFolder => self.action_new_folder(),
            NewAlbum => self.action_new_album(),
            RenameItem => self.action_rename_item(),
            DeleteItem => self.action_delete_item(),
            Import => self.action_import(),
            PerformImport(request) => self.perform_file_import(&request),
            ImportLibrary => self.action_import_library(),
            DropLibFile(target, type_, source) => self.action_drop_libfile(target, type_, source),
        }
    }

    fn button_press_event(&self, x: f64, y: f64) {
        if let Some(widgets) = self.widgets.get() {
            if widgets
                .librarytree
                .model()
                .and_then(|m| m.downcast::<gtk4::SingleSelection>().ok())
                .and_then(|m| m.selected_item())
                .is_some()
            {
                widgets
                    .context_menu
                    .set_pointing_to(Some(&gdk4::Rectangle::new(x as i32, y as i32, 1, 1)));
                widgets.context_menu.popup();
            }
        }
    }

    fn on_libtree_selection(&self) {
        let mut content = ContentView::Empty;
        if let Some((type_, id)) = self.selected_item_id() {
            if let Some(client) = self.client.upgrade() {
                content = match type_ {
                    TreeItemType::Folder => {
                        client.query_folder_content(id);
                        ContentView::Folder(id)
                    }
                    TreeItemType::Keyword => {
                        client.query_keyword_content(id);
                        ContentView::Keyword(id)
                    }
                    TreeItemType::Album => {
                        client.query_album_content(id);
                        ContentView::Album(id)
                    }
                    _ => {
                        dbg_out!("Something selected of type {:?}", type_);
                        ContentView::Empty
                    }
                }
            }
        }
        // XXX
        // disable DeleteItem of type != folder or album
        self.selection_changed.emit(content);
    }

    fn row_expanded_collapsed(&self, pos: u32, expanded: bool) {
        if let Some(item) = self
            .widgets
            .get()
            .and_then(|widgets| widgets.treemodel.item(pos))
            .and_then(|item| item.downcast_ref::<gtk4::TreeListRow>()?.item())
            .and_then(|item| item.downcast::<Item>().ok())
        {
            let type_ = item.tree_item_type();
            if let Some(key) = match type_ {
                TreeItemType::Folders => Some("workspace_folders_expanded"),
                TreeItemType::Projects => Some("workspace_projects_expanded"),
                TreeItemType::Keywords => Some("workspace_keywords_expanded"),
                TreeItemType::Albums => Some("workspace_albums_expanded"),
                // Not an error. This is no-op
                _ => None,
            } {
                self.cfg.set_value(key, &expanded.to_string());
            }
        }
    }

    fn action_new_folder(&self) {
        if let Some(client) = self.client.upgrade() {
            let window = self
                .widget()
                .ancestor(gtk4::Window::static_type())
                .and_then(|w| w.downcast::<gtk4::Window>().ok());
            npc_fwk::toolkit::request::request_name(
                window.as_ref(),
                &i18n("New folder"),
                &i18n("Folder _name:"),
                Some(&i18n("Untitled folder")),
                move |name| {
                    dbg_out!("Create folder {}", &name);
                    client.create_folder(name.to_string(), None);
                },
            );
        }
    }

    fn action_rename_album(&self, album: db::LibraryId, name: &str) {
        if let Some(client) = self.client.upgrade() {
            let window = self
                .widget()
                .ancestor(gtk4::Window::static_type())
                .and_then(|w| w.downcast::<gtk4::Window>().ok());
            npc_fwk::toolkit::request::request_name(
                window.as_ref(),
                &i18n("Rename album"),
                &i18n("Album _name:"),
                // XXX fix this
                Some(name),
                move |name| {
                    dbg_out!("Rename album {}", &name);
                    client.rename_album(album, name.to_string());
                },
            );
        }
    }

    /// Rename the selected item
    fn action_rename_item(&self) {
        if let Some(item) = self.selected_item() {
            let id = item.id();
            let name = item.label();
            let type_ = item.tree_item_type();
            match type_ {
                TreeItemType::Album => self.action_rename_album(id, &name),
                _ => err_out!("Wrong type {:?}", type_),
            }
        }
    }

    /// Delete the selected item
    fn action_delete_item(&self) {
        if let Some((type_, id)) = self.selected_item_id() {
            match type_ {
                TreeItemType::Folder => self.action_delete_folder(id),
                TreeItemType::Album => self.action_delete_album(id),
                _ => err_out!("Wrong type {:?}", type_),
            }
        }
    }

    fn action_delete_folder(&self, id: db::LibraryId) {
        let window = self
            .widget()
            .ancestor(gtk4::Window::static_type())
            .and_then(|w| w.downcast::<gtk4::Window>().ok());
        let dialog = npc_fwk::toolkit::confirm::request(
            &i18n("Delete selected folder?"),
            &i18n("The folder will be deleted."),
            Some(i18n("_Delete")),
            true,
            window.as_ref(),
        );
        dialog.connect_response(
            None,
            glib::clone!(@strong dialog, @strong self.client as client => move |_, response| {
                if response == "confirm" {
                    if let Some(client) = client.upgrade() {
                        client.delete_folder(id);
                    }
                }
                dialog.destroy();
            }),
        );
        dialog.show();
    }

    fn action_new_album(&self) {
        if let Some(client) = self.client.upgrade() {
            let window = self
                .widget()
                .ancestor(gtk4::Window::static_type())
                .and_then(|w| w.downcast::<gtk4::Window>().ok());
            npc_fwk::toolkit::request::request_name(
                window.as_ref(),
                &i18n("New Album"),
                &i18n("Album _name:"),
                Some(&i18n("Untitled album")),
                move |name| {
                    client.create_album(name.to_string(), -1);
                },
            );
        }
    }

    fn action_delete_album(&self, id: db::LibraryId) {
        let window = self
            .widget()
            .ancestor(gtk4::Window::static_type())
            .and_then(|w| w.downcast::<gtk4::Window>().ok());
        let dialog = npc_fwk::toolkit::confirm::request(
            &i18n("Delete selected album?"),
            &i18n("The album will be deleted."),
            Some(i18n("_Delete")),
            true,
            window.as_ref(),
        );
        dialog.connect_response(
            None,
            glib::clone!(@strong dialog, @strong self.client as client => move |_, response| {
                if response == "confirm" {
                    if let Some(client) = client.upgrade() {
                        client.delete_album(id);
                    }
                }
                dialog.destroy();
            }),
        );
        dialog.show();
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
        if let Some(client) = self.client.upgrade() {
            let dest_dir = request.dest_dir();
            let client = client.sender().clone();
            importer.do_import(
                source,
                dest_dir,
                Box::new(
                    move |path: &std::path::Path,
                          files: &npc_fwk::utils::files::FileList,
                          manage| {
                        client.import_files(
                            path.to_string_lossy().to_string(),
                            files.0.clone(),
                            manage,
                        );
                    },
                ),
            );
        }
    }

    fn action_import(&self) {
        let import_dialog = super::dialogs::ImportDialog::new();
        let parent = self
            .widget()
            .root()
            .and_then(|root| root.downcast::<gtk4::Window>().ok());
        import_dialog.run_modal(
            parent.as_ref(),
            glib::clone!(@weak import_dialog, @strong self.tx as tx => move |dialog, response| {
                dbg_out!("import dialog response: {}", response);
                let request = import_dialog.import_request();
                dialog.close();
                if let Some(request) = request {
                    if response == 0 {
                        on_err_out!(tx.send(Event::PerformImport(request)));
                    }
                }
            }),
        );
    }

    fn action_import_library(&self) {
        use crate::niepce::ui::dialogs::ImportLibraryDialog;

        if let Some(client) = self.client.upgrade() {
            let parent = self
                .widget()
                .root()
                .and_then(|root| root.downcast_ref::<gtk4::Window>().cloned());
            let dialog = ImportLibraryDialog::new(client);
            dialog.run(parent.as_ref());
            dbg_out!("dialog out of scope");
        }
    }

    /// A `LibFile` with `source` id was dropped onto `target` of `type_`.
    /// Act upon it.
    fn action_drop_libfile(
        &self,
        target: db::LibraryId,
        type_: TreeItemType,
        source: Vec<db::LibraryId>,
    ) {
        dbg_out!(
            "dropped files id {:?} onto a {:?}({})",
            source,
            type_,
            target
        );
        use TreeItemType::*;
        match type_ {
            Trash => {
                // let source_container = self.selected_item_id();
            }
            Album => {
                if let Some(client) = self.client.upgrade() {
                    let client_redo = client.clone();
                    let redo_source = source.clone();
                    npc_fwk::toolkit::undo_do_command(
                        &i18n("Add to Album"),
                        Box::new(move || {
                            client_redo.add_to_album(&redo_source, target);
                            npc_fwk::toolkit::Storage::Void
                        }),
                        Box::new(move |_| client.remove_from_album(&source, target)),
                    );
                }
            }
            Keyword => {}
            _ => err_out!("Unhandled drop target of type {:?}", type_),
        }
    }

    fn selected_item(&self) -> Option<Item> {
        self.widgets
            .get()?
            .librarytree
            .model()
            .and_then(|selection| {
                selection
                    .downcast::<gtk4::SingleSelection>()
                    .ok()?
                    .selected_item()
            })
            .and_then(|item| item.downcast_ref::<gtk4::TreeListRow>()?.item())
            .and_then(|item| item.downcast::<Item>().ok())
    }

    /// Get the selected item id and type in the workspace.
    fn selected_item_id(&self) -> Option<(TreeItemType, db::LibraryId)> {
        self.selected_item()
            .map(|item| (item.tree_item_type(), item.id()))
    }

    fn add_folder_item(&self, folder: &db::LibFolder) {
        if let Some(widgets) = self.widgets.get() {
            let icon = if folder.virtual_type() == db::libfolder::FolderVirtualType::Trash {
                if let Some(client) = self.client.upgrade() {
                    client.set_trash_id(folder.id());
                }
                &self.icon_trash
            } else {
                &self.icon_roll
            };

            if let Some(pos) = widgets.add_folder_item(folder, icon) {
                if folder.expanded() {
                    widgets.expand_row(pos);
                }
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

    fn add_album_item(&self, album: &db::Album) {
        if let Some(widgets) = self.widgets.get() {
            widgets.add_album_item(album, &self.icon_album);
            if let Some(client) = self.client.upgrade() {
                client.count_album(album.id());
            }
        } else {
            err_out!("couldn't get widgets");
        }
    }

    fn remove_album_item(&self, id: db::LibraryId) {
        if let Some(widgets) = self.widgets.get() {
            widgets.remove_album_item(id);
        } else {
            err_out!("couldn't get widgets");
        }
    }

    /// Add a toplevel item
    fn add_toplevel_item(
        treestore: &gtk4::TreeListModel,
        icon: &gio::Icon,
        label: &str,
        type_: TreeItemType,
    ) -> Option<gtk4::TreeListRow> {
        let store = treestore.model().downcast::<gio::ListStore>().ok()?;

        let idx = store.n_items();
        store.append(&Item::with_values(icon, label, 0, type_));

        treestore.row(idx)
    }

    /// Add an item as a child of subtree.
    fn add_item(
        subtree: &gtk4::TreeListRow,
        icon: &gio::Icon,
        label: &str,
        id: db::LibraryId,
        type_: TreeItemType,
    ) -> Option<u32> {
        // XXX probably there is a different way
        let item = subtree.item()?.downcast::<Item>().expect("not an item");
        item.create_children().and_then(|children| {
            dbg_out!(
                "children created for item {:?} {}",
                item.tree_item_type(),
                item.label()
            );

            let idx = children.n_items();
            dbg_out!("store has {} items", idx);
            children
                .append(&Item::with_values(icon, label, id, type_))
                .ok()
                .map(|_| idx)
                .or_else(|| {
                    err_out!("Coudln't add item {}", label);
                    None
                })
        })
    }

    pub fn on_lib_notification(&self, ln: &LibNotification) {
        dbg_out!("notification for workspace {:?}", ln);
        match ln {
            LibNotification::AddedFolder(f) => self.add_folder_item(f),
            LibNotification::FolderDeleted(id) => self.remove_folder_item(*id),
            LibNotification::AddedKeyword(k) => self.add_keyword_item(k),
            LibNotification::AddedAlbum(a) => self.add_album_item(a),
            LibNotification::AlbumDeleted(id) => self.remove_album_item(*id),
            LibNotification::FolderCounted(count)
            | LibNotification::KeywordCounted(count)
            | LibNotification::AlbumCounted(count) => {
                dbg_out!("count for container {} is {}", count.id, count.count);
                let type_ = match ln {
                    LibNotification::FolderCounted(_) => TreeItemType::Folders,
                    LibNotification::KeywordCounted(_) => TreeItemType::Keywords,
                    LibNotification::AlbumCounted(_) => TreeItemType::Albums,
                    _ => unreachable!(),
                };
                if let Some(widgets) = self.widgets.get() {
                    widgets.set_count(type_, count.id, count.count as i32);
                } else {
                    err_out!("No widget");
                }
            }
            LibNotification::FolderCountChanged(count)
            | LibNotification::KeywordCountChanged(count)
            | LibNotification::AlbumCountChanged(count) => {
                dbg_out!("count change for container {} is {}", count.id, count.count);
                let type_ = match ln {
                    LibNotification::FolderCountChanged(_) => TreeItemType::Folders,
                    LibNotification::KeywordCountChanged(_) => TreeItemType::Keywords,
                    LibNotification::AlbumCountChanged(_) => TreeItemType::Albums,
                    _ => unreachable!(),
                };
                if let Some(widgets) = self.widgets.get() {
                    widgets.increase_count(type_, count.id, count.count as i32);
                }
            }
            LibNotification::AlbumRenamed(id, name) => {
                if let Some(widgets) = self.widgets.get() {
                    widgets.rename_item(TreeItemType::Albums, *id, name);
                }
            }
            _ => {}
        }
    }
}
