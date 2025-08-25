/*
 * niepce - niepce/ui/image_list_store.rs
 *
 * Copyright (C) 2020-2025 Hubert Figuière
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

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::BTreeMap;
use std::sync::Arc;

use gtk4::prelude::*;
use npc_fwk::{gdk4, gio, gtk4};

use super::image_grid_view::ImageListItem;
use npc_engine::catalog::LibraryId;
use npc_engine::catalog::libfile::{FileStatus, LibFile};
use npc_engine::catalog::props::NiepceProperties as Np;
use npc_engine::catalog::props::NiepcePropertyIdx as Npi;
use npc_engine::library::notification::{LibNotification, MetadataChange};
use npc_engine::library::thumbnail_cache::ThumbnailCache;
use npc_engine::libraryclient::{ClientInterface, LibraryClient};
use npc_fwk::PropertyValue;
use npc_fwk::toolkit::Configuration;
use npc_fwk::{dbg_out, err_out};

#[derive(Clone, Copy)]
enum CurrentContainer {
    None,
    Folder(LibraryId),
    #[allow(dead_code)]
    Keyword(LibraryId),
    #[allow(dead_code)]
    Album(LibraryId),
}

/// The Image list store.
/// It wraps the tree model/store.
pub struct ImageListStore {
    store: gio::ListStore,
    model: gtk4::SingleSelection,
    config: Arc<Configuration>,
    current: Cell<CurrentContainer>,
    idmap: RefCell<BTreeMap<LibraryId, u32>>,
    image_loading_icon: OnceCell<gtk4::IconPaintable>,
}

impl ImageListStore {
    pub fn new(config: Arc<Configuration>) -> Self {
        let store = gio::ListStore::new::<ImageListItem>();
        let model = gtk4::SingleSelection::new(Some(store.clone()));
        model.set_autoselect(false);
        model.set_can_unselect(true);

        Self {
            store,
            model,
            config,
            current: Cell::new(CurrentContainer::None),
            idmap: RefCell::new(BTreeMap::new()),
            image_loading_icon: OnceCell::new(),
        }
    }

    /// Return the number of items in the store.
    pub fn is_empty(&self) -> bool {
        self.store.n_items() == 0
    }

    /// Return the number of items in the store.
    pub fn len(&self) -> usize {
        self.store.n_items() as usize
    }

    /// Return the `Gtk::SelectionModel`
    pub fn selection_model(&self) -> &gtk4::SingleSelection {
        &self.model
    }

    fn get_loading_icon(&self) -> &gtk4::IconPaintable {
        self.image_loading_icon.get_or_init(|| {
            gtk4::IconTheme::default().lookup_icon(
                "image-loading",
                &[],
                32,
                1,
                gtk4::TextDirection::None,
                gtk4::IconLookupFlags::empty(),
            )
        })
    }

    fn is_property_interesting(idx: Np) -> bool {
        (idx == Np::Index(Npi::NpXmpRatingProp))
            || (idx == Np::Index(Npi::NpXmpLabelProp))
            || (idx == Np::Index(Npi::NpTiffOrientationProp))
            || (idx == Np::Index(Npi::NpNiepceFlagProp))
    }

    pub fn pos_from_id(&self, id: LibraryId) -> Option<u32> {
        self.idmap.borrow().get(&id).cloned()
    }

    /// Clear the content of the store.
    pub fn clear_content(&self) {
        // clear the map before the list.
        self.idmap.borrow_mut().clear();
        self.store.remove_all();
    }

    fn add_libfile(&self, f: &LibFile) {
        let icon = self.get_loading_icon().clone();
        let iter = self.add_row(Some(icon.upcast()), f.clone(), FileStatus::Ok);
        self.idmap.borrow_mut().insert(f.id(), iter);
    }

    fn add_libfiles(&self, content: &[LibFile]) {
        for f in content.iter() {
            self.add_libfile(f);
        }
    }

    /// Process the notification.
    /// Returns false if it hasn't been
    pub fn on_lib_notification(
        &self,
        notification: &LibNotification,
        client: &Arc<LibraryClient>,
        thumbnail_cache: &ThumbnailCache,
    ) -> bool {
        use LibNotification::*;

        match *notification {
            XmpNeedsUpdate => {
                let write_xmp = self
                    .config
                    .value("write_xmp_automatically", "0")
                    .parse::<bool>()
                    .unwrap_or(false);
                client.process_xmp_update_queue(write_xmp);
                true
            }
            FolderContentQueried(ref c)
            | KeywordContentQueried(ref c)
            | AlbumContentQueried(ref c) => {
                self.current.set(match *notification {
                    FolderContentQueried(_) => CurrentContainer::Folder(c.id),
                    KeywordContentQueried(_) => CurrentContainer::Keyword(c.id),
                    AlbumContentQueried(_) => CurrentContainer::Album(c.id),
                    _ => CurrentContainer::None,
                });
                self.clear_content();
                dbg_out!("received folder content file # {}", c.content.len());
                self.add_libfiles(&c.content);
                // request thumbnails c.content
                thumbnail_cache.request(&c.content);
                true
            }
            FileMoved(ref param) => {
                if let CurrentContainer::Folder(current_folder) = self.current.get() {
                    dbg_out!("File moved. Current folder {}", current_folder);
                    if param.from == current_folder {
                        // remove from list
                        dbg_out!("from this folder");
                        if let Some(pos) = self.pos_from_id(param.file) {
                            self.store.remove(pos);
                            self.idmap.borrow_mut().remove(&param.file);
                        }
                    } else if param.to == current_folder {
                        // XXX add to list. but this isn't likely to happen atm.
                    }
                }
                true
            }
            FileStatusChanged(ref status) => {
                if let Some(pos) = self.idmap.borrow().get(&status.id) {
                    if let Some(item) = self.store.item(*pos).and_downcast::<ImageListItem>() {
                        item.set_file_status(status.status);
                    }
                }
                true
            }
            MetadataChanged(ref m) => {
                dbg_out!("metadata changed {:?}", m.meta);
                // only interested in a few props
                if Self::is_property_interesting(m.meta) {
                    if let Some(pos) = self.idmap.borrow().get(&m.id) {
                        self.set_property(*pos, m);
                    }
                }
                true
            }
            ThumbnailLoaded(ref t) => {
                let pixbuf = gdk4::Texture::from(&t.pix);
                self.set_thumbnail(t.id, &pixbuf);
                true
            }
            _ => false,
        }
    }

    pub fn get_file_id_at_pos(&self, pos: u32) -> LibraryId {
        self.store
            .item(pos)
            .and_downcast_ref::<ImageListItem>()
            .and_then(|item| item.file())
            .map(|file| file.id())
            .unwrap_or(0)
    }

    pub fn file(&self, id: LibraryId) -> Option<LibFile> {
        self.idmap.borrow().get(&id).and_then(|pos| {
            self.store
                .item(*pos)
                .and_downcast_ref::<ImageListItem>()
                .and_then(ImageListItem::file)
        })
    }

    pub fn add_row(
        &self,
        thumbnail: Option<gdk4::Paintable>,
        file: LibFile,
        file_status: FileStatus,
    ) -> u32 {
        let item = ImageListItem::new(thumbnail, Some(file), file_status);
        self.store.append(&item);
        self.store.n_items() - 1
    }

    pub fn set_thumbnail(&self, id: LibraryId, thumb: &gdk4::Texture) {
        if let Some(pos) = self.idmap.borrow().get(&id) {
            if let Some(item) = self.store.item(*pos).and_downcast_ref::<ImageListItem>() {
                let thumb = thumb.clone();
                item.set_thumbnail(Some(thumb.upcast::<gdk4::Paintable>()));
            }
        }
    }

    pub fn set_property(&self, pos: u32, change: &MetadataChange) {
        if let Some(item) = self.store.item(pos).and_downcast_ref::<ImageListItem>() {
            if let Some(mut file) = item.file() {
                assert!(file.id() == change.id);
                let meta = change.meta;
                if let PropertyValue::Int(value) = change.value {
                    // XXX can we make this less suboptimal
                    file.set_property(meta, value);
                    item.set_file(Some(file));
                } else {
                    err_out!("Wrong property type");
                }
            } else {
                err_out!("No file found");
            }
        }
    }
}
