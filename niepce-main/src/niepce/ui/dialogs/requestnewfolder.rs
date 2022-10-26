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

use std::sync::Arc;

use gettextrs::gettext;
use gtk4::prelude::*;
use gtk4::{Dialog, Entry, Label};

use npc_fwk::dbg_out;

use crate::libraryclient::{ClientInterface, LibraryClient};

pub fn request(client: Arc<LibraryClient>, parent: Option<&gtk4::Window>) {
    let dialog = Dialog::with_buttons(
        Some("New folder"),
        parent,
        gtk4::DialogFlags::MODAL,
        &[
            (&gettext("OK"), gtk4::ResponseType::Ok),
            (&gettext("Cancel"), gtk4::ResponseType::Cancel),
        ],
    );
    let label = Label::with_mnemonic(&gettext("Folder _name:"));
    let content_area = dialog.content_area();
    content_area.append(&label);
    let entry = Entry::new();
    entry.set_text("foobar");
    entry.add_mnemonic_label(&label);
    content_area.append(&entry);

    dialog.set_modal(true);

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
