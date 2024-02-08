/*
 * niepce - niepce/ui/niepce_window.rs
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
use std::rc::Rc;

use adw::prelude::*;
use gettextrs::gettext as i18n;
use gtk4::gio;
use gtk4::glib::translate::*;
use once_cell::unsync::OnceCell;

use npc_engine::db;
use npc_engine::library::notification::LibNotification;
use npc_engine::libraryclient::{ClientInterface, LibraryClientHost};
use npc_fwk::base::rgbcolour::RgbColour;
use npc_fwk::base::Moniker;
use npc_fwk::toolkit::{self, Controller, ControllerImpl, UiController, WindowController};
use npc_fwk::{dbg_out, err_out, on_err_out};

use super::film_strip_controller::FilmStripController;
use super::module_shell::ModuleShell;
use super::workspace_controller::WorkspaceController;
use crate::{config, NotificationCenter};

enum Event {
    Delete,
    ToggleToolsVisible,
    EditLabels,
    OpenCatalog(std::path::PathBuf),
    NewLibraryCreated,
    AddedLabel(db::Label),
    LabelChanged(db::Label),
    LabelDeleted(db::LibraryId),
    DatabaseReady,
    DatabaseNeedUpgrade(i32),
}

struct Widgets {
    widget_: gtk4::Widget,
    vbox: gtk4::Box,
    hbox: gtk4::Paned,
    main_menu: gio::Menu,
    header: gtk4::HeaderBar,

    notif_center: NotificationCenter,
}

struct ShellWidgets {
    _workspace: Rc<WorkspaceController>,
    shell: Rc<ModuleShell>,
    _filmstrip: Rc<FilmStripController>,
    _statusbar: gtk4::Label,
}

struct NiepceWindow {
    imp_: RefCell<ControllerImpl>,
    tx: npc_fwk::toolkit::Sender<Event>,
    window: gtk4::ApplicationWindow,
    libraryclient: RefCell<Option<Rc<LibraryClientHost>>>,
    configuration: RefCell<Option<Rc<toolkit::Configuration>>>,

    widgets: OnceCell<Widgets>,
    shell_widgets: OnceCell<ShellWidgets>,
}

impl Controller for NiepceWindow {
    type InMsg = Event;

    npc_fwk::controller_imp_imp!(imp_);

    fn dispatch(&self, e: Event) {
        use Event::*;

        match e {
            Delete => {
                if let Some(widgets) = self.shell_widgets.get() {
                    widgets.shell.action_edit_delete()
                }
            }
            EditLabels => self.on_action_edit_labels(),
            ToggleToolsVisible => {
                // XXX todo
            }
            OpenCatalog(catalog) => {
                self.open_catalog(&catalog);
            }
            NewLibraryCreated => self.create_initial_labels(),
            DatabaseReady => {
                self.create_module_shell();
                dbg_out!("Database ready.");
            }
            DatabaseNeedUpgrade(v) => {
                dbg_out!("Database need upgrade {}.", v);
                let dialog = npc_fwk::toolkit::confirm::request(
                    &i18n("Catalog needs to be upgraded"),
                    &i18n("The catalog will be upgraded to the latest version. A copy of the old version will be save. Upgrade?"),
                    Some(i18n("_Upgrade")),
                    false,
                    Some(self.window()),
                );
                dialog.connect_response(
                    None,
                    glib::clone!(@strong self.libraryclient as client => move |dialog, response| {
                        if response == "confirm" {
                            if let Some(client_host) = client.borrow().as_ref() {
                                client_host.client().upgrade_library_from_sync(v);
                            }
                        }
                        dialog.destroy();
                    }),
                );
                dialog.present();
            }
            AddedLabel(label) => {
                if let Some(host) = &*self.libraryclient.borrow() {
                    host.ui_provider().add_label(&label);
                }
            }
            LabelChanged(label) => {
                if let Some(host) = &*self.libraryclient.borrow() {
                    host.ui_provider().update_label(&label);
                }
            }
            LabelDeleted(id) => {
                if let Some(host) = &*self.libraryclient.borrow() {
                    host.ui_provider().delete_label(id);
                }
            }
        }
    }
}

impl UiController for NiepceWindow {
    fn widget(&self) -> &gtk4::Widget {
        &self
            .widgets
            .get_or_init(|| {
                let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
                let hbox = gtk4::Paned::new(gtk4::Orientation::Horizontal);
                let header = gtk4::HeaderBar::new();
                header.set_show_title_buttons(true);

                let main_menu = gio::Menu::new();
                let menu_btn = gtk4::MenuButton::new();
                menu_btn.set_direction(gtk4::ArrowType::None);
                menu_btn.set_menu_model(Some(&main_menu));
                header.pack_end(&menu_btn);

                let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
                button_box.add_css_class("linked");
                let undo_button = gtk4::Button::with_label(&i18n("Undo"));
                undo_button.set_icon_name("edit-undo-symbolic");
                undo_button.set_action_name(Some("win.Undo"));
                let redo_button = gtk4::Button::with_label(&i18n("Redo"));
                redo_button.set_icon_name("edit-redo-symbolic");
                redo_button.set_action_name(Some("win.Redo"));
                button_box.append(&undo_button);
                button_box.append(&redo_button);
                header.pack_start(&button_box);

                let import_button = gtk4::Button::with_label(&i18n("Import..."));
                import_button.set_action_name(Some("workspace.Import"));
                header.pack_start(&import_button);

                if config::PROFILE == "Devel" {
                    self.window().add_css_class("devel");
                }

                self.window().set_titlebar(Some(&header));
                self.window().set_size_request(600, 400);

                // Main hamburger menu
                let section = gio::Menu::new();
                main_menu.append_section(None, &section);
                section.append(Some(&i18n("New Catalog...")), Some("app.NewCatalog"));
                section.append(Some(&i18n("Open Catalog...")), Some("app.OpenCatalog"));

                let section = gio::Menu::new();
                main_menu.append_section(None, &section);
                section.append(Some(&i18n("Hide tools")), Some("win.ToggleToolsVisible"));
                section.append(Some(&i18n("Edit Labels...")), Some("win.EditLabels"));
                section.append(Some(&i18n("Preferences...")), Some("app.Preferences"));

                let section = gio::Menu::new();
                main_menu.append_section(None, &section);
                section.append(Some(&i18n("Help")), Some("app.Help"));
                section.append(Some(&i18n("About")), Some("app.About"));

                self.window.set_child(Some(&vbox));

                let tx = self.tx.clone();
                let notif_center = NotificationCenter::new();
                notif_center.signal_notify.connect(move |n| {
                    Self::lib_notification(&tx, n);
                });

                self.actions();

                Widgets {
                    widget_: vbox.clone().upcast(),
                    vbox,
                    hbox,
                    main_menu,
                    header,
                    notif_center,
                }
            })
            .widget_
    }

    fn actions(&self) -> Option<(&str, &gio::ActionGroup)> {
        let tx = self.tx.clone();
        let group: &gio::ActionMap = self.window.upcast_ref();
        action!(
            group,
            "Close",
            glib::clone!(@strong self.window as window => move |_, _| {
                window.close()
            })
        );

        npc_fwk::toolkit::create_undo_action(group);
        npc_fwk::toolkit::create_redo_action(group);

        action!(group, "Cut", move |_, _| {});
        action!(group, "Copy", move |_, _| {});
        action!(group, "Paste", move |_, _| {});
        npc_fwk::sending_action!(group, "Delete", tx, Event::Delete);

        npc_fwk::sending_action!(group, "ToggleToolsVisible", tx, Event::ToggleToolsVisible);
        npc_fwk::sending_action!(group, "EditLabels", tx, Event::EditLabels);

        Some(("win", self.window.upcast_ref()))
    }
}

impl WindowController for NiepceWindow {
    fn window(&self) -> &gtk4::Window {
        self.window.upcast_ref()
    }
}

impl NiepceWindow {
    pub fn new(app: &gtk4::Application) -> Rc<NiepceWindow> {
        let (tx, rx) = npc_fwk::toolkit::channel();

        let ctrl = Rc::new(NiepceWindow {
            imp_: RefCell::new(ControllerImpl::default()),
            tx,
            window: gtk4::ApplicationWindow::new(app),
            libraryclient: RefCell::new(None),
            configuration: RefCell::new(None),

            widgets: OnceCell::new(),
            shell_widgets: OnceCell::new(),
        });

        npc_fwk::toolkit::channels::receiver_attach(
            rx,
            glib::clone!(@strong ctrl => move |e| {
                ctrl.dispatch(e);
            }),
        );

        ctrl
    }

    fn create_initial_labels(&self) {
        dbg_out!("create initial labels");
        let client = self.libraryclient.borrow();
        if let Some(ref libraryclient) = *client {
            let client = libraryclient.client();
            client.create_label(
                i18n("Label 1"),
                RgbColour::new(55769, 9509, 4369).to_string(),
            );
            client.create_label(
                i18n("Label 2"),
                RgbColour::new(24929, 55769, 4369).to_string(),
            );
            client.create_label(
                i18n("Label 3"),
                RgbColour::new(4369, 50629, 55769).to_string(),
            );
            client.create_label(
                i18n("Label 4"),
                RgbColour::new(35209, 4369, 55769).to_string(),
            );
            client.create_label(
                i18n("Label 5"),
                RgbColour::new(35209, 4369, 55769).to_string(),
            );
        }
    }

    fn lib_notification(tx: &npc_fwk::toolkit::Sender<Event>, n: LibNotification) -> bool {
        use LibNotification::*;

        match n {
            LibCreated => npc_fwk::send_async_local!(Event::NewLibraryCreated, tx),
            AddedLabel(label) => npc_fwk::send_async_local!(Event::AddedLabel(label), tx),
            LabelChanged(label) => npc_fwk::send_async_local!(Event::LabelChanged(label), tx),
            LabelDeleted(label_id) => npc_fwk::send_async_local!(Event::LabelDeleted(label_id), tx),
            DatabaseReady => npc_fwk::send_async_local!(Event::DatabaseReady, tx),
            DatabaseNeedUpgrade(version) => {
                npc_fwk::send_async_local!(Event::DatabaseNeedUpgrade(version), tx)
            }
            _ => (),
        }
        true
    }

    fn set_title(&self, title: &str) {
        if let Some(widgets) = self.widgets.get() {
            let title = format!("{} - {}", i18n("Niepce Digital"), title);
            let label = gtk4::Label::new(Some(&title));
            widgets.header.set_title_widget(Some(&label));
        }
    }

    /// Opening a library has been requested
    fn on_open_catalog(&self) {
        let app = npc_fwk::ffi::Application_app();
        let cfg = &app.config().cfg;
        let reopen = cfg.value("reopen_last_catalog", "0");
        let cat_moniker = if reopen == "1" {
            cfg.value("last_open_catalog", "")
        } else {
            "".into()
        };
        if cat_moniker.is_empty() {
            self.prompt_open_catalog();
        } else {
            let moniker = Moniker::from(&cat_moniker);
            dbg_out!("Last catalog is {}", &cat_moniker);
            if !self.open_catalog(&std::path::PathBuf::from(moniker.path())) {
                err_out!("catalog {:?} cannot be open. Prompting.", &moniker);
                self.prompt_open_catalog();
            }
        }
    }

    fn prompt_open_catalog(&self) {
        let dialog = gtk4::FileChooserDialog::new(
            Some(&i18n("Open catalog")),
            Some(self.window()),
            gtk4::FileChooserAction::SelectFolder,
            &[
                (&i18n("Cancel"), gtk4::ResponseType::Cancel),
                (&i18n("Open"), gtk4::ResponseType::Accept),
            ],
        );
        dialog.set_modal(true);
        dialog.set_create_folders(true);
        dialog.connect_response(
            glib::clone!(@strong dialog, @strong self.tx as tx => move |_, response| {
                if response == gtk4::ResponseType::Accept {
                    if let Some(catalog_to_create) = dialog.file().and_then(|f| f.path()) {
                        let catalog_to_create2 = catalog_to_create.clone();
                        npc_fwk::send_async_local!(Event::OpenCatalog(catalog_to_create2), tx);
                        let app = npc_fwk::ffi::Application_app();
                        let cfg = &app.config().cfg;
                        cfg.set_value("last_open_catalog", &catalog_to_create.to_string_lossy());
                    }
                    dialog.destroy();
                }
            }),
        );
        dialog.present();
    }

    /// Actually open a catalog
    fn open_catalog(&self, catalog: &std::path::Path) -> bool {
        dbg_out!("opening catalog {:?}", catalog);
        // This is a fatal logic error. Everything should have been initialized
        let channel = self
            .widgets
            .get()
            .map(|w| w.notif_center.channel())
            .unwrap();
        let moniker = Moniker::from(&*catalog.to_string_lossy());
        self.libraryclient
            .replace(Some(Rc::new(LibraryClientHost::new(&moniker, channel))));
        self.set_title(&moniker.to_string());

        let mut config_path = catalog.to_path_buf();
        config_path.push("config.ini");
        self.configuration
            .replace(Some(Rc::new(toolkit::Configuration::from_file(
                config_path,
            ))));

        true
    }

    fn create_module_shell(&self) {
        dbg_out!("creating module shell");

        let client = self.libraryclient.borrow();

        if let Some(c) = self.libraryclient.borrow().as_ref() {
            c.client().get_all_labels();
        }

        let module_shell = ModuleShell::new(client.as_ref().unwrap());
        let module_widget = module_shell.widget();

        if let Some(notif_center) = self.widgets.get().map(|w| &w.notif_center) {
            let module_shell = module_shell.clone();
            notif_center
                .signal_notify
                .connect(move |ln| module_shell.on_lib_notification(&ln));
        }

        // We really expect cfg to be available
        let configuration = self.configuration.borrow();
        let cfg = configuration.as_ref().unwrap();
        let client = client.as_ref().unwrap().client();
        let workspace = WorkspaceController::new(cfg.clone(), client);
        if let Some(actions) = workspace.actions() {
            self.window.insert_action_group(actions.0, Some(actions.1));
        }
        workspace.selection_changed.connect(
            glib::clone!(@weak module_shell => move |content| module_shell.on_content_will_change(content)),
        );
        if let Some(notif_center) = self.widgets.get().map(|w| &w.notif_center) {
            let workspace = workspace.clone();
            notif_center
                .signal_notify
                .connect(move |ln| workspace.on_lib_notification(&ln));
        }

        let hbox = &self.widgets.get().as_ref().unwrap().hbox;
        hbox.set_wide_handle(true);
        hbox.set_start_child(Some(workspace.widget()));
        hbox.set_end_child(Some(module_widget));
        // set the databinder for the `"workspace_splitter"` bound to hbox `position`

        let filmstrip = FilmStripController::new(module_shell.image_list_store());
        let vbox = &self.widgets.get().as_ref().unwrap().vbox;
        vbox.append(hbox);
        vbox.append(filmstrip.widget());

        let statusbar = gtk4::Label::new(Some(&i18n("Ready")));
        statusbar.set_xalign(0.0);
        vbox.append(&statusbar);

        filmstrip.grid_view().connect_activate(glib::clone!(
        @weak module_shell.selection_controller.handler as handler => move |_, pos| {
            handler.activated(pos)
        }));

        // `ShellWidget` isn't `Debug` so we can't unwrap.
        let _ = self.shell_widgets.set(ShellWidgets {
            _workspace: workspace.clone(),
            shell: module_shell,
            _filmstrip: filmstrip,
            _statusbar: statusbar,
        });

        workspace.startup();
    }

    fn on_action_edit_labels(&self) {
        dbg_out!("edit labels");
        if let Some(ref libclient) = *self.libraryclient.borrow() {
            let editlabel_dialog = crate::ffi::edit_labels_new(libclient);
            let parent: *mut gtk4::ffi::GtkWindow = self.window().to_glib_none().0;
            unsafe {
                editlabel_dialog.run_modal(
                    parent as *mut crate::ffi::GtkWindow,
                    move |p, _| {
                        drop(p);
                    },
                    editlabel_dialog.clone(),
                );
            }
        }
    }
}

/// C++ wrapper around the Rc.
/// Only used for the ffi
pub struct NiepceWindowWrapper(Rc<NiepceWindow>);

/// # Safety
/// Dereference a pointer.
pub unsafe fn niepce_window_new(app: *mut crate::ffi::GtkApplication) -> Box<NiepceWindowWrapper> {
    let app = app as *mut gtk4::ffi::GtkApplication;
    Box::new(NiepceWindowWrapper(NiepceWindow::new(&from_glib_none(app))))
}

impl NiepceWindowWrapper {
    pub fn on_open_catalog(&self) {
        self.0.on_open_catalog();
    }

    pub fn on_ready(&self) {
        self.0.on_ready();
    }

    // Return a GtkWidget
    pub fn widget(&self) -> *mut crate::ffi::GtkWidget {
        let w: *mut gtk4::ffi::GtkWidget = self.0.widget().to_glib_none().0;
        w as *mut crate::ffi::GtkWidget
    }

    // Return a GtkWindow
    pub fn window(&self) -> *mut crate::ffi::GtkWindow {
        let w: *mut gtk4::ffi::GtkWindow = self.0.window().to_glib_none().0;
        w as *mut crate::ffi::GtkWindow
    }

    // Return a GMenu
    pub fn menu(&self) -> *mut crate::ffi::GMenu {
        if let Some(widgets) = self.0.widgets.get() {
            let m: *mut gio::ffi::GMenu = widgets.main_menu.to_glib_none().0;
            m as *mut crate::ffi::GMenu
        } else {
            std::ptr::null_mut()
        }
    }
}
