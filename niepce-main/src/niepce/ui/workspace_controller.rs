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
use std::rc::Rc;

use gettextrs::gettext;
use gtk4::gio;
use gtk4::glib;
use gtk4::glib::Type;
use gtk4::prelude::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use once_cell::unsync::OnceCell;

use npc_engine::db;
use npc_fwk::toolkit::{self, Controller, ControllerImpl, UiController};
use npc_fwk::{err_out, on_err_out};

#[derive(FromPrimitive)]
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
    RowExpanded(gtk4::TreeIter, gtk4::TreePath),
    RowCollapsed(gtk4::TreeIter, gtk4::TreePath),
    NewFolder,
    DeleteFolder,
    Import,
}

pub struct WorkspaceController {
    imp_: RefCell<ControllerImpl>,
    tx: glib::Sender<Event>,
    cfg: Rc<toolkit::Configuration>,
    widgets: OnceCell<Widgets>,
    action_group: OnceCell<gio::ActionGroup>,

    icon_folder: gio::Icon,
    icon_project: gio::Icon,
    icon_keyword: gio::Icon,
}

struct Widgets {
    widget_: gtk4::Widget,
    treestore: gtk4::TreeStore,
    librarytree: gtk4::TreeView,
    context_menu: gtk4::PopoverMenu,

    folder_node: gtk4::TreeIter,
    project_node: gtk4::TreeIter,
    keywords_node: gtk4::TreeIter,
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
                    project_node,
                    folder_node,
                    keywords_node,
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
    pub fn new(cfg: Rc<toolkit::Configuration>) -> Rc<WorkspaceController> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let ctrl = Rc::new(WorkspaceController {
            imp_: RefCell::new(ControllerImpl::default()),
            tx,

            cfg,
            widgets: OnceCell::new(),
            action_group: OnceCell::new(),
            icon_folder: gio::ThemedIcon::new("folder-symbolic").upcast(),
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

    fn dispatch(&self, e: Event) {
        use Event::*;

        match e {
            ButtonPress(x, y) => self.button_press_event(x, y),
            RowExpanded(iter, _) => self.row_expanded_collapsed(iter, true),
            RowCollapsed(iter, _) => self.row_expanded_collapsed(iter, false),
            NewFolder => self.action_new_folder(),
            DeleteFolder => self.action_delete_folder(),
            Import => self.action_import(),
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

    fn action_new_folder(&self) {}

    fn action_delete_folder(&self) {}

    fn action_import(&self) {}

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
}
