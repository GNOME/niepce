/*
 * niepce - niepce/ui/niepce_application.rs
 *
 * Copyright (C) 2024-2025 Hubert Figuière
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
use std::ffi::CString;
use std::rc::Rc;
use std::sync::{Arc, RwLock, Weak};

use adw::prelude::*;
use gettextrs::gettext as i18n;
use npc_fwk::{adw, gdk4, gio, glib, gtk4};

use crate::config;
use crate::niepce::ui::PreferencesDialog;
use crate::niepce::ui::niepce_window::NiepceWindow;
use npc_fwk::base::Moniker;

use npc_fwk::toolkit::{
    AppController, Configuration, Controller, ControllerImplCell, DialogController, UiController,
    UndoHistory, UndoTransaction, WindowController, WindowSize, gtk_utils,
};
use npc_fwk::{controller_imp_imp, err_out, send_async_any};

pub enum Event {
    FileOpen,
    Preferences,
    About,
    Quit,
}

pub struct NiepceApplication {
    imp_: ControllerImplCell<Event, ()>,
    this: RwLock<Weak<NiepceApplication>>,
    config: Arc<Configuration>,
    undo_history: UndoHistory,
    app: adw::Application,
    main_window: RefCell<Option<Rc<NiepceWindow>>>,
}

impl Controller for NiepceApplication {
    type InMsg = Event;
    type OutMsg = ();

    controller_imp_imp!(imp_);

    fn dispatch(&self, msg: Event) {
        match msg {
            Event::FileOpen => {
                if let Some(win) = self.main_window.borrow().as_ref() {
                    win.prompt_open_catalog();
                }
            }
            Event::About => self.action_about(),
            Event::Preferences => self.action_preferences(),
            Event::Quit => (),
        }
    }
}

impl AppController for NiepceApplication {
    fn begin_undo(&self, transaction: UndoTransaction) {
        self.undo_history.add(transaction);
    }

    fn undo_history(&self) -> &UndoHistory {
        &self.undo_history
    }

    fn config(&self) -> Arc<Configuration> {
        self.config.clone()
    }
}

impl NiepceApplication {
    pub fn new() -> Arc<NiepceApplication> {
        let undo_history = UndoHistory::default();
        let config_path = Configuration::make_config_path(config::PACKAGE);
        let config = Arc::new(Configuration::from_file(config_path));
        let gtkapp = adw::Application::new(Some(config::APP_ID), gio::ApplicationFlags::default());
        let app = Arc::new(NiepceApplication {
            imp_: ControllerImplCell::default(),
            this: RwLock::new(Weak::new()),
            config,
            undo_history,
            app: gtkapp,
            main_window: RefCell::default(),
        });

        let this = Arc::downgrade(&app);
        *app.this.write().unwrap() = this;

        // This will panic for there is no default display.
        let theme = gtk4::IconTheme::for_display(&gdk4::Display::default().unwrap());
        theme.add_resource_path("/net/figuiere/Niepce/pixmaps");
        theme.add_resource_path("/net/figuiere/Niepce/icons");

        app.app.connect_activate(glib::clone!(
            #[weak]
            app,
            move |_| {
                let win = app.main_window.borrow();
                if let Some(win) = win.clone() {
                    win.window().present();
                }
            }
        ));
        app.app.connect_startup(glib::clone!(
            #[weak]
            app,
            move |_| app.on_startup()
        ));
        <Self as AppController>::start(&app);

        app
    }

    pub const NIEPCE_OPEN_ENV: &str = "NIEPCE_OPEN";

    /// Reopen the app with a specific catalog.
    /// This literally relaunch the executable.
    pub fn reopen_with(catalog_path: &str) {
        let catalog = Moniker::from(catalog_path);
        unsafe { std::env::set_var(Self::NIEPCE_OPEN_ENV, catalog.to_string()) };
        let self_path = CString::new(
            std::env::current_exe()
                .expect("Coudln't get current exe")
                .as_os_str()
                .as_encoded_bytes(),
        )
        .unwrap();
        nix::unistd::execv(&self_path, &[&self_path]).expect("execv failed");
    }

    pub fn weak(&self) -> Weak<Self> {
        self.this.read().unwrap().clone()
    }

    /// Return the toolkit application.
    pub fn app(&self) -> &adw::Application {
        &self.app
    }

    pub fn main(&self) {
        self.app.run();
    }

    fn on_startup(&self) {
        self.init_actions();

        if let Some(this) = Weak::upgrade(&self.weak()) {
            let window = NiepceWindow::new(&this);
            self.main_window.replace(Some(window.clone()));
            window.widget();
            window.on_open_catalog();
        } else {
            err_out!("No more application!");
        }
    }

    fn init_actions(&self) {
        let group = self.app.upcast_ref::<gio::ActionMap>();
        let sender = self.sender();
        gtk_utils::add_action(
            group,
            "OpenCatalog",
            move |_, _| send_async_any!(Event::FileOpen, sender),
            Some("app"),
            Some("<Primary>o"),
        );
        let sender = self.sender();
        gtk_utils::add_action(
            group,
            "Preferences",
            move |_, _| send_async_any!(Event::Preferences, sender),
            None,
            None,
        );
        let sender = self.sender();
        gtk_utils::add_action(
            group,
            "Help",
            move |_, _| send_async_any!(Event::About, sender),
            None,
            None,
        );
        let sender = self.sender();
        gtk_utils::add_action(
            group,
            "About",
            move |_, _| send_async_any!(Event::About, sender),
            None,
            None,
        );
        let sender = self.sender();
        gtk_utils::add_action(
            group,
            "Quit",
            move |_, _| send_async_any!(Event::Quit, sender),
            Some("app"),
            Some("<Primary>q"),
        );
    }

    fn action_about(&self) {
        let win = self.main_window.borrow();
        let win = win.as_ref().map(|win| win.window());

        let dlg = adw::AboutWindow::new();
        dlg.set_application_name("Niepce Digital");
        dlg.set_version(config::VERSION);
        dlg.set_application_icon(config::APP_ID);
        dlg.set_license_type(gtk4::License::Gpl30);
        dlg.set_comments(&i18n("A digital photo application."));
        dlg.set_transient_for(win);
        dlg.present();
    }

    fn action_preferences(&self) {
        let win = self.main_window.borrow();
        let win = win.as_ref().map(|win| win.window());

        let dialog = PreferencesDialog::new(self);
        dialog.run_modal(win, WindowSize::Default, |_| {});
    }
}
