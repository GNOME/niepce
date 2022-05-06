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

use std::collections::BTreeMap;
use std::ptr;

use glib::translate::*;
use gtk4::prelude::*;

use once_cell::unsync::OnceCell;

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

/// The Image list store.
/// It wraps the tree model/store.
pub struct ImageListStore {
    store: gtk4::ListStore,
    current_folder: LibraryId,
    current_keyword: LibraryId,
    idmap: BTreeMap<LibraryId, gtk4::TreeIter>,
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
            current_folder: 0,
            current_keyword: 0,
            idmap: BTreeMap::new(),
            image_loading_icon: OnceCell::new(),
        }
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

    fn get_iter_from_id(&self, id: LibraryId) -> Option<&gtk4::TreeIter> {
        self.idmap.get(&id)
    }

    fn clear_content(&mut self) {
        // clear the map before the list.
        self.idmap.clear();
        self.store.clear();
    }

    fn add_libfile(&mut self, f: &LibFile) {
        let icon = self.get_loading_icon().clone();
        let thumb_icon = icon.clone();
        let iter = self.add_row(
            Some(icon.upcast()),
            f,
            Some(thumb_icon.upcast()),
            FileStatus::Ok,
        );
        self.idmap.insert(f.id(), iter);
    }

    fn add_libfiles(&mut self, content: &[LibFile]) {
        for f in content.iter() {
            self.add_libfile(f);
        }
    }

    /// Process the notification.
    /// Returns false if it hasn't been
    pub fn on_lib_notification(
        &mut self,
        notification: &LibNotification,
        thumbnail_cache: &ThumbnailCache,
    ) -> bool {
        use self::LibNotification::*;

        match *notification {
            FolderContentQueried(ref c) | KeywordContentQueried(ref c) => {
                match *notification {
                    FolderContentQueried(_) => {
                        self.current_folder = c.id;
                        self.current_keyword = 0;
                    }
                    KeywordContentQueried(_) => {
                        self.current_folder = 0;
                        self.current_keyword = c.id;
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
                dbg_out!("File moved. Current folder {}", self.current_folder);
                if self.current_folder != 0 {
                    if param.from == self.current_folder {
                        // remove from list
                        dbg_out!("from this folder");
                        if let Some(iter) = self.get_iter_from_id(param.file) {
                            self.store.remove(iter);
                            self.idmap.remove(&param.file);
                        }
                    } else if param.to == self.current_folder {
                        // XXX add to list. but this isn't likely to happen atm.
                    }
                }
                true
            }
            FileStatusChanged(ref status) => {
                if let Some(iter) = self.idmap.get(&status.id) {
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
                    if let Some(iter) = self.idmap.get(&m.id) {
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
        if let Some(iter) = self.store.iter(&path) {
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

    pub fn get_file(&self, id: LibraryId) -> Option<LibFile> {
        if let Some(iter) = self.idmap.get(&id) {
            self.store
                .get_value(&iter, ColIndex::File as i32)
                .get::<&StoreLibFile>()
                .map(|v| v.0.clone())
                .ok()
        } else {
            None
        }
    }

    pub fn add_row(
        &mut self,
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

    pub fn set_thumbnail(&mut self, id: LibraryId, thumb: &gdk_pixbuf::Pixbuf) {
        if let Some(iter) = self.idmap.get(&id) {
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
            .get_value(&iter, ColIndex::File as i32)
            .get::<&StoreLibFile>()
        {
            assert!(libfile.0.id() == change.id);
            let meta = change.meta;
            if let PropertyValue::Int(value) = change.value {
                let mut file = libfile.0.clone();
                file.set_property(meta, value);
                self.store
                    .set_value(&iter, ColIndex::File as u32, &StoreLibFile(file).to_value());
            } else {
                err_out!("Wrong property type");
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn npc_image_list_store_new() -> *mut ImageListStore {
    let box_ = Box::new(ImageListStore::new());
    Box::into_raw(box_)
}

/// # Safety
/// Dereference pointer.
#[no_mangle]
pub unsafe extern "C" fn npc_image_list_store_delete(self_: *mut ImageListStore) {
    assert!(!self_.is_null());
    Box::from_raw(self_);
}

/// Return the gobj for the GtkListStore. You must ref it to hold it.
#[no_mangle]
pub extern "C" fn npc_image_list_store_gobj(self_: &ImageListStore) -> *mut gtk4_sys::GtkListStore {
    self_.store.to_glib_none().0
}

/// Return the ID of the file at the given GtkTreePath
///
/// # Safety
/// Use glib pointers.
#[no_mangle]
pub unsafe extern "C" fn npc_image_list_store_get_file_id_at_path(
    self_: &ImageListStore,
    path: *const gtk4_sys::GtkTreePath,
) -> LibraryId {
    assert!(!path.is_null());
    self_.get_file_id_at_path(&from_glib_borrow(path))
}

/// # Safety
/// Dereference pointers.
#[no_mangle]
pub unsafe extern "C" fn npc_image_list_store_add_row(
    self_: &mut ImageListStore,
    thumb: *mut gdk_pixbuf_sys::GdkPixbuf,
    file: *const LibFile,
    strip_thumb: *mut gdk_pixbuf_sys::GdkPixbuf,
    status: FileStatus,
) -> gtk4_sys::GtkTreeIter {
    let thumb: Option<gdk_pixbuf::Pixbuf> = from_glib_none(thumb);
    let strip_thumb: Option<gdk_pixbuf::Pixbuf> = from_glib_none(strip_thumb);
    let thumb = thumb.as_ref().map(gdk4::Texture::for_pixbuf);
    let strip_thumb = strip_thumb.as_ref().map(gdk4::Texture::for_pixbuf);
    *self_
        .add_row(
            thumb.map(|t| t.upcast()),
            &*file,
            strip_thumb.map(|t| t.upcast()),
            status,
        )
        .to_glib_none()
        .0
}

#[no_mangle]
pub extern "C" fn npc_image_list_store_get_iter_from_id(
    self_: &mut ImageListStore,
    id: LibraryId,
) -> *const gtk4_sys::GtkTreeIter {
    self_.idmap.get(&id).to_glib_none().0
}

#[no_mangle]
pub extern "C" fn npc_image_list_store_get_file(
    self_: &mut ImageListStore,
    id: LibraryId,
) -> *mut LibFile {
    if let Some(libfile) = self_.get_file(id) {
        Box::into_raw(Box::new(libfile))
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn npc_image_list_store_on_lib_notification(
    self_: &mut ImageListStore,
    notification: &LibNotification,
    thumbnail_cache: &ThumbnailCache,
) -> bool {
    self_.on_lib_notification(notification, thumbnail_cache)
}

#[no_mangle]
pub extern "C" fn npc_image_list_store_clear_content(self_: &mut ImageListStore) {
    self_.clear_content()
}
