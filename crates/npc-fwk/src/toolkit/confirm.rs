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

use crate::gtk4;
use adw::prelude::*;
use gettextrs::gettext as i18n;

/// Confirmation request dialog using Adwaita. `confirm` is the
/// optional confirmation label.  Response will be `"confirm"` if
/// confirmed.  If `destructive` is `true` the confirmation button
/// will be styled appropriately.
pub fn request(
    heading: &str,
    body: &str,
    confirm: Option<String>,
    destructive: bool,
    parent: Option<&impl IsA<gtk4::Window>>,
) -> adw::MessageDialog {
    let dialog = adw::MessageDialog::new(parent, Some(heading), Some(body));

    let confirm = confirm.unwrap_or_else(|| i18n("C_onfirm"));
    dialog.add_response("cancel", &i18n("_Cancel"));
    dialog.add_response("confirm", &confirm);
    dialog.set_default_response(Some("confirm"));
    dialog.set_close_response("cancel");

    let appearance = if destructive {
        adw::ResponseAppearance::Destructive
    } else {
        adw::ResponseAppearance::Suggested
    };
    dialog.set_response_appearance("confirm", appearance);

    dialog.set_modal(true);
    dialog
}
