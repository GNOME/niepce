/*
 * niepce - toolkit/gtk_utils.rs
 *
 * Copyright (C) 2022-2025 Hubert Figuière
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

use crate::gio;
use crate::glib;
use crate::gtk4;
use gtk4::prelude::*;

/// Create an action group and add all these actions
/// to it. The group is returned from the expression.
/// ```ignore
/// sending_action_group!(
///        tx,
///        ("ActionName1", Event::TheFirstAction),
///        ("ActionName2", Event::TheSecondAction)
/// );
/// ```
#[macro_export]
macro_rules! sending_action_group {
    ( $sender:expr, $( ( $name: expr, $event:expr ) ),* ) => {
        {
            let group = gio::SimpleActionGroup::new();
            let tx = $sender.clone();

            $(
                $crate::sending_action!(group, $name, tx, $event);
            )*

            group
        }
    }
}

/// Create a sending action with `name` that will send an `event`
/// through the `sender` and add it to the `group`.
///
/// ```ignore
/// sending_action!(group, "ActionName", tx, Event::TheAction);
/// ```
/// Will create an action with the name `ActionName` to send `TheAction`
/// onto `tx` (`tx` is `npc_fwk::toolkit::Sender<Event>`),
/// and add it to the `group`.
#[macro_export]
macro_rules! sending_action {
    ( $group:expr, $name:expr, $sender:expr, $event:expr ) => {{
        let tx = $sender.clone();
        gtk_macros::action!($group, $name, move |_, _| {
            $crate::send_async_local!($event, tx);
        });
    }};
}

pub fn add_action<F>(
    group: &gio::ActionMap,
    name: &str,
    f: F,
    context: Option<&str>,
    accel: Option<&str>,
) -> gio::SimpleAction
where
    F: Fn(&gio::SimpleAction, Option<&glib::Variant>) + 'static,
{
    let action = gio::SimpleAction::new(name, None);
    group.add_action(&action);
    action.connect_activate(f);
    if let Some(context) = context
        && let Some(accel) = accel
    {
        let detail = format!("{context}.{name}");
        gtk4::Application::default().set_accels_for_action(&detail, &[accel]);
    }

    action
}

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
