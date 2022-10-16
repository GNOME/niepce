use gettextrs::gettext as i18n;

use npc_engine::db::NiepcePropertyIdx;
use npc_fwk::toolkit::widgets::{MetaDT, MetadataFormat, MetadataSectionFormat};

lazy_static::lazy_static! {
    static ref FORMATS: Vec<MetadataSectionFormat> = vec![
        MetadataSectionFormat{
            section: i18n("File Information"),
            formats: vec![
                MetadataFormat{ label: i18n("File Name:"), id: NiepcePropertyIdx::NpFileNameProp as u32, type_: MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Folder:"), id: NiepcePropertyIdx::NpFolderProp as u32, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("File Type:"), id: NiepcePropertyIdx::NpFileTypeProp as u32, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("File Size:"), id: NiepcePropertyIdx::NpFileSizeProp as u32, type_:MetaDT::SIZE, readonly: true },
                MetadataFormat{ label: i18n("Sidecar Files:"), id: NiepcePropertyIdx::NpSidecarsProp as u32, type_:MetaDT::STRING_ARRAY, readonly: true },
            ]
        },
        MetadataSectionFormat{
            section: i18n("Camera Information"),
            formats: vec![
                MetadataFormat{ label: i18n("Make:"), id: NiepcePropertyIdx::NpTiffMakeProp as u32, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Model:"), id: NiepcePropertyIdx::NpTiffModelProp as u32, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Lens:"), id: NiepcePropertyIdx::NpExifAuxLensProp as u32, type_:MetaDT::STRING, readonly: true },
            ]
        },
        MetadataSectionFormat{
            section: i18n("Shooting Information"),
            formats: vec![
                MetadataFormat{ label: i18n("Exposure Program:"), id: NiepcePropertyIdx::NpExifExposureProgramProp as u32, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Speed:"), id: NiepcePropertyIdx::NpExifExposureTimeProp as u32, type_:MetaDT::FRAC, readonly: true },
                MetadataFormat{ label: i18n("Aperture:"), id: NiepcePropertyIdx::NpExifFNumberPropProp as u32, type_:MetaDT::FRAC_DEC, readonly: true },
                MetadataFormat{ label: i18n("ISO:"), id: NiepcePropertyIdx::NpExifIsoSpeedRatingsProp as u32, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Exposure Bias:"), id: NiepcePropertyIdx::NpExifExposureBiasProp as u32, type_:MetaDT::FRAC_DEC, readonly: true },
                MetadataFormat{ label: i18n("Flash:"), id: NiepcePropertyIdx::NpExifFlashFiredProp as u32, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Flash compensation:"), id: NiepcePropertyIdx::NpExifAuxFlashCompensationProp as u32, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Focal length:"), id: NiepcePropertyIdx::NpExifFocalLengthProp as u32, type_:MetaDT::FRAC_DEC, readonly: true },
                MetadataFormat{ label: i18n("White balance:"), id: NiepcePropertyIdx::NpExifWbProp as u32, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Date:"), id: NiepcePropertyIdx::NpExifDateTimeOriginalProp as u32, type_:MetaDT::DATE, readonly: false },
            ]
        },
        MetadataSectionFormat{
            section: i18n("IPTC"),
            formats: vec![
                MetadataFormat{ label: i18n("Headline:"), id: NiepcePropertyIdx::NpIptcHeadlineProp as u32, type_:MetaDT::STRING, readonly: false },
                MetadataFormat{ label: i18n("Caption:"), id: NiepcePropertyIdx::NpIptcDescriptionProp as u32, type_:MetaDT::TEXT, readonly: false },
                MetadataFormat{ label: i18n("Rating:"), id: NiepcePropertyIdx::NpXmpRatingProp as u32, type_:MetaDT::STAR_RATING, readonly: false },
                // FIXME change this type to the right one when there is a widget
                MetadataFormat{ label: i18n("Label:"), id: NiepcePropertyIdx::NpXmpLabelProp as u32, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Keywords:"), id: NiepcePropertyIdx::NpIptcKeywordsProp as u32, type_:MetaDT::STRING_ARRAY, readonly: false },
            ]
        },
        MetadataSectionFormat{
            section: i18n("Rights"),
            formats: vec![]
        },
    ];
}

pub fn get_format() -> &'static [MetadataSectionFormat] {
    &FORMATS
}
