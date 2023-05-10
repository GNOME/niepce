/*
 * niepce - niepce/ui/dialogs/importer/camera_importer_ui.rs
 *
 * Copyright (C) 2017-2023 Hubert Figuière
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

use super::{ImporterUI, SourceSelectedCallback};
use npc_engine::importer::{CameraImporter, ImportBackend};
use npc_fwk::on_err_out;
use npc_fwk::toolkit;

enum Event {
    CameraSelected,
}

#[derive(Default)]
struct Widgets {
    parent: Option<gtk4::Window>,
    camera_list_combo: Option<gtk4::ComboBoxText>,
    select_camera: Option<gtk4::Button>,
    source_selected_cb: Option<SourceSelectedCallback>,
}

pub(super) struct CameraImporterUI {
    tx: glib::Sender<Event>,
    name: String,
    backend: Rc<dyn ImportBackend>,
    widgets: RefCell<Widgets>,
}

impl CameraImporterUI {
    pub fn new() -> Rc<CameraImporterUI> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let widget = Rc::new(CameraImporterUI {
            tx,
            name: i18n("Camera"),
            backend: Rc::new(CameraImporter::default()),
            widgets: RefCell::default(),
        });

        rx.attach(
            None,
            glib::clone!(@strong widget => move |e| {
                widget.dispatch(e);
                glib::Continue(true)
            }),
        );

        widget
    }

    fn dispatch(&self, e: Event) {
        match e {
            Event::CameraSelected => self.select_camera(),
        }
    }

    fn select_camera(&self) {
        if let Some(source) = self
            .widgets
            .borrow()
            .camera_list_combo
            .as_ref()
            .and_then(|combo| combo.active_id())
        {
            let datetime = chrono::Local::now();
            let today = format!("{}", datetime.format("%F %H%M%S"));
            let dest_dir = i18n("Camera import ") + &today;
            if let Some(callback) = &self.widgets.borrow().source_selected_cb {
                callback(&source, &dest_dir);
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

    fn setup_widget(&self, parent: &gtk4::Window) -> gtk4::Widget {
        let builder = gtk4::Builder::from_resource("/net/figuiere/Niepce/ui/cameraimporterui.ui");
        get_widget!(builder, gtk4::Grid, main_widget);
        get_widget!(builder, gtk4::Button, select_camera_btn);
        select_camera_btn.connect_clicked(glib::clone!(@strong self.tx as tx =>
            move |_| on_err_out!(tx.send(Event::CameraSelected));
        ));
        get_widget!(builder, gtk4::ComboBoxText, camera_list_combo);

        let mut widgets = self.widgets.borrow_mut();
        widgets.parent = Some(parent.clone());
        widgets.select_camera = Some(select_camera_btn.clone());
        widgets.camera_list_combo = Some(camera_list_combo.clone());

        toolkit::GpDeviceList::instance().detect();

        // XXX restore the selection from the preferences.
        for (idx, device) in toolkit::GpDeviceList::instance().list().iter().enumerate() {
            camera_list_combo.append(Some(&device.port), &device.model);
            if idx == 0 {
                camera_list_combo.set_active_id(Some(&device.port));
            }
        }
        if camera_list_combo.active_id().is_none() {
            camera_list_combo.append(Some(""), &i18n("No camera found"));
            camera_list_combo.set_active_id(Some(""));
            camera_list_combo.set_sensitive(false);
            select_camera_btn.set_sensitive(false);
        }

        main_widget.upcast::<gtk4::Widget>()
    }

    fn set_source_selected_callback(&self, callback: Box<dyn Fn(&str, &str)>) {
        self.widgets.borrow_mut().source_selected_cb = Some(callback);
    }
}
