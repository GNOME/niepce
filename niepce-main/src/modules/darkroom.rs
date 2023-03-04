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
use npc_engine::db;
use npc_fwk::toolkit::widgets::Dock;
use npc_fwk::toolkit::{Controller, ControllerImpl, UiController};

pub struct DarkroomModule {
    imp_: RefCell<ControllerImpl>,
    widget: gtk4::Widget,
    _toolbox_controller: cxx::SharedPtr<crate::ffi::ToolboxController>,
    image: cxx::SharedPtr<npc_craw::Image>,
    imagefile: RefCell<Option<db::LibFile>>,
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
        npc_craw::ffi::init();
        let image = npc_craw::ffi::image_new();
        let toolbox_controller = crate::ffi::toolbox_controller_new();
        let widget = Self::build_widget(&image, &toolbox_controller);

        let module = Rc::new(Self {
            imp_: RefCell::new(ControllerImpl::default()),
            widget,
            image,
            _toolbox_controller: toolbox_controller,
            imagefile: RefCell::new(None),
            need_reload: Cell::new(true),
            active: Cell::new(false),
        });
        selection_controller.handler.signal_selected.connect(
            glib::clone!(@weak selection_controller, @weak module => move |id| {
                let file = selection_controller.file(id);
                module.set_image(file);
            }),
        );

        module
    }

    fn build_widget(
        image: &cxx::SharedPtr<npc_craw::Image>,
        toolbox_controller: &cxx::SharedPtr<crate::ffi::ToolboxController>,
    ) -> gtk4::Widget {
        let splitview = gtk4::Paned::new(gtk4::Orientation::Horizontal);
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        splitview.set_start_child(Some(&vbox));
        let imagecanvas = image_canvas::ImageCanvas::new();
        imagecanvas.set_hexpand(true);
        imagecanvas.set_vexpand(true);
        vbox.append(&imagecanvas);

        imagecanvas.set_image(image);

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

        if let Some(file) = self.imagefile.borrow().as_ref() {
            // currently we treat RAW + JPEG as RAW.
            // TODO: have a way to actually choose the JPEG.
            let file_type = file.file_type();
            let is_raw = (file_type == db::FileType::Raw) || (file_type == db::FileType::RawJpeg);
            let path = file.path().to_string_lossy();

            self.image.reload(&path, is_raw, file.orientation());
        } else if let Ok(p) =
            gdk_pixbuf::Pixbuf::from_resource("/org/gnome/Niepce/pixmaps/niepce-image-generic.png")
        {
            let p: *mut gdk_pixbuf_sys::GdkPixbuf = p.to_glib_none().0;
            unsafe {
                self.image.reload_pixbuf(p as *mut npc_craw::ffi::GdkPixbuf);
            }
        }

        self.need_reload.set(false);
    }

    pub fn set_image(&self, file: Option<db::LibFile>) {
        self.imagefile.replace(file);
        self.need_reload.set(true);

        if self.need_reload.get() && self.active.get() {
            self.reload_image();
        }
    }
}
