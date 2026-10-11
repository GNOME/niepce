/*
 * niepce - engine/metadata/analog_exif.rs
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

//! Analog Exif metadata.
//! Source https://analogexif.sourceforge.net/help/analogexif-xmp.php

use gettextrs::gettext as i18n;

use npc_fwk::toolkit::widgets::{MetaDT, MetadataFormat, MetadataSectionFormat};

use crate::catalog::NiepcePropertyIdx as Npi;

pub const ANALOG_EXIF_NAMESPACE: &str = "http://analogexif.sourceforge.net/ns";
pub const ANALOG_EXIF_NS_PREFIX: &str = "ae";

const AE_EXPOSURE_NUMBER: &str = "ExposureNumber";
const AE_LENS_SERIAL_NUMBER: &str = "LensSerialNumber";
const AE_ROLL_ID: &str = "RollId";
const AE_FILM_MAKER: &str = "FilmMaker";
const AE_FILM: &str = "Film";
const AE_FILM_ALIAS: &str = "FilmAlias";
const AE_FILM_GRAIN: &str = "FilmGrain";
const AE_FILM_TYPE: &str = "FilmType";

/*
 * Predefined string value. Film type. Available values:
 */
const FILM_TYPES: [&str; 86] = [
    "135",
    "120",
    "220",
    "APS",
    "4×5",
    "8×10",
    "Type 600",
    "127",
    "Disc",
    "Paper",
    "126",
    "101",
    "102",
    "103",
    "104",
    "105",
    "106",
    "107",
    "108",
    "109",
    "110",
    "111",
    "112",
    "113",
    "114",
    "115",
    "116",
    "117",
    "118",
    "119",
    "121",
    "122",
    "123",
    "124",
    "125",
    "128",
    "129",
    "235",
    "335",
    "435",
    "518",
    "520",
    "522",
    "523",
    "616",
    "617",
    "620",
    "645",
    "828",
    "35",
    "Minox",
    "Karat",
    "Rapid",
    "SL",
    "K 16",
    "1⅝×2⅛",
    "2×2½",
    "2×3",
    "2½×3½",
    "3×4",
    "3⅛×4⅛",
    "3¼×4¼",
    "3¼×5½",
    "4¾×6½",
    "4½×5½",
    "4×10",
    "5×7",
    "7×17",
    "8×20",
    "8½×6½",
    "11×14",
    "12×20",
    "14×17",
    "16×20",
    "20×24",
    "6.5×9",
    "9×12",
    "10×15",
    "13×18",
    "18×24",
    "24×30",
    "SX-70",
    "Type 37",
    "Type 47",
    "Type 88",
    "Type 100",
];

lazy_static::lazy_static! {
    pub static ref ANALOG_EXIF_FORMATS: [MetadataSectionFormat; 1] = [
        MetadataSectionFormat{
            section: i18n("Film"),
            condition_prop: Some(Npi::NiepceIsAnalog as u32),
            formats: vec![
                MetadataFormat{ label: i18n("Frame Number:"), id: Npi::AnalogExposureNum as u32, type_:MetaDT::String, readonly: false },
                MetadataFormat{ label: i18n("Roll:"), id: Npi::AnalogRollId as u32, type_:MetaDT::String, readonly: false },
                MetadataFormat{ label: i18n("Film:"), id: Npi::AnalogFilm as u32, type_:MetaDT::String, readonly: false },
                MetadataFormat{ label: i18n("Film Make:"), id: Npi::AnalogFilmMaker as u32, type_:MetaDT::String, readonly: false },
                MetadataFormat{ label: i18n("Film Format:"), id: Npi::AnalogFilmType as u32, type_:MetaDT::Choices(FILM_TYPES.iter().map(|s| s.to_string()).collect()), readonly: false },
                MetadataFormat{ label: i18n("Process:"), id: Npi::AnalogDeveloperProcess as u32, type_:MetaDT::String, readonly: false },
                MetadataFormat{ label: i18n("Filter:"), id: Npi::AnalogFilter as u32, type_:MetaDT::String, readonly: false },
                MetadataFormat{ label: i18n("Scanner:"), id: Npi::AnalogScanner as u32, type_:MetaDT::String, readonly: false },
                MetadataFormat{ label: i18n("Scanner Make:"), id: Npi::AnalogScannerMaker as u32, type_:MetaDT::String, readonly: false },
                MetadataFormat{ label: i18n("Scanner Software:"), id: Npi::AnalogScannerSoftware as u32, type_:MetaDT::String, readonly: false },
            ]
        },
    ];
}

