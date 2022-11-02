/*
 * niepce - eng/db/libmetadata.rs
 *
 * Copyright (C) 2017-2022 Hubert Figuière
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

use std::ops::DerefMut;

use chrono::Utc;

use super::libfile::FileType;
use super::props;
use super::NiepceProperties as Np;
use super::{FromDb, LibraryId};
use crate::NiepcePropertyBag;
use npc_fwk::base::date::Date;
use npc_fwk::toolkit::widgets::WrappedPropertyBag;
use npc_fwk::utils::exempi::{NS_DC, NS_XAP};
use npc_fwk::{dbg_out, err_out};
use npc_fwk::{xmp_date_from, PropertyBag, PropertySet, PropertyValue, XmpMeta};

#[derive(Clone, Debug)]
pub struct LibMetadata {
    xmp: XmpMeta,
    id: LibraryId,
    pub sidecars: Vec<String>,
    pub file_type: FileType,
    pub name: String,
    pub folder: String,
}

struct IndexToXmp {
    pub ns: &'static str,
    pub property: &'static str,
}

fn property_index_to_xmp(meta: Np) -> Option<IndexToXmp> {
    props::PROP_TO_XMP_MAP.get(&meta).map(|t| IndexToXmp {
        ns: t.0,
        property: t.1,
    })
}

impl LibMetadata {
    pub fn new(id: LibraryId) -> LibMetadata {
        LibMetadata {
            xmp: XmpMeta::new(),
            id,
            sidecars: vec![],
            file_type: FileType::Unknown,
            name: String::new(),
            folder: String::new(),
        }
    }

    pub fn new_with_xmp(id: LibraryId, xmp: XmpMeta) -> LibMetadata {
        LibMetadata {
            xmp,
            id,
            sidecars: vec![],
            file_type: FileType::Unknown,
            name: String::new(),
            folder: String::new(),
        }
    }

    pub fn id(&self) -> LibraryId {
        self.id
    }

    pub fn serialize_inline(&self) -> String {
        self.xmp.serialize_inline()
    }

    fn get_metadata(&self, meta: Np) -> Option<PropertyValue> {
        let index_to_xmp = property_index_to_xmp(meta)?;

        let mut prop_flags = exempi::PropFlags::default();
        let mut xmp_result =
            self.xmp
                .xmp
                .get_property(index_to_xmp.ns, index_to_xmp.property, &mut prop_flags);
        if xmp_result.is_ok() && prop_flags.contains(exempi::PropFlags::ARRAY_IS_ALTTEXT) {
            if let Ok((_, value)) = self.xmp.xmp.get_localized_text(
                index_to_xmp.ns,
                index_to_xmp.property,
                "",
                "x-default",
                &mut prop_flags,
            ) {
                xmp_result = Ok(value);
            }
        }
        Some(PropertyValue::String(String::from(
            xmp_result.ok()?.to_str(),
        )))
    }

    pub fn set_metadata(&mut self, meta: Np, value: &PropertyValue) -> bool {
        if let Some(ix) = property_index_to_xmp(meta) {
            match *value {
                PropertyValue::Empty => {
                    return self.xmp.xmp.delete_property(ix.ns, ix.property).is_ok()
                }
                PropertyValue::Int(i) => {
                    return self
                        .xmp
                        .xmp
                        .set_property_i32(ix.ns, ix.property, i, exempi::PropFlags::NONE)
                        .is_ok()
                }
                PropertyValue::String(ref s) => {
                    if s.is_empty() {
                        return self.xmp.xmp.delete_property(ix.ns, ix.property).is_ok();
                    } else if self
                        .xmp
                        .xmp
                        .set_property(ix.ns, ix.property, s, exempi::PropFlags::NONE)
                        .is_err()
                    {
                        if exempi::get_error() == exempi::Error::BadXPath {
                            return self
                                .xmp
                                .xmp
                                .set_localized_text(
                                    ix.ns,
                                    ix.property,
                                    "",
                                    "x-default",
                                    s,
                                    exempi::PropFlags::NONE,
                                )
                                .is_ok();
                        }
                    } else {
                        return true;
                    }
                }
                PropertyValue::StringArray(ref sa) => {
                    if self.xmp.xmp.delete_property(ix.ns, ix.property).is_err() {
                        err_out!("Error deleting property {}", &ix.property);
                        return false;
                    }
                    for s in sa {
                        if self
                            .xmp
                            .xmp
                            .append_array_item(
                                ix.ns,
                                ix.property,
                                exempi::PropFlags::VALUE_IS_ARRAY,
                                s,
                                exempi::PropFlags::NONE,
                            )
                            .is_err()
                        {
                            err_out!("Error appending array item {} in property {}", &s, &ix.ns);
                            return false;
                        }
                    }
                    return true;
                }
                PropertyValue::Date(ref d) => {
                    let xmp_date = xmp_date_from(d);
                    return self
                        .xmp
                        .xmp
                        .set_property_date(ix.ns, ix.property, &xmp_date, exempi::PropFlags::NONE)
                        .is_ok();
                }
            }
            err_out!(
                "error setting property {}:{} {}",
                ix.ns,
                ix.property,
                exempi::get_error() as u32
            );
            return false;
        }
        err_out!("Unknown property {:?}", meta);
        false
    }

    pub fn to_properties(&self, propset: &PropertySet<Np>) -> Box<NiepcePropertyBag> {
        use super::NiepcePropertyIdx::*;
        let mut property_bag = Box::new(NiepcePropertyBag::default());
        let props = &mut property_bag.deref_mut().0;
        for prop_id in propset {
            match *prop_id {
                Np::Index(NpXmpRatingProp) => {
                    if let Some(rating) = self.xmp.rating() {
                        props.set_value(*prop_id, PropertyValue::Int(rating));
                    }
                }
                Np::Index(NpXmpLabelProp) => {
                    if let Some(label) = self.xmp.label() {
                        props.set_value(*prop_id, PropertyValue::String(label));
                    }
                }
                Np::Index(NpTiffOrientationProp) => {
                    if let Some(orientation) = self.xmp.orientation() {
                        props.set_value(*prop_id, PropertyValue::Int(orientation));
                    }
                }
                Np::Index(NpExifDateTimeOriginalProp) => {
                    if let Some(date) = self.xmp.creation_date().map(Date) {
                        props.set_value(*prop_id, PropertyValue::Date(date));
                    }
                }
                Np::Index(NpIptcKeywordsProp) => {
                    let mut iter = exempi::XmpIterator::new(
                        &self.xmp.xmp,
                        NS_DC,
                        "subject",
                        exempi::IterFlags::JUST_LEAF_NODES,
                    );
                    let mut keywords: Vec<String> = vec![];
                    let mut schema = exempi::XmpString::new();
                    let mut name = exempi::XmpString::new();
                    let mut value = exempi::XmpString::new();
                    let mut flags = exempi::PropFlags::default();
                    while iter.next(&mut schema, &mut name, &mut value, &mut flags) {
                        keywords.push(String::from(value.to_str()));
                    }
                    props.set_value(*prop_id, PropertyValue::StringArray(keywords));
                }
                Np::Index(NpFileNameProp) => {
                    props.set_value(*prop_id, PropertyValue::String(self.name.clone()));
                }
                Np::Index(NpFileTypeProp) => {
                    let file_type: &str = self.file_type.into();
                    props.set_value(*prop_id, PropertyValue::String(String::from(file_type)));
                }
                Np::Index(NpFileSizeProp) => {}
                Np::Index(NpFolderProp) => {
                    props.set_value(*prop_id, PropertyValue::String(self.folder.clone()));
                }
                Np::Index(NpSidecarsProp) => {
                    props.set_value(*prop_id, PropertyValue::StringArray(self.sidecars.clone()));
                }
                _ => {
                    if let Some(propval) = self.get_metadata(*prop_id) {
                        props.set_value(*prop_id, propval);
                    } else {
                        dbg_out!("missing prop {:?}", prop_id);
                    }
                }
            }
        }
        property_bag
    }

    // cxx
    pub fn to_wrapped_properties(&self, propset: &PropertySet<Np>) -> *mut WrappedPropertyBag {
        let bag = self.to_properties(propset);
        Box::into_raw(Box::new(WrappedPropertyBag(into_u32(*bag))))
    }

    pub fn touch(&mut self) -> bool {
        let xmpdate = xmp_date_from(&Utc::now());
        self.xmp
            .xmp
            .set_property_date(NS_XAP, "MetadataDate", &xmpdate, exempi::PropFlags::NONE)
            .is_ok()
    }
}

impl FromDb for LibMetadata {
    fn read_db_columns() -> &'static str {
        "files.id,xmp,file_type,files.name,folders.name"
    }

    fn read_db_tables() -> &'static str {
        "files LEFT JOIN folders ON folders.id = files.parent_id"
    }

    fn read_db_where_id() -> &'static str {
        "files.id"
    }

    fn read_from(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let id: LibraryId = row.get(0)?;
        let xmp: String = row.get(1)?;

        let mut xmpmeta = XmpMeta::new();
        xmpmeta.unserialize(&xmp);
        let mut libmeta = LibMetadata::new_with_xmp(id, xmpmeta);
        let col: i32 = row.get(2)?;
        libmeta.file_type = FileType::from(col);
        libmeta.name = row.get(3)?;
        libmeta.folder = row.get(4)?;
        Ok(libmeta)
    }
}

fn into_u32(from: NiepcePropertyBag) -> PropertyBag<u32> {
    PropertyBag {
        bag: from.bag.iter().map(|v| (*v).into()).collect(),
        map: from
            .map
            .iter()
            .map(|(k, v)| ((*k).into(), v.clone()))
            .collect(),
    }
}
