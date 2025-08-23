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
mod dest_folders;
mod directory_importer_ui;
mod importer_ui;
mod thumb_item;
mod thumb_item_row;

use camera_importer_ui::CameraImporterUI;
use directory_importer_ui::DirectoryImporterUI;
use importer_ui::{ImporterMsg, ImporterUI};

use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use gettextrs::gettext as i18n;
use gtk_macros::get_widget;
use gtk4::prelude::*;
use i18n_format::i18n_fmt;
use npc_fwk::{adw, gdk4, gio, glib, gtk4};
use num_traits::ToPrimitive;

use crate::niepce::ui::{ImageGridView, MetadataPaneController};
use dest_folders::DestFoldersIn;
use npc_engine::importer::{DatePathFormat, ImportBackend, ImportRequest, ImportedFile, Importer};
use npc_engine::libraryclient::LibraryClient;
use npc_fwk::base::Executor;
use npc_fwk::toolkit::{
    self, Controller, ControllerImplCell, DialogController, ListViewRow, Receiver, Sender,
    Thumbnail, UiController,
};
use npc_fwk::utils::normalize_for_display;
use npc_fwk::{Date, controller_imp_imp, dbg_out, err_out, send_async_any, trace_out};
use thumb_item::ThumbItem;
use thumb_item_row::ThumbItemRow;

pub enum Event {
    /// Set Source `source` and `copy`
    SetSource(Option<String>, bool),
    /// Sent when the source needs to be refreshed.
    RefreshSource(Option<String>),
    /// The import source changed. `id` in the combo box. The import
    /// source is either directory or camera (currently).
    ImportSourceChanged(String),
    /// The destination was changed in the UI. Path of the dest folder.
    DestChanged(PathBuf),
    /// The `DatePathFormat` has been changed.
    SetDatePathFormat(DatePathFormat),
    PreviewReceived(String, Option<Thumbnail>, Option<Date>),
    PreviewsDone,
    AppendFiles(Vec<Box<dyn ImportedFile>>),
    Cancel,
    Import,
}

struct Widgets {
    dialog: adw::Window,
    import_source_combo_model: Rc<toolkit::ComboModel<String>>,
    importer_ui_stack: gtk4::Stack,
    dest_folders: Rc<dest_folders::DestFolders>,
    destination_help: gtk4::Label,
    images_list_model: gio::ListStore,
    image_count: gtk4::Label,
    preview_spinner: gtk4::Spinner,

    importers: HashMap<String, Rc<dyn ImporterUI>>,
    current_importer: RefCell<Option<Rc<dyn ImporterUI>>>,
    importer_tx: Sender<ImporterMsg>,
}

