/*
 * niepce - engine/mod.rs
 *
 * Copyright (C) 2017-2022 Hubert Figuière
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

pub mod db;
pub mod importer;
pub mod library;
pub mod libraryclient;

use db::NiepceProperties;
pub use library::thumbnail_cache::ThumbnailCache;

// must be a tuple for cxx
#[derive(Default)]
pub struct PropertySet(npc_fwk::PropertySet<db::NiepceProperties>);

impl PropertySet {
    fn add(&mut self, v: u32) {
        self.0.insert(NiepceProperties::from(v));
    }
}

impl std::ops::Deref for PropertySet {
    type Target = npc_fwk::PropertySet<db::NiepceProperties>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use npc_fwk::PropertyValue;

// must be a tuple for cxx
#[derive(Clone, Default)]
pub struct PropertyBag(npc_fwk::PropertyBag<db::NiepceProperties>);

impl PropertyBag {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn key_by_index(&self, idx: usize) -> u32 {
        self.0.bag[idx].into()
    }

    fn contains_key(&self, key: &u32) -> bool {
        let key = db::NiepceProperties::from(*key);
        self.0.contains_key(&key)
    }

    fn value_unchecked(&self, key: u32) -> &PropertyValue {
        self.0
            .map
            .get(&db::NiepceProperties::from(key))
            .expect("no such value")
    }
}

impl std::ops::Deref for PropertyBag {
    type Target = npc_fwk::PropertyBag<db::NiepceProperties>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PropertyBag {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn property_set_new() -> Box<PropertySet> {
    Box::new(PropertySet::default())
}

pub type NiepcePropertySet = PropertySet;
pub type NiepcePropertyBag = PropertyBag;

use crate::db::{Keyword, Label, LibFile, LibFolder, LibMetadata};
use crate::library::notification::{LcChannel, LibNotification};
use crate::libraryclient::{
    LibraryClientHost, LibraryClientWrapper,
    UIDataProvider,
};

#[cxx::bridge(namespace = "eng")]
pub mod ffi {
    #[namespace = "fwk"]
    extern "C++" {
        include!("fwk/cxx_prelude.hpp");
        include!("fwk/cxx_colour_bindings.hpp");

        type Moniker = npc_fwk::base::Moniker;
        type RgbColour = npc_fwk::base::rgbcolour::RgbColour;
        type PropertyValue = npc_fwk::PropertyValue;
        type WrappedPropertyBag = npc_fwk::toolkit::widgets::WrappedPropertyBag;
    }

    extern "Rust" {
        type LcChannel;
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[repr(u32)]
    pub enum NiepcePropertyIdx {
        NpFileNameProp,
        NpFileTypeProp,
        NpFileSizeProp,
        NpFolderProp,
        NpSidecarsProp,
        NpXmpRatingProp,
        NpXmpLabelProp,
        NpTiffOrientationProp,
        NpTiffMakeProp,
        NpTiffModelProp,
        NpExifAuxLensProp,
        NpExifExposureProgramProp,
        NpExifExposureTimeProp,
        NpExifFNumberPropProp,
        NpExifIsoSpeedRatingsProp,
        NpExifExposureBiasProp,
        NpExifFlashFiredProp,
        NpExifAuxFlashCompensationProp,
        NpExifWbProp,
        NpExifDateTimeOriginalProp,
        NpExifFocalLengthProp,
        NpExifGpsLongProp,
        NpExifGpsLatProp,
        NpIptcHeadlineProp,
        NpIptcDescriptionProp,
        NpIptcKeywordsProp,
        NpNiepceFlagProp,
        NpNiepceXmpPacket,
    }

    #[repr(i32)]
    #[derive(PartialEq, Clone, Copy, Eq)]
    pub enum Managed {
        NO = 0,
        YES = 1,
    }

    #[repr(i32)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum FileType {
        /// Don't know
        Unknown = 0,
        /// Camera Raw
        Raw = 1,
        /// Bundle of RAW + processed. Don't assume JPEG.
        RawJpeg = 2,
        /// Processed Image
        Image = 3,
        /// Video
        Video = 4,
    }

    extern "Rust" {
        type Keyword;

        fn id(&self) -> i64;
        fn keyword(&self) -> &str;
    }

    impl Box<Keyword> {}

    extern "Rust" {
        type Label;

        fn colour(&self) -> &RgbColour;
        fn label(&self) -> &str;
        fn id(&self) -> i64;
        fn clone_boxed(&self) -> Box<Label>;
    }

    extern "Rust" {
        type LibFile;

        #[cxx_name = "path"]
        fn path_str(&self) -> String;
        fn id(&self) -> i64;
        fn folder_id(&self) -> i64;
        fn orientation(&self) -> i32;
        // The type is `FileType`.
        fn file_type(&self) -> FileType;
        #[cxx_name = "property"]
        fn property_int(&self, idx: u32) -> i32;
        #[cxx_name = "set_property"]
        fn set_property_int(&mut self, idx: u32, v: i32);
    }

    impl Box<LibFile> {}

    #[repr(i32)]
    #[derive(Clone, Debug)]
    pub enum FolderVirtualType {
        NONE = 0,
        TRASH = 1,
    }

    extern "Rust" {
        type LibFolder;

        fn id(&self) -> i64;
        fn name(&self) -> &str;
        fn expanded(&self) -> bool;
        fn virtual_type(&self) -> FolderVirtualType;
    }

    extern "Rust" {
        type LibMetadata;

        fn id(&self) -> i64;
        fn to_properties(&self, propset: &PropertySet) -> Box<PropertyBag>;
        fn to_wrapped_properties(&self, propset: &PropertySet) -> *mut WrappedPropertyBag;
    }

    #[namespace = "fwk"]
    extern "Rust" {
        type PropertyBag;

        fn is_empty(&self) -> bool;
        fn len(&self) -> usize;
        fn contains_key(&self, key: &u32) -> bool;
        #[cxx_name = "value"]
        fn value_unchecked(&self, key: u32) -> &PropertyValue;
        fn key_by_index(&self, idx: usize) -> u32;
    }

    #[namespace = "fwk"]
    extern "Rust" {
        type PropertySet;

        #[cxx_name = "PropertySet_new"]
        fn property_set_new() -> Box<PropertySet>;
        fn add(&mut self, v: u32);
    }

    extern "Rust" {
        type ThumbnailCache;
    }

    #[repr(i32)]
    #[allow(non_camel_case_types)]
    pub enum NotificationType {
        NONE,
        NEW_LIBRARY_CREATED,
        DatabaseNeedUpgrade,
        DatabaseReady,
        ADDED_FOLDER,
        ADDED_FILE,
        ADDED_FILES,
        ADDED_KEYWORD,
        ADDED_LABEL,
        ADDED_ALBUM,
        ADDED_TO_ALBUM,
        ALBUM_CONTENT_QUERIED,
        ALBUM_COUNTED,
        ALBUM_COUNT_CHANGE,
        AlbumDeleted,
        FOLDER_CONTENT_QUERIED,
        FOLDER_DELETED,
        FOLDER_COUNTED,
        FOLDER_COUNT_CHANGE,
        KEYWORD_CONTENT_QUERIED,
        KEYWORD_COUNTED,
        KEYWORD_COUNT_CHANGE,
        METADATA_QUERIED,
        METADATA_CHANGED,
        LABEL_CHANGED,
        LABEL_DELETED,
        XMP_NEEDS_UPDATE,
        FILE_MOVED,
        FILE_STATUS_CHANGED,
        ThumbnailLoaded,
    }

    extern "Rust" {
        type LibNotification;

        fn type_(&self) -> NotificationType;
        fn id(&self) -> i64;
        fn get_libmetadata(&self) -> &LibMetadata;
    }

    extern "Rust" {
        type UIDataProvider;

        #[cxx_name = "addLabel"]
        fn add_label(&self, label: &Label);
        #[cxx_name = "updateLabel"]
        fn update_label(&self, label: &Label);
        #[cxx_name = "deleteLabel"]
        fn delete_label(&self, label: i64);
        fn label_count(&self) -> usize;
        fn label_at(&self, idx: usize) -> *mut Label;
        #[cxx_name = "colourForLabel"]
        fn colour_for_label(&self, idx: i64) -> RgbColour;
    }

    extern "Rust" {
        type LibraryClientWrapper;

        fn request_metadata(&self, id: i64);
        fn delete_label(&self, id: i64);
        fn update_label(&self, id: i64, new_name: String, new_colour: String);
        fn create_label_sync(&self, name: String, colour: String) -> i64;
    }

    extern "Rust" {
        type LibraryClientHost;

        #[cxx_name = "getDataProvider"]
        fn ui_provider(&self) -> &UIDataProvider;
        fn client(&self) -> &LibraryClientWrapper;
        #[cxx_name = "thumbnailCache"]
        fn thumbnail_cache(&self) -> &ThumbnailCache;
    }
}
