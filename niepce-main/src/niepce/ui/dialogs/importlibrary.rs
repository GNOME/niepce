/*
 * niepce - niepce/ui/dialogs/importlibrary.rs
 *
 * Copyright (C) 2021-2025 Hubert Figuière
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

mod lrimport_root_row;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use gettextrs::gettext as i18n;
use gtk4::Builder;
use gtk4::prelude::*;
use i18n_format::i18n_fmt;
use npc_fwk::{gio, glib, gtk4};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use npc_engine::importer::LibraryImporter;
use npc_engine::libraryclient::LibraryClient;
use npc_fwk::toolkit::{self, Controller, ControllerImplCell};
use npc_fwk::{controller_imp_imp, dbg_out, err_out, on_err_out};

use lrimport_root_row::LrImportRootRow;

#[repr(i32)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
enum Page {
    FileSelection = 0,
    Roots = 1,
    StartImport = 2,
    Progress = 3,
    Done = 4,
}

#[derive(Default)]
struct ImportState {
    /// The path of the library to import
    library_path: Option<PathBuf>,
    /// The importer
    importer: Option<Box<dyn LibraryImporter>>,
    /// The widget to set the label to.
    importer_name_label: Option<gtk4::Label>,
    /// The import button.
    import_file_button: Option<gtk4::Button>,
    /// The map for root folders.
    root_remapping: HashMap<String, (String, bool)>,
}

impl ImportState {
    /// Remap a root folder
    fn remap_root(&mut self, p: String, v: String) {
        if let Some(ref mut entry) = self.root_remapping.get_mut(&p) {
            entry.0 = v;
        } else {
            self.root_remapping.insert(p, (v, true));
        }
    }

    /// Enable/disable root folder
    fn enable_root(&mut self, p: &str, b: bool) {
        if let Some(ref mut entry) = self.root_remapping.get_mut(p) {
            entry.1 = b;
        }
    }

    /// Perform the root remap
    fn importer_root_remap(&mut self) {
        if let Some(importer) = &mut self.importer {
            for (root, (dest, enabled)) in &self.root_remapping {
                if *enabled {
                    importer.map_root_folder(root, dest);
                }
            }
        }
    }
}

type ImportStateRef = Rc<RefCell<ImportState>>;

#[derive(Clone)]
pub enum Command {
    /// Request to select a file
    SelectFile,
    /// File selector accept
    SetFile(std::path::PathBuf),
    /// We just located a root. Sent for each.
    FoundRoot(String),
    /// We are done loading root
    RootsDone,
    /// Update the remap
    Remap((String, String)),
    /// Update the root enabled
    SetRootEnabled((String, bool)),
    /// Close the assistant
    Close,
}

pub struct ImportLibraryDialog {
    imp_: ControllerImplCell<Command, ()>,
    #[allow(deprecated)]
    assistant: gtk4::Assistant,
    client: Arc<LibraryClient>,
    state: ImportStateRef,
    roots_list: Option<gtk4::ListBox>,
}

impl Controller for ImportLibraryDialog {
    type InMsg = Command;
    type OutMsg = ();

    controller_imp_imp!(imp_);

    fn dispatch(&self, command: Command) {
        match command {
            Command::SetFile(p) => self.library_file_set(p),
            Command::SelectFile => self.select_file(),
            Command::FoundRoot(p) => {
                #[allow(deprecated)]
                if self.assistant.current_page() == Page::Roots.to_i32().unwrap() {
                    // add the root to the list.
                    self.state.borrow_mut().remap_root(p.clone(), p.clone());
                    if let Some(roots_list) = &self.roots_list {
                        let row = LrImportRootRow::new(p.clone());

                        let sender = self.sender();
                        let p2 = p.clone();
                        row.connect_changed(move |w| {
                            let v = w.text().to_string();
                            let p = p2.clone();
                            npc_fwk::send_async_local!(Command::Remap((p, v)), sender);
                        });

                        let sender = self.sender();
                        row.connect_toggled(move |w| {
                            let v = w.is_active();
                            let p = p.clone();
                            npc_fwk::send_async_local!(Command::SetRootEnabled((p, v)), sender);
                        });
                        roots_list.insert(&row, -1);
                    }
                } else {
                    err_out!("Received FoundRoot({:?}) on wrong page", p);
                }
            }
            Command::RootsDone => {
                #[allow(deprecated)]
                if self.assistant.current_page() == Page::Roots.to_i32().unwrap() {
                    // we are done with all.
                    self.set_page_complete(Page::Roots);
                } else {
                    err_out!("Received RootsDone on wrong page");
                }
            }
            Command::Remap((p, v)) => {
                dbg_out!("Remap {} to {}", p, v);
                self.state.borrow_mut().remap_root(p, v);
            }
            Command::SetRootEnabled((p, v)) => {
                dbg_out!("{} {}", if v { "Enable" } else { "Disable" }, p);
                self.state.borrow_mut().enable_root(&p, v);
            }
            Command::Close => {
                self.cancel();
            }
        }
    }
}

impl ImportLibraryDialog {
    #[allow(deprecated)]
    pub fn new(client: Arc<LibraryClient>) -> Rc<Self> {
        let assistant = gtk4::Assistant::new();

        let mut dlg = Rc::new(Self {
            imp_: ControllerImplCell::default(),
            assistant: assistant.clone(),
            client,
            state: Rc::new(RefCell::new(ImportState::default())),
            roots_list: None,
        });

        <Self as Controller>::start(&dlg);

        let sender = dlg.sender();
        assistant.connect_cancel(move |_| {
            npc_fwk::send_async_local!(Command::Close, sender);
        });
        let sender = dlg.sender();
        assistant.connect_close(move |_| {
            npc_fwk::send_async_local!(Command::Close, sender);
        });
        assistant.set_forward_page_func(Self::forward_page);

        let builder = Builder::new();
        if let Err(result) = builder.add_from_resource("/net/figuiere/Niepce/ui/importlibrary.ui") {
            // XXX fix the we should actually report the error
            err_out!("couldn't find ui file: {}", result);
            return dlg;
        }
        if let Some(page) = builder.object::<gtk4::Widget>("fileselection-page") {
            let idx = assistant.insert_page(&page, -1);
            assistant.set_page_type(&page, gtk4::AssistantPageType::Intro);
            assistant.set_current_page(idx);
            toolkit::assistant::set_page_index(&page, Page::FileSelection.to_i32().unwrap());

            if let Some(file_chooser) = builder.object::<gtk4::Button>("file_chooser") {
                let sender = dlg.sender();
                file_chooser.connect_clicked(move |_| {
                    npc_fwk::send_async_local!(Command::SelectFile, sender);
                });
                dlg.state.borrow_mut().import_file_button = Some(file_chooser);
            }
        }
        if let Some(page) = builder.object::<gtk4::Widget>("roots-page") {
            assistant.insert_page(&page, -1);
            toolkit::assistant::set_page_index(&page, Page::Roots.to_i32().unwrap());

            dlg.state.borrow_mut().importer_name_label =
                builder.object::<gtk4::Label>("roots-importer-name");
        }
        if let Some(page) = builder.object::<gtk4::Widget>("start-import-page") {
            assistant.insert_page(&page, -1);
            assistant.set_page_type(&page, gtk4::AssistantPageType::Confirm);
            toolkit::assistant::set_page_index(&page, Page::StartImport.to_i32().unwrap());
        }

        if let Some(page) = builder.object::<gtk4::Widget>("import-progress-page") {
            assistant.insert_page(&page, -1);
            assistant.set_page_type(&page, gtk4::AssistantPageType::Progress);
            toolkit::assistant::set_page_index(&page, Page::Progress.to_i32().unwrap());
        }

        if let Some(page) = builder.object::<gtk4::Widget>("done-page") {
            assistant.insert_page(&page, -1);
            assistant.set_page_type(&page, gtk4::AssistantPageType::Summary);
            toolkit::assistant::set_page_index(&page, Page::Done.to_i32().unwrap());
        }

        if let Some(ref mut dlg) = Rc::get_mut(&mut dlg) {
            dlg.roots_list = builder.object::<gtk4::ListBox>("roots-list");
        }

        <Self as Controller>::start(&dlg);

        assistant.connect_prepare(glib::clone!(
            #[strong]
            dlg,
            move |_, p| dlg.prepare_page(p)
        ));

        dlg
    }

    #[allow(deprecated)]
    pub fn run(&self, parent: Option<&gtk4::Window>) {
        self.assistant.set_transient_for(parent);
        self.assistant.set_modal(true);
        self.assistant.present();
    }

    fn forward_page(current: i32) -> i32 {
        dbg_out!("forward_page: {}", current);
        match Page::from_i32(current) {
            Some(Page::FileSelection) => Page::Roots.to_i32().unwrap(),
            Some(Page::Roots) => Page::StartImport.to_i32().unwrap(),
            Some(Page::StartImport) => Page::Progress.to_i32().unwrap(),
            Some(Page::Progress) => Page::Done.to_i32().unwrap(),
            _ => Page::FileSelection.to_i32().unwrap(),
        }
    }

    /// Prepare to display the roots folder page.
    fn prepare_roots_page(&self) {
        // Clear the list.
        if let Some(roots_list) = &self.roots_list {
            while let Some(child) = roots_list.first_child() {
                child.unparent();
                roots_list.remove(&child);
            }
        }

        let label = self.state.borrow().importer_name_label.clone();
        if let Some(ref mut importer) = self.state.borrow_mut().importer {
            if let Some(label) = label {
                let importing = i18n_fmt! {
                    // Translators: {} is the import source.
                    i18n_fmt("Importing from {}", importer.name())
                };
                label.set_text(&importing);
            }
            let roots = importer.root_folders();
            for root in roots {
                dbg_out!("Found root folder {}", &root);
                npc_fwk::send_async_local!(Command::FoundRoot(root), self.sender());
            }
            npc_fwk::send_async_local!(Command::RootsDone, self.sender());
        }
    }

    fn prepare_page(&self, page: &gtk4::Widget) {
        let idx = toolkit::assistant::get_page_index(page).and_then(Page::from_i32);
        dbg_out!("Prepare page {:?}", idx);
        match idx {
            Some(Page::Roots) => self.prepare_roots_page(),
            Some(Page::StartImport) => self.set_page_complete(Page::StartImport),
            Some(Page::Progress) => self.perform_import(),
            _ => {}
        }
    }

    /// Prepare the progress page. Actually run the import.
    fn perform_import(&self) {
        dbg_out!("Perform import");
        self.state.borrow_mut().importer_root_remap();
        if let Some(importer) = &mut self.state.borrow_mut().importer {
            importer
                .import_library(&self.client)
                .expect("import library");
        }
        self.set_page_complete(Page::Progress);
        #[allow(deprecated)]
        self.assistant.commit();
    }

    #[allow(deprecated)]
    fn set_page_complete(&self, page: Page) {
        let idx = page.to_i32().unwrap();
        if let Some(w) = self.assistant.nth_page(idx) {
            self.assistant.set_page_complete(&w, true);
        } else {
            err_out!("Couldn't find page {:?}", page);
        }
    }

    fn library_file_set(&self, path: std::path::PathBuf) {
        // Check if we can find an importer.
        let mut importer = npc_engine::importer::find_importer(&path);
        if let Some(ref mut importer) = importer {
            // XXX actually handle the error. This should be a failure.
            on_err_out!(importer.init_importer(&path));
        }

        if let Some(ref button) = self.state.borrow().import_file_button {
            button.set_label(&path.to_string_lossy());
        }
        self.state.borrow_mut().library_path = Some(path);
        self.state.borrow_mut().importer = importer;
        dbg_out!("Page FileSelection complete");
        self.set_page_complete(Page::FileSelection);
    }

    fn select_file(&self) {
        let message = i18n("Select Lightroom™ Library");
        #[allow(deprecated)]
        let file_dialog = gtk4::FileChooserDialog::new(
            Some(&message),
            Some(&self.assistant),
            gtk4::FileChooserAction::Open,
            &[
                (&i18n("Import"), gtk4::ResponseType::Accept),
                (&i18n("Cancel"), gtk4::ResponseType::Cancel),
            ],
        );
        let sender = self.sender();
        #[allow(deprecated)]
        file_dialog.connect_response(move |d, response| {
            if response == gtk4::ResponseType::Accept {
                dbg_out!("Accept");
                #[allow(deprecated)]
                if let Some(file) = d.file().as_ref().and_then(gio::prelude::FileExt::path) {
                    dbg_out!("Lr file: {:?}", file);
                    npc_fwk::send_async_local!(Command::SetFile(file), sender);
                }
            }
            d.close();
        });

        file_dialog.present();
    }

    fn cancel(&self) {
        dbg_out!("Assistant cancel");
        self.assistant.destroy();
    }
}
