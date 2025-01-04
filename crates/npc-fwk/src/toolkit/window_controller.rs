/*
 * niepce - crates/npc-fwk/src/toolkit/window_controller.rs
 *
 * Copyright (C) 2024-2025 Hubert Figuière
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

use std::rc::Rc;
use std::sync::Arc;

use crate::gio;
use crate::glib;
use crate::gtk4;
use gtk4::prelude::*;
use serde::{Deserialize, Serialize};

use super::AppController;
use super::Configuration;

#[derive(Serialize, Deserialize)]
struct State {
    x: i32,
    y: i32,
    max: bool,
}

pub trait WindowController {
    fn window(&self) -> &gtk4::Window;

    /// The config key for the state.
    fn state_key(&self) -> Option<&str> {
        None
    }

    /// The configuration
    fn configuration(&self) -> Option<Rc<Configuration>> {
        None
    }

    /// What to do when the close request.
    fn on_close(&self) {}

    /// Initialize the state saving if needed
    fn init_state<T: WindowController + 'static>(this: &Rc<T>) {
        this.window().connect_close_request(glib::clone!(
            #[weak]
            this,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_| {
                if let Some(cfg) = this.configuration() {
                    this.save_state(&cfg);
                    dbg_out!("State saved");
                }
                this.on_close();
                glib::Propagation::Proceed
            }
        ));
    }

    fn save_state(&self, cfg: &Configuration) {
        if let Some(key) = self.state_key() {
            let window = self.window();
            let size = window.default_size();
            let state = State {
                x: size.0,
                y: size.1,
                max: window.is_maximized(),
            };

            if let Ok(j) = serde_json::to_string(&state) {
                cfg.set_value(key, &j);
            }
        }
    }

    fn load_state(&self, cfg: &Configuration) {
        if let Some(key) = self.state_key() {
            if let Some(state) = cfg.value_opt(key) {
                if let Ok(state) = serde_json::from_str::<State>(&state) {
                    let window = self.window();
                    window.set_default_size(state.x, state.y);
                    window.set_maximized(state.max);
                    dbg_out!("loaded state");
                } else {
                    err_out!("Couldn't deserialise window state");
                }
            } else {
                err_out!("Couldn't load state");
            }
        }
    }
}

/// Create an undo action, with accel, and automatic state handling.
pub fn create_undo_action(
    app: Arc<dyn AppController>,
    action_map: &gio::ActionMap,
) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("Undo", None);
    action.connect_activate(glib::clone!(
        #[strong]
        app,
        move |_, _| {
            app.undo_history().undo();
        }
    ));
    action_map.add_action(&action);
    gtk4::Application::default().set_accels_for_action("win.Undo", &["<control>Z"]);

    app.undo_history().signal_changed.connect(glib::clone!(
        #[strong]
        app,
        #[weak]
        action,
        move |_| {
            let history = app.undo_history();
            action.set_enabled(history.has_undo());
            // let label = history.next_undo();
            // Maybe gio::SimpleAction isn't the best.
            // action.set_label(&format!("Undo {}", label));
        }
    ));

    action
}

/// Create an redo action, with accel, and automatic state handling.
pub fn create_redo_action(
    app: Arc<dyn AppController>,
    action_map: &gio::ActionMap,
) -> gio::SimpleAction {
    let action = gio::SimpleAction::new("Redo", None);
    action.connect_activate(glib::clone!(
        #[strong]
        app,
        move |_, _| {
            app.undo_history().redo();
        }
    ));
    action_map.add_action(&action);
    gtk4::Application::default().set_accels_for_action("win.Redo", &["<control><shift>Z"]);

    app.undo_history().signal_changed.connect(glib::clone!(
        #[strong]
        app,
        #[weak]
        action,
        move |_| {
            let history = app.undo_history();
            action.set_enabled(history.has_redo());
            // let label = history.next_redo();
            // action.set_label(&format!("Redo {}", label));
        }
    ));

    action
}
