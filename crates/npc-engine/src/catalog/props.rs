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

use crate::metadata::xmp;

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

impl TryFrom<u32> for NiepcePropertyIdx {
    type Error = npc_fwk::Error;

    fn try_from(v: u32) -> Result<NiepcePropertyIdx, npc_fwk::Error> {
        if v < NiepcePropertyIdx::_PropertyEnd as u32 {
            Ok(unsafe { std::mem::transmute::<u32, NiepcePropertyIdx>(v) })
        } else {
            Err(npc_fwk::anyerror!("Invalid property value"))
        }
    }
}

lazy_static! {
    pub static ref PROP_TO_XMP_MAP: std::collections::HashMap<NiepcePropertyIdx, (&'static str, &'static str)> = hashmap! {
        NiepcePropertyIdx::XmpRatingProp => (NS_XMP, "Rating"),
        NiepcePropertyIdx::XmpLabelProp => (NS_XMP, "Label"),
        NiepcePropertyIdx::TiffOrientationProp => (NS_TIFF, "Orientation"),
        NiepcePropertyIdx::TiffMakeProp => (NS_TIFF, "Make"),
        NiepcePropertyIdx::TiffModelProp => (NS_TIFF, "Model"),
        NiepcePropertyIdx::ExifAuxLensProp => (NS_EXIF_AUX, "Lens"),
        NiepcePropertyIdx::ExifExposureProgramProp => (NS_EXIF, "ExposureProgram"),
        NiepcePropertyIdx::ExifExposureTimeProp => (NS_EXIF, "ExposureTime"),
        NiepcePropertyIdx::ExifFNumberPropProp => (NS_EXIF, "FNumber"),
        NiepcePropertyIdx::ExifIsoSpeedRatingsProp => (NS_EXIF, "ISOSpeedRatings"),
        NiepcePropertyIdx::ExifExposureBiasProp => (NS_EXIF, "ExposureBiasValue"),
        NiepcePropertyIdx::ExifFlashFiredProp => (NS_EXIF, "Flash/exif:Fired"),
        NiepcePropertyIdx::ExifAuxFlashCompensationProp => (NS_EXIF_AUX, "FlashCompensation"),
        NiepcePropertyIdx::ExifWbProp => (NS_EXIF, "WhiteBalance"),
        NiepcePropertyIdx::ExifDateTimeOriginalProp => (NS_EXIF, "DateTimeOriginal"),
        NiepcePropertyIdx::ExifFocalLengthProp => (NS_EXIF, "FocalLength"),
        NiepcePropertyIdx::ExifGpsLongProp => (NS_EXIF, "GPSLongitude"),
        NiepcePropertyIdx::ExifGpsLatProp => (NS_EXIF, "GPSLatitude"),
        NiepcePropertyIdx::IptcHeadlineProp => (NS_PHOTOSHOP, "Headline"),
        NiepcePropertyIdx::IptcDescriptionProp => (NS_DC, "description"),
        NiepcePropertyIdx::IptcKeywordsProp => (NS_DC, "subject"),
        NiepcePropertyIdx::NiepceFlagProp => (xmp::NIEPCE_XMP_NAMESPACE, "Flag"),
        NiepcePropertyIdx::NiepceRenderEngineProp => (xmp::NIEPCE_XMP_NAMESPACE, "RenderEngine"),
    };
}
