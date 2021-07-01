/*
 * niepce - niepce/ui/image_list_store.rs
 *
 * Copyright (C) 2020-2021 Hubert Figuière
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

use gdk_pixbuf;
use gdk_pixbuf_sys;
use glib::translate::*;
use gtk;
use gtk::prelude::*;
use gtk_sys;

use once_cell::unsync::OnceCell;

use npc_engine::db::libfile::{FileStatus, LibFile};
use npc_engine::db::props::NiepceProperties as Np;
use npc_engine::db::props::NiepcePropertyIdx::*;
use npc_engine::db::LibraryId;
use npc_engine::library::notification::{LibNotification, MetadataChange};
use npc_engine::library::thumbnail_cache::ThumbnailCache;
use npc_fwk::toolkit::gdk_utils;
use npc_fwk::PropertyValue;

/// Wrap a libfile into something that can be in a glib::Value
#[derive(Clone, GBoxed)]
#[gboxed(type_name = "StoreLibFile", nullable)]
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
    store: gtk::ListStore,
    current_folder: LibraryId,
    current_keyword: LibraryId,
    idmap: BTreeMap<LibraryId, gtk::TreeIter>,
    image_loading_icon: OnceCell<Option<gdk_pixbuf::Pixbuf>>,
}

impl ImageListStore {
    pub fn new() -> Self {
        let col_types: [glib::Type; 4] = [
            gdk_pixbuf::Pixbuf::static_type(),
            StoreLibFile::static_type(),
            gdk_pixbuf::Pixbuf::static_type(),
            glib::Type::I32,
        ];

        let store = gtk::ListStore::new(&col_types);

        Self {
            store,
            current_folder: 0,
            current_keyword: 0,
            idmap: BTreeMap::new(),
            image_loading_icon: OnceCell::new(),
        }
    }

    fn get_loading_icon(&self) -> Option<&gdk_pixbuf::Pixbuf> {
        self.image_loading_icon
            .get_or_init(|| {
                if let Some(theme) = gtk::IconTheme::default() {
                    if let Ok(icon) =
                        theme.load_icon("image-loading", 32, gtk::IconLookupFlags::USE_BUILTIN)
                    {
                        icon
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .as_ref()
    }

    fn is_property_interesting(idx: Np) -> bool {
        return (idx == Np::Index(NpXmpRatingProp))
            || (idx == Np::Index(NpXmpLabelProp))
            || (idx == Np::Index(NpTiffOrientationProp))
            || (idx == Np::Index(NpNiepceFlagProp));
    }

    fn get_iter_from_id(&self, id: LibraryId) -> Option<&gtk::TreeIter> {
        self.idmap.get(&id)
    }

    fn clear_content(&mut self) {
        // clear the map before the list.
        self.idmap.clear();
        self.store.clear();
    }

    fn add_libfile(&mut self, f: &LibFile) {
        let icon = self.get_loading_icon().map(|v| v.clone());
        let iter = self.add_row(
            icon.clone().as_ref(),
            f,
            gdk_utils::gdkpixbuf_scale_to_fit(icon.as_ref(), 100).as_ref(),
            FileStatus::Ok,
        );
        self.idmap.insert(f.id(), iter);
    }

    fn add_libfiles(&mut self, content: &Vec<LibFile>) {
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
                if let Some(iter) = self.get_iter_from_id(t.id) {
                    let pixbuf = t.pix.make_pixbuf();
                    self.store.set(
                        iter,
                        &[
                            (ColIndex::Thumb as u32, &pixbuf),
                            (
                                ColIndex::StripThumb as u32,
                                &gdk_utils::gdkpixbuf_scale_to_fit(pixbuf.as_ref(), 100),
                            ),
                        ],
                    );
                }
                true
            }
            _ => false,
        }
    }

    pub fn get_file_id_at_path(&self, path: &gtk::TreePath) -> LibraryId {
        if let Some(iter) = self.store.iter(&path) {
            if let Ok(libfile) = self
                .store
                .value(&iter, ColIndex::File as i32)
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
                .value(&iter, ColIndex::File as i32)
                .get::<&StoreLibFile>()
                .map(|v| v.0.clone())
                .ok()
        } else {
            None
        }
    }

    pub fn add_row(
        &mut self,
        thumb: Option<&gdk_pixbuf::Pixbuf>,
        file: &LibFile,
        strip_thumb: Option<&gdk_pixbuf::Pixbuf>,
        status: FileStatus,
    ) -> gtk::TreeIter {
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
            let strip_thumb = gdk_utils::gdkpixbuf_scale_to_fit(Some(thumb), 100);
            assert!(thumb.ref_count() > 0);
            self.store.set(
                iter,
                &[
                    (ColIndex::Thumb as u32, thumb),
                    (ColIndex::StripThumb as u32, &strip_thumb),
                ],
            );
        }
    }

    pub fn set_property(&self, iter: &gtk::TreeIter, change: &MetadataChange) {
        if let Ok(libfile) = self
            .store
            .value(&iter, ColIndex::File as i32)
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
pub unsafe extern "C" fn npc_image_list_store_new() -> *mut ImageListStore {
    let box_ = Box::new(ImageListStore::new());
    Box::into_raw(box_)
}

#[no_mangle]
pub unsafe extern "C" fn npc_image_list_store_delete(self_: *mut ImageListStore) {
    assert!(!self_.is_null());
    Box::from_raw(self_);
}

/// Return the gobj for the GtkListStore. You must ref it to hold it.
#[no_mangle]
pub unsafe extern "C" fn npc_image_list_store_gobj(
    self_: &ImageListStore,
) -> *mut gtk_sys::GtkListStore {
    self_.store.to_glib_none().0
}

/// Return the ID of the file at the given GtkTreePath
#[no_mangle]
pub unsafe extern "C" fn npc_image_list_store_get_file_id_at_path(
    self_: &ImageListStore,
    path: *const gtk_sys::GtkTreePath,
) -> LibraryId {
    assert!(!path.is_null());
    self_.get_file_id_at_path(&from_glib_borrow(path))
}

#[no_mangle]
pub unsafe extern "C" fn npc_image_list_store_add_row(
    self_: &mut ImageListStore,
    thumb: *mut gdk_pixbuf_sys::GdkPixbuf,
    file: *const LibFile,
    strip_thumb: *mut gdk_pixbuf_sys::GdkPixbuf,
    status: FileStatus,
) -> gtk_sys::GtkTreeIter {
    let thumb: Option<gdk_pixbuf::Pixbuf> = from_glib_none(thumb);
    let strip_thumb: Option<gdk_pixbuf::Pixbuf> = from_glib_none(strip_thumb);
    *self_
        .add_row(thumb.as_ref(), &*file, strip_thumb.as_ref(), status)
        .to_glib_none()
        .0
}

#[no_mangle]
pub unsafe extern "C" fn npc_image_list_store_get_iter_from_id(
    self_: &mut ImageListStore,
    id: LibraryId,
) -> *const gtk_sys::GtkTreeIter {
    self_.idmap.get(&id).to_glib_none().0
}

#[no_mangle]
pub unsafe extern "C" fn npc_image_list_store_get_file(
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
pub unsafe extern "C" fn npc_image_list_store_on_lib_notification(
    self_: &mut ImageListStore,
    notification: &LibNotification,
    thumbnail_cache: &ThumbnailCache,
) -> bool {
    self_.on_lib_notification(notification, thumbnail_cache)
}

#[no_mangle]
pub unsafe extern "C" fn npc_image_list_store_clear_content(self_: &mut ImageListStore) {
    self_.clear_content()
}
