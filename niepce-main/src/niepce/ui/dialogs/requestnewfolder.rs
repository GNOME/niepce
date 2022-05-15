/*
 * niepce - niepce/ui/dialogs/requestnewfolder.rs
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

use gettextrs::gettext;
use glib::translate::*;
use gtk4::prelude::*;
use gtk4::{Dialog, Entry, Label};

use npc_fwk::dbg_out;

use crate::libraryclient::{ClientInterface, LibraryClientWrapper};

/// # Safety
/// Use raw pointers.
#[no_mangle]
pub unsafe extern "C" fn dialog_request_new_folder(
    client: &mut LibraryClientWrapper,
    parent: *mut gtk4_sys::GtkWindow,
) {
    let parent = gtk4::Window::from_glib_none(parent);
    let dialog = Dialog::with_buttons(
        Some("New folder"),
        Some(&parent),
        gtk4::DialogFlags::MODAL,
        &[
            (&gettext("OK"), gtk4::ResponseType::Ok),
            (&gettext("Cancel"), gtk4::ResponseType::Cancel),
        ],
    );
    let label = Label::with_mnemonic(gettext("Folder _name:").as_str());
    let content_area = dialog.content_area();
    content_area.append(&label);
    let entry = Entry::new();
    entry.set_text("foobar");
    entry.add_mnemonic_label(&label);
    content_area.append(&entry);

    dialog.set_modal(true);

    let client = client.client();
    dialog.connect_response(glib::clone!(@strong entry => move |dialog, response| {
        let folder_name = entry.text();
        let cancel = response != gtk4::ResponseType::Ok;
        if !cancel {
            dbg_out!("Create folder {}", &folder_name);
            client.create_folder(folder_name.to_string(), None);
        }
        dialog.close();
    }));
    dialog.show();
}
