/*
 * niepce - niepce/ui/dialogs/requestnewfolder.rs
 *
 * Copyright (C) 2017-2021 Hubert Figuière
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
use gtk::prelude::*;
use gtk::{Dialog, Entry, Label};

use npc_engine::libraryclient::{ClientInterface, LibraryClientWrapper};

/// # Safety
/// Use raw pointers.
#[no_mangle]
pub unsafe extern "C" fn dialog_request_new_folder(
    client: &mut LibraryClientWrapper,
    parent: *mut gtk_sys::GtkWindow,
) {
    let parent = gtk::Window::from_glib_none(parent);
    let dialog = Dialog::with_buttons(
        Some("New folder"),
        Some(&parent),
        gtk::DialogFlags::MODAL,
        &[
            (&gettext("OK"), gtk::ResponseType::Ok),
            (&gettext("Cancel"), gtk::ResponseType::Cancel),
        ],
    );
    let label = Label::with_mnemonic(gettext("Folder _name:").as_str());
    dialog.content_area().pack_start(&label, true, false, 4);
    let entry = Entry::new();
    entry.set_text("foobar");
    entry.add_mnemonic_label(&label);
    dialog.content_area().pack_end(&entry, true, false, 4);

    dialog.content_area().show_all();
    let cancel = dialog.run() != gtk::ResponseType::Ok;
    let folder_name = entry.text();
    dialog.destroy();
    if !cancel {
        client
            .unwrap_mut()
            .create_folder(folder_name.to_string(), None);
    }
}