impl Widgets {
    // XXX This could be a forwarder if ImportUI were a Controller.
    fn setup(&self, importer_rx: Receiver<ImporterMsg>, tx_out: Sender<Event>) {
        toolkit::channels::receiver_attach(importer_rx, move |msg| match msg {
            ImporterMsg::SetSource(source, copy) => {
                npc_fwk::send_async_local!(Event::SetSource(source, copy), tx_out);
            }
            ImporterMsg::RefreshSource(source) => {
                npc_fwk::send_async_local!(Event::RefreshSource(source), tx_out);
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
        //
        self.dest_folders.send(DestFoldersIn::Clear);
    }

    fn importer_changed(&self, source: &str) {
        let importer = self.importers.get(source);
        self.current_importer.replace(importer.cloned());
        self.importer_ui_stack.set_visible_child_name(source);
    }

    fn set_copy_mode(&self, copy: bool) {
        trace_out!("set_copy_mode(): {copy}");
        self.dest_folders.set_copy_mode(copy);
    }
}

struct DestEntry {
    idx: u32,
    dest: Option<PathBuf>,
}

#[derive(Default)]
struct State {
    source: Option<String>,
    import_count: usize,
    copy_dest_dir: PathBuf,
    full_dest_dir: Option<PathBuf>,
    copy: bool,
    sorting_disabled: bool,
    sorting_format: DatePathFormat,
}

pub struct ImportDialog {
    imp_: ControllerImplCell<Event, ImportRequest>,
    cfg: Rc<toolkit::Configuration>,
    client: Arc<LibraryClient>,

    widgets: OnceCell<Widgets>,
    state: RefCell<State>,
    // map images name to position in list store.
    images_list_map: RefCell<HashMap<String, DestEntry>>,

    list_task: Executor,
    thumbnail_task: Executor,
}

impl Controller for ImportDialog {
    type InMsg = Event;
    type OutMsg = ImportRequest;

    controller_imp_imp!(imp_);

    fn dispatch(&self, e: Event) {
        match e {
            Event::SetSource(source, copy) => self.set_source(source.as_deref(), copy),
            Event::RefreshSource(source) => self.refresh_source(source.as_deref()),
            Event::ImportSourceChanged(source) => self.import_source_changed(&source),
            Event::DestChanged(dest_dir) => self.handle_dest_changed(Some(dest_dir)),
            Event::SetDatePathFormat(f) => {
                if let Some(widgets) = self.widgets.get() {
                    widgets.dest_folders.send(DestFoldersIn::SortingChanged(f));
                }
                self.set_sorting_format(f)
            }
            Event::PreviewReceived(path, thumbnail, date) => {
                if let Some(widgets) = self.widgets.get() {
                    widgets
                        .dest_folders
                        .send(DestFoldersIn::PreviewReceived(date));
                }
                self.preview_received(&path, thumbnail, date)
            }
            Event::PreviewsDone => {
                if let Some(widgets) = self.widgets.get() {
                    widgets.preview_spinner.stop();
                }
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
                let builder = gtk4::Builder::from_resource("/net/figuiere/Niepce/ui/import.ui");
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
                get_widget!(builder, gtk4::ListView, destination_folders);
                let dest_folders = dest_folders::DestFolders::new(
                    self.client.clone(),
                    destination_folders,
                    &self.cfg,
                );
                get_widget!(builder, gtk4::Label, image_count);
                get_widget!(builder, gtk4::Spinner, preview_spinner);
                let sender = self.sender();
                dest_folders.set_forwarder(Some(Box::new(glib::clone!(move |event| {
                    use dest_folders::DestFoldersOut::*;
                    match event {
                        SelectedFolder(dest_dir) => {
                            trace_out!("Selected folder {}", dest_dir.name());
                            let dest_dir = dest_dir.dest().clone();
                            trace_out!("DestChanged {dest_dir:?}");
                            send_async_any!(Event::DestChanged(dest_dir), sender);
                        }
                        DeselectAll => {
                            dbg_out!("Deselected All");
                            // XXX todo
                        }
                    }
                }))));
                get_widget!(builder, gtk4::Stack, importer_ui_stack);

                get_widget!(builder, gtk4::DropDown, import_source_combo);
                let import_source_combo_model = toolkit::ComboModel::<String>::new();
                let sender = self.sender();
                import_source_combo_model.bind(&import_source_combo, move |value| {
                    let source = value.to_string();
                    send_async_any!(Event::ImportSourceChanged(source), sender);
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
                    trace_out!("setting format {format:?}");
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

                get_widget!(builder, gtk4::Label, destination_help);

                let (importer_tx, importer_rx) = npc_fwk::toolkit::channel();
                let mut widgets = Widgets {
                    dialog: import_dialog,
                    import_source_combo_model,
                    importer_ui_stack,
                    dest_folders,
                    destination_help,
                    images_list_model,
                    image_count,
                    preview_spinner,
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
    pub fn new(client: Arc<LibraryClient>, cfg: Rc<toolkit::Configuration>) -> Rc<Self> {
        let state = State {
            copy_dest_dir: cfg
                .value_opt("base_import_dest_dir")
                .map(PathBuf::from)
                .or_else(|| glib::user_special_dir(glib::UserDirectory::Pictures))
                .unwrap_or_else(glib::home_dir),
            ..Default::default()
        };
        dbg_out!("base import dest dir {:?}", state.copy_dest_dir);
        let dialog = Rc::new(ImportDialog {
            imp_: ControllerImplCell::default(),
            cfg,
            client,
            widgets: OnceCell::new(),
            state: RefCell::new(state),
            images_list_map: RefCell::default(),
            list_task: Executor::new("list content".into()),
            thumbnail_task: Executor::new("import thumbnailing".into()),
        });

        <Self as DialogController>::start(&dialog);

        dialog
    }

    pub fn import_request(&self) -> Option<ImportRequest> {
        let source = self.source();
        if source.is_none() {
            err_out!("Requested import without source");
            return None;
        }
        let dest_dir = self.dest_dir();
        if dest_dir.is_none() {
            err_out!("Requested import without dest");
            return None;
        }
        self.widgets
            .get()?
            .current_importer
            .borrow()
            .as_ref()
            .map(|importer| {
                ImportRequest::new(
                    source.unwrap(),
                    dest_dir.as_deref().unwrap(),
                    importer.backend(),
                )
                .set_sorting(self.sorting_format())
            })
    }

    fn update_import_count(&self) {
        if let Some(widgets) = self.widgets.get() {
            let import_count = self.state.borrow().import_count;
            widgets.image_count.set_label(&i18n_fmt! {
                i18n_fmt("{} _Images to import", import_count)
            });
        }
    }

    fn clear_import_list(&self) {
        if let Some(widgets) = self.widgets.get() {
            widgets.clear_import_list();
        }
        self.images_list_map.borrow_mut().clear();
        self.state.borrow_mut().import_count = 0;
        self.update_import_count();
    }

    /// The import source change: dir or camera.
    fn import_source_changed(&self, source: &str) {
        if let Some(widgets) = self.widgets.get() {
            widgets.importer_changed(source);
            widgets
                .current_importer
                .borrow()
                .as_ref()
                .inspect(|importer| importer.state_update());
            self.state.borrow_mut().source = None;
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

    fn refresh_source(&self, source: Option<&str>) {
        self.clear_import_list();

        if let Some(importer) = self.importer()
            && let Some(source) = source
        {
            if let Some(widgets) = self.widgets.get() {
                widgets.preview_spinner.start();
            }
            let sender = self.sender();
            importer.list_source_content(
                &self.list_task,
                source,
                Box::new(move |files| {
                    npc_fwk::send_async_any!(Event::AppendFiles(files), sender);
                }),
            );
        }

        self.state.borrow_mut().source = source.map(|x| x.to_string());
    }

    fn set_source(&self, source: Option<&str>, copy: bool) {
        trace_out!("Set source {source:?} {copy}");
        let dest_dir = if !copy {
            source.as_ref().map(PathBuf::from)
        } else {
            Some(self.state.borrow().copy_dest_dir.clone())
        };
        if self.state.borrow().source.as_deref() != source {
            self.refresh_source(source);
        }
        self.set_copy(copy);
        // Select the destdir in the list
        self.widgets.get().inspect(|widgets| {
            widgets
                .dest_folders
                .send(DestFoldersIn::SelectPath(dest_dir.clone()));
        });
    }

    fn set_copy(&self, copy: bool) {
        trace_out!("set copy {copy}");
        self.widgets.get().unwrap().set_copy_mode(copy);
        self.state.borrow_mut().copy = copy;
        self.state.borrow_mut().sorting_disabled = !copy;
        self.widgets.get().inspect(|widgets| {
            widgets
                .dest_folders
                .send(DestFoldersIn::SortingChanged(self.sorting_format()));
        });
    }

    fn handle_dest_changed(&self, dest_dir: Option<PathBuf>) {
        trace_out!("handle dest dir {dest_dir:?}");
        let widgets = self.widgets.get().unwrap();
        if let Some(dest_dir) = &dest_dir {
            let copy = self.state.borrow().copy;
            let norm_dir =
                normalize_for_display(dest_dir, self.client.base_directory().as_ref(), true);
            if copy {
                widgets.destination_help.set_label(&i18n_fmt! {
                    // Translation note: {} is the directory path.
                    i18n_fmt("Will images copy to \"{}\".", norm_dir.unwrap_or_else(|_| dest_dir.to_string_lossy().to_string()))
                });
                self.state.borrow_mut().copy_dest_dir = dest_dir.clone();
                self.cfg
                    .set_value("base_import_dest_dir", &dest_dir.to_string_lossy());
            } else {
                widgets.destination_help.set_label(&i18n_fmt! {
                    // Translation note: {} is the directory path.
                    i18n_fmt("Will import \"{}\" by reference.", norm_dir.unwrap_or_else(|_| dest_dir.to_string_lossy().to_string()))
                });
            }
        }
        self.state.borrow_mut().full_dest_dir = dest_dir;
    }

    /// Return the active sorting format. If sorting is disabled
    /// like with non copy import, then it returns `NoPath`.
    fn sorting_format(&self) -> DatePathFormat {
        let disabled = self.state.borrow().sorting_disabled;
        if disabled {
            DatePathFormat::NoPath
        } else {
            self.state.borrow().sorting_format
        }
    }

    /// Set the date sorting format.
    fn set_sorting_format(&self, format: DatePathFormat) {
        let mut state = self.state.borrow_mut();
        state.sorting_format = format;
        if let Some(sorting) = format.to_u32() {
            self.cfg.set_value("import_sorting", &sorting.to_string());
        }
    }

    fn append_files_to_import(&self, files: &[Box<dyn ImportedFile>]) {
        let count = files.len();
        let paths: Vec<String> = files
            .iter()
            .map(|f| {
                let path = f.path();
                trace_out!("selected {}", &path);
                if let Some(widgets) = self.widgets.get() {
                    widgets
                        .images_list_model
                        .append(&ThumbItem::new(f.as_ref()));
                    self.images_list_map.borrow_mut().insert(
                        path.to_string(),
                        DestEntry {
                            idx: widgets.images_list_model.n_items() - 1,
                            dest: None,
                        },
                    );
                }
                path.to_string()
            })
            .collect();
        self.state.borrow_mut().import_count += count;
        self.update_import_count();
        if let Some(widgets) = self.widgets.get() {
            widgets.preview_spinner.start();
        }

        if let Some(importer) = self.importer()
            && let Some(source) = &self.state.borrow().source
        {
            let sender = self.sender();
            importer.get_previews_for(
                &self.thumbnail_task,
                source,
                paths,
                Box::new(move |path, thumbnail, date| {
                    if let Some(path) = path {
                        npc_fwk::send_async_any!(
                            Event::PreviewReceived(path, thumbnail, date),
                            sender
                        );
                    } else {
                        npc_fwk::send_async_any!(Event::PreviewsDone, sender);
                    }
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

        trace_out!("preview and date received {:?}", date);

        let state = self.state.borrow();
        let dest_dir = &state.full_dest_dir;
        if dest_dir.is_none() {
            err_out!(
                "Dest dir is None. This shouldn't have happened. Maybe some left over previews???"
            );
            return;
        }
        let dest_dir = dest_dir.as_ref().unwrap();
        let dest = Some(Importer::dest_dir_for_date(
            dest_dir,
            date.as_ref(),
            self.sorting_format(),
        ));
        if let Some(entry) = self.images_list_map.borrow_mut().get_mut(path) {
            self.widgets.get().inspect(|widgets| {
                widgets
                    .images_list_model
                    .item(entry.idx)
                    .and_downcast::<ThumbItem>()
                    .inspect(|item| {
                        item.set_pixbuf(thumbnail.map(|ref t| {
                            let texture: gdk4::Texture = t.into();
                            texture.upcast::<gdk4::Paintable>()
                        }));
                        item.set_date(date);
                    });
                if self.state.borrow().copy {
                    if let Some(dest) = &dest {
                        widgets
                            .dest_folders
                            .send(DestFoldersIn::DestDirFile(dest.clone()));
                    }
                } else if let Some(parent) = PathBuf::from(path).parent() {
                    widgets
                        .dest_folders
                        .send(DestFoldersIn::DestDirFile(parent.to_path_buf()));
                }
            });
            entry.dest = dest;
        }
    }

    fn source(&self) -> Option<String> {
        self.state.borrow().source.clone()
    }

    fn dest_dir(&self) -> Option<PathBuf> {
        self.state.borrow().full_dest_dir.clone()
    }
}
