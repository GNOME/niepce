/*
 * niepce - crates/npc-fwk/src/toolkit/uicontroller.rs
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

use std::rc::Rc;

use crate::gio;
use crate::gtk4;
use gtk4::prelude::*;

use super::Controller;

/// UI Controller
pub trait UiController: Controller {
    /// Get the widget. Will lazy load it.
    fn widget(&self) -> &gtk4::Widget;

    /// Get the action group if any. Lazy loaded
    fn actions(&self) -> Option<(&str, &gio::ActionGroup)> {
        None
    }
}

/// Specify a Window size.
pub enum WindowSize {
    /// Default window size.
    Default,
    /// Same size as parent.
    Parent,
}

/// A Dialog Controller to handle dialogs asynchronously
///
/// It should start itself calling DialogController::start()
///
pub trait DialogController: UiController {
    fn dialog(&self) -> &adw::Window;

    /// Close the dialog. This will stop the controller dispatch.
    /// This is usually called in a "close-request" signal.
    fn close(&self) {
        self.dialog().close();
        self.stop();
    }

    /// Unlike the regular start this keep a strong reference.  For
    /// dialogs the idea is that it will be holder the controller ref.
    /// as the work async.
    fn start<T: DialogController + 'static>(this: &Rc<T>) {
        let rx = this.receiver();
        let ctrl = this.clone();
        super::channels::receiver_attach(rx, move |e| {
            dbg_out!(
                "dialog dispatching for {}:{:p}",
                std::any::type_name::<Self>(),
                Rc::as_ptr(&ctrl)
            );
            ctrl.dispatch(e);
        });
    }

    /// Run the dialog modal. Will call `callback` when it is closed
    /// in success.
    fn run_modal<F>(&self, parent: Option<&gtk4::Window>, size: WindowSize, callback: F)
    where
        F: Fn(Self::OutMsg) + 'static,
    {
        let dialog = self.dialog();
        self.set_forwarder(Some(Box::new(callback)));
        dialog.set_transient_for(parent);
        dialog.set_modal(true);
        match size {
            WindowSize::Parent => {
                if let Some(parent) = parent {
                    let w = parent.width();
                    let h = parent.height();
                    dialog.set_default_size(w, h);
                }
            }
            WindowSize::Default => {}
        }
        dialog.present();
    }
}
