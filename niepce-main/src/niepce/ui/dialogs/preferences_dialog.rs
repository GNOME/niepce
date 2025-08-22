/*
 * niepce - niepce/ui/preferences_dialog.rs
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

use std::rc::Rc;

use adw::prelude::*;
use npc_fwk::{adw, glib, gtk4};

use npc_fwk::controller_imp_imp;
use npc_fwk::toolkit::{
    AppController, Controller, ControllerImplCell, DialogController, UiController,
};

use crate::NiepceApplication;

pub enum Event {
    Close,
}

pub struct PreferencesDialog {
    imp_: ControllerImplCell<Event, ()>,
    dialog: adw::Window,
}

impl Controller for PreferencesDialog {
    type InMsg = Event;
    type OutMsg = ();

    controller_imp_imp!(imp_);

    fn dispatch(&self, msg: Event) {
        match msg {
            Event::Close => self.close(),
        }
    }
}

impl UiController for PreferencesDialog {
    fn widget(&self) -> &gtk4::Widget {
        self.dialog.upcast_ref()
    }
}

impl DialogController for PreferencesDialog {
    fn dialog(&self) -> &adw::Window {
        &self.dialog
    }
}

impl PreferencesDialog {
    pub fn new(app: &NiepceApplication) -> Rc<PreferencesDialog> {
        let builder = gtk4::Builder::from_resource("/net/figuiere/Niepce/ui/preferences_dialog.ui");
        get_widget!(builder, adw::Window, preferences);
        get_widget!(builder, adw::SwitchRow, reopen_checkbutton);
        get_widget!(builder, adw::SwitchRow, write_xmp_checkbutton);
        get_widget!(builder, adw::SwitchRow, dark_theme_checkbox);

        let cfg = &app.config();

        cfg.to_switchrow(&reopen_checkbutton, "reopen_last_catalog", "0");
        reopen_checkbutton.connect_active_notify(glib::clone!(
            #[weak]
            cfg,
            move |w| {
                cfg.from_switchrow(w, "reopen_last_catalog");
            }
        ));

        cfg.to_switchrow(&write_xmp_checkbutton, "write_xmp_automatically", "0");
        write_xmp_checkbutton.connect_active_notify(glib::clone!(
            #[weak]
            cfg,
            move |w| {
                cfg.from_switchrow(w, "write_xmp_automatically");
            }
        ));

        cfg.to_switchrow(&dark_theme_checkbox, "ui_dark_theme", "0");
        dark_theme_checkbox.connect_active_notify(glib::clone!(
            #[weak]
            cfg,
            move |w| {
                cfg.from_switchrow(w, "ui_dark_theme");
            }
        ));

        let ctrl = Rc::new(PreferencesDialog {
            imp_: ControllerImplCell::default(),
            dialog: preferences,
        });

        <Self as DialogController>::start(&ctrl);
        ctrl
    }
}
