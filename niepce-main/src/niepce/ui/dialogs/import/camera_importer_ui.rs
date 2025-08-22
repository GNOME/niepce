/*
 * niepce - niepce/ui/dialogs/importer/camera_importer_ui.rs
 *
 * Copyright (C) 2017-2025 Hubert Figuière
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
use std::rc::Rc;

use gettextrs::gettext as i18n;
use gtk4::prelude::*;
use npc_fwk::{glib, gtk4};

use super::{ImporterMsg, ImporterUI};
use npc_engine::importer::{CameraImporter, ImportBackend};
use npc_fwk::controller_imp_imp;
use npc_fwk::toolkit;
use npc_fwk::toolkit::{Controller, ControllerImplCell, Sender};

pub enum Event {
    CameraSelected,
}

#[derive(Default)]
struct State {
    source: Option<String>,
}

#[derive(Default)]
struct Widgets {
    parent: Option<gtk4::Window>,
    camera_list_combo: Option<gtk4::DropDown>,
    camera_list_model: Rc<toolkit::ComboModel<String>>,
    select_camera: Option<gtk4::Button>,
    tx: Option<Sender<ImporterMsg>>,
}

pub(super) struct CameraImporterUI {
    imp_: ControllerImplCell<Event, ()>,
    name: String,
    backend: Rc<dyn ImportBackend>,
    widgets: RefCell<Widgets>,
    state: RefCell<State>,
}

impl Controller for CameraImporterUI {
    type InMsg = Event;
    type OutMsg = ();

    controller_imp_imp!(imp_);

    fn dispatch(&self, e: Event) {
        match e {
            Event::CameraSelected => self.select_camera(),
        }
    }
}

impl CameraImporterUI {
    pub fn new() -> Rc<CameraImporterUI> {
        let widget = Rc::new(CameraImporterUI {
            imp_: ControllerImplCell::default(),
            name: i18n("Camera"),
            backend: Rc::new(CameraImporter::default()),
            widgets: RefCell::default(),
            state: RefCell::default(),
        });

        <Self as Controller>::start(&widget);

        widget
    }

    fn select_camera(&self) {
        if let Some(source) = self
            .widgets
            .borrow()
            .camera_list_combo
            .as_ref()
            .map(|combo| combo.selected())
        {
            let source = self
                .widgets
                .borrow()
                .camera_list_model
                .value(source as usize);
            if let Some(tx) = &self.widgets.borrow().tx.clone() {
                let source = Some(source.clone());
                {
                    let mut state = self.state.borrow_mut();
                    state.source = source.clone();
                }
                npc_fwk::send_async_local!(ImporterMsg::SetSource(source, true), tx);
            }
        }
    }
}

impl ImporterUI for CameraImporterUI {
    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> String {
        self.backend.id().to_string()
    }

    fn backend(&self) -> Rc<dyn ImportBackend> {
        self.backend.clone()
    }

    fn setup_widget(&self, parent: &gtk4::Window, tx: Sender<ImporterMsg>) -> gtk4::Widget {
        let builder = gtk4::Builder::from_resource("/net/figuiere/Niepce/ui/camera_importer_ui.ui");
        get_widget!(builder, gtk4::Grid, main_widget);
        get_widget!(builder, gtk4::Button, select_camera_btn);
        let sender = self.sender();
        select_camera_btn.connect_clicked(glib::clone!(
            #[strong]
            sender,
            move |_| npc_fwk::send_async_local!(Event::CameraSelected, sender),
        ));
        get_widget!(builder, gtk4::DropDown, camera_list_combo);

        let mut widgets = self.widgets.borrow_mut();
        widgets.parent = Some(parent.clone());
        widgets.select_camera = Some(select_camera_btn.clone());
        widgets.camera_list_combo = Some(camera_list_combo.clone());
        widgets.tx = Some(tx);

        toolkit::GpDeviceList::instance().detect();

        widgets
            .camera_list_model
            .bind(&camera_list_combo, move |_| {});
        // XXX restore the selection from the preferences.
        for device in toolkit::GpDeviceList::instance().list().iter() {
            widgets
                .camera_list_model
                .push(&device.model, device.port.clone());
        }
        if widgets.camera_list_model.is_empty() {
            widgets
                .camera_list_model
                .push(&i18n("No camera found"), String::default());
            camera_list_combo.set_sensitive(false);
            select_camera_btn.set_sensitive(false);
        }

        main_widget.upcast::<gtk4::Widget>()
    }

    fn state_update(&self) {
        if let Some(tx) = &self.widgets.borrow().tx.clone() {
            let source = self.state.borrow().source.clone();
            npc_fwk::send_async_local!(ImporterMsg::SetSource(source, true), tx);
        }
    }
}
