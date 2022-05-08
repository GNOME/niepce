/*
 * niepce - niepce/ui/dialogs/confirm.rs
 *
 * Copyright (C) 2017-2022 Hubert Figuière
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

use libc::c_char;
use std::ffi::CStr;

use glib::translate::*;
use gtk4::prelude::*;
use gtk4::MessageDialog;

/// # Safety
/// Use raw pointers.
#[no_mangle]
pub unsafe extern "C" fn dialog_confirm(
    message: *const c_char,
    parent: *mut gtk4_sys::GtkWindow,
) -> *mut gtk4_sys::GtkMessageDialog {
    let msg = CStr::from_ptr(message).to_string_lossy();
    let parent = gtk4::Window::from_glib_none(parent);
    let dialog = MessageDialog::new(
        Some(&parent),
        gtk4::DialogFlags::MODAL,
        gtk4::MessageType::Question,
        gtk4::ButtonsType::YesNo,
        &*msg,
    );

    dialog.set_modal(true);

    dialog.to_glib_none().0
}
