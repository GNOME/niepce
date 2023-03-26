/*
 * niepce - niepce/ui/metadata_pane_controllerd.rs
 *
 * Copyright (C) 2022-2023 Hubert Figuière
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

use gettextrs::gettext as i18n;

use npc_engine::db::NiepcePropertyIdx;
use npc_fwk::toolkit::widgets::{MetaDT, MetadataFormat, MetadataSectionFormat};

lazy_static::lazy_static! {
    static ref FORMATS: Vec<MetadataSectionFormat> = vec![
        MetadataSectionFormat{
            section: i18n("File Information"),
            formats: vec![
                MetadataFormat{ label: i18n("File Name:"), id: NiepcePropertyIdx::NpFileNameProp.repr, type_: MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Folder:"), id: NiepcePropertyIdx::NpFolderProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("File Type:"), id: NiepcePropertyIdx::NpFileTypeProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("File Size:"), id: NiepcePropertyIdx::NpFileSizeProp.repr, type_:MetaDT::SIZE, readonly: true },
                MetadataFormat{ label: i18n("Sidecar Files:"), id: NiepcePropertyIdx::NpSidecarsProp.repr, type_:MetaDT::STRING_ARRAY, readonly: true },
            ]
        },
        MetadataSectionFormat{
            section: i18n("Camera Information"),
            formats: vec![
                MetadataFormat{ label: i18n("Make:"), id: NiepcePropertyIdx::NpTiffMakeProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Model:"), id: NiepcePropertyIdx::NpTiffModelProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Lens:"), id: NiepcePropertyIdx::NpExifAuxLensProp.repr, type_:MetaDT::STRING, readonly: true },
            ]
        },
        MetadataSectionFormat{
            section: i18n("Shooting Information"),
            formats: vec![
                MetadataFormat{ label: i18n("Exposure Program:"), id: NiepcePropertyIdx::NpExifExposureProgramProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Speed:"), id: NiepcePropertyIdx::NpExifExposureTimeProp.repr, type_:MetaDT::FRAC, readonly: true },
                MetadataFormat{ label: i18n("Aperture:"), id: NiepcePropertyIdx::NpExifFNumberPropProp.repr, type_:MetaDT::FRAC_DEC, readonly: true },
                MetadataFormat{ label: i18n("ISO:"), id: NiepcePropertyIdx::NpExifIsoSpeedRatingsProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Exposure Bias:"), id: NiepcePropertyIdx::NpExifExposureBiasProp.repr, type_:MetaDT::FRAC_DEC, readonly: true },
                MetadataFormat{ label: i18n("Flash:"), id: NiepcePropertyIdx::NpExifFlashFiredProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Flash compensation:"), id: NiepcePropertyIdx::NpExifAuxFlashCompensationProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Focal length:"), id: NiepcePropertyIdx::NpExifFocalLengthProp.repr, type_:MetaDT::FRAC_DEC, readonly: true },
                MetadataFormat{ label: i18n("White balance:"), id: NiepcePropertyIdx::NpExifWbProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Date:"), id: NiepcePropertyIdx::NpExifDateTimeOriginalProp.repr, type_:MetaDT::DATE, readonly: false },
            ]
        },
        MetadataSectionFormat{
            section: i18n("IPTC"),
            formats: vec![
                MetadataFormat{ label: i18n("Headline:"), id: NiepcePropertyIdx::NpIptcHeadlineProp.repr, type_:MetaDT::STRING, readonly: false },
                MetadataFormat{ label: i18n("Caption:"), id: NiepcePropertyIdx::NpIptcDescriptionProp.repr, type_:MetaDT::TEXT, readonly: false },
                MetadataFormat{ label: i18n("Rating:"), id: NiepcePropertyIdx::NpXmpRatingProp.repr, type_:MetaDT::STAR_RATING, readonly: false },
                // FIXME change this type to the right one when there is a widget
                MetadataFormat{ label: i18n("Label:"), id: NiepcePropertyIdx::NpXmpLabelProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Keywords:"), id: NiepcePropertyIdx::NpIptcKeywordsProp.repr, type_:MetaDT::STRING_ARRAY, readonly: false },
            ]
        },
        MetadataSectionFormat{
            section: i18n("Rights"),
            formats: vec![]
        },
        MetadataSectionFormat{
            section: i18n("Processing"),
            formats: vec![
                MetadataFormat{ label: i18n("Process:"), id: NiepcePropertyIdx::NpNiepceRenderEngineProp.repr, type_: MetaDT::STRING, readonly: true },
            ]
        },
    ];
}

pub fn get_format() -> &'static [MetadataSectionFormat] {
    &FORMATS
}
