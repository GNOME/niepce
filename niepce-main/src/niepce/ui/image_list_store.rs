/*
 * niepce - niepce/ui/image_list_store.rs
 *
 * Copyright (C) 2020-2022 Hubert Figuière
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

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::ffi::c_char;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;

use glib::translate::*;
use gtk4::prelude::*;
use once_cell::unsync::OnceCell;

use crate::libraryclient::{ClientInterface, LibraryClient};
use npc_engine::db::libfile::{FileStatus, LibFile};
use npc_engine::db::props::NiepceProperties as Np;
use npc_engine::db::props::NiepcePropertyIdx::*;
use npc_engine::db::LibraryId;
use npc_engine::library::notification::{LibNotification, MetadataChange};
use npc_engine::library::thumbnail_cache::ThumbnailCache;
use npc_fwk::toolkit::gdk_utils;
use npc_fwk::PropertyValue;
use npc_fwk::{dbg_out, err_out};

/// Wrap a libfile into something that can be in a glib::Value
#[derive(Clone, glib::Boxed)]
#[boxed_type(name = "StoreLibFile", nullable)]
pub struct StoreLibFile(pub LibFile);

#[repr(i32)]
pub enum ColIndex {
    Thumb = 0,
    File = 1,
    StripThumb = 2,
    FileStatus = 3,
}

/// Binding raw because it's a Rc.
#[derive(Default)]
pub struct ImageListStoreWrap(pub Rc<ImageListStore>);

impl std::ops::Deref for ImageListStoreWrap {
    type Target = ImageListStore;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl ImageListStoreWrap {
    /// # Safety
    /// Deref a pointer
    pub unsafe fn unwrap_ref(&self) -> &ImageListStore {
        &*Rc::as_ptr(&self.0)
    }

    // cxx
    /// Clone to a new `Box`.
    pub fn clone_(&self) -> Box<Self> {
        Box::new(Self(self.0.clone()))
    }
}

/// The Image list store.
/// It wraps the tree model/store.
pub struct ImageListStore {
    store: gtk4::ListStore,
    current_folder: Cell<LibraryId>,
    current_keyword: Cell<LibraryId>,
    idmap: RefCell<BTreeMap<LibraryId, gtk4::TreeIter>>,
    image_loading_icon: OnceCell<gtk4::IconPaintable>,
}

impl Default for ImageListStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageListStore {
    pub fn new() -> Self {
        let col_types: [glib::Type; 4] = [
            gdk4::Paintable::static_type(),
            StoreLibFile::static_type(),
            gdk4::Paintable::static_type(),
            glib::Type::I32,
        ];

        let store = gtk4::ListStore::new(&col_types);

        Self {
            store,
            current_folder: Cell::new(0),
            current_keyword: Cell::new(0),
            idmap: RefCell::new(BTreeMap::new()),
            image_loading_icon: OnceCell::new(),
        }
    }

    /// Return the `GtkListStore`
    pub fn liststore(&self) -> &gtk4::ListStore {
        &self.store
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
        (idx == Np::Index(NpXmpRatingProp))
            || (idx == Np::Index(NpXmpLabelProp))
            || (idx == Np::Index(NpTiffOrientationProp))
            || (idx == Np::Index(NpNiepceFlagProp))
    }

    // cxx
    pub fn get_iter_from_id_(&self, id: i64) -> *const c_char {
        self.idmap
            .borrow()
            .get(&id)
            .map(|iter| {
                let c_iter: *const gtk4_sys::GtkTreeIter = iter.to_glib_none().0;
                c_iter as *const c_char
            })
            .unwrap_or(ptr::null())
    }

    pub fn get_iter_from_id(&self, id: LibraryId) -> Option<gtk4::TreeIter> {
        self.idmap.borrow().get(&id).cloned()
    }

    /// Clear the content of the store.
    pub fn clear_content(&self) {
        // clear the map before the list.
        self.idmap.borrow_mut().clear();
        self.store.clear();
    }

    fn add_libfile(&self, f: &LibFile) {
        let icon = self.get_loading_icon().clone();
        let thumb_icon = icon.clone();
        let iter = self.add_row(
            Some(icon.upcast()),
            f,
            Some(thumb_icon.upcast()),
            FileStatus::Ok,
        );
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
        use self::LibNotification::*;

        match *notification {
            XmpNeedsUpdate => {
                let app = npc_fwk::ffi::Application_app();
                let cfg = &app.config().cfg;
                let write_xmp = cfg
                    .value("write_xmp_automatically", "0")
                    .parse::<bool>()
                    .unwrap_or(false);
                client.process_xmp_update_queue(write_xmp);
                true
            }
            FolderContentQueried(ref c) | KeywordContentQueried(ref c) => {
                match *notification {
                    FolderContentQueried(_) => {
                        self.current_folder.set(c.id);
                        self.current_keyword.set(0);
                    }
                    KeywordContentQueried(_) => {
                        self.current_folder.set(0);
                        self.current_keyword.set(c.id);
                    }
                    _ => {}
                }
                self.clear_content();
                dbg_out!("received folder content file # {}", c.content.len());
                self.add_libfiles(&c.content);
                // request thumbnails c.content
                thumbnail_cache.request(&c.content);
                true
            }
            FileMoved(ref param) => {
                dbg_out!("File moved. Current folder {}", self.current_folder.get());
                if self.current_folder.get() != 0 {
                    if param.from == self.current_folder.get() {
                        // remove from list
                        dbg_out!("from this folder");
                        if let Some(iter) = self.get_iter_from_id(param.file) {
                            self.store.remove(&iter);
                            self.idmap.borrow_mut().remove(&param.file);
                        }
                    } else if param.to == self.current_folder.get() {
                        // XXX add to list. but this isn't likely to happen atm.
                    }
                }
                true
            }
            FileStatusChanged(ref status) => {
                if let Some(iter) = self.idmap.borrow().get(&status.id) {
                    self.store.set_value(
                        iter,
                        ColIndex::FileStatus as u32,
                        &(status.status as i32).to_value(),
                    );
                }
                true
            }
            MetadataChanged(ref m) => {
                dbg_out!("metadata changed {:?}", m.meta);
                // only interested in a few props
                if Self::is_property_interesting(m.meta) {
                    if let Some(iter) = self.idmap.borrow().get(&m.id) {
                        self.set_property(iter, m);
                    }
                }
                true
            }
            ThumbnailLoaded(ref t) => {
                if let Some(pixbuf) = t.pix.make_pixbuf() {
                    self.set_thumbnail(t.id, &pixbuf);
                }
                true
            }
            _ => false,
        }
    }

    pub fn get_file_id_at_path(&self, path: &gtk4::TreePath) -> LibraryId {
        if let Some(iter) = self.store.iter(path) {
            if let Ok(libfile) = self
                .store
                .get_value(&iter, ColIndex::File as i32)
                .get::<&StoreLibFile>()
            {
                return libfile.0.id();
            }
        }
        0
    }

    // cxx
    pub fn get_file_(&self, id: LibraryId) -> *mut LibFile {
        if let Some(file) = self.file(id) {
            Box::into_raw(Box::new(file))
        } else {
            ptr::null_mut()
        }
    }

    pub fn file(&self, id: LibraryId) -> Option<LibFile> {
        if let Some(iter) = self.idmap.borrow().get(&id) {
            self.store
                .get_value(iter, ColIndex::File as i32)
                .get::<&StoreLibFile>()
                .map(|v| v.0.clone())
                .ok()
        } else {
            None
        }
    }

    pub fn add_row(
        &self,
        thumb: Option<gdk4::Paintable>,
        file: &LibFile,
        strip_thumb: Option<gdk4::Paintable>,
        status: FileStatus,
    ) -> gtk4::TreeIter {
        let iter = self.store.append();
        let store_libfile = StoreLibFile(file.clone());
        self.store.set(
            &iter,
            &[
                (ColIndex::Thumb as u32, &thumb),
                (ColIndex::File as u32, &store_libfile),
                (ColIndex::StripThumb as u32, &strip_thumb),
                (ColIndex::FileStatus as u32, &(status as i32)),
            ],
        );
        iter
    }

    pub fn set_thumbnail(&self, id: LibraryId, thumb: &gdk_pixbuf::Pixbuf) {
        if let Some(iter) = self.idmap.borrow().get(&id) {
            let strip_thumb = gdk_utils::gdkpixbuf_scale_to_fit(Some(thumb), 100)
                .map(|pix| gdk4::Texture::for_pixbuf(&pix));
            let thumb = gdk4::Texture::for_pixbuf(thumb);
            assert!(thumb.ref_count() > 0);
            self.store.set(
                iter,
                &[
                    (ColIndex::Thumb as u32, &thumb),
                    (ColIndex::StripThumb as u32, &strip_thumb),
                ],
            );
        }
    }

    pub fn set_property(&self, iter: &gtk4::TreeIter, change: &MetadataChange) {
        if let Ok(libfile) = self
            .store
            .get_value(iter, ColIndex::File as i32)
            .get::<&StoreLibFile>()
        {
            assert!(libfile.0.id() == change.id);
            let meta = change.meta;
            if let PropertyValue::Int(value) = change.value {
                let mut file = libfile.0.clone();
                file.set_property(meta, value);
                self.store
                    .set_value(iter, ColIndex::File as u32, &StoreLibFile(file).to_value());
            } else {
                err_out!("Wrong property type");
            }
        }
    }

    // cxx
    /// Return the gobj for the GtkListStore. You must ref it to hold it.
    pub fn gobj(&self) -> *mut c_char {
        let w: *mut gtk4_sys::GtkListStore = self.store.to_glib_none().0;
        w as *mut c_char
    }

    // cxx
    /// Return the ID of the file at the given GtkTreePath
    ///
    /// # Safety
    /// Use glib pointers.
    pub unsafe fn get_file_id_at_path_(&self, path: *const c_char) -> i64 {
        assert!(!path.is_null());
        let path = path as *const gtk4_sys::GtkTreePath;
        self.get_file_id_at_path(&from_glib_borrow(path))
    }
}
