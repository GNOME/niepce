/*
 * niepce - npc-fwk/toolkit/request.rs
 *
 * Copyright (C) 2017-2024 Hubert Figuière
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

use crate::glib;
use crate::gtk4;
use adw::prelude::*;
use gettextrs::gettext as i18n;
use gtk4::{Entry, Label};

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
    let dialog = adw::Window::new();
    let label = Label::with_mnemonic(label);
    let content_area = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    dialog.set_content(Some(&content_area));

    let header_bar = gtk4::HeaderBar::new();
    let title = Label::with_mnemonic(title);
    header_bar.set_show_title_buttons(false);
    header_bar.set_title_widget(Some(&title));
    content_area.append(&header_bar);
    content_area.append(&label);

    let entry = Entry::new();
    if let Some(value) = value {
        entry.set_text(value);
    }
    entry.add_mnemonic_label(&label);
    content_area.append(&entry);

    let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    content_area.append(&button_box);
    let cancel_button = gtk4::Button::with_label(&i18n("Cancel"));
    button_box.append(&cancel_button);
    cancel_button.connect_clicked(glib::clone!(
        #[strong]
        dialog,
        move |_| {
            dialog.close();
        }
    ));

    let ok_button = gtk4::Button::with_label(&i18n("OK"));
    button_box.append(&ok_button);
    ok_button.connect_clicked(glib::clone!(
        #[strong]
        entry,
        #[strong]
        dialog,
        move |_| {
            let name = entry.text();
            action(&name);

            dialog.close();
        }
    ));

    dialog.set_transient_for(parent);
    dialog.set_modal(true);

    dialog.present();
}
