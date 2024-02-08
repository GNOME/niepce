/*
 * niepce - niepce/modules/darkroom.rs
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
use npc_engine::db::NiepceProperties as Np;
use npc_engine::db::NiepcePropertyIdx as Npi;
use npc_engine::db::{self, LibMetadata, LibraryId};
use npc_engine::library::notification::{ImageRendered, LibNotification, MetadataChange};
use npc_engine::library::{RenderEngine, RenderMsg, RenderParams};
use npc_engine::libraryclient::{ClientInterface, LibraryClientHost};
use npc_fwk::base::Size;
use npc_fwk::toolkit::widgets::Dock;
use npc_fwk::toolkit::{Controller, ControllerImpl, UiController};
use npc_fwk::{dbg_out, on_err_out};

pub enum Msg {
    SetRenderEngine(String),
}

pub struct DarkroomModule {
    imp_: RefCell<ControllerImpl>,
    tx: npc_fwk::toolkit::Sender<Msg>,
    client: Rc<LibraryClientHost>,
    widget: gtk4::Widget,
    worker: RenderWorker,
    imagecanvas: ImageCanvas,
    overlay: adw::ToastOverlay,
    engine_combo: gtk4::ComboBoxText,
    engine_combo_change: Option<glib::SignalHandlerId>,
    toolbox_controller: cxx::UniquePtr<crate::ffi::ToolboxController>,
    file: RefCell<Option<db::LibFile>>,
    render_params: RefCell<Option<RenderParams>>,
    need_reload: Cell<bool>,
    active: Cell<bool>,
    loading_toast: RefCell<Option<adw::Toast>>,
}

impl Controller for DarkroomModule {
    type InMsg = Msg;

    npc_fwk::controller_imp_imp!(imp_);

    fn dispatch(&self, msg: Msg) {
        match msg {
            Msg::SetRenderEngine(ref engine) => {
                // XXX make this a command with undo
                dbg_out!("Render engine changed in UI");
                if let Some(ref file) = *self.file.borrow() {
                    self.client.client().set_metadata(
                        file.id(),
                        Np::Index(Npi::NpNiepceRenderEngineProp),
                        &npc_fwk::base::PropertyValue::String(engine.clone()),
                    );
                }
            }
        }
    }
}

impl UiController for DarkroomModule {
    fn widget(&self) -> &gtk4::Widget {
        &self.widget
    }
}

impl LibraryModule for DarkroomModule {
    fn set_active(&self, active: bool) {
        self.active.set(active);
        if active {
            // The logic here is that the `SelectionController` will request
            // the metadata. But if inactive, it's possible the module
            // doesn't have the metadata so we'll have to request it.
            if let Some(ref file) = *self.file.borrow() {
                if file.metadata().is_none() {
                    dbg_out!("Requesting metadata");
                    self.client.client().request_metadata(file.id());
                } else {
                    self.reload_image(self.render_params.borrow().clone());
                }
            }
        }
    }

    fn widget(&self) -> &gtk4::Widget {
        &self.widget
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
        let engine_combo = gtk4::ComboBoxText::new();
        let (tx, rx) = npc_fwk::toolkit::channel();

        let mut module = Self {
            imp_: RefCell::new(ControllerImpl::default()),
            tx,
            client: client_host.clone(),
            widget,
            imagecanvas,
            overlay,
            engine_combo,
            engine_combo_change: None,
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

        npc_fwk::toolkit::channels::receiver_attach(
            rx,
            glib::clone!(@strong module => move |e| {
                module.dispatch(e)
            }),
        );

        selection_controller.handler.signal_selected.connect(
            glib::clone!(@weak selection_controller, @weak module => move |id| {
                let file = selection_controller.file(id);
                module.set_image(file);
            }),
        );

        module
    }

    /// Set the active engine in the UI, but don't emit the signal change
    /// This is for when we set the UI value.
    fn set_active_engine(&self, engine: Option<&str>) {
        if let Some(ref handler) = self.engine_combo_change {
            glib::signal::signal_handler_block(&self.engine_combo, handler);
        }
        self.engine_combo.set_active_id(engine);
        if let Some(ref handler) = self.engine_combo_change {
            glib::signal::signal_handler_unblock(&self.engine_combo, handler);
        }
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

    /// Check that the current file is `id`.
    fn is_current_file_id(&self, id: LibraryId) -> bool {
        self.file
            .borrow()
            .as_ref()
            .map(|file| file.id() == id)
            .unwrap_or(false)
    }

    /// We received a rendered image.
    fn rendered_image_received(&self, rendered: &ImageRendered) {
        dbg_out!("Got bitmap");
        if self.is_current_file_id(rendered.id) {
            let b = rendered.image.clone();
            self.imagecanvas.set_image(b);
            self.remove_loading_toast();
        } else {
            dbg_out!("Received bitmap for {}, not the current", rendered.id);
        }
    }

    /// We received a metadata change.
    fn metadata_change_received(&self, changed: &MetadataChange) {
        if self.is_current_file_id(changed.id)
            && changed.meta == Np::Index(Npi::NpNiepceRenderEngineProp)
        {
            if let Some(engine) = changed.value.string() {
                self.set_engine(engine);
            }
        }
    }

    /// We received metadata.
    fn metadata_received(&self, metadata: &LibMetadata) {
        if !self.is_current_file_id(metadata.id()) {
            return;
        }

        if let Some(ref mut file) = *self.file.borrow_mut() {
            dbg_out!(
                "Checking file: current {} received {}",
                file.id(),
                metadata.id()
            );
            dbg_out!("Got metadata for {}", metadata.id());
            if file.metadata.is_some() {
                return;
            }
            file.metadata = Some(metadata.clone());
        }
        let params = self.file.borrow().as_ref().map(|file| {
            let params = self.params_for_metadata(file);
            self.render_params.replace(Some(params.clone()));

            let key = params.engine().key();
            self.set_active_engine(Some(key));

            self.need_reload.set(true);
            params
        });
        self.reload_image(params);
    }

    pub fn on_lib_notification(&self, ln: &LibNotification) {
        match ln {
            LibNotification::ImageRendered(rendered) => self.rendered_image_received(rendered),
            LibNotification::MetadataChanged(changed) => self.metadata_change_received(changed),
            LibNotification::MetadataQueried(metadata) => self.metadata_received(metadata),
            _ => {}
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
        self.engine_combo
            .append(Some(RenderEngine::Ncr.key()), "Niepce Camera Raw");
        self.engine_combo
            .append(Some(RenderEngine::Rt.key()), "RawTherapee");
        let tx = self.tx.clone();
        self.engine_combo_change = Some(self.engine_combo.connect_changed(move |combo| {
            if let Some(id) = combo.active_id().map(|id| id.to_string()) {
                npc_fwk::toolkit::send_async_local!(Msg::SetRenderEngine(id), tx);
            }
        }));
        dock.vbox().append(&self.engine_combo);
        let toolbox = unsafe {
            gtk4::Widget::from_glib_none(
                self.toolbox_controller.pin_mut().build_widget() as *const gtk4::ffi::GtkWidget
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
            on_err_out!(self.worker.send(RenderMsg::Reload(params.clone())));
            if let Some(render) = params {
                let cache = self.client.thumbnail_cache();
                cache.request_render(file.clone(), render, Some(self.worker.sender().clone()));
            }
            self.need_reload.set(false);
        }
    }

    /// Build the `RenderParams` from the metadata.
    fn params_for_metadata(&self, file: &db::LibFile) -> RenderParams {
        // If we have metadata, use them.
        let engine = file.metadata().and_then(|metadata| {
            metadata
                .get_metadata(Np::Index(Npi::NpNiepceRenderEngineProp))?
                .string()
                .and_then(RenderEngine::from_key)
        });
        let engine = match engine {
            None => {
                // We shall explicitly save the default engine.
                let e = RenderEngine::default();
                self.client.client().set_metadata(
                    file.id(),
                    Np::Index(Npi::NpNiepceRenderEngineProp),
                    &npc_fwk::base::PropertyValue::String(e.key().to_string()),
                );
                e
            }
            Some(e) => e,
        };

        RenderParams::new_preview(file, engine, Size::default())
    }

    pub fn set_image(&self, file: Option<db::LibFile>) {
        self.need_reload.set(true);
        self.file.replace(file.clone());

        if let Some(ref file) = file {
            on_err_out!(self.worker.send(RenderMsg::SetImage(Some(file.clone()))));
            if file.metadata().is_some() {
                let params = self.params_for_metadata(file);
                self.render_params.replace(Some(params.clone()));

                let key = params.engine().key();
                self.set_active_engine(Some(key));

                if self.need_reload.get() && self.active.get() {
                    self.reload_image(Some(params));
                }
            }
        }
    }

    fn set_engine(&self, engine: &str) {
        if let Some(engine) = RenderEngine::from_key(engine) {
            if let Some(ref mut params) = *self.render_params.borrow_mut() {
                // Don't retrigger render if the engine didn't change.
                if params.engine() != engine {
                    params.set_engine(engine);
                    self.set_active_engine(Some(engine.key()));
                    self.need_reload.set(true);
                }
            }
            self.reload_image(self.render_params.borrow().clone());
        }
    }
}
