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
use std::collections::HashMap;
use std::rc::Rc;

use gettextrs::gettext as i18n;
use glib::SignalHandlerId;
use gtk4::prelude::*;
use npc_fwk::{glib, gtk4};

use npc_engine::catalog;
use npc_engine::catalog::NiepcePropertyIdx;
use npc_engine::library::notification::MetadataChange;
use npc_engine::{NiepcePropertyBag, NiepcePropertySet};
use npc_fwk::toolkit::widgets::MetadataPropertyBag;
use npc_fwk::toolkit::widgets::{MetaDT, MetadataFormat, MetadataSectionFormat, MetadataWidget};
use npc_fwk::toolkit::{Controller, ControllerImplCell, UiController};
use npc_fwk::{PropertyBag, PropertyValue, dbg_out, err_out, send_async_local};

lazy_static::lazy_static! {
    static ref FORMATS: Vec<MetadataSectionFormat> = {
        let formats = vec![
            MetadataSectionFormat{
                section: i18n("File Information"),
                condition_prop: None,
                formats: vec![
                    MetadataFormat{ label: i18n("File Name:"), id: NiepcePropertyIdx::FileName as u32, type_: MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Folder:"), id: NiepcePropertyIdx::Folder as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("File Type:"), id: NiepcePropertyIdx::FileType as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("File Size:"), id: NiepcePropertyIdx::FileSize as u32, type_:MetaDT::Size, readonly: true },
                    MetadataFormat{ label: i18n("Sidecar Files:"), id: NiepcePropertyIdx::Sidecars as u32, type_:MetaDT::StringArray, readonly: true },
                ]
            },
            MetadataSectionFormat{
                section: i18n("Camera Information"),
                condition_prop: None,
                formats: vec![
                    MetadataFormat{ label: i18n("Make:"), id: NiepcePropertyIdx::TiffMake as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Model:"), id: NiepcePropertyIdx::TiffModel as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Lens:"), id: NiepcePropertyIdx::ExifAuxLens as u32, type_:MetaDT::String, readonly: true },
                ]
            },
            MetadataSectionFormat{
                section: i18n("Shooting Information"),
                condition_prop: None,
                formats: vec![
                    MetadataFormat{ label: i18n("Exposure Program:"), id: NiepcePropertyIdx::ExifExposureProgram as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Speed:"), id: NiepcePropertyIdx::ExifExposureTime as u32, type_:MetaDT::Frac, readonly: true },
                    MetadataFormat{ label: i18n("Aperture:"), id: NiepcePropertyIdx::ExifFNumber as u32, type_:MetaDT::FracDec, readonly: true },
                    MetadataFormat{ label: i18n("ISO:"), id: NiepcePropertyIdx::ExifIsoSpeedRatings as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Exposure Bias:"), id: NiepcePropertyIdx::ExifExposureBias as u32, type_:MetaDT::FracDec, readonly: true },
                    MetadataFormat{ label: i18n("Flash:"), id: NiepcePropertyIdx::ExifFlashFired as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Flash compensation:"), id: NiepcePropertyIdx::ExifAuxFlashCompensation as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Focal length:"), id: NiepcePropertyIdx::ExifFocalLength as u32, type_:MetaDT::FracDec, readonly: true },
                    MetadataFormat{ label: i18n("White balance:"), id: NiepcePropertyIdx::ExifWb as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Date:"), id: NiepcePropertyIdx::ExifDateTimeOriginal as u32, type_:MetaDT::Date, readonly: false },
                ]
            },
            MetadataSectionFormat{
                section: i18n("IPTC"),
                condition_prop: None,
                formats: vec![
                    MetadataFormat{ label: i18n("Headline:"), id: NiepcePropertyIdx::IptcHeadline as u32, type_:MetaDT::String, readonly: false },
                    MetadataFormat{ label: i18n("Caption:"), id: NiepcePropertyIdx::IptcDescription as u32, type_:MetaDT::Text, readonly: false },
                    MetadataFormat{ label: i18n("Rating:"), id: NiepcePropertyIdx::XmpRating as u32, type_:MetaDT::StarRating, readonly: false },
                    // FIXME change this type to the right one when there is a widget
                    MetadataFormat{ label: i18n("Label:"), id: NiepcePropertyIdx::XmpLabel as u32, type_:MetaDT::String, readonly: true },
                    MetadataFormat{ label: i18n("Keywords:"), id: NiepcePropertyIdx::IptcKeywords as u32, type_:MetaDT::StringArray, readonly: false },
                ]
            },
            MetadataSectionFormat{
                section: i18n("Rights"),
                condition_prop: None,
                formats: vec![]
            },
            MetadataSectionFormat{
                section: i18n("Processing"),
                condition_prop: None,
                formats: vec![
                    MetadataFormat{ label: i18n("Process:"), id: NiepcePropertyIdx::NiepceRenderEngine as u32, type_: MetaDT::String, readonly: true },
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
    propmaps: RefCell<HashMap<catalog::LibraryId, NiepcePropertyBag>>,
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
            propmaps: RefCell::default(),
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
            let conditional = current.condition_prop.is_some();
            w.set_data_format(Some(current.clone()));
            let sender = self.sender();
            let sig_id = w.connect_metadata_changed(glib::clone!(
                #[strong]
                sender,
                move |_, new, old| {
                    send_async_local!(MetadataInputMsg::MetadataChanged(new.0, old.0), sender);
                }
            ));
            w.set_visible(!conditional);
            self.widgets.push((w, sig_id));
        }
    }

    fn build_properties(
        metas: &HashMap<catalog::LibraryId, NiepcePropertyBag>,
    ) -> NiepcePropertyBag {
        let mut mixed_properties = NiepcePropertyBag::default();
        for (idx, properties) in metas.values().enumerate() {
            if idx == 0 {
                mixed_properties = properties.clone();
            } else {
                mixed_properties.merge_mixed(properties.clone());
            }
        }
        mixed_properties
    }

    pub fn display(&self, metadatas: Option<&Vec<catalog::LibMetadata>>) {
        dbg_out!("displaying metadatas");
        self.propmaps.borrow_mut().clear();

        if let Some(metas) = metadatas {
            {
                let mut propmaps = self.propmaps.borrow_mut();
                for meta in metas {
                    let properties = meta.to_properties(&self.propset);
                    propmaps.insert(meta.id(), properties);
                }
            }
            let mixed_properties = Self::build_properties(&self.propmaps.borrow());

            // XXX this is bad performance. The problem is the widget
            // is generic and uses generic properties.
            let into = PropertyBag::<u32>::from(mixed_properties);
            // XXX we have multiple copies of the property bag. That's
            // not a good idea.
            for element in &self.widgets {
                let widget = &element.0;
                widget.set_data_source(Some(into.clone()));
                let mut visible = false;
                if let Some(condition) = widget
                    .data_format()
                    .as_ref()
                    .and_then(|fmt| fmt.condition_prop)
                {
                    if let Some(v) = into.get(&condition).and_then(PropertyValue::boolean) {
                        visible = v;
                    }
                } else {
                    // If no condition, it's always visible.
                    visible = true;
                }
                widget.set_visible(visible);
            }
        } else {
            for element in &self.widgets {
                element.0.set_data_source(None);
            }
        }
    }

    /// Update the metadata. Will check it is relevant.
    pub fn update(&self, change: &MetadataChange) {
        {
            let mut propmaps = self.propmaps.borrow_mut();
            for id in &change.ids {
                if let Some(bag) = propmaps.get_mut(id) {
                    bag.set_value(change.meta, change.value.clone());
                } else {
                    err_out!("Update: the file id {id} isn't found");
                }
            }
        }

        let mixed_properties = Self::build_properties(&self.propmaps.borrow());

        for element in &self.widgets {
            if let Some(value) = mixed_properties.get(&change.meta) {
                element.0.update_data(change.meta as u32, value.clone());
            }
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
