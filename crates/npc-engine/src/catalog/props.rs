/*
 * niepce - engine/catalog/props.rs
 *
 * Copyright (C) 2021-2026 Hubert Figuière
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

use npc_fwk::utils::xmp::{NS_DC, NS_EXIF, NS_EXIF_AUX, NS_PHOTOSHOP, NS_TIFF, NS_XMP};
mod xmp {
    pub use npc_fwk::utils::xmp::NIEPCE_XMP_NAMESPACE;
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum NiepcePropertyIdx {
    FileNameProp,
    FileTypeProp,
    FileSizeProp,
    FolderProp,
    SidecarsProp,
    XmpRatingProp,
    XmpLabelProp,
    TiffOrientationProp,
    TiffMakeProp,
    TiffModelProp,
    ExifAuxLensProp,
    ExifExposureProgramProp,
    ExifExposureTimeProp,
    ExifFNumberPropProp,
    ExifIsoSpeedRatingsProp,
    ExifExposureBiasProp,
    ExifFlashFiredProp,
    ExifAuxFlashCompensationProp,
    ExifWbProp,
    ExifDateTimeOriginalProp,
    ExifFocalLengthProp,
    ExifGpsLongProp,
    ExifGpsLatProp,
    IptcHeadlineProp,
    IptcDescriptionProp,
    IptcKeywordsProp,
    NiepceFlagProp,
    NiepceRenderEngineProp,
    NiepceXmpPacket,
    // Always keep this last.
    _PropertyEnd,
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
        if v > 0 && v < NiepcePropertyIdx::_PropertyEnd as u32 {
            Self::Index(unsafe { std::mem::transmute::<u32, NiepcePropertyIdx>(v) })
        } else {
            Self::Other(v)
        }
    }
}
lazy_static! {
    pub static ref PROP_TO_XMP_MAP: std::collections::HashMap<NiepceProperties, (&'static str, &'static str)> = hashmap! {
    NiepceProperties::Index(NiepcePropertyIdx::XmpRatingProp) => (NS_XMP, "Rating"),
    NiepceProperties::Index(NiepcePropertyIdx::XmpLabelProp) => (NS_XMP, "Label"),
    NiepceProperties::Index(NiepcePropertyIdx::TiffOrientationProp) => (NS_TIFF, "Orientation"),
    NiepceProperties::Index(NiepcePropertyIdx::TiffMakeProp) => (NS_TIFF, "Make"),
    NiepceProperties::Index(NiepcePropertyIdx::TiffModelProp) => (NS_TIFF, "Model"),
    NiepceProperties::Index(NiepcePropertyIdx::ExifAuxLensProp) => (NS_EXIF_AUX, "Lens"),
    NiepceProperties::Index(NiepcePropertyIdx::ExifExposureProgramProp) => (NS_EXIF, "ExposureProgram"),
    NiepceProperties::Index(NiepcePropertyIdx::ExifExposureTimeProp) => (NS_EXIF, "ExposureTime"),
    NiepceProperties::Index(NiepcePropertyIdx::ExifFNumberPropProp) => (NS_EXIF, "FNumber"),
    NiepceProperties::Index(NiepcePropertyIdx::ExifIsoSpeedRatingsProp) => (NS_EXIF, "ISOSpeedRatings"),
    NiepceProperties::Index(NiepcePropertyIdx::ExifExposureBiasProp) => (NS_EXIF, "ExposureBiasValue"),
    NiepceProperties::Index(NiepcePropertyIdx::ExifFlashFiredProp) => (NS_EXIF, "Flash/exif:Fired"),
    NiepceProperties::Index(NiepcePropertyIdx::ExifAuxFlashCompensationProp) => (NS_EXIF_AUX, "FlashCompensation"),
    NiepceProperties::Index(NiepcePropertyIdx::ExifWbProp) => (NS_EXIF, "WhiteBalance"),
    NiepceProperties::Index(NiepcePropertyIdx::ExifDateTimeOriginalProp) => (NS_EXIF, "DateTimeOriginal"),
    NiepceProperties::Index(NiepcePropertyIdx::ExifFocalLengthProp) => (NS_EXIF, "FocalLength"),
    NiepceProperties::Index(NiepcePropertyIdx::ExifGpsLongProp) => (NS_EXIF, "GPSLongitude"),
    NiepceProperties::Index(NiepcePropertyIdx::ExifGpsLatProp) => (NS_EXIF, "GPSLatitude"),
    NiepceProperties::Index(NiepcePropertyIdx::IptcHeadlineProp) => (NS_PHOTOSHOP, "Headline"),
    NiepceProperties::Index(NiepcePropertyIdx::IptcDescriptionProp) => (NS_DC, "description"),
    NiepceProperties::Index(NiepcePropertyIdx::IptcKeywordsProp) => (NS_DC, "subject"),
    NiepceProperties::Index(NiepcePropertyIdx::NiepceFlagProp) => (xmp::NIEPCE_XMP_NAMESPACE, "Flag"),
    NiepceProperties::Index(NiepcePropertyIdx::NiepceRenderEngineProp) => (xmp::NIEPCE_XMP_NAMESPACE, "RenderEngine"),
    };
}
