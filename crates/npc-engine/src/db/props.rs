/*
 * niepce - engine/db/props.rs
 *
 * Copyright (C) 2021-2023 Hubert Figuière
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

use lazy_static::lazy_static;
use maplit::hashmap;
use npc_fwk::utils::exempi::NS_AUX as NS_EXIF_AUX;
use npc_fwk::utils::exempi::{NS_DC, NS_EXIF, NS_PHOTOSHOP, NS_TIFF, NS_XAP};
mod xmp {
    pub use npc_fwk::utils::exempi::NIEPCE_XMP_NAMESPACE;
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(unused_parens)]
#[repr(C)]
pub enum NiepceProperties {
    Index(NiepcePropertyIdx),
    Other(u32),
}

impl From<NiepceProperties> for u32 {
    fn from(v: NiepceProperties) -> u32 {
        match v {
            NiepceProperties::Index(i) => i as u32,
            NiepceProperties::Other(i) => i,
        }
    }
}

impl From<u32> for NiepceProperties {
    fn from(v: u32) -> NiepceProperties {
        if v > 0 && v < NiepcePropertyIdx::_NpPropertyEnd as u32 {
            Self::Index(unsafe { std::mem::transmute::<u32, NiepcePropertyIdx>(v) })
        } else {
            Self::Other(v)
        }
    }
}
lazy_static! {
    pub static ref PROP_TO_XMP_MAP: std::collections::HashMap<NiepceProperties, (&'static str, &'static str)> = hashmap! {
    NiepceProperties::Index(NiepcePropertyIdx::NpXmpRatingProp) => (NS_XAP, "Rating"),
    NiepceProperties::Index(NiepcePropertyIdx::NpXmpLabelProp) => (NS_XAP, "Label"),
    NiepceProperties::Index(NiepcePropertyIdx::NpTiffOrientationProp) => (NS_TIFF, "Orientation"),
    NiepceProperties::Index(NiepcePropertyIdx::NpTiffMakeProp) => (NS_TIFF, "Make"),
    NiepceProperties::Index(NiepcePropertyIdx::NpTiffModelProp) => (NS_TIFF, "Model"),
    NiepceProperties::Index(NiepcePropertyIdx::NpExifAuxLensProp) => (NS_EXIF_AUX, "Lens"),
    NiepceProperties::Index(NiepcePropertyIdx::NpExifExposureProgramProp) => (NS_EXIF, "ExposureProgram"),
    NiepceProperties::Index(NiepcePropertyIdx::NpExifExposureTimeProp) => (NS_EXIF, "ExposureTime"),
    NiepceProperties::Index(NiepcePropertyIdx::NpExifFNumberPropProp) => (NS_EXIF, "FNumber"),
    NiepceProperties::Index(NiepcePropertyIdx::NpExifIsoSpeedRatingsProp) => (NS_EXIF, "ISOSpeedRatings"),
    NiepceProperties::Index(NiepcePropertyIdx::NpExifExposureBiasProp) => (NS_EXIF, "ExposureBiasValue"),
    NiepceProperties::Index(NiepcePropertyIdx::NpExifFlashFiredProp) => (NS_EXIF, "Flash/exif:Fired"),
    NiepceProperties::Index(NiepcePropertyIdx::NpExifAuxFlashCompensationProp) => (NS_EXIF_AUX, "FlashCompensation"),
    NiepceProperties::Index(NiepcePropertyIdx::NpExifWbProp) => (NS_EXIF, "WhiteBalance"),
    NiepceProperties::Index(NiepcePropertyIdx::NpExifDateTimeOriginalProp) => (NS_EXIF, "DateTimeOriginal"),
    NiepceProperties::Index(NiepcePropertyIdx::NpExifFocalLengthProp) => (NS_EXIF, "FocalLength"),
    NiepceProperties::Index(NiepcePropertyIdx::NpExifGpsLongProp) => (NS_EXIF, "GPSLongitude"),
    NiepceProperties::Index(NiepcePropertyIdx::NpExifGpsLatProp) => (NS_EXIF, "GPSLatitude"),
    NiepceProperties::Index(NiepcePropertyIdx::NpIptcHeadlineProp) => (NS_PHOTOSHOP, "Headline"),
    NiepceProperties::Index(NiepcePropertyIdx::NpIptcDescriptionProp) => (NS_DC, "description"),
    NiepceProperties::Index(NiepcePropertyIdx::NpIptcKeywordsProp) => (NS_DC, "subject"),
    NiepceProperties::Index(NiepcePropertyIdx::NpNiepceFlagProp) => (xmp::NIEPCE_XMP_NAMESPACE, "Flag"),
    NiepceProperties::Index(NiepcePropertyIdx::NpNiepceRenderEngineProp) => (xmp::NIEPCE_XMP_NAMESPACE, "RenderEngine"),
    };
}