const AE_DEVELOPER: &str = "Developer";
const AE_DEVELOPER_PROCESS: &str = "DeveloperProcess";
const AE_DEVELOPER_MAKER: &str = "DeveloperMaker";
const AE_DEVELOPER_DILUTION: &str = "DeveloperDilution";
const AE_DEVELOP_TIME: &str = "DevelopTime";
const AE_LAB: &str = "Lab";
const AE_LAB_ADDRESS: &str = "LabAddress";
const AE_FILTER: &str = "Filter";
const AE_SCANNER_MAKER: &str = "ScannerMaker";
const AE_SCANNER: &str = "Scanner";
const AE_SCANNER_SOFTWARE: &str = "ScannerSoftware";

pub(crate) const PROP_TO_XMP_MAP: [(Npi, (&str, &str)); 19] = [
    (
        Npi::AnalogExposureNum,
        (ANALOG_EXIF_NAMESPACE, AE_EXPOSURE_NUMBER),
    ),
    (
        Npi::AnalogLensSerialNumber,
        (ANALOG_EXIF_NAMESPACE, AE_LENS_SERIAL_NUMBER),
    ),
    (Npi::AnalogRollId, (ANALOG_EXIF_NAMESPACE, AE_ROLL_ID)),
    (Npi::AnalogFilmMaker, (ANALOG_EXIF_NAMESPACE, AE_FILM_MAKER)),
    (Npi::AnalogFilm, (ANALOG_EXIF_NAMESPACE, AE_FILM)),
    (Npi::AnalogFilmAlias, (ANALOG_EXIF_NAMESPACE, AE_FILM_ALIAS)),
    (Npi::AnalogFilmGrain, (ANALOG_EXIF_NAMESPACE, AE_FILM_GRAIN)),
    (Npi::AnalogFilmType, (ANALOG_EXIF_NAMESPACE, AE_FILM_TYPE)),
    (Npi::AnalogDeveloper, (ANALOG_EXIF_NAMESPACE, AE_DEVELOPER)),
    (
        Npi::AnalogDeveloperProcess,
        (ANALOG_EXIF_NAMESPACE, AE_DEVELOPER_PROCESS),
    ),
    (
        Npi::AnalogDeveloperMaker,
        (ANALOG_EXIF_NAMESPACE, AE_DEVELOPER_MAKER),
    ),
    (
        Npi::AnalogDeveloperDilution,
        (ANALOG_EXIF_NAMESPACE, AE_DEVELOPER_DILUTION),
    ),
    (
        Npi::AnalogDevelopTime,
        (ANALOG_EXIF_NAMESPACE, AE_DEVELOP_TIME),
    ),
    (Npi::AnalogLab, (ANALOG_EXIF_NAMESPACE, AE_LAB)),
    (
        Npi::AnalogLabAddress,
        (ANALOG_EXIF_NAMESPACE, AE_LAB_ADDRESS),
    ),
    (Npi::AnalogFilter, (ANALOG_EXIF_NAMESPACE, AE_FILTER)),
    (
        Npi::AnalogScannerMaker,
        (ANALOG_EXIF_NAMESPACE, AE_SCANNER_MAKER),
    ),
    (Npi::AnalogScanner, (ANALOG_EXIF_NAMESPACE, AE_SCANNER)),
    (
        Npi::AnalogScannerSoftware,
        (ANALOG_EXIF_NAMESPACE, AE_SCANNER_SOFTWARE),
    ),
];

pub(crate) fn is_analog(xmp: &npc_fwk::XmpMeta) -> bool {
    xmp.xmp.has_property(ANALOG_EXIF_NAMESPACE, AE_FILM)
        || xmp.xmp.has_property(ANALOG_EXIF_NAMESPACE, AE_SCANNER)
}
