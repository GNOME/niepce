/*
 * niepce - ui/dialogs/import/directory_importer_ui.rs
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
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;

use gettextrs::gettext as i18n;
use gtk4::prelude::*;

use npc_engine::importer::{DirectoryImporter, ImportBackend};
use npc_fwk::{on_err_out, toolkit};

use super::{ImporterUI, SourceSelectedCallback};

enum Event {
    CopyToggled(bool),
    SelectDirectories,
    SetDirectoryName(Option<PathBuf>),
    SourceSelected(String, String),
}

#[derive(Default)]
struct Widgets {
    directory_name: Option<gtk4::Label>,
    parent: Option<gtk4::Window>,
    source_selected_cb: Option<SourceSelectedCallback>,
}

pub(super) struct DirectoryImporterUI {
    tx: glib::Sender<Event>,
    name: String,
    cfg: Rc<toolkit::Configuration>,
    backend: RefCell<Rc<DirectoryImporter>>,
    widgets: RefCell<Widgets>,
}

impl DirectoryImporterUI {
    pub fn new(cfg: Rc<toolkit::Configuration>) -> Rc<DirectoryImporterUI> {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let widget = Rc::new(DirectoryImporterUI {
            tx,
            name: i18n("Directory"),
            cfg,
            backend: RefCell::new(Rc::new(DirectoryImporter::default())),
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
            Event::CopyToggled(t) => self.copy_toggled(t),
            Event::SelectDirectories => self.do_select_directories(),
            Event::SetDirectoryName(name) => {
                if let Some(directory_name) = &self.widgets.borrow().directory_name {
                    directory_name.set_text(name.as_ref().and_then(|p| p.to_str()).unwrap_or(""));
                }
            }
            Event::SourceSelected(source, dest_dir) => {
                if let Some(source_selected_cb) = &self.widgets.borrow().source_selected_cb {
                    source_selected_cb(&source, &dest_dir);
                }
            }
        }
    }

    fn do_select_directories(&self) {
        let dialog = gtk4::FileChooserDialog::new(
            Some(&i18n("Import picture folder")),
            self.widgets.borrow().parent.as_ref(),
            gtk4::FileChooserAction::SelectFolder,
            &[
                (&i18n("Cancel"), gtk4::ResponseType::Cancel),
                (&i18n("Select"), gtk4::ResponseType::Ok),
            ],
        );
        dialog.set_select_multiple(false);

        if let Some(last_import_location) = self.cfg.value_opt("last_dir_import_location") {
            let file = gio::File::for_path(last_import_location);
            on_err_out!(dialog.set_current_folder(Some(&file)));
        }
        dialog.connect_response(glib::clone!(@strong self.tx as tx, @weak self.cfg as cfg => move |dialog, response| {
            let mut source = None;
            if response == gtk4::ResponseType::Ok {
                source = dialog.file().and_then(|f| f.path());
                let dest_dir = source.as_ref().and_then(|p| p.file_name()?.to_str())
                    .unwrap_or("");
                if let Some(source) = source.as_ref().and_then(|p| p.to_str()).map(|s| s.to_string()) {
                    cfg.set_value("last_dir_import_location", &source);
                    on_err_out!(tx.send(Event::SourceSelected(source, dest_dir.to_string())));
                }
            }
            on_err_out!(tx.send(Event::SetDirectoryName(source)));
            dialog.close()
        }));

        dialog.show();
    }

    fn copy_toggled(&self, toggle: bool) {
        if let Some(ref mut backend) = Rc::get_mut(&mut self.backend.borrow_mut()) {
            backend.set_copy(toggle);
        }
        self.cfg.set_value("dir_import_copy", &toggle.to_string());
    }
}

impl ImporterUI for DirectoryImporterUI {
    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> String {
        self.backend.borrow().id().to_string()
    }

    fn backend(&self) -> Rc<dyn ImportBackend> {
        self.backend.borrow().clone()
    }

    fn setup_widget(&self, parent: &gtk4::Window) -> gtk4::Widget {
        let builder = gtk4::Builder::from_resource("/org/gnome/Niepce/ui/directoryimporterui.ui");
        get_widget!(builder, gtk4::Box, main_widget);
        get_widget!(builder, gtk4::Button, select_directories);
        select_directories.connect_clicked(glib::clone!(@strong self.tx as tx =>
            move |_| on_err_out!(tx.send(Event::SelectDirectories));
        ));
        get_widget!(builder, gtk4::Label, directory_name);
        get_widget!(builder, gtk4::CheckButton, copy_files);
        copy_files.connect_toggled(glib::clone!(@strong self.tx as tx =>
            move |check| on_err_out!(tx.send(Event::CopyToggled(check.is_active())));
        ));
        copy_files.set_active(
            bool::from_str(&self.cfg.value("dir_import_copy", "false")).unwrap_or(false),
        );

        let mut widgets = self.widgets.borrow_mut();
        widgets.parent = Some(parent.clone());
        widgets.directory_name = Some(directory_name);

        main_widget.upcast::<gtk4::Widget>()
    }

    fn set_source_selected_callback(&self, callback: Box<dyn Fn(&str, &str)>) {
        self.widgets.borrow_mut().source_selected_cb = Some(callback);
    }
}
