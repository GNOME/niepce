/*
 * niepce - niepce/ui/metadata_pane_controller.rs
 *
 * Copyright (C) 2022-2026 Hubert Figuière
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

use std::cell::RefCell;
use std::rc::Rc;

use gettextrs::gettext as i18n;
use glib::SignalHandlerId;
use gtk4::prelude::*;
use npc_fwk::{glib, gtk4};

use npc_engine::NiepcePropertySet;
use npc_engine::catalog;
use npc_engine::catalog::NiepcePropertyIdx;
use npc_engine::library::notification::MetadataChange;
use npc_fwk::toolkit::widgets::MetadataPropertyBag;
use npc_fwk::toolkit::widgets::{MetaDT, MetadataFormat, MetadataSectionFormat, MetadataWidget};
use npc_fwk::toolkit::{Controller, ControllerImplCell, UiController};
use npc_fwk::{PropertyBag, dbg_out, send_async_local};

lazy_static::lazy_static! {
    static ref FORMATS: Vec<MetadataSectionFormat> = {
        let formats = vec![
            MetadataSectionFormat{
                section: i18n("File Information"),
                formats: vec![
                    MetadataFormat{ label: i18n("File Name:"), id: NiepcePropertyIdx::FileNameProp as u32, type_: MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Folder:"), id: NiepcePropertyIdx::FolderProp as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("File Type:"), id: NiepcePropertyIdx::FileTypeProp as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("File Size:"), id: NiepcePropertyIdx::FileSizeProp as u32, type_:MetaDT::Size, readonly: true },
                    MetadataFormat{ label: i18n("Sidecar Files:"), id: NiepcePropertyIdx::SidecarsProp as u32, type_:MetaDT::StringArray, readonly: true },
                ]
            },
            MetadataSectionFormat{
                section: i18n("Camera Information"),
                formats: vec![
                    MetadataFormat{ label: i18n("Make:"), id: NiepcePropertyIdx::TiffMakeProp as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Model:"), id: NiepcePropertyIdx::TiffModelProp as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Lens:"), id: NiepcePropertyIdx::ExifAuxLensProp as u32, type_:MetaDT::String, readonly: true },
                ]
            },
            MetadataSectionFormat{
                section: i18n("Shooting Information"),
                formats: vec![
                    MetadataFormat{ label: i18n("Exposure Program:"), id: NiepcePropertyIdx::ExifExposureProgramProp as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Speed:"), id: NiepcePropertyIdx::ExifExposureTimeProp as u32, type_:MetaDT::Frac, readonly: true },
                    MetadataFormat{ label: i18n("Aperture:"), id: NiepcePropertyIdx::ExifFNumberPropProp as u32, type_:MetaDT::FracDec, readonly: true },
                    MetadataFormat{ label: i18n("ISO:"), id: NiepcePropertyIdx::ExifIsoSpeedRatingsProp as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Exposure Bias:"), id: NiepcePropertyIdx::ExifExposureBiasProp as u32, type_:MetaDT::FracDec, readonly: true },
                    MetadataFormat{ label: i18n("Flash:"), id: NiepcePropertyIdx::ExifFlashFiredProp as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Flash compensation:"), id: NiepcePropertyIdx::ExifAuxFlashCompensationProp as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Focal length:"), id: NiepcePropertyIdx::ExifFocalLengthProp as u32, type_:MetaDT::FracDec, readonly: true },
                    MetadataFormat{ label: i18n("White balance:"), id: NiepcePropertyIdx::ExifWbProp as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Date:"), id: NiepcePropertyIdx::ExifDateTimeOriginalProp as u32, type_:MetaDT::Date, readonly: false },
                ]
            },
            MetadataSectionFormat{
                section: i18n("IPTC"),
                formats: vec![
                    MetadataFormat{ label: i18n("Headline:"), id: NiepcePropertyIdx::IptcHeadlineProp as u32, type_:MetaDT::String, readonly: false },
                    MetadataFormat{ label: i18n("Caption:"), id: NiepcePropertyIdx::IptcDescriptionProp as u32, type_:MetaDT::Text, readonly: false },
                    MetadataFormat{ label: i18n("Rating:"), id: NiepcePropertyIdx::XmpRatingProp as u32, type_:MetaDT::StarRating, readonly: false },
                    // FIXME change this type to the right one when there is a widget
                    MetadataFormat{ label: i18n("Label:"), id: NiepcePropertyIdx::XmpLabelProp as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Keywords:"), id: NiepcePropertyIdx::IptcKeywordsProp as u32, type_:MetaDT::StringArray, readonly: false },
                ]
            },
            MetadataSectionFormat{
                section: i18n("Rights"),
                formats: vec![]
            },
            MetadataSectionFormat{
                section: i18n("Processing"),
                formats: vec![
                    MetadataFormat{ label: i18n("Process:"), id: NiepcePropertyIdx::NiepceRenderEngineProp as u32, type_: MetaDT::String, readonly: true },
                ]
            },
        ];

        formats
    };
}

fn formats() -> &'static [MetadataSectionFormat] {
    &FORMATS
}

pub enum MetadataInputMsg {
    MetadataChanged(MetadataPropertyBag, MetadataPropertyBag),
}

pub enum MetadataOutputMsg {
    MetadataChanged(MetadataPropertyBag, MetadataPropertyBag),
}

pub struct MetadataPaneController {
    imp_: ControllerImplCell<MetadataInputMsg, MetadataOutputMsg>,
    vbox: gtk4::Box,
    widgets: Vec<(MetadataWidget, SignalHandlerId)>,
    propset: NiepcePropertySet,
    fileid: RefCell<Vec<catalog::LibraryId>>,
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
            imp_: ControllerImplCell::default(),
            vbox: gtk4::Box::new(gtk4::Orientation::Vertical, 0),
            widgets: vec![],
            propset: NiepcePropertySet::default(),
            fileid: RefCell::default(),
        };

        ctrl.build_widget();

        let ctrl = Rc::new(ctrl);

        <Self as Controller>::start(&ctrl);

        ctrl
    }

    fn build_property_set(&mut self) {
        let formats = formats();
        for current in formats {
            for format in &current.formats {
                self.propset
                    .insert(NiepcePropertyIdx::try_from(format.id).unwrap());
            }
        }
    }

    fn build_widget(&mut self) {
        self.build_property_set();
        let formats = formats();
        for current in formats {
            let w = MetadataWidget::new(&current.section);
            self.vbox.append(&w);
            w.set_data_format(Some(current.clone()));
            let sender = self.sender();
            let sig_id = w.connect_metadata_changed(glib::clone!(
                #[strong]
                sender,
                move |_, new, old| {
                    send_async_local!(MetadataInputMsg::MetadataChanged(new.0, old.0), sender);
                }
            ));
            self.widgets.push((w, sig_id));
        }
    }

    pub fn display(&self, metadatas: Option<&Vec<catalog::LibMetadata>>) {
        dbg_out!("displaying metadatas");
        if let Some(metas) = metadatas {
            let fileids = metas.iter().map(|meta| meta.id()).collect::<Vec<_>>();
            self.fileid.replace(fileids);

            let mut mixed_properties = PropertyBag::<NiepcePropertyIdx>::default();
            for (idx, meta) in metas.iter().enumerate() {
                let properties = meta.to_properties(&self.propset);
                if idx == 0 {
                    mixed_properties = properties;
                } else {
                    mixed_properties.merge_mixed(properties);
                }
            }

            // XXX this is bad performance. The problem is the widget
            // is generic and uses generic properties.
            let into = PropertyBag::<u32>::from(mixed_properties);
            // XXX we have multiple copies of the property bag. That's
            // not a good idea.
            for element in &self.widgets {
                element.0.set_data_source(Some(into.clone()));
            }
        } else {
            self.fileid.replace(vec![]);
            for element in &self.widgets {
                element.0.set_data_source(None);
            }
        }
    }

    /// Update the metadata. Will check it is relevant.
    pub fn update(&self, change: &MetadataChange) {
        if self
            .fileid
            .borrow()
            .iter()
            .find(|v| **v == change.id)
            .is_none()
        {
            return;
        }
        for element in &self.widgets {
            element
                .0
                .update_data(change.meta as u32, change.value.clone());
        }
    }
}

#[cfg(test)]
mod test {
    use npc_engine::catalog::NiepcePropertyIdx;

    #[test]
    fn test_format_valid_properties() {
        let formats = super::formats();
        for current in formats {
            for format in &current.formats {
                NiepcePropertyIdx::try_from(format.id).unwrap();
            }
        }
    }
}
