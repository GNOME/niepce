/*
 * niepce - niepce/ui/dialogs/import/mod.rs
 *
 * Copyright (C) 2008-2024 Hubert Figuière
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
use importer_ui::{ImporterUI, SourceSelectedCallback};

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use glib::translate::*;
use gtk4::prelude::*;
use gtk_macros::get_widget;
use num_traits::FromPrimitive;
use once_cell::sync::OnceCell;

use crate::ffi;
use npc_engine::importer::{DatePathFormat, ImportBackend, ImportRequest, ImportedFile};
use npc_fwk::toolkit::{self, Thumbnail};
use npc_fwk::{dbg_out, Date};
use thumb_item::ThumbItem;
use thumb_item_row::ThumbItemRow;

enum Event {
    /// Set Source `source` and `dest_dir`
    SetSource(String, String),
    /// The source changed. `id` in the combo box.
    SourceChanged(String),
    /// The `DatePathFormat` has been changed.
    SetDatePathFormat(DatePathFormat),
    PreviewReceived(String, Option<Thumbnail>, Option<Date>),
    AppendFiles(Vec<Box<dyn ImportedFile>>),
}

struct Widgets {
    dialog: adw::Window,
    import_source_combo: gtk4::ComboBoxText,
    importer_ui_stack: gtk4::Stack,
    destination_folder: gtk4::Entry,
    images_list_model: gio::ListStore,

    importers: HashMap<String, Rc<dyn ImporterUI>>,
    current_importer: RefCell<Option<Rc<dyn ImporterUI>>>,
}

impl Widgets {
    fn add_importer_ui(
        &mut self,
        importer: Rc<dyn ImporterUI>,
        tx: npc_fwk::toolkit::Sender<Event>,
    ) {
        self.import_source_combo
            .append(Some(&importer.id()), importer.name());

        dbg_out!("setting up importer widget for {}", &importer.id());
        let importer_widget = importer.setup_widget(self.dialog.upcast_ref::<gtk4::Window>());
        self.importer_ui_stack
            .add_named(&importer_widget, Some(&importer.id()));
        importer.set_source_selected_callback(Box::new(move |source, dest_dir| {
            let source = source.to_string();
            let dest_dir = dest_dir.to_string();
            npc_fwk::send_async_local!(Event::SetSource(source, dest_dir), tx);
        }));

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
    tx: npc_fwk::toolkit::Sender<Event>,
    base_dest_dir: PathBuf,
    cfg: Rc<toolkit::Configuration>,

    widgets: OnceCell<Widgets>,
    state: RefCell<State>,
}

impl ImportDialog {
    pub fn new(cfg: Rc<toolkit::Configuration>) -> Rc<Self> {
        let (tx, rx) = npc_fwk::toolkit::channel();

        let base_dest_dir = cfg
            .value_opt("base_import_dest_dir")
            .map(PathBuf::from)
            .or_else(|| glib::user_special_dir(glib::UserDirectory::Pictures))
            .unwrap_or_else(glib::home_dir);
        let dialog = Rc::new(ImportDialog {
            tx,
            base_dest_dir,
            cfg,
            widgets: OnceCell::new(),
            state: RefCell::new(State::default()),
        });

        npc_fwk::toolkit::channels::receiver_attach(
            rx,
            glib::clone!(@weak dialog => move |e| {
                dialog.dispatch(e);
            }),
        );

        dialog
    }

    fn dispatch(&self, e: Event) {
        match e {
            Event::SetSource(source, destdir) => self.set_source(&source, &destdir),
            Event::SourceChanged(source) => self.import_source_changed(&source),
            Event::SetDatePathFormat(f) => self.set_sorting_format(f),
            Event::PreviewReceived(path, thumbnail, date) => {
                self.preview_received(&path, thumbnail, date)
            }
            Event::AppendFiles(files) => self.append_files_to_import(&files),
        }
    }

    fn setup_widget<F>(&self, callback: F) -> &adw::Window
    where
        F: Fn(&adw::Window) + 'static,
    {
        &self
            .widgets
            .get_or_init(|| {
                let builder =
                    gtk4::Builder::from_resource("/net/figuiere/Niepce/ui/importdialog.ui");
                get_widget!(builder, adw::Window, import_dialog);
                // get_widget!(builder, gtk4::ComboBox, date_tz_combo);
                get_widget!(builder, gtk4::Button, cancel_button);
                cancel_button.connect_clicked(
                    glib::clone!(@weak import_dialog => move |_| import_dialog.close()),
                );
                get_widget!(builder, gtk4::Button, import_button);
                import_button.connect_clicked(
                    glib::clone!(@weak import_dialog => move |_| callback(&import_dialog)),
                );
                get_widget!(builder, gtk4::Entry, destination_folder);
                get_widget!(builder, gtk4::Stack, importer_ui_stack);
                get_widget!(builder, gtk4::ComboBoxText, import_source_combo);
                get_widget!(builder, gtk4::DropDown, date_sorting_combo);
                let string_list = gtk4::StringList::new(&[
                    "No Sorting",
                    "YYYYMMDD",
                    "YYYY/MMDD",
                    "YYYY/MM/DD",
                    "YYYY/YYYYMMDD",
                ]);
                date_sorting_combo.set_model(Some(&string_list));

                get_widget!(builder, gtk4::ScrolledWindow, attributes_scrolled);
                let mut metadata_pane = ffi::metadata_pane_controller_new();
                let w = unsafe {
                    gtk4::Widget::from_glib_none(
                        metadata_pane.pin_mut().build_widget() as *mut gtk4::ffi::GtkWidget
                    )
                };
                // add
                attributes_scrolled.set_child(Some(&w));

                get_widget!(builder, gtk4::ScrolledWindow, images_list_scrolled);
                let images_list_model = gio::ListStore::new::<ThumbItem>();
                let selection_model = gtk4::SingleSelection::new(Some(images_list_model.clone()));
                let image_gridview = crate::ImageGridView::new(selection_model, None, None);
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
                            thumb_item
                                .bind_property("name", &row, "filename")
                                .sync_create()
                                .build();
                            thumb_item
                                .bind_property("pixbuf", &row, "image")
                                .sync_create()
                                .build();
                        }
                    }
                });

                images_list_scrolled.set_child(Some(&*image_gridview));
                images_list_scrolled
                    .set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);

                let mut widgets = Widgets {
                    dialog: import_dialog,
                    import_source_combo: import_source_combo.clone(),
                    importer_ui_stack,
                    destination_folder,
                    images_list_model,
                    importers: HashMap::new(),
                    current_importer: RefCell::new(None),
                };

                let importer = DirectoryImporterUI::new(self.cfg.clone());
                widgets.add_importer_ui(importer, self.tx.clone());
                let importer = CameraImporterUI::new();
                widgets.add_importer_ui(importer, self.tx.clone());

                import_source_combo.connect_changed(
                    glib::clone!(@strong self.tx as tx => move |combo| {
                        if let Some(source) = combo.active_id() {
                            npc_fwk::send_async_local!(Event::SourceChanged(source.to_string()), tx);
                        }
                    }),
                );
                date_sorting_combo.connect_selected_item_notify(
                    glib::clone!(@strong self.tx as tx => move |dropdown| {
                        dbg_out!("selected format {}", dropdown.selected());
                        if let Some(format) = DatePathFormat::from_u32(dropdown.selected()) {
                            dbg_out!("setting format {format:?}");
                            npc_fwk::send_async_local!(Event::SetDatePathFormat(format), tx);
                        }
                    }),
                );

                let last_importer = self.cfg.value("last_importer", "DirectoryImporter");
                import_source_combo.set_active_id(Some(&last_importer));

                widgets
            })
            .dialog
    }

    pub fn run_modal<F>(&self, parent: Option<&gtk4::Window>, callback: F)
    where
        F: Fn(&adw::Window) + 'static,
    {
        let dialog = self.setup_widget(callback);
        dialog.set_transient_for(parent);
        dialog.set_modal(true);
        dialog.present();
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
            let tx = self.tx.clone();
            importer.list_source_content(
                source,
                Box::new(move |files| {
                    npc_fwk::send_async_any!(Event::AppendFiles(files), tx);
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
            let tx = self.tx.clone();
            importer.get_previews_for(
                &self.state.borrow().source,
                paths,
                Box::new(move |path, thumbnail, date| {
                    npc_fwk::send_async_any!(Event::PreviewReceived(path, thumbnail, date), tx);
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
                    .map(|item| {
                        item.set_pixbuf(thumbnail.and_then(|t| t.make_pixbuf()));
                        item.set_date(date);

                        item
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
