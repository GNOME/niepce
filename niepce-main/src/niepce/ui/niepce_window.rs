/*
 * niepce - niepce/ui/niepce_window.rs
 *
 * Copyright (C) 2022-2025 Hubert Figuière
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

use std::cell::{OnceCell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;
use gettextrs::gettext as i18n;
use npc_fwk::{adw, gio, glib, gtk4};

use npc_engine::catalog;
use npc_engine::library::CatalogPreferences;
use npc_engine::library::notification::LibNotification;
use npc_engine::libraryclient::{ClientInterface, ClientInterfaceSync, LibraryClientHost};
use npc_fwk::base::Moniker;
use npc_fwk::base::rgbcolour::RgbColour;
use npc_fwk::toolkit::{
    self, AppController, Controller, ControllerImplCell, DialogController, UiController,
    WindowController, WindowSize,
};
use npc_fwk::{dbg_out, err_out};

use super::dialogs::EditLabels;
use super::film_strip_controller::FilmStripController;
use super::module_shell::ModuleShell;
use super::workspace_controller::WorkspaceController;
use crate::NiepceApplication;
use crate::{NotificationCenter, config};

pub enum Event {
    Delete,
    ToggleToolsVisible,
    EditLabels,
    OpenCatalog(std::path::PathBuf),
    NewLibraryCreated,
    AddedLabel(catalog::Label),
    LabelChanged(catalog::Label),
    LabelDeleted(catalog::LibraryId),
    DatabaseReady,
    DatabaseNeedUpgrade(i32),
    InitialisePrefs(Vec<(String, String)>),
    UpdatePrefs(String, String),
}

struct Widgets {
    widget: gtk4::Widget,
    vbox: gtk4::Box,
    hbox: gtk4::Paned,
    header: gtk4::HeaderBar,
    statusbar: gtk4::Label,
    filmstrip: RefCell<Option<gtk4::Widget>>,

    notif_center: NotificationCenter,
}

impl Widgets {
    fn with_sender(tx: toolkit::Sender<<NiepceWindow as Controller>::InMsg>) -> Widgets {
        let builder = gtk4::Builder::from_resource("/net/figuiere/Niepce/ui/niepce_window.ui");
        get_widget!(builder, gtk4::Box, vbox);
        get_widget!(builder, gtk4::Paned, hbox);
        get_widget!(builder, gtk4::Label, statusbar);
        get_widget!(builder, gtk4::HeaderBar, header);

        let notif_center = NotificationCenter::new();
        notif_center.signal_notify.connect(move |n| {
            NiepceWindow::lib_notification(&tx, n);
        });

        Widgets {
            widget: vbox.clone().upcast(),
            vbox,
            hbox,
            header,
            statusbar,
            filmstrip: RefCell::new(None),
            notif_center,
        }
    }

    fn set_workspace(&self, workspace: Option<&gtk4::Widget>) {
        self.hbox.set_start_child(workspace);
    }

    fn set_shell(&self, shell: Option<&gtk4::Widget>) {
        self.hbox.set_end_child(shell);
    }

    fn set_status(&self, status: &str) {
        self.statusbar.set_label(status);
    }

    fn set_filmstrip(&self, filmstrip: &gtk4::Widget) {
        if let Some(old_strip) = &*self.filmstrip.borrow() {
            self.vbox.remove(old_strip);
        }
        self.vbox.insert_child_after(filmstrip, Some(&self.hbox));
        self.filmstrip.replace(Some(filmstrip.clone()));
    }
}

struct ShellWidgets {
    _workspace: Rc<WorkspaceController>,
    shell: Rc<ModuleShell>,
    _filmstrip: Rc<FilmStripController>,
}

pub struct NiepceWindow {
    imp_: ControllerImplCell<Event, ()>,
    app: Arc<NiepceApplication>,
    window: gtk4::ApplicationWindow,
    libraryclient: RefCell<Option<Rc<LibraryClientHost>>>,
    configuration: RefCell<Option<Rc<toolkit::Configuration>>>,

    widgets: OnceCell<Widgets>,
    shell_widgets: OnceCell<ShellWidgets>,
}

impl Controller for NiepceWindow {
    type InMsg = Event;
    type OutMsg = ();

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
                self.on_close();
                NiepceApplication::reopen_with(catalog.to_str().unwrap());
                // We have terminated here.
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
                    &i18n(
                        "The catalog will be upgraded to the latest version. A copy of the old version will be saved. Upgrade?",
                    ),
                    Some(i18n("_Upgrade")),
                    false,
                    Some(self.window()),
                );
                dialog.connect_response(
                    None,
                    glib::clone!(
                        #[strong(rename_to = client)]
                        self.libraryclient,
                        move |dialog, response| {
                            if response == "confirm" {
                                if let Some(client_host) = client.borrow().as_ref() {
                                    client_host.client().upgrade_catalog_from_sync(v);
                                }
                            }
                            dialog.destroy();
                        }
                    ),
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
            InitialisePrefs(prefs) => {
                self.configuration
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .imp()
                    .initialise(&prefs);
                self.load_state(self.configuration.borrow().as_ref().unwrap());
            }
            UpdatePrefs(key, value) => {
                self.configuration
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .imp()
                    .initialise(&[(key, value)]);
            }
        }
    }
}

impl UiController for NiepceWindow {
    fn widget(&self) -> &gtk4::Widget {
        &self
            .widgets
            .get_or_init(|| {
                let widgets = Widgets::with_sender(self.sender());

                if config::PROFILE == "Devel" {
                    self.window().add_css_class("devel");
                }

                self.window().set_titlebar(Some(&widgets.header));
                self.window().set_size_request(600, 400);
                self.window.set_child(Some(&widgets.widget));
                self.actions();

                widgets
            })
            .widget
    }

    fn actions(&self) -> Option<(&str, &gio::ActionGroup)> {
        let tx = self.sender();
        let group: &gio::ActionMap = self.window.upcast_ref();
        action!(
            group,
            "Close",
            glib::clone!(
                #[strong(rename_to = window)]
                self.window,
                move |_, _| window.close()
            )
        );

        npc_fwk::toolkit::create_undo_action(self.app.clone(), group);
        npc_fwk::toolkit::create_redo_action(self.app.clone(), group);

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

    fn on_close(&self) {
        if let Some(client) = self.libraryclient.borrow().as_ref() {
            client.close();
        }
    }

    fn state_key(&self) -> Option<&str> {
        Some("catalog-window")
    }

    fn configuration(&self) -> Option<Rc<toolkit::Configuration>> {
        self.configuration.borrow().clone()
    }
}

impl NiepceWindow {
    pub fn new(app: &Arc<NiepceApplication>) -> Rc<NiepceWindow> {
        let ctrl = Rc::new(NiepceWindow {
            imp_: ControllerImplCell::default(),
            app: app.clone(),
            window: gtk4::ApplicationWindow::new(app.app()),
            libraryclient: RefCell::new(None),
            configuration: RefCell::new(None),

            widgets: OnceCell::new(),
            shell_widgets: OnceCell::new(),
        });

        <Self as Controller>::start(&ctrl);
        <Self as WindowController>::init_state(&ctrl);

        ctrl
    }

    fn create_initial_labels(&self) {
        dbg_out!("create initial labels");
        let client = self.libraryclient.borrow();
        if let Some(ref libraryclient) = *client {
            let client = libraryclient.client();
            client.create_label(i18n("Label 1"), RgbColour::new(55769, 9509, 4369));
            client.create_label(i18n("Label 2"), RgbColour::new(24929, 55769, 4369));
            client.create_label(i18n("Label 3"), RgbColour::new(4369, 50629, 55769));
            client.create_label(i18n("Label 4"), RgbColour::new(35209, 4369, 55769));
            client.create_label(i18n("Label 5"), RgbColour::new(35209, 4369, 55769));
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
            Prefs(prefs) => {
                npc_fwk::send_async_any!(Event::InitialisePrefs(prefs), tx)
            }
            PrefChanged(key, value) => {
                // Treat this like an initialisation
                npc_fwk::send_async_any!(Event::UpdatePrefs(key, value), tx)
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
    pub fn on_open_catalog(&self) {
        let cat_moniker = if let Ok(reopen) = std::env::var(NiepceApplication::NIEPCE_OPEN_ENV) {
            reopen
        } else {
            let cfg = &self.app.config();
            let reopen = cfg.value("reopen_last_catalog", "0");
            if reopen == "1" {
                cfg.value("last_open_catalog", "")
            } else {
                "".into()
            }
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

    /// Request to open a catalog.
    #[allow(deprecated)]
    pub fn prompt_open_catalog(&self) {
        let filter = gtk4::FileFilter::new();
        filter.add_pattern("*.npcat");
        filter.add_mime_type("application/x-niepce-catalog");

        // Can't use FileDialog because it will use the file portal
        // which we don't want because we can't request the whole directory
        let dialog = gtk4::FileChooserDialog::new(
            Some(&i18n("Open catalog")),
            Some(self.window()),
            gtk4::FileChooserAction::Open,
            &[
                (&i18n("Cancel"), gtk4::ResponseType::Cancel),
                (&i18n("Open"), gtk4::ResponseType::Accept),
            ],
        );
        dialog.add_filter(&filter);
        dialog.set_modal(true);
        dialog.set_create_folders(true);
        let tx = self.sender();
        dialog.connect_response(glib::clone!(
            #[weak(rename_to=app)]
            self.app,
            #[strong]
            dialog,
            #[strong]
            tx,
            move |_, response| {
                match response {
                    gtk4::ResponseType::Accept => {
                        if let Some(catalog_to_create) = dialog.file().and_then(|f| f.path()) {
                            let catalog_to_create2 = catalog_to_create.clone();
                            npc_fwk::send_async_local!(Event::OpenCatalog(catalog_to_create2), tx);
                            let cfg = &app.config();
                            cfg.set_value(
                                "last_open_catalog",
                                &catalog_to_create.to_string_lossy(),
                            );
                        }
                        dialog.destroy();
                    }
                    gtk4::ResponseType::Cancel => dialog.destroy(),
                    _ => err_out!("File chooser: unknown response {response}"),
                }
            }
        ));
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
        let client = Rc::new(LibraryClientHost::new(&moniker, channel));
        self.libraryclient.replace(Some(client.clone()));
        self.set_title(&moniker.to_string());

        self.configuration
            .replace(Some(Rc::new(toolkit::Configuration::from_impl(Box::new(
                CatalogPreferences::new(client.client().clone()),
            )))));

        self.configuration.borrow().as_ref().unwrap().imp().start();
        true
    }

    fn create_module_shell(&self) {
        dbg_out!("creating module shell");

        let client_host = self.libraryclient.borrow();
        let client_host = client_host.as_ref().unwrap();

        client_host.client().get_all_labels();

        let module_shell = ModuleShell::new(client_host, self.app.weak().clone());
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
        let client = client_host.client();
        let workspace = WorkspaceController::new(self.app.weak().clone(), cfg.clone(), client);
        if let Some(actions) = workspace.actions() {
            self.window.insert_action_group(actions.0, Some(actions.1));
        }
        workspace.selection_changed.connect(glib::clone!(
            #[weak]
            module_shell,
            move |content| module_shell.on_content_will_change(content)
        ));
        if let Some(notif_center) = self.widgets.get().map(|w| &w.notif_center) {
            let workspace = workspace.clone();
            notif_center
                .signal_notify
                .connect(move |ln| workspace.on_lib_notification(&ln));
        }

        let widgets = self.widgets.get();
        let widgets = widgets.as_ref().unwrap();
        widgets.set_workspace(Some(workspace.widget()));
        widgets.set_shell(Some(module_widget));

        // set the databinder for the `"workspace_splitter"` bound to hbox `position`

        let filmstrip = FilmStripController::new(module_shell.image_list_store());
        widgets.set_filmstrip(filmstrip.widget());
        widgets.set_status(&i18n("Ready"));

        let sender = module_shell.selection_sender();
        filmstrip.grid_view().connect_activate(glib::clone!(
            #[strong]
            sender,
            move |_, pos| {
                npc_fwk::send_async_local!(
                    super::selection_controller::SelectionInMsg::Activated(pos),
                    sender
                )
            }
        ));

        // `ShellWidget` isn't `Debug` so we can't unwrap.
        let _ = self.shell_widgets.set(ShellWidgets {
            _workspace: workspace.clone(),
            shell: module_shell,
            _filmstrip: filmstrip,
        });

        workspace.startup();
    }

    fn on_action_edit_labels(&self) {
        dbg_out!("edit labels");
        if let Some(ref libclient) = *self.libraryclient.borrow() {
            let editlabel_dialog = EditLabels::new(libclient, self.app.weak().clone());
            editlabel_dialog.run_modal(Some(self.window()), WindowSize::Default, move |_| {});
        }
    }
}
