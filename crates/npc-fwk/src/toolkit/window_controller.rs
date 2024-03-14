/*
 * niepce - crates/npc-fwk/src/toolkit/window_controller.rs
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

use std::sync::Arc;

use gtk4::prelude::*;

use super::AppController;

pub trait WindowController {
    fn window(&self) -> &gtk4::Window;
}

/// Create an undo action, with accel, and automatic state handling.
pub fn create_undo_action(
    app: &Arc<dyn AppController>,
    action_map: &gio::ActionMap,
) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("Undo", None);
    action.connect_activate(glib::clone!(@strong app => move |_, _| {
        app.undo_history().undo();
    }));
    action_map.add_action(&action);
    gtk4::Application::default().set_accels_for_action("win.Undo", &["<control>Z"]);

    app.undo_history()
        .signal_changed
        .connect(glib::clone!(@strong app, @weak action => move |_| {
            let history = app.undo_history();
            action.set_enabled(history.has_undo());
            // let label = history.next_undo();
            // Maybe gio::SimpleAction isn't the best.
            // action.set_label(&format!("Undo {}", label));
        }));

    action
}

/// Create an redo action, with accel, and automatic state handling.
pub fn create_redo_action(
    app: &Arc<dyn AppController>,
    action_map: &gio::ActionMap,
) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("Redo", None);
    action.connect_activate(glib::clone!(@strong app => move |_, _| {
        app.undo_history().redo();
    }));
    action_map.add_action(&action);
    gtk4::Application::default().set_accels_for_action("win.Redo", &["<control><shift>Z"]);

    app.undo_history()
        .signal_changed
        .connect(glib::clone!(@strong app, @weak action => move |_| {
            let history = app.undo_history();
            action.set_enabled(history.has_redo());
            // let label = history.next_redo();
            // action.set_label(&format!("Redo {}", label));
        }));

    action
}
