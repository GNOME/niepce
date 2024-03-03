/*
 * niepce - npc_engine/lib.rs
 *
 * Copyright (C) 2017-2024 Hubert Figuière
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
    pub fn add(&mut self, v: u32) {
        self.0.insert(NiepceProperties::from(v));
    }
}

impl std::ops::Deref for PropertySet {
    type Target = npc_fwk::PropertySet<db::NiepceProperties>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// must be a tuple for cxx
#[derive(Clone, Default)]
pub struct PropertyBag(pub npc_fwk::PropertyBag<db::NiepceProperties>);

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
    Box::<PropertySet>::default()
}

pub type NiepcePropertySet = PropertySet;
pub type NiepcePropertyBag = PropertyBag;

use crate::db::{Label, LibMetadata};
use crate::library::notification::LibNotification;
use crate::libraryclient::{LibraryClientHost, LibraryClientWrapper, UIDataProvider};

#[cxx::bridge(namespace = "eng")]
pub mod ffi {
    #[namespace = "fwk"]
    extern "C++" {
        include!("fwk/cxx_prelude.hpp");
        include!("fwk/cxx_colour_bindings.hpp");

        type RgbColour = npc_fwk::base::rgbcolour::RgbColour;
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
        NpNiepceRenderEngineProp,
        NpNiepceXmpPacket,
        // Always keep this last.
        _NpPropertyEnd,
    }

    extern "Rust" {
        type Label;

        fn colour(&self) -> &RgbColour;
        fn label(&self) -> &str;
        fn id(&self) -> i64;
        fn clone_boxed(&self) -> Box<Label>;
    }

    extern "Rust" {
        type LibMetadata;

        fn id(&self) -> i64;
        fn to_properties(&self, propset: &PropertySet) -> Box<PropertyBag>;
    }

    #[namespace = "fwk"]
    extern "Rust" {
        type PropertyBag;

        fn is_empty(&self) -> bool;
        fn len(&self) -> usize;
        fn contains_key(&self, key: &u32) -> bool;
        fn key_by_index(&self, idx: usize) -> u32;
    }

    #[namespace = "fwk"]
    extern "Rust" {
        type PropertySet;

        #[cxx_name = "PropertySet_new"]
        fn property_set_new() -> Box<PropertySet>;
        fn add(&mut self, v: u32);
    }

    #[repr(i32)]
    pub enum NotificationType {
        None,
        NewLibraryCreated,
        DatabaseNeedUpgrade,
        DatabaseReady,
        AddedFolder,
        AddedFile,
        AddedFiles,
        AddedKeyword,
        AddedLabel,
        AddedAlbum,
        AddedToAlbum,
        RemovedFromAlbum,
        AlbumContentQueried,
        AlbumCounted,
        AlbumCountChange,
        AlbumDeleted,
        AlbumRenamed,
        FolderContentQueried,
        FolderDeleted,
        FolderCounted,
        FolderCountChange,
        KeywordContentQueried,
        KeywordCounted,
        KeywordCountChange,
        #[cxx_name = "METADATA_QUERIED"]
        MetadataQueried,
        #[cxx_name = "METADATA_CHANGED"]
        MetadataChanged,
        LabelChanged,
        LabelDeleted,
        XmpNeedsUpdate,
        FileMoved,
        FileStatusChanged,
        ThumbnailLoaded,
        ImageRendered,
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
    }
}
