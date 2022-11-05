/*
 * niepce - niepce/modules/darkroom.rs
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

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use glib::translate::*;

use crate::ffi::{darkroom_module_new, DarkroomModule};
use crate::niepce::ui::LibraryModule;
use crate::SelectionController;
use npc_fwk::toolkit::{Controller, ControllerImpl, UiController};

/// The proxy handle the interface between the Rust module shell
/// and the C++ implementation. Ideally this will become the whole
/// implementation and renamed to just `DarkroomModule`.
pub struct DarkroomModuleProxy {
    imp_: RefCell<ControllerImpl>,
    module: cxx::SharedPtr<DarkroomModule>,
    widget: gtk4::Widget,
}

impl Controller for DarkroomModuleProxy {
    /// What to do when ready.
    fn on_ready(&self) {}

    /// Return the implementation
    fn imp(&self) -> Ref<'_, ControllerImpl> {
        self.imp_.borrow()
    }

    /// Return the mutable implementation
    fn imp_mut(&self) -> RefMut<'_, ControllerImpl> {
        self.imp_.borrow_mut()
    }
}

impl UiController for DarkroomModuleProxy {
    fn widget(&self) -> &gtk4::Widget {
        &self.widget
    }

    fn actions(&self) -> Option<(&str, &gio::ActionGroup)> {
        None
    }
}

impl LibraryModule for DarkroomModuleProxy {
    fn set_active(&self, active: bool) {
        self.module.set_active(active);
    }

    fn menu(&self) -> Option<&gio::Menu> {
        None
    }
}

impl DarkroomModuleProxy {
    pub fn new(selection_controller: &Rc<SelectionController>) -> Self {
        let module = darkroom_module_new();
        let widget = unsafe {
            gtk4::Widget::from_glib_none(module.build_widget() as *mut gtk4_sys::GtkWidget)
        };
        selection_controller.handler.signal_selected.connect(
            glib::clone!(@weak selection_controller, @strong module => move |id| {
                let file = selection_controller.file(id);
                if let Some(file) = file {
                    unsafe { module.set_image(Box::into_raw(Box::new(file))); }
                } else {
                    unsafe { module.set_image(std::ptr::null_mut()); }
                }
            }),
        );
        Self {
            imp_: RefCell::new(ControllerImpl::default()),
            module,
            widget,
        }
    }
}
