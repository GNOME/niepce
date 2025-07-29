/*
 * niepce - niepce/ui/dialogs/import.rs
 *
 * Copyright (C) 2008-2025 Hubert Figuière
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

mod camera_importer_ui;
mod directory_importer_ui;
mod importer_ui;
mod thumb_item;
mod thumb_item_row;

use camera_importer_ui::CameraImporterUI;
use directory_importer_ui::DirectoryImporterUI;
use importer_ui::{ImporterMsg, ImporterUI};

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use gettextrs::gettext as i18n;
use gtk_macros::get_widget;
use gtk4::prelude::*;
use npc_fwk::{adw, gdk4, gio, glib, gtk4};
use num_traits::ToPrimitive;
use once_cell::sync::OnceCell;

use crate::niepce::ui::{ImageGridView, MetadataPaneController};
use npc_engine::importer::{DatePathFormat, ImportBackend, ImportRequest, ImportedFile};
use npc_fwk::toolkit::{
    self, Controller, ControllerImplCell, DialogController, ListViewRow, Receiver, Sender,
    Thumbnail, UiController,
};
use npc_fwk::{Date, controller_imp_imp, dbg_out, send_async_any};
use thumb_item::ThumbItem;
use thumb_item_row::ThumbItemRow;

pub enum Event {
    /// Set Source `source` and `dest_dir`
    SetSource(String, String),
    /// The source changed. `id` in the combo box.
    SourceChanged(String),
    /// The `DatePathFormat` has been changed.
    SetDatePathFormat(DatePathFormat),
    PreviewReceived(String, Option<Thumbnail>, Option<Date>),
    AppendFiles(Vec<Box<dyn ImportedFile>>),
    Cancel,
    Import,
}

struct Widgets {
    dialog: adw::Window,
    import_source_combo_model: Rc<toolkit::ComboModel<String>>,
    importer_ui_stack: gtk4::Stack,
    destination_folder: gtk4::Entry,
    images_list_model: gio::ListStore,

    importers: HashMap<String, Rc<dyn ImporterUI>>,
    current_importer: RefCell<Option<Rc<dyn ImporterUI>>>,
    importer_tx: Sender<ImporterMsg>,
}

impl Widgets {
    // XXX This could be a forwarder if ImportUI were a Controller.
    fn setup(&self, importer_rx: Receiver<ImporterMsg>, tx_out: Sender<Event>) {
        toolkit::channels::receiver_attach(importer_rx, move |msg| {
            match msg {
                ImporterMsg::SetSource(source, dest_dir) => {
                    npc_fwk::send_async_local!(Event::SetSource(source, dest_dir), tx_out);
                }
                ImporterMsg::SetCopy(_copy) => {
                    // XXX do something
                }
            }
        });
    }

    fn add_importer_ui(&mut self, importer: Rc<dyn ImporterUI>) {
        self.import_source_combo_model
            .push(importer.name(), importer.id());

        dbg_out!("setting up importer widget for {}", &importer.id());
        let importer_widget = importer.setup_widget(
            self.dialog.upcast_ref::<gtk4::Window>(),
            self.importer_tx.clone(),
        );
        self.importer_ui_stack
            .add_named(&importer_widget, Some(&importer.id()));

        self.importers.insert(importer.id(), importer.clone());
    }

    fn clear_import_list(&self) {
        self.images_list_model.remove_all();
        self.destination_folder.set_text("");
    }

    fn importer_changed(&self, source: &str) {
        self.current_importer
            .replace(self.importers.get(source).cloned());
        self.importer_ui_stack.set_visible_child_name(source);
    }
}

#[derive(Default)]
struct State {
    source: String,
    dest_dir: PathBuf,
    sorting_format: DatePathFormat,
    // map images name to position in list store.
    images_list_map: HashMap<String, u32>,
}

pub struct ImportDialog {
    imp_: ControllerImplCell<Event, ImportRequest>,
    base_dest_dir: PathBuf,
    cfg: Rc<toolkit::Configuration>,

    widgets: OnceCell<Widgets>,
    state: RefCell<State>,
}

impl Controller for ImportDialog {
    type InMsg = Event;
    type OutMsg = ImportRequest;

    controller_imp_imp!(imp_);

    fn dispatch(&self, e: Event) {
        match e {
            Event::SetSource(source, destdir) => self.set_source(&source, &destdir),
            Event::SourceChanged(source) => self.import_source_changed(&source),
            Event::SetDatePathFormat(f) => self.set_sorting_format(f),
            Event::PreviewReceived(path, thumbnail, date) => {
                self.preview_received(&path, thumbnail, date)
            }
            Event::AppendFiles(files) => self.append_files_to_import(&files),
            Event::Cancel => self.close(),
            Event::Import => {
                if let Some(request) = self.import_request() {
                    self.emit(request);
                }
                self.close();
            }
        }
    }
}

impl UiController for ImportDialog {
    fn widget(&self) -> &gtk4::Widget {
        self.dialog().upcast_ref()
    }
}

impl DialogController for ImportDialog {
    fn dialog(&self) -> &adw::Window {
        &self
            .widgets
            .get_or_init(|| {
                let builder =
                    gtk4::Builder::from_resource("/net/figuiere/Niepce/ui/importdialog.ui");
                get_widget!(builder, adw::Window, import_dialog);
                get_widget!(builder, gtk4::DropDown, date_tz_combo);
                let string_list =
                    toolkit::ComboModel::with_map(&[("Date is local", 0), ("Date is UTC", 1)]);
                string_list.bind(&date_tz_combo, |_| {});
                get_widget!(builder, gtk4::Button, cancel_button);
                let sender = self.sender();
                cancel_button.connect_clicked(move |_| {
                    send_async_any!(Event::Cancel, sender);
                });
                get_widget!(builder, gtk4::Button, import_button);
                let sender = self.sender();
                import_button.connect_clicked(move |_| {
                    send_async_any!(Event::Import, sender);
                });
                get_widget!(builder, gtk4::Entry, destination_folder);
                get_widget!(builder, gtk4::Stack, importer_ui_stack);

                get_widget!(builder, gtk4::DropDown, import_source_combo);
                let import_source_combo_model = toolkit::ComboModel::<String>::new();
                let sender = self.sender();
                import_source_combo_model.bind(&import_source_combo, move |value| {
                    let source = value.to_string();
                    send_async_any!(Event::SourceChanged(source), sender);
                });

                get_widget!(builder, gtk4::DropDown, date_sorting_combo);
                let string_list = toolkit::ComboModel::with_map(&[
                    (&i18n("No Sorting"), DatePathFormat::NoPath),
                    ("YYYYMMDD", DatePathFormat::YearMonthDay),
                    ("YYYY/MMDD", DatePathFormat::YearSlashMonthDay),
                    ("YYYY/MM/DD", DatePathFormat::YearSlashMonthSlashDay),
                    ("YYYY/YYYYMMDD", DatePathFormat::YearSlashYearMonthDay),
                ]);
                let sender = self.sender();
                string_list.bind(&date_sorting_combo, move |value| {
                    let format = *value;
                    dbg_out!("setting format {format:?}");
                    send_async_any!(Event::SetDatePathFormat(format), sender);
                });
                if let Some(sorting) = self
                    .cfg
                    .value_opt("import_sorting")
                    .and_then(|sorting| sorting.parse::<u32>().ok())
                {
                    date_sorting_combo.set_selected(sorting);
                }

                get_widget!(builder, gtk4::DropDown, preset_combo);
                let string_list = toolkit::ComboModel::with_map(&[(&i18n("No preset"), "NONE")]);
                string_list.bind(&preset_combo, |_| {});

                get_widget!(builder, gtk4::ScrolledWindow, attributes_scrolled);
                let metadata_pane = MetadataPaneController::new();
                let w = metadata_pane.widget();
                // add
                attributes_scrolled.set_child(Some(w));

                get_widget!(builder, gtk4::ScrolledWindow, images_list_scrolled);
                let images_list_model = gio::ListStore::new::<ThumbItem>();
                let selection_model = gtk4::SingleSelection::new(Some(images_list_model.clone()));
                let image_gridview = ImageGridView::new(selection_model, None, None);
                let factory = gtk4::SignalListItemFactory::new();
                image_gridview.set_factory(Some(&factory));
                factory.connect_setup(move |_, item| {
                    if let Some(list_item) = item.downcast_ref::<gtk4::ListItem>() {
                        let child = ThumbItemRow::new();
                        list_item.set_child(Some(&child));
                    }
                });
                factory.connect_bind(move |_, item| {
                    if let Some(list_item) = item.downcast_ref::<gtk4::ListItem>() {
                        if let Some(row) = list_item.child().and_downcast::<ThumbItemRow>() {
                            let thumb_item = list_item.item().and_downcast::<ThumbItem>().unwrap();
                            row.bind(&thumb_item, None);
                        }
                    }
                });
                factory.connect_unbind(move |_, item| {
                    if let Some(row) = item
                        .downcast_ref::<gtk4::ListItem>()
                        .and_then(|list_item| list_item.child().and_downcast::<ThumbItemRow>())
                    {
                        row.unbind();
                    }
                });

                images_list_scrolled.set_child(Some(&*image_gridview));
                images_list_scrolled
                    .set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);

                let (importer_tx, importer_rx) = npc_fwk::toolkit::channel();
                let mut widgets = Widgets {
                    dialog: import_dialog,
                    import_source_combo_model,
                    importer_ui_stack,
                    destination_folder,
                    images_list_model,
                    importers: HashMap::new(),
                    current_importer: RefCell::new(None),
                    importer_tx,
                };

                widgets.setup(importer_rx, self.sender());

                let importer = DirectoryImporterUI::new(self.cfg.clone());
                widgets.add_importer_ui(importer);
                let importer = CameraImporterUI::new();
                widgets.add_importer_ui(importer);

                let last_importer = self.cfg.value("last_importer", "DirectoryImporter");
                if let Some(selected) = widgets.import_source_combo_model.index_of(&last_importer) {
                    import_source_combo.set_selected(selected as u32);
                }

                widgets
            })
            .dialog
    }
}

impl ImportDialog {
    pub fn new(cfg: Rc<toolkit::Configuration>) -> Rc<Self> {
        let base_dest_dir = cfg
            .value_opt("base_import_dest_dir")
            .map(PathBuf::from)
            .or_else(|| glib::user_special_dir(glib::UserDirectory::Pictures))
            .unwrap_or_else(glib::home_dir);
        let dialog = Rc::new(ImportDialog {
            imp_: ControllerImplCell::default(),
            base_dest_dir,
            cfg,
            widgets: OnceCell::new(),
            state: RefCell::new(State::default()),
        });

        <Self as DialogController>::start(&dialog);

        dialog
    }

    pub fn import_request(&self) -> Option<ImportRequest> {
        self.widgets
            .get()?
            .current_importer
            .borrow()
            .as_ref()
            .map(|importer| {
                ImportRequest::new(self.source(), self.dest_dir(), importer.backend())
                    .set_sorting(self.sorting_format())
            })
    }

    fn clear_import_list(&self) {
        if let Some(widgets) = self.widgets.get() {
            widgets.clear_import_list();
        }
        let mut state = self.state.borrow_mut();
        state.images_list_map.clear();
    }

    fn import_source_changed(&self, source: &str) {
        if let Some(widgets) = self.widgets.get() {
            widgets.importer_changed(source);
            self.state.borrow_mut().source = "".to_string();
            self.clear_import_list();
            self.cfg.set_value("last_importer", source);
        }
    }

    /// Get importer backend from the importer UI.
    fn importer(&self) -> Option<Rc<dyn ImportBackend>> {
        self.widgets
            .get()?
            .current_importer
            .borrow()
            .as_ref()
            .map(|v| v.backend())
    }

    fn set_source(&self, source: &str, dest_dir: &str) {
        self.clear_import_list();

        if let Some(importer) = self.importer() {
            let sender = self.sender();
            importer.list_source_content(
                source,
                Box::new(move |files| {
                    npc_fwk::send_async_any!(Event::AppendFiles(files), sender);
                }),
            );
        }

        let full_dest_dir = self.base_dest_dir.join(dest_dir);
        let mut state = self.state.borrow_mut();
        state.source = source.to_string();
        state.dest_dir = full_dest_dir;

        if let Some(widgets) = self.widgets.get() {
            widgets.destination_folder.set_text(dest_dir);
        }
    }

    fn sorting_format(&self) -> DatePathFormat {
        self.state.borrow().sorting_format
    }

    /// Set the date sorting format.
    fn set_sorting_format(&self, format: DatePathFormat) {
        let mut state = self.state.borrow_mut();
        state.sorting_format = format;
        // XXX handle the UI
        if let Some(sorting) = format.to_u32() {
            self.cfg.set_value("import_sorting", &sorting.to_string());
        }
    }

    fn append_files_to_import(&self, files: &[Box<dyn ImportedFile>]) {
        let paths: Vec<String> = files
            .iter()
            .map(|f| {
                let path = f.path();
                dbg_out!("selected {}", &path);
                if let Some(widgets) = self.widgets.get() {
                    widgets
                        .images_list_model
                        .append(&ThumbItem::new(f.as_ref()));
                    self.state
                        .borrow_mut()
                        .images_list_map
                        .insert(path.to_string(), widgets.images_list_model.n_items() - 1);
                }
                path.to_string()
            })
            .collect();

        if let Some(importer) = self.importer() {
            let sender = self.sender();
            importer.get_previews_for(
                &self.state.borrow().source,
                paths,
                Box::new(move |path, thumbnail, date| {
                    npc_fwk::send_async_any!(Event::PreviewReceived(path, thumbnail, date), sender);
                }),
            );
        }
    }

    /// A preview was received. Update the UI to show it in the grid view.
    /// And eventually a date (original).
    /// If either `thumbnail` and `date` are `None` then this is a no-op.
    fn preview_received(&self, path: &str, thumbnail: Option<Thumbnail>, date: Option<Date>) {
        if thumbnail.is_none() && date.is_none() {
            return;
        }

        dbg_out!("preview and date received {:?}", date);

        if let Some(idx) = self.state.borrow_mut().images_list_map.get(path) {
            self.widgets.get().and_then(|widgets| {
                widgets
                    .images_list_model
                    .item(*idx)
                    .and_downcast::<ThumbItem>()
                    .inspect(|item| {
                        item.set_pixbuf(thumbnail.map(|ref t| {
                            let texture: gdk4::Texture = t.into();
                            texture.upcast::<gdk4::Paintable>()
                        }));
                        item.set_date(date);
                    })
            });
        }
    }

    fn source(&self) -> String {
        self.state.borrow().source.clone()
    }

    fn dest_dir(&self) -> PathBuf {
        self.state.borrow().dest_dir.to_path_buf()
    }
}
