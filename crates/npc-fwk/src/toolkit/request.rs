/*
 * niepce - npc-fwk/toolkit/request.rs
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

use gettextrs::gettext as i18n;
use gtk4::prelude::*;
use gtk4::{Dialog, Entry, Label};

/// Request a name. On Ok call `action`
///
/// `title` is the dialog title
/// `label` is the label
/// `value` is the optional default name.
pub fn request_name<F: Fn(&str) + 'static>(
    parent: Option<&gtk4::Window>,
    title: &str,
    label: &str,
    value: Option<&str>,
    action: F,
) {
    let dialog = Dialog::with_buttons(
        Some(title),
        parent,
        gtk4::DialogFlags::MODAL,
        &[
            (&i18n("OK"), gtk4::ResponseType::Ok),
            (&i18n("Cancel"), gtk4::ResponseType::Cancel),
        ],
    );
    let label = Label::with_mnemonic(label);
    let content_area = dialog.content_area();
    content_area.set_spacing(12);
    content_area.append(&label);
    let entry = Entry::new();
    if let Some(value) = value {
        entry.set_text(value);
    }
    entry.add_mnemonic_label(&label);
    content_area.append(&entry);

    dialog.set_modal(true);

    dialog.connect_response(glib::clone!(@strong entry => move |dialog, response| {
        let name = entry.text();
        if response == gtk4::ResponseType::Ok {
            action(&name);
        }
        dialog.close();
    }));
    dialog.present();
}
