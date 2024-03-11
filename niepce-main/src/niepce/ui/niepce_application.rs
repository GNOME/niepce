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

use glib::translate::*;

use crate::niepce::ui::PreferencesDialog;

use npc_fwk::toolkit::DialogController;

pub unsafe fn action_preferences(parent: *mut crate::ffi::GtkWindow) {
    let dialog = PreferencesDialog::new();
    let parent: gtk4::Window = from_glib_none(parent as *mut gtk4::ffi::GtkWindow);
    dialog.run_modal(Some(&parent), |_| {});
}
