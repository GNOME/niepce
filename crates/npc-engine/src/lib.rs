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

#[cxx::bridge(namespace = "eng")]
pub mod ffi {
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

    #[namespace = "fwk"]
    extern "Rust" {
        type PropertyBag;

        fn is_empty(&self) -> bool;
        fn len(&self) -> usize;
        fn contains_key(&self, key: &u32) -> bool;
        fn key_by_index(&self, idx: usize) -> u32;
    }
}
