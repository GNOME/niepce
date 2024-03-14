/*
 * niepce - niepce/ui/app_controller.rs
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

use crate::toolkit::{Configuration, Controller, UndoHistory, UndoTransaction};

/// AppController trait allow getting a few App only pieces. Notably
/// app configuration and undo.
pub trait AppController {
    fn begin_undo(&self, transaction: UndoTransaction);
    fn undo_history(&self) -> &UndoHistory;
    fn config(&self) -> &Configuration;
}

/// This implement the singleton as the application, there can only be
/// one.
pub trait AppControllerSingleton {
    /// Get the singleton.
    fn singleton<T: AppController>() -> Arc<dyn AppController> {
        Self::singleton_init::<T>(None)
    }

    /// Initialise the singleton from the controller.  It is an error
    /// to pass `None` if the singleton hasn't been created.
    fn singleton_init<T: AppController>(
        init: Option<Arc<dyn AppController>>,
    ) -> Arc<dyn AppController> {
        static mut INSTANCE: Option<Arc<dyn AppController>> = None;
        static INIT: std::sync::Once = std::sync::Once::new();

        INIT.call_once(|| unsafe {
            INSTANCE = init;
        });

        unsafe { INSTANCE.clone().unwrap() }
    }

    /// Start the controller event loop
    /// Like `Controller::start<>` but takes an `Arc<>`
    fn start<T: Controller + 'static>(this: &Arc<T>) {
        let rx = this.receiver();
        let ctrl = this.clone();
        super::channels::receiver_attach(rx, move |e| {
            dbg_out!(
                "dialog dispatching for {}:{:p}",
                std::any::type_name::<T>(),
                Arc::as_ptr(&ctrl)
            );
            ctrl.dispatch(e);
        });
    }

    /// Create the singleton with the controller.
    fn create<T>(app: Arc<T>)
    where
        T: AppController + 'static,
    {
        Self::singleton_init::<T>(Some(app));
    }
}
