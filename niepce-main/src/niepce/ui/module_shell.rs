/*
 * niepce - niepce/ui/module_shell.rs
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

use gettextrs::gettext;
use glib::Cast;
use gtk4::prelude::*;

use super::{
    GridViewModuleProxy, ImageListStore, LibraryModule, ModuleShellWidget, SelectionController,
};
use crate::libraryclient::{ClientInterface, LibraryClientHost};
use npc_engine::db;
use npc_engine::library::notification::LibNotification;
use npc_fwk::toolkit::gtk_utils::add_menu_action;
use npc_fwk::toolkit::{to_controller, Controller, ControllerImpl, UiController};
use npc_fwk::{dbg_out, err_out, on_err_out};

enum Event {
    ModuleActivated(String),
    ModuleDeactivated(String),
}

pub struct ModuleShell {
    imp_: RefCell<ControllerImpl>,
    tx: glib::Sender<Event>,
    widget: ModuleShellWidget,
    action_group: gio::SimpleActionGroup,
    pub selection_controller: Rc<SelectionController>,
    // currently a proxy that will bridge the C++ implementation
    gridview: Rc<GridViewModuleProxy>,
    menu: gio::Menu,
    module_menu: gio::Menu,
    client: Rc<LibraryClientHost>,
    modules: RefCell<HashMap<String, Rc<dyn LibraryModule>>>,
}

impl ModuleShell {
    pub fn new(client_host: &Rc<LibraryClientHost>) -> Rc<ModuleShell> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
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
                client_host.ui_provider(),
            )),
            selection_controller,
            menu,
            module_menu: gio::Menu::new(),
            client: client_host.clone(),
            modules: RefCell::new(HashMap::default()),
        });

        rx.attach(
            None,
            glib::clone!(@strong shell => move |e| {
                shell.dispatch(e);
                glib::Continue(true)
            }),
        );

        shell.add(&to_controller(shell.selection_controller.clone()));
        shell
            .widget
            .insert_action_group("shell", Some(&shell.action_group));

        let group = shell.action_group.upcast_ref::<gio::ActionMap>();
        add_menu_action(
            group,
            "PrevImage",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.select_previous()
            }),
            &shell.menu,
            Some(&gettext("Back")),
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
            Some(&gettext("Forward")),
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
            Some(&gettext("Rotate Left")),
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
            Some(&gettext("Rotate Right")),
            Some("shell"),
            Some("bracketright"),
        );

        let section = gio::Menu::new();
        shell.menu.append_section(None, &section);
        let submenu = gio::Menu::new();
        section.append_submenu(Some(&gettext("Set Label")), &submenu);

        add_menu_action(
            group,
            "SetLabel6",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_label(1)
            }),
            &submenu,
            Some(&gettext("Label 6")),
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
            Some(&gettext("Label 7")),
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
            Some(&gettext("Label 8")),
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
            Some(&gettext("Label 9")),
            Some("shell"),
            Some("<Primary>9"),
        );

        let submenu = gio::Menu::new();
        section.append_submenu(Some(&gettext("Set Rating")), &submenu);
        add_menu_action(
            group,
            "SetRating0",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_rating(0)
            }),
            &submenu,
            Some(&gettext("Unrated")),
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
            Some(&gettext("Rating 1")),
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
            Some(&gettext("Rating 2")),
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
            Some(&gettext("Rating 3")),
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
            Some(&gettext("Rating 4")),
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
            Some(&gettext("Rating 5")),
            Some("shell"),
            Some("<Primary>5"),
        );

        let submenu = gio::Menu::new();
        section.append_submenu(Some(&gettext("Set Flag")), &submenu);
        add_menu_action(
            group,
            "SetFlagReject",
            glib::clone!(
            @weak shell.selection_controller as selection_controller => move |_, _| {
                selection_controller.set_flag(-1)
            }),
            &submenu,
            Some(&gettext("Flag as Rejected")),
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
            Some(&gettext("Unflagged")),
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
            Some(&gettext("Flag as Pick")),
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
            Some(&gettext("Write metadata")),
            Some("shell"),
            None,
        );

        shell.menu.append_section(None, &shell.module_menu);
        shell.widget.menu_button().set_menu_model(Some(&shell.menu));

        shell.add_library_module(&shell.gridview, "grid", &gettext("Catalog"));
        shell.selection_controller.add_selectable(&shell.gridview);

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

        //shell.darkroom = darkroom_module_new();
        //shell.add_library_module(shell.darkroom, "darkroom", &gettext("Darkroom"));
        //shell.mapm = darkroom_module_new();
        //shell.add_library_module(shell.mapm, "map", &gettext("Map"));

        shell.widget.connect(
            "activated",
            true,
            glib::clone!(@strong tx => move |value| {
                let name = value[1].get::<&str>().expect("Failed to convert callback parameter");
                on_err_out!(tx.send(Event::ModuleActivated(name.to_string())));
                None
            }),
        );
        let tx = shell.tx.clone();
        shell.widget.connect(
            "deactivated",
            true,
            glib::clone!(@strong tx => move |value| {
                let name = value[1].get::<&str>().expect("Failed to convert callback parameter");
                on_err_out!(tx.send(Event::ModuleDeactivated(name.to_string())));
                None
            }),
        );
        shell
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
        // XXX
        //mapm.on_lib_notification(ln)
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

    pub fn on_content_will_change(&self) {
        self.selection_controller.content_will_change();
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
        if let Some(_libfile) = store.file(id) {
            // XXX
            // self.darkroom.set_image(libfile)
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
    fn imp(&self) -> Ref<'_, ControllerImpl> {
        self.imp_.borrow()
    }

    fn imp_mut(&self) -> RefMut<'_, ControllerImpl> {
        self.imp_.borrow_mut()
    }

    fn on_ready(&self) {}
}

impl UiController for ModuleShell {
    fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref()
    }

    fn actions(&self) -> Option<(&str, &gio::ActionGroup)> {
        Some(("shell", self.action_group.upcast_ref()))
    }
}
