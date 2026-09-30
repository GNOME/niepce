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

use npc_fwk::PropertyIndex;
use npc_fwk::utils::xmp::{NS_DC, NS_EXIF, NS_EXIF_AUX, NS_PHOTOSHOP, NS_TIFF, NS_XMP};

use crate::metadata::xmp;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum NiepcePropertyIdx {
    FileName,
    FileType,
    FileSize,
    Folder,
    Sidecars,
    XmpRating,
    XmpLabel,
    TiffOrientation,
    TiffMake,
    TiffModel,
    ExifAuxLens,
    ExifExposureProgram,
    ExifExposureTime,
    ExifFNumber,
    ExifIsoSpeedRatings,
    ExifExposureBias,
    ExifFlashFired,
    ExifAuxFlashCompensation,
    ExifWb,
    ExifDateTimeOriginal,
    ExifFocalLength,
    ExifGpsLong,
    ExifGpsLat,
    IptcHeadline,
    IptcDescription,
    IptcKeywords,
    NiepceIsRaw,
    NiepceFlag,
    NiepceRenderEngine,
    NiepceXmpPacket,
    NiepcePp3ForRaw,
    NiepcePp3ForJpeg,
    // Always keep this last.
    _PropertyEnd,
}

impl PropertyIndex for NiepcePropertyIdx {}

impl From<NiepcePropertyIdx> for u32 {
    fn from(pi: NiepcePropertyIdx) -> u32 {
        pi as u32
    }
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
    pub static ref PROP_TO_XMP_MAP: std::collections::HashMap<NiepcePropertyIdx, (&'static str, &'static str)> = {
        let map = hashmap! {
            NiepcePropertyIdx::XmpRating => (NS_XMP, "Rating"),
            NiepcePropertyIdx::XmpLabel => (NS_XMP, "Label"),
            NiepcePropertyIdx::TiffOrientation => (NS_TIFF, "Orientation"),
            NiepcePropertyIdx::TiffMake => (NS_TIFF, "Make"),
            NiepcePropertyIdx::TiffModel => (NS_TIFF, "Model"),
            NiepcePropertyIdx::ExifAuxLens => (NS_EXIF_AUX, "Lens"),
            NiepcePropertyIdx::ExifExposureProgram => (NS_EXIF, "ExposureProgram"),
            NiepcePropertyIdx::ExifExposureTime => (NS_EXIF, "ExposureTime"),
            NiepcePropertyIdx::ExifFNumber => (NS_EXIF, "FNumber"),
            NiepcePropertyIdx::ExifIsoSpeedRatings => (NS_EXIF, "ISOSpeedRatings"),
            NiepcePropertyIdx::ExifExposureBias => (NS_EXIF, "ExposureBiasValue"),
            NiepcePropertyIdx::ExifFlashFired => (NS_EXIF, "Flash/exif:Fired"),
            NiepcePropertyIdx::ExifAuxFlashCompensation => (NS_EXIF_AUX, "FlashCompensation"),
            NiepcePropertyIdx::ExifWb => (NS_EXIF, "WhiteBalance"),
            NiepcePropertyIdx::ExifDateTimeOriginal => (NS_EXIF, "DateTimeOriginal"),
            NiepcePropertyIdx::ExifFocalLength => (NS_EXIF, "FocalLength"),
            NiepcePropertyIdx::ExifGpsLong => (NS_EXIF, "GPSLongitude"),
            NiepcePropertyIdx::ExifGpsLat => (NS_EXIF, "GPSLatitude"),
            NiepcePropertyIdx::IptcHeadline => (NS_PHOTOSHOP, "Headline"),
            NiepcePropertyIdx::IptcDescription => (NS_DC, "description"),
            NiepcePropertyIdx::IptcKeywords => (NS_DC, "subject"),
            NiepcePropertyIdx::NiepceFlag => (xmp::NIEPCE_XMP_NAMESPACE, "Flag"),
            NiepcePropertyIdx::NiepceRenderEngine => (xmp::NIEPCE_XMP_NAMESPACE, "RenderEngine"),
            NiepcePropertyIdx::NiepcePp3ForRaw => (xmp::NIEPCE_XMP_NAMESPACE, xmp::PP3_FOR_RAW),
            NiepcePropertyIdx::NiepcePp3ForJpeg => (xmp::NIEPCE_XMP_NAMESPACE, xmp::PP3_FOR_JPEG),
        };

        map
    };
}
