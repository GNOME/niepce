/*
 * niepce - niepce/ui/niepce_application.rs
 *
 * Copyright (C) 2024 Hubert Figuière
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
use std::sync::Arc;

use adw::prelude::*;
use gettextrs::gettext as i18n;

use crate::config;
use crate::niepce::ui::niepce_window::NiepceWindow;
use crate::niepce::ui::PreferencesDialog;

use npc_fwk::toolkit::{
    gtk_utils, AppController, AppControllerSingleton, Configuration, Controller, ControllerImpl,
    DialogController, UiController, UndoHistory, UndoTransaction, WindowController,
};
use npc_fwk::{controller_imp_imp, send_async_any};

pub enum Event {
    FileOpen,
    Preferences,
    About,
    Quit,
}

pub struct NiepceApplication {
    imp_: RefCell<ControllerImpl<Event, ()>>,
    config: Configuration,
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
            Event::FileOpen => (),
            Event::About => {
                let win = self.main_window.borrow();
                let win = win.as_ref().map(|win| win.window());
                action_about(win);
            }
            Event::Preferences => {
                let win = self.main_window.borrow();
                let win = win.as_ref().map(|win| win.window());
                action_preferences(win);
            }
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

    fn config(&self) -> &Configuration {
        &self.config
    }
}

impl AppControllerSingleton for NiepceApplication {}

impl NiepceApplication {
    pub fn instance() -> Arc<dyn AppController> {
        <Self as AppControllerSingleton>::singleton::<Self>()
    }

    pub fn new() -> Arc<NiepceApplication> {
        let undo_history = UndoHistory::default();
        let config_path = Configuration::make_config_path(config::PACKAGE);
        let config = Configuration::from_file(config_path);
        let gtkapp = adw::Application::new(Some(config::APP_ID), gio::ApplicationFlags::FLAGS_NONE);
        let app = Arc::new(NiepceApplication {
            imp_: RefCell::default(),
            config,
            undo_history,
            app: gtkapp,
            main_window: RefCell::default(),
        });
        <Self as AppControllerSingleton>::create(app.clone());

        // This will panic for there is no default display.
        let theme = gtk4::IconTheme::for_display(&gdk4::Display::default().unwrap());
        theme.add_resource_path("/net/figuiere/Niepce/pixmaps");
        theme.add_resource_path("/net/figuiere/Niepce/icons");

        app.app
            .connect_activate(glib::clone!(@weak app => move |_| {
                let win = app.main_window.borrow();
                if let Some(win) = win.clone() {
                    win.window().present();
                }
            }));
        app.app.connect_startup(glib::clone!(@weak app => move |_| {
            app.on_startup()
        }));
        <Self as AppControllerSingleton>::start(&app);

        app
    }

    pub fn main(&self) {
        self.app.run();
    }

    fn on_startup(&self) {
        self.init_actions();

        let window = NiepceWindow::new(self.app.upcast_ref());
        self.main_window.replace(Some(window.clone()));
        window.widget();
        window.on_open_catalog();
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
}

fn action_about(parent: Option<&gtk4::Window>) {
    let dlg = adw::AboutWindow::new();
    dlg.set_application_name("Niepce Digital");
    dlg.set_version(config::VERSION);
    dlg.set_application_icon(config::APP_ID);
    dlg.set_license_type(gtk4::License::Gpl30);
    dlg.set_comments(&i18n("A digital photo application."));
    dlg.set_transient_for(parent);
    dlg.present();
}

fn action_preferences(parent: Option<&gtk4::Window>) {
    let dialog = PreferencesDialog::new();
    dialog.run_modal(parent, |_| {});
}
