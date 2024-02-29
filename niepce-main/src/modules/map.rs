/*
 * niepce - niepce/modules/map.rs
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

use std::cell::RefCell;

use glib::translate::*;

use crate::ffi::{map_module_new, MapModule};
use crate::niepce::ui::LibraryModule;
use npc_engine::library::notification::LibNotification;
use npc_fwk::toolkit::{Controller, ControllerImpl, UiController};

pub struct MapModuleProxy {
    imp_: RefCell<ControllerImpl<<MapModuleProxy as Controller>::InMsg>>,
    module: cxx::UniquePtr<MapModule>,
    widget: gtk4::Widget,
}

impl Controller for MapModuleProxy {
    type InMsg = ();

    npc_fwk::controller_imp_imp!(imp_);
}

impl UiController for MapModuleProxy {
    fn widget(&self) -> &gtk4::Widget {
        &self.widget
    }
}

impl LibraryModule for MapModuleProxy {
    fn set_active(&self, active: bool) {
        self.module.set_active(active);
    }

    fn widget(&self) -> &gtk4::Widget {
        &self.widget
    }
}

impl Default for MapModuleProxy {
    fn default() -> Self {
        let mut module = map_module_new();
        let widget = unsafe {
            gtk4::Widget::from_glib_none(
                module.pin_mut().build_widget() as *mut gtk4::ffi::GtkWidget
            )
        };
        Self {
            imp_: RefCell::new(ControllerImpl::default()),
            module,
            widget,
        }
    }
}

impl MapModuleProxy {
    pub fn on_lib_notification(&self, ln: &LibNotification) {
        self.module.on_lib_notification(ln);
    }
}
