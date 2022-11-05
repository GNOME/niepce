/*
 * niepce - toolkit/gtk_utils.rs
 *
 * Copyright (C) 2022 Hubert Figuière
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

use gtk4::prelude::*;

pub fn add_menu_action<F>(
    group: &gio::ActionMap,
    name: &str,
    f: F,
    menu: &gio::Menu,
    label: Option<&str>,
    context: Option<&str>,
    accel: Option<&str>,
) -> gio::SimpleAction
where
    F: Fn(&gio::SimpleAction, Option<&glib::Variant>) + 'static,
{
    let action = gio::SimpleAction::new(name, None);
    group.add_action(&action);
    action.connect_activate(f);
    if label.is_some() && context.is_some() {
        let detail = format!("{}.{}", context.unwrap(), name);
        menu.append(label, Some(&detail));
        if let Some(accel) = accel {
            gtk4::Application::default().set_accels_for_action(&detail, &[accel]);
        }
    }

    action
}
