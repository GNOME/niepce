/*
 * niepce - crates/npc-fwk/src/toolkit/controller.rs
 *
 * Copyright (C) 2022-2024 Hubert Figuière
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

use std::cell::{Ref, RefMut};

/// Use this macro inside the impl to implement `imp()` and `imp_mut()`
///
/// ```rust,ignore
/// impl Controller for MyController {
///     npc_fwk::controller_imp_imp!()
/// }
/// ```
#[macro_export]
macro_rules! controller_imp_imp {
    ( $f:ident ) => {
        fn imp(&self) -> std::cell::Ref<'_, $crate::toolkit::ControllerImpl> {
            self.$f.borrow()
        }

        fn imp_mut(&self) -> std::cell::RefMut<'_, $crate::toolkit::ControllerImpl> {
            self.$f.borrow_mut()
        }
    };
}

#[derive(Default)]
pub struct ControllerImpl {}

pub trait Controller {
    type InMsg;

    /// Notify the controller is ready. Will notify children and call on_ready()
    fn ready(&self) {
        dbg_out!("ready");
        self.on_ready();
    }

    /// What to do when ready.
    fn on_ready(&self) {}

    /// Dispatch input message.
    fn dispatch(&self, _message: Self::InMsg) {}

    /// Return the implementation
    /// Implemented via controller_imp_imp!()
    fn imp(&self) -> Ref<'_, ControllerImpl>;
    /// Return the mutable implementation
    /// Implemented via controller_imp_imp!()
    fn imp_mut(&self) -> RefMut<'_, ControllerImpl>;
}
