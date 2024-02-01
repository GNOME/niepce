/*
 * niepce - niepce/ui/module_shell.rs
 *
 * Copyright (C) 2022-2024 Hubert Figuière
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

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gettextrs::gettext as i18n;
use glib::Cast;
use gtk4::prelude::*;

use super::{
    GridViewModuleProxy, ImageListStore, LibraryModule, ModuleShellWidget, SelectionController,
};
use crate::modules::{DarkroomModule, MapModuleProxy};
use npc_engine::db;
use npc_engine::library::notification::LibNotification;
use npc_engine::libraryclient::LibraryClientHost;
use npc_fwk::toolkit::gtk_utils::add_menu_action;
use npc_fwk::toolkit::{to_controller, Controller, ControllerImpl, UiController};
use npc_fwk::{dbg_out, on_err_out};

enum Event {
    ModuleActivated(String),
    ModuleDeactivated(String),
}

pub struct ModuleShell {
    imp_: RefCell<ControllerImpl>,
    tx: npc_fwk::toolkit::Sender<Event>,
    widget: ModuleShellWidget,
    action_group: gio::SimpleActionGroup,
    pub selection_controller: Rc<SelectionController>,
    // currently a proxy that will bridge the C++ implementation
    gridview: Rc<GridViewModuleProxy>,
    mapm: Rc<MapModuleProxy>,
    darkroom: Rc<DarkroomModule>,
    menu: gio::Menu,
    module_menu: gio::Menu,
    client: Rc<LibraryClientHost>,
    modules: RefCell<HashMap<String, Rc<dyn LibraryModule>>>,
}

impl ModuleShell {
    pub fn new(client_host: &Rc<LibraryClientHost>) -> Rc<ModuleShell> {
        let (tx, rx) = npc_fwk::toolkit::channel();
        let selection_controller = SelectionController::new(client_host);
        let menu = gio::Menu::new();
        let shell = Rc::new(ModuleShell {
            imp_: RefCell::new(ControllerImpl::default()),
            tx: tx.clone(),
            widget: ModuleShellWidget::new(),
            action_group: gio::SimpleActionGroup::new(),
            gridview: Rc::new(GridViewModuleProxy::new(
                &selection_controller,
                &menu,
                client_host,
            )),
            mapm: Rc::new(MapModuleProxy::default()),
            darkroom: DarkroomModule::new(&selection_controller, client_host),
            selection_controller,
            menu,
            module_menu: gio::Menu::new(),
            client: client_host.clone(),
            modules: RefCell::new(HashMap::default()),
        });

        npc_fwk::toolkit::channels::receiver_attach(
            rx,
            glib::clone!(@strong shell => move |e| {
                shell.dispatch(e);
            }),
        );

        shell.add(&to_controller(shell.selection_controller.clone()));
        shell
            .widget
            .insert_action_group("shell", Some(&shell.action_group));

        Self::build_gridview_context_menu(&shell);
        shell.widget.menu_button().set_menu_model(Some(&shell.menu));

        shell.add_library_module(&shell.gridview, "grid", &i18n("Catalog"));
        shell.gridview.grid_view.connect_activate(glib::clone!(
        @weak shell.selection_controller.handler as handler => move |_, pos| {
            handler.activated(pos)
        }));

        shell.selection_controller.handler.signal_selected.connect(
            glib::clone!(@weak shell => move |id| {
                shell.on_image_selected(id);
            }),
        );
        shell.selection_controller.handler.signal_activated.connect(
            glib::clone!(@weak shell => move |id| {
                shell.on_image_activated(id);
            }),
        );

        // built-in modules;
        shell.add_library_module(&shell.darkroom, "darkroom", &i18n("Darkroom"));
        shell.add_library_module(&shell.mapm, "map", &i18n("Map"));

        shell.widget.connect(
            "activated",
            true,
            glib::clone!(@strong tx => move |value| {
                let name = value[1].get::<&str>().expect("Failed to convert callback parameter").to_string();
                npc_fwk::send_async_local!(Event::ModuleActivated(name), tx);
                None
            }),
        );
        let tx = shell.tx.clone();
        shell.widget.connect(
            "deactivated",
            true,
            glib::clone!(@strong tx => move |value| {
                let name = value[1].get::<&str>().expect("Failed to convert callback parameter").to_string();
                npc_fwk::send_async_local!(Event::ModuleDeactivated(name), tx);
                None
            }),
        );
        shell
    }

    /// Build the GridView context menu.
    fn build_gridview_context_menu(shell: &Rc<Self>) {
        let group = shell.action_group.upcast_ref::<gio::ActionMap>();
        add_menu_action(
            group,
            "PrevImage",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.select_previous()
            }),
            &shell.menu,
            Some(&i18n("Back")),
            Some("shell"),
            Some("Left"),
        );
        add_menu_action(
            group,
            "NextImage",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.select_next()
            }),
            &shell.menu,
            Some(&i18n("Forward")),
            Some("shell"),
            Some("Right"),
        );

        let section = gio::Menu::new();
        shell.menu.append_section(None, &section);
        add_menu_action(
            group,
            "RotateLeft",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.rotate(-90)
            }),
            &section,
            Some(&i18n("Rotate Left")),
            Some("shell"),
            Some("bracketleft"),
        );
        add_menu_action(
            group,
            "RotateRight",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.rotate(90)
            }),
            &section,
            Some(&i18n("Rotate Right")),
            Some("shell"),
            Some("bracketright"),
        );

        let section = gio::Menu::new();
        shell.menu.append_section(None, &section);
        let submenu = gio::Menu::new();
        section.append_submenu(Some(&i18n("Set Label")), &submenu);

        add_menu_action(
            group,
            "SetLabel6",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_label(1)
            }),
            &submenu,
            Some(&i18n("Label 6")),
            Some("shell"),
            Some("<Primary>6"),
        );
        add_menu_action(
            group,
            "SetLabel7",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_label(2)
            }),
            &submenu,
            Some(&i18n("Label 7")),
            Some("shell"),
            Some("<Primary>7"),
        );
        add_menu_action(
            group,
            "SetLabel8",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_label(3)
            }),
            &submenu,
            Some(&i18n("Label 8")),
            Some("shell"),
            Some("<Primary>8"),
        );
        add_menu_action(
            group,
            "SetLabel9",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_label(4)
            }),
            &submenu,
            Some(&i18n("Label 9")),
            Some("shell"),
            Some("<Primary>9"),
        );

        let submenu = gio::Menu::new();
        section.append_submenu(Some(&i18n("Set Rating")), &submenu);
        add_menu_action(
            group,
            "SetRating0",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_rating(0)
            }),
            &submenu,
            Some(&i18n("Unrated")),
            Some("shell"),
            Some("<Primary>0"),
        );
        add_menu_action(
            group,
            "SetRating1",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_rating(1)
            }),
            &submenu,
            Some(&i18n("Rating 1")),
            Some("shell"),
            Some("<Primary>1"),
        );
        add_menu_action(
            group,
            "SetRating2",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_rating(2)
            }),
            &submenu,
            Some(&i18n("Rating 2")),
            Some("shell"),
            Some("<Primary>2"),
        );
        add_menu_action(
            group,
            "SetRating3",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_rating(3)
            }),
            &submenu,
            Some(&i18n("Rating 3")),
            Some("shell"),
            Some("<Primary>3"),
        );
        add_menu_action(
            group,
            "SetRating4",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_rating(4)
            }),
            &submenu,
            Some(&i18n("Rating 4")),
            Some("shell"),
            Some("<Primary>4"),
        );
        add_menu_action(
            group,
            "SetRating5",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_rating(5)
            }),
            &submenu,
            Some(&i18n("Rating 5")),
            Some("shell"),
            Some("<Primary>5"),
        );

        let submenu = gio::Menu::new();
        section.append_submenu(Some(&i18n("Set Flag")), &submenu);
        add_menu_action(
            group,
            "SetFlagReject",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_flag(-1)
            }),
            &submenu,
            Some(&i18n("Flag as Rejected")),
            Some("shell"),
            Some("<Primary><Shift>x"),
        );
        add_menu_action(
            group,
            "SetFlagNone",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_flag(0)
            }),
            &submenu,
            Some(&i18n("Unflagged")),
            Some("shell"),
            Some("<Primary><Shift>u"),
        );
        add_menu_action(
            group,
            "SetFlagPick",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_flag(1)
            }),
            &submenu,
            Some(&i18n("Flag as Pick")),
            Some("shell"),
            Some("<Primary><Shift>p"),
        );

        let section = gio::Menu::new();
        shell.menu.append_section(None, &section);
        add_menu_action(
            group,
            "WriteMetadata",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.write_metadata()
            }),
            &section,
            Some(&i18n("Write metadata")),
            Some("shell"),
            None,
        );

        let section = gio::Menu::new();
        shell.menu.append_section(None, &section);
        add_menu_action(
            group,
            "Delete",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.delete_from_view()
            }),
            &section,
            Some(&i18n("Delete")),
            Some("shell"),
            None,
        );

        shell.menu.append_section(None, &shell.module_menu);
    }

    fn dispatch(&self, e: Event) {
        use Event::*;
        match e {
            ModuleActivated(ref name) => self.module_activated(name),
            ModuleDeactivated(ref name) => self.module_deactivated(name),
        }
    }

    pub fn image_list_store(&self) -> Rc<ImageListStore> {
        self.selection_controller.list_store().0.clone()
    }

    pub fn on_lib_notification(&self, ln: &LibNotification) {
        self.gridview.on_lib_notification(ln, self.client.client());
        self.darkroom.on_lib_notification(ln);
        self.mapm.on_lib_notification(ln);
        self.selection_controller
            .on_lib_notification(ln, self.client.thumbnail_cache());
    }

    pub fn action_edit_delete(&self) {
        self.selection_controller.move_to_trash();
    }

    fn add_library_module<T: LibraryModule + 'static>(
        &self,
        module: &Rc<T>,
        name: &str,
        label: &str,
    ) {
        self.add(&to_controller(module.clone()));
        let widget = module.widget();
        self.widget.append_page(widget, name, label);
        self.modules
            .borrow_mut()
            .insert(name.to_string(), module.clone());
    }

    pub fn on_content_will_change(&self, content: super::ContentView) {
        self.selection_controller.content_will_change(content);
    }

    fn on_image_selected(&self, id: db::LibraryId) {
        dbg_out!("Selected callback for {}", id);
        if id > 0 {
            self.client.client().request_metadata(id);
        } else {
            self.gridview.display_none()
        }
    }

    fn on_image_activated(&self, id: db::LibraryId) {
        dbg_out!("Activated callback for {}", id);
        let store = &self.selection_controller.list_store().0;
        if let Some(libfile) = store.file(id) {
            self.darkroom.set_image(Some(libfile));
            self.widget.activate_page("darkroom");
        }
    }

    fn module_activated(&self, name: &str) {
        if let Some(module) = self.modules.borrow().get(name) {
            if let Some(menu) = module.menu() {
                self.module_menu.append_section(None, menu);
            }
            module.set_active(true);
        }
    }

    fn module_deactivated(&self, name: &str) {
        if let Some(module) = self.modules.borrow().get(name) {
            self.module_menu.remove_all();
            module.set_active(false);
        }
    }
}

impl Controller for ModuleShell {
    npc_fwk::controller_imp_imp!(imp_);
}

impl UiController for ModuleShell {
    fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }

    fn actions(&self) -> Option<(&str, &gio::ActionGroup)> {
        Some(("shell", self.action_group.upcast_ref()))
    }
}
