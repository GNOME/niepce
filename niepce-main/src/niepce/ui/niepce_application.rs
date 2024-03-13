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

use adw::prelude::*;
use glib::translate::*;
use gettextrs::gettext as i18n;

use crate::config;
use crate::niepce::ui::PreferencesDialog;

use npc_fwk::toolkit::DialogController;

pub unsafe fn action_about(parent: *mut crate::ffi::GtkWindow) {
    let parent: gtk4::Window = from_glib_none(parent as *mut gtk4::ffi::GtkWindow);
    let dlg = adw::AboutWindow::new();
    dlg.set_application_name("Niepce Digital");
    dlg.set_version(config::VERSION);
    dlg.set_application_icon(config::APP_ID);
    dlg.set_license_type(gtk4::License::Gpl30);
    dlg.set_comments(&i18n("A digital photo application."));
    dlg.set_transient_for(Some(&parent));
    dlg.present();
}

pub unsafe fn action_preferences(parent: *mut crate::ffi::GtkWindow) {
    let dialog = PreferencesDialog::new();
    let parent: gtk4::Window = from_glib_none(parent as *mut gtk4::ffi::GtkWindow);
    dialog.run_modal(Some(&parent), |_| {});
}
