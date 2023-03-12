/*
 * niepce - niepce/modules/darkroom.rs
 *
 * Copyright (C) 2022-2023 Hubert Figuière
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

pub(super) mod image_canvas;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gettextrs::gettext as i18n;
use glib::translate::*;
use gtk4::prelude::*;
use i18n_format::i18n_fmt;

use crate::niepce::ui::LibraryModule;
use crate::SelectionController;
use image_canvas::ImageCanvas;
use npc_craw::{RenderImpl, RenderWorker};
use npc_engine::db;
use npc_engine::library::notification::LibNotification;
use npc_engine::library::{RenderMsg, RenderParams};
use npc_engine::libraryclient::LibraryClientHost;
use npc_fwk::base::Size;
use npc_fwk::toolkit::widgets::Dock;
use npc_fwk::toolkit::{Controller, ControllerImpl, UiController};
use npc_fwk::{dbg_out, on_err_out};

pub struct DarkroomModule {
    imp_: RefCell<ControllerImpl>,
    client: Rc<LibraryClientHost>,
    widget: gtk4::Widget,
    worker: RenderWorker,
    imagecanvas: ImageCanvas,
    overlay: adw::ToastOverlay,
    toolbox_controller: cxx::SharedPtr<crate::ffi::ToolboxController>,
    file: RefCell<Option<db::LibFile>>,
    render_params: RefCell<Option<RenderParams>>,
    need_reload: Cell<bool>,
    active: Cell<bool>,
    loading_toast: RefCell<Option<adw::Toast>>,
}

impl Controller for DarkroomModule {
    npc_fwk::controller_imp_imp!(imp_);
}

impl UiController for DarkroomModule {
    fn widget(&self) -> &gtk4::Widget {
        &self.widget
    }

    fn actions(&self) -> Option<(&str, &gio::ActionGroup)> {
        None
    }
}

impl LibraryModule for DarkroomModule {
    fn set_active(&self, active: bool) {
        self.active.set(active);
        if active {
            self.reload_image(self.render_params.borrow().clone());
        }
    }

    fn menu(&self) -> Option<&gio::Menu> {
        None
    }
}

impl DarkroomModule {
    pub fn new(
        selection_controller: &Rc<SelectionController>,
        client_host: &Rc<LibraryClientHost>,
    ) -> Rc<Self> {
        let worker = RenderWorker::new(RenderImpl::new());
        let imagecanvas = ImageCanvas::new();
        let overlay = adw::ToastOverlay::new();
        let toolbox_controller = crate::ffi::toolbox_controller_new();
        let widget: gtk4::Widget = gtk4::Paned::new(gtk4::Orientation::Horizontal).into();

        let mut module = Self {
            imp_: RefCell::new(ControllerImpl::default()),
            client: client_host.clone(),
            widget,
            imagecanvas,
            overlay,
            worker,
            toolbox_controller,
            file: RefCell::new(None),
            render_params: RefCell::new(None),
            need_reload: Cell::new(true),
            active: Cell::new(false),
            loading_toast: RefCell::new(None),
        };

        module.build_widget();

        let module = Rc::new(module);
        selection_controller.handler.signal_selected.connect(
            glib::clone!(@weak selection_controller, @weak module => move |id| {
                let file = selection_controller.file(id);
                module.set_image(file);
            }),
        );

        module
    }

    /// Remove the toast indicating loading.
    fn remove_loading_toast(&self) {
        if let Some(ref toast) = *self.loading_toast.borrow() {
            toast.dismiss();
        }
        self.loading_toast.replace(None);
    }

    /// Show the toast indicating loading for `path`
    fn show_loading_toast(&self, path: &std::path::Path) {
        // Make sure the current one is dismissed.
        if let Some(ref toast) = *self.loading_toast.borrow() {
            toast.dismiss();
        }
        let toast = adw::Toast::new(&if let Some(filename) =
            path.file_name().map(|s| s.to_string_lossy())
        {
            i18n_fmt! {
                // Translators: {} is replaced by the file name.
                i18n_fmt("Loading \"{}\"...", filename)
            }
        } else {
            i18n("Loading...")
        });
        toast.set_timeout(0);
        self.loading_toast.replace(Some(toast.clone()));
        self.overlay.add_toast(toast);
    }

    pub fn on_lib_notification(&self, ln: &LibNotification) {
        if let LibNotification::ImageRendered(b) = ln {
            // XXX this is suboptimal
            dbg_out!("Got bitmap");
            self.imagecanvas.set_image(b.clone());
            self.remove_loading_toast();
        }
    }

    fn build_widget(&mut self) {
        let splitview = self
            .widget
            .downcast_ref::<gtk4::Paned>()
            .expect("Failed to downcast to Paned");
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        splitview.set_start_child(Some(&vbox));
        splitview.set_wide_handle(true);
        self.imagecanvas.set_hexpand(true);
        self.imagecanvas.set_vexpand(true);
        self.overlay.set_child(Some(&self.imagecanvas));
        vbox.append(&self.overlay);

        let toolbar = crate::niepce::ui::imagetoolbar::image_toolbar_new();
        vbox.append(&toolbar);
        let dock = Dock::new();
        let toolbox = unsafe {
            gtk4::Widget::from_glib_none(
                self.toolbox_controller.build_widget() as *const gtk4_sys::GtkWidget
            )
        };
        dock.vbox().append(&toolbox);
        splitview.set_end_child(Some(&dock));
        splitview.set_resize_end_child(false);
    }

    fn reload_image(&self, params: Option<RenderParams>) {
        if !self.need_reload.get() {
            return;
        }
        if let Some(ref file) = *self.file.borrow() {
            self.show_loading_toast(file.path());
            on_err_out!(self.worker.send(RenderMsg::Reload(params)));
            let cache = self.client.thumbnail_cache();
            cache.request_render(file.clone(), Some(self.worker.sender().clone()));

            self.need_reload.set(false);
        }
    }

    pub fn set_image(&self, file: Option<db::LibFile>) {
        self.need_reload.set(true);
        self.file.replace(file.clone());

        let params = file
            .as_ref()
            .map(|file| RenderParams::new_preview(file.id(), Size::default()));
        on_err_out!(self.worker.send(RenderMsg::SetImage(file)));
        self.render_params.replace(params.clone());

        if self.need_reload.get() && self.active.get() {
            self.reload_image(params);
        }
    }
}
