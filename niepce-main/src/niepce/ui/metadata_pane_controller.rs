/*
 * niepce - niepce/ui/metadata_pane_controller.rs
 *
 * Copyright (C) 2022-2024 Hubert Figuière
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

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gettextrs::gettext as i18n;
use glib::SignalHandlerId;
use gtk4::prelude::*;

use npc_engine::db;
use npc_engine::db::NiepcePropertyIdx;
use npc_engine::PropertySet;
use npc_fwk::toolkit::widgets::WrappedPropertyBag;
use npc_fwk::toolkit::widgets::{MetaDT, MetadataFormat, MetadataSectionFormat, MetadataWidget};
use npc_fwk::toolkit::{Controller, ControllerImpl, UiController};
use npc_fwk::{dbg_out, send_async_local, PropertyBag};

lazy_static::lazy_static! {
    static ref FORMATS: Vec<MetadataSectionFormat> = vec![
        MetadataSectionFormat{
            section: i18n("File Information"),
            formats: vec![
                MetadataFormat{ label: i18n("File Name:"), id: NiepcePropertyIdx::NpFileNameProp.repr, type_: MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Folder:"), id: NiepcePropertyIdx::NpFolderProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("File Type:"), id: NiepcePropertyIdx::NpFileTypeProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("File Size:"), id: NiepcePropertyIdx::NpFileSizeProp.repr, type_:MetaDT::SIZE, readonly: true },
                MetadataFormat{ label: i18n("Sidecar Files:"), id: NiepcePropertyIdx::NpSidecarsProp.repr, type_:MetaDT::StringArray, readonly: true },
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
                MetadataFormat{ label: i18n("Aperture:"), id: NiepcePropertyIdx::NpExifFNumberPropProp.repr, type_:MetaDT::FracDec, readonly: true },
                MetadataFormat{ label: i18n("ISO:"), id: NiepcePropertyIdx::NpExifIsoSpeedRatingsProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Exposure Bias:"), id: NiepcePropertyIdx::NpExifExposureBiasProp.repr, type_:MetaDT::FracDec, readonly: true },
                MetadataFormat{ label: i18n("Flash:"), id: NiepcePropertyIdx::NpExifFlashFiredProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Flash compensation:"), id: NiepcePropertyIdx::NpExifAuxFlashCompensationProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Focal length:"), id: NiepcePropertyIdx::NpExifFocalLengthProp.repr, type_:MetaDT::FracDec, readonly: true },
                MetadataFormat{ label: i18n("White balance:"), id: NiepcePropertyIdx::NpExifWbProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Date:"), id: NiepcePropertyIdx::NpExifDateTimeOriginalProp.repr, type_:MetaDT::DATE, readonly: false },
            ]
        },
        MetadataSectionFormat{
            section: i18n("IPTC"),
            formats: vec![
                MetadataFormat{ label: i18n("Headline:"), id: NiepcePropertyIdx::NpIptcHeadlineProp.repr, type_:MetaDT::STRING, readonly: false },
                MetadataFormat{ label: i18n("Caption:"), id: NiepcePropertyIdx::NpIptcDescriptionProp.repr, type_:MetaDT::TEXT, readonly: false },
                MetadataFormat{ label: i18n("Rating:"), id: NiepcePropertyIdx::NpXmpRatingProp.repr, type_:MetaDT::StarRating, readonly: false },
                // FIXME change this type to the right one when there is a widget
                MetadataFormat{ label: i18n("Label:"), id: NiepcePropertyIdx::NpXmpLabelProp.repr, type_:MetaDT::STRING, readonly: true },
                MetadataFormat{ label: i18n("Keywords:"), id: NiepcePropertyIdx::NpIptcKeywordsProp.repr, type_:MetaDT::StringArray, readonly: false },
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

pub enum MetadataInputMsg {
    MetadataChanged(WrappedPropertyBag, WrappedPropertyBag),
}

pub enum MetadataOutputMsg {
    MetadataChanged(WrappedPropertyBag, WrappedPropertyBag),
}

pub struct MetadataPaneController {
    imp_: RefCell<ControllerImpl<MetadataInputMsg, MetadataOutputMsg>>,
    vbox: gtk4::Box,
    widgets: Vec<(MetadataWidget, SignalHandlerId)>,
    propset: PropertySet,
    fileid: Cell<db::LibraryId>,
}

impl Controller for MetadataPaneController {
    npc_fwk::controller_imp_imp!(imp_);

    type InMsg = MetadataInputMsg;
    type OutMsg = MetadataOutputMsg;

    fn dispatch(&self, msg: MetadataInputMsg) {
        let MetadataInputMsg::MetadataChanged(new, old) = msg;
        self.emit(MetadataOutputMsg::MetadataChanged(new, old))
    }
}

impl UiController for MetadataPaneController {
    fn widget(&self) -> &gtk4::Widget {
        self.vbox.upcast_ref()
    }
}

impl MetadataPaneController {
    pub fn new() -> Rc<MetadataPaneController> {
        let mut ctrl = MetadataPaneController {
            imp_: RefCell::default(),
            vbox: gtk4::Box::new(gtk4::Orientation::Vertical, 0),
            widgets: vec![],
            propset: PropertySet::default(),
            fileid: Cell::new(0),
        };

        ctrl.build_widget();

        let ctrl = Rc::new(ctrl);

        <Self as Controller>::start(&ctrl);

        ctrl
    }

    fn build_property_set(&mut self) {
        let formats = get_format();
        for current in formats {
            for format in &current.formats {
                self.propset.add(format.id);
            }
        }
    }

    fn build_widget(&mut self) {
        self.build_property_set();
        let formats = get_format();
        for current in formats {
            let w = MetadataWidget::new(&current.section);
            self.vbox.append(&w);
            w.set_data_format(Some(current.clone()));
            let sender = self.sender();
            let sig_id =
                w.connect_metadata_changed(glib::clone!(@strong sender => move |_, new, old| {
                    send_async_local!(MetadataInputMsg::MetadataChanged(new, old), sender);
                }));
            self.widgets.push((w, sig_id));
        }
    }

    pub fn displayed(&self) -> db::LibraryId {
        self.fileid.get()
    }

    pub fn display(&self, id: db::LibraryId, metadata: Option<&db::LibMetadata>) {
        self.fileid.set(id);
        dbg_out!("displaying metadata");
        if let Some(meta) = metadata {
            let properties = meta.to_properties(&self.propset);

            // XXX this is bad performance. The problem is the widget
            // is generic and uses generic properties.
            //
            // Also can we implement this as `From<>` ?
            let mut into = PropertyBag::<u32>::new();
            for key in properties.0.bag.iter() {
                if let Some(elem) = properties.0.map.get(key) {
                    into.set_value(u32::from(*key), elem.clone());
                }
            }
            // XXX we have multiple copies of the property bag. That's not a good idea.
            for element in &self.widgets {
                element.0.set_data_source(Some(into.clone()));
            }
        } else {
            for element in &self.widgets {
                element.0.set_data_source(None);
            }
        }
    }
}
