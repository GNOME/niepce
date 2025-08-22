/*
 * niepce - ui/dialogs/import/directory_importer_ui.rs
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
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;

use gettextrs::gettext as i18n;
use gtk4::prelude::*;
use npc_fwk::{adw, gio, glib, gtk4};

use npc_engine::importer::{DirectoryImporter, ImportBackend};
use npc_fwk::toolkit::{Controller, ControllerImplCell, Sender};
use npc_fwk::{controller_imp_imp, on_err_out, toolkit};

use super::{ImporterMsg, ImporterUI};

pub enum Event {
    /// Copuy has been toggled.
    CopyToggled(bool),
    /// Recurisve has been toggled.
    RecursiveToggled(bool),
    /// Request the UI to select directories.
    SelectDirectories,
    /// Internal set the directory name in the widget.
    SetDirectoryName(Option<PathBuf>),
    /// Source directory was selected.
    SourceSelected(String),
    /// We need to rescan
    RefreshSource,
}

#[derive(Default)]
struct State {
    source: Option<String>,
}

#[derive(Default)]
struct Widgets {
    directory_name: Option<adw::ButtonContent>,
    parent: Option<gtk4::Window>,
    copy_files: Option<gtk4::CheckButton>,
    tx: Option<Sender<ImporterMsg>>,
}

pub(super) struct DirectoryImporterUI {
    imp_: ControllerImplCell<Event, ()>,
    name: String,
    cfg: Rc<toolkit::Configuration>,
    backend: RefCell<Rc<DirectoryImporter>>,
    widgets: RefCell<Widgets>,
    state: RefCell<State>,
}

impl Controller for DirectoryImporterUI {
    type InMsg = Event;
    type OutMsg = ();

    controller_imp_imp!(imp_);

    fn dispatch(&self, e: Event) {
        match e {
            Event::CopyToggled(t) => self.copy_toggled(t),
            Event::RecursiveToggled(t) => self.recursive_toggled(t),
            Event::SelectDirectories => self.do_select_directories(),
            Event::SetDirectoryName(name) => {
                if let Some(directory_name) = &self.widgets.borrow().directory_name {
                    if let Some(name) = name {
                        directory_name.set_label(name.to_str().unwrap_or(""));
                    } else {
                        directory_name.set_label(&i18n("Choose Directory"));
                    }
                }
            }
            Event::SourceSelected(source) => {
                if let Some(tx) = &self.widgets.borrow().tx.clone() {
                    let source = source.clone();
                    let is_copy = self.backend.borrow().copy();
                    {
                        let mut state = self.state.borrow_mut();
                        state.source = Some(source.clone());
                    }
                    npc_fwk::send_async_local!(ImporterMsg::SetSource(Some(source), is_copy), tx);
                }
            }
            Event::RefreshSource => {
                if let Some(tx) = &self.widgets.borrow().tx.clone()
                    && let Some(source) = self.state.borrow().source.clone()
                {
                    npc_fwk::send_async_local!(ImporterMsg::RefreshSource(Some(source)), tx);
                }
            }
        }
    }
}

impl DirectoryImporterUI {
    pub fn new(cfg: Rc<toolkit::Configuration>) -> Rc<DirectoryImporterUI> {
        let widget = Rc::new(DirectoryImporterUI {
            imp_: ControllerImplCell::default(),
            name: i18n("Directory"),
            cfg,
            backend: RefCell::new(Rc::new(DirectoryImporter::default())),
            widgets: RefCell::default(),
            state: RefCell::default(),
        });

        <Self as Controller>::start(&widget);

        widget
    }

    fn do_select_directories(&self) {
        #[allow(deprecated)]
        let dialog = gtk4::FileChooserDialog::new(
            Some(&i18n("Import picture folder")),
            self.widgets.borrow().parent.as_ref(),
            gtk4::FileChooserAction::SelectFolder,
            &[
                (&i18n("Cancel"), gtk4::ResponseType::Cancel),
                (&i18n("Select"), gtk4::ResponseType::Ok),
            ],
        );
        #[allow(deprecated)]
        dialog.set_select_multiple(false);

        if let Some(last_import_location) = self.cfg.value_opt("last_dir_import_location") {
            let file = gio::File::for_path(last_import_location);
            #[allow(deprecated)]
            let result = dialog.set_current_folder(Some(&file));
            on_err_out!(result);
        }
        let sender = self.sender();
        #[allow(deprecated)]
        dialog.connect_response(glib::clone!(
            #[strong]
            sender,
            #[weak(rename_to = cfg)]
            self.cfg,
            move |dialog, response| {
                if response == gtk4::ResponseType::DeleteEvent {
                    return;
                }
                let mut source = None;
                #[allow(deprecated)]
                if response == gtk4::ResponseType::Ok {
                    source = dialog.file().and_then(|f| f.path());
                    if let Some(source) = source
                        .as_ref()
                        .and_then(|p| p.to_str())
                        .map(|s| s.to_string())
                    {
                        cfg.set_value("last_dir_import_location", &source);
                        npc_fwk::send_async_local!(Event::SourceSelected(source), sender);
                    }
                }
                npc_fwk::send_async_local!(Event::SetDirectoryName(source), sender);
                dialog.close()
            }
        ));

        dialog.present();
    }

    fn copy_toggled(&self, toggle: bool) {
        if let Some(ref mut backend) = Rc::get_mut(&mut self.backend.borrow_mut()) {
            backend.set_copy(toggle);
        }
        self.cfg.set_value("dir_import_copy", &toggle.to_string());
        if let Some(tx) = &self.widgets.borrow().tx.clone() {
            let source = self.state.borrow().source.clone();
            npc_fwk::send_async_local!(ImporterMsg::SetSource(source, toggle), tx);
        }
    }

    fn recursive_toggled(&self, toggle: bool) {
        if let Some(ref mut backend) = Rc::get_mut(&mut self.backend.borrow_mut()) {
            backend.set_recursive(toggle);
        }
        self.cfg
            .set_value("dir_import_recursive", &toggle.to_string());
        let sender = self.sender();
        npc_fwk::send_async_local!(Event::RefreshSource, sender);
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

    fn setup_widget(&self, parent: &gtk4::Window, tx: Sender<ImporterMsg>) -> gtk4::Widget {
        let builder =
            gtk4::Builder::from_resource("/net/figuiere/Niepce/ui/directory_importer_ui.ui");
        get_widget!(builder, gtk4::Grid, main_widget);
        get_widget!(builder, gtk4::Button, select_directories);
        let sender = self.sender();
        select_directories.connect_clicked(glib::clone!(
            #[strong]
            sender,
            move |_| npc_fwk::send_async_local!(Event::SelectDirectories, sender),
        ));
        get_widget!(builder, adw::ButtonContent, select_dir_content);
        get_widget!(builder, gtk4::CheckButton, copy_files);
        get_widget!(builder, gtk4::CheckButton, recursive);
        let sender = self.sender();
        copy_files.connect_toggled(glib::clone!(
            #[strong]
            sender,
            move |check| {
                let is_active = check.is_active();
                npc_fwk::send_async_local!(Event::CopyToggled(is_active), sender);
            }
        ));
        copy_files.set_active(
            bool::from_str(&self.cfg.value("dir_import_copy", "false")).unwrap_or(false),
        );
        let sender = self.sender();
        recursive.connect_toggled(glib::clone!(
            #[strong]
            sender,
            move |check| {
                let is_active = check.is_active();
                npc_fwk::send_async_local!(Event::RecursiveToggled(is_active), sender);
            }
        ));
        recursive.set_active(
            bool::from_str(&self.cfg.value("dir_import_recursive", "false")).unwrap_or(false),
        );

        let mut widgets = self.widgets.borrow_mut();
        widgets.parent = Some(parent.clone());
        widgets.directory_name = Some(select_dir_content);
        widgets.copy_files = Some(copy_files);
        widgets.tx = Some(tx);

        main_widget.upcast::<gtk4::Widget>()
    }

    fn state_update(&self) {
        let widgets = self.widgets.borrow();
        if let Some(tx) = &widgets.tx.clone() {
            let source = self.state.borrow().source.clone();
            let is_copy = widgets
                .copy_files
                .as_ref()
                .map(|check| check.is_active())
                .unwrap_or(false);
            npc_fwk::send_async_local!(ImporterMsg::SetSource(source, is_copy), tx);
        }
    }
}
