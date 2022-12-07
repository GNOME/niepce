/*
 * niepce - niepce/ui/module_shell.rs
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

use npc_engine::library::notification::LibNotification;
use npc_engine::libraryclient::{LibraryClientHost, LibraryClientWrapper};
use npc_fwk::toolkit::{Controller, ControllerImpl, UiController};

use crate::ffi::{grid_view_module_new, GridViewModule};
use crate::niepce::ui::{LibraryModule, SelectionController};

pub struct GridViewModuleProxy {
    imp_: RefCell<ControllerImpl>,
    module: cxx::SharedPtr<GridViewModule>,
    widget: gtk4::Widget,
    pub grid_view: gtk4::GridView,
}

impl Controller for GridViewModuleProxy {
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

impl UiController for GridViewModuleProxy {
    fn widget(&self) -> &gtk4::Widget {
        // In this the assumption is that widget has been set at
        // construction time from the C++ impl and since the only way
        // to do so is by calling the new associated function, it
        // should be ok.
        &self.widget
    }

    fn actions(&self) -> Option<(&str, &gio::ActionGroup)> {
        None
    }
}

impl LibraryModule for GridViewModuleProxy {
    fn set_active(&self, _active: bool) {}

    fn menu(&self) -> Option<&gio::Menu> {
        None
    }
}

impl GridViewModuleProxy {
    pub fn new(
        selection_controller: &Rc<SelectionController>,
        menu: &gio::Menu,
        libclient_host: &LibraryClientHost,
    ) -> Self {
        let menu: *const gio_sys::GMenu = menu.to_glib_none().0;
        let module = unsafe {
            // Silence clippy because we borrow the selection controller for the cxx
            // bindings. It'll go away.
            #[allow(clippy::borrow_deref_ref)]
            grid_view_module_new(
                &*selection_controller,
                menu as *const crate::ffi::GMenu,
                libclient_host,
            )
        };
        let widget = unsafe {
            gtk4::Widget::from_glib_none(module.build_widget() as *const gtk4_sys::GtkWidget)
        };
        let grid_view = unsafe {
            gtk4::GridView::from_glib_none(module.image_list() as *const gtk4_sys::GtkGridView)
        };
        GridViewModuleProxy {
            imp_: RefCell::new(ControllerImpl::default()),
            module,
            widget,
            grid_view,
        }
    }

    pub fn on_lib_notification(&self, ln: &LibNotification, client: &LibraryClientWrapper) {
        self.module.on_lib_notification(ln, client);
    }

    pub fn display_none(&self) {
        self.module.display_none();
    }
}