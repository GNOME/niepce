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

use glib::translate::*;
use gtk4::prelude::*;

use crate::niepce::ui::LibraryModule;
use crate::SelectionController;
use image_canvas::ImageCanvas;
use npc_craw::{RenderImpl, RenderMsg, RenderWorker};
use npc_engine::db;
use npc_fwk::toolkit::widgets::Dock;
use npc_fwk::toolkit::{Controller, ControllerImpl, UiController};
use npc_fwk::{dbg_out, on_err_out};

pub struct DarkroomModule {
    imp_: RefCell<ControllerImpl>,
    widget: gtk4::Widget,
    worker: RenderWorker,
    imagecanvas: ImageCanvas,
    tx: glib::Sender<RenderMsg>,
    _toolbox_controller: cxx::SharedPtr<crate::ffi::ToolboxController>,
    need_reload: Cell<bool>,
    active: Cell<bool>,
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
            self.reload_image();
        }
    }

    fn menu(&self) -> Option<&gio::Menu> {
        None
    }
}

impl DarkroomModule {
    pub fn new(selection_controller: &Rc<SelectionController>) -> Rc<Self> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let worker = RenderWorker::new(RenderImpl::new());
        let imagecanvas = ImageCanvas::new();
        let toolbox_controller = crate::ffi::toolbox_controller_new();
        let widget = Self::build_widget(&imagecanvas, &toolbox_controller);

        let module = Rc::new(Self {
            imp_: RefCell::new(ControllerImpl::default()),
            widget,
            imagecanvas,
            worker,
            tx,
            _toolbox_controller: toolbox_controller,
            need_reload: Cell::new(true),
            active: Cell::new(false),
        });
        selection_controller.handler.signal_selected.connect(
            glib::clone!(@weak selection_controller, @weak module => move |id| {
                let file = selection_controller.file(id);
                module.set_image(file);
            }),
        );
        rx.attach(
            None,
            glib::clone!(@strong module => move |e| {
                if let RenderMsg::Bitmap(b) = e {
                    dbg_out!("Got bitmap");
                    module.imagecanvas.set_image(b);
                }
                glib::Continue(true)
            }),
        );

        module
    }

    fn build_widget(
        imagecanvas: &ImageCanvas,
        toolbox_controller: &cxx::SharedPtr<crate::ffi::ToolboxController>,
    ) -> gtk4::Widget {
        let splitview = gtk4::Paned::new(gtk4::Orientation::Horizontal);
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        splitview.set_start_child(Some(&vbox));
        imagecanvas.set_hexpand(true);
        imagecanvas.set_vexpand(true);
        vbox.append(imagecanvas);

        let toolbar = crate::niepce::ui::imagetoolbar::image_toolbar_new();
        vbox.append(&toolbar);
        let dock = Dock::new();
        let toolbox = unsafe {
            gtk4::Widget::from_glib_none(
                toolbox_controller.build_widget() as *const gtk4_sys::GtkWidget
            )
        };
        dock.vbox().append(&toolbox);
        splitview.set_end_child(Some(&dock));
        splitview.set_resize_end_child(false);

        splitview.into()
    }

    fn reload_image(&self) {
        if !self.need_reload.get() {
            return;
        }

        on_err_out!(self.worker.send(RenderMsg::Reload));
        on_err_out!(self.worker.send(RenderMsg::GetBitmap(self.tx.clone())));

        self.need_reload.set(false);
    }

    pub fn set_image(&self, file: Option<db::LibFile>) {
        self.need_reload.set(true);

        on_err_out!(self.worker.send(RenderMsg::SetImage(file)));

        if self.need_reload.get() && self.active.get() {
            self.reload_image();
        }
    }
}
