/*
 * niepce - eng/db/libmetadata.rs
 *
 * Copyright (C) 2017-2025 Hubert Figuière
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

use super::NiepceProperties as Np;
use super::libfile::FileType;
use super::props;
use super::{FromDb, LibraryId};
use crate::NiepcePropertyBag;
use npc_fwk::utils::exempi::{NS_DC, NS_XAP};
use npc_fwk::{DateExt, PropertySet, PropertyValue, XmpMeta};
use npc_fwk::{dbg_out, err_out};

#[derive(Clone, Debug)]
pub struct LibMetadata {
    /// The XmpMeta block
    xmp_meta: XmpMeta,
    /// Library ID this is associated to.
    id: LibraryId,
    /// List of sidecards
    pub sidecars: Vec<String>,
    pub file_type: FileType,
    /// name to be displayed
    pub name: String,
    /// Folder name for display
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
            xmp_meta: XmpMeta::new(),
            id,
            sidecars: vec![],
            file_type: FileType::Unknown,
            name: String::new(),
            folder: String::new(),
        }
    }

    /// Create a LibMetadata with an XmpMeta.
    pub fn new_with_xmp(id: LibraryId, xmp_meta: XmpMeta) -> LibMetadata {
        LibMetadata {
            xmp_meta,
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
        self.xmp_meta.serialize_inline()
    }

    pub fn get_metadata(&self, meta: Np) -> Option<PropertyValue> {
        let index_to_xmp = property_index_to_xmp(meta)?;

        let mut prop_flags = exempi2::PropFlags::default();
        let mut xmp_result =
            self.xmp_meta
                .xmp
                .get_property(index_to_xmp.ns, index_to_xmp.property, &mut prop_flags);
        if xmp_result.is_ok() && prop_flags.contains(exempi2::PropFlags::ARRAY_IS_ALTTEXT) {
            if let Ok((_, value)) = self.xmp_meta.xmp.get_localized_text(
                index_to_xmp.ns,
                index_to_xmp.property,
                "",
                "x-default",
                &mut prop_flags,
            ) {
                xmp_result = Ok(value);
            }
        }
        Some(PropertyValue::String(String::from(&xmp_result.ok()?)))
    }

    pub fn set_metadata(&mut self, meta: Np, value: &PropertyValue) -> bool {
        if let Some(ix) = property_index_to_xmp(meta) {
            match *value {
                PropertyValue::Empty => {
                    return self
                        .xmp_meta
                        .xmp
                        .delete_property(ix.ns, ix.property)
                        .is_ok();
                }
                PropertyValue::Int(i) => {
                    return self
                        .xmp_meta
                        .xmp
                        .set_property_i32(ix.ns, ix.property, i, exempi2::PropFlags::NONE)
                        .is_ok();
                }
                PropertyValue::String(ref s) => {
                    if s.is_empty() {
                        return self
                            .xmp_meta
                            .xmp
                            .delete_property(ix.ns, ix.property)
                            .is_ok();
                    } else if let Err(err) = self.xmp_meta.xmp.set_property(
                        ix.ns,
                        ix.property,
                        s,
                        exempi2::PropFlags::NONE,
                    ) {
                        if err == exempi2::Error(exempi2::XmpError::BadXPath) {
                            return self
                                .xmp_meta
                                .xmp
                                .set_localized_text(
                                    ix.ns,
                                    ix.property,
                                    "",
                                    "x-default",
                                    s,
                                    exempi2::PropFlags::NONE,
                                )
                                .is_ok();
                        }
                    } else {
                        return true;
                    }
                }
                PropertyValue::StringArray(ref sa) => {
                    if self
                        .xmp_meta
                        .xmp
                        .delete_property(ix.ns, ix.property)
                        .is_err()
                    {
                        err_out!("Error deleting property {}", &ix.property);
                        return false;
                    }
                    for s in sa {
                        if self
                            .xmp_meta
                            .xmp
                            .append_array_item(
                                ix.ns,
                                ix.property,
                                exempi2::PropFlags::VALUE_IS_ARRAY,
                                s,
                                exempi2::PropFlags::NONE,
                            )
                            .is_err()
                        {
                            err_out!("Error appending array item {} in property {}", &s, &ix.ns);
                            return false;
                        }
                    }
                    return true;
                }
                PropertyValue::Date(d) => {
                    return self
                        .xmp_meta
                        .xmp
                        .set_property_date(
                            ix.ns,
                            ix.property,
                            &d.into_xmpdate(),
                            exempi2::PropFlags::NONE,
                        )
                        .is_ok();
                }
            }
            err_out!("error setting property {}:{}", ix.ns, ix.property);
            return false;
        }
        err_out!("Unknown property {:?}", meta);
        false
    }

    pub fn to_properties(&self, propset: &PropertySet<Np>) -> NiepcePropertyBag {
        use super::NiepcePropertyIdx as Npi;
        let mut property_bag = NiepcePropertyBag::default();
        let props = &mut property_bag;
        for prop_id in propset {
            match *prop_id {
                Np::Index(Npi::NpXmpRatingProp) => {
                    if let Some(rating) = self.xmp_meta.rating() {
                        props.set_value(*prop_id, PropertyValue::Int(rating));
                    }
                }
                Np::Index(Npi::NpXmpLabelProp) => {
                    if let Some(label) = self.xmp_meta.label() {
                        props.set_value(*prop_id, PropertyValue::String(label));
                    }
                }
                Np::Index(Npi::NpTiffOrientationProp) => {
                    if let Some(orientation) = self.xmp_meta.orientation() {
                        props.set_value(*prop_id, PropertyValue::Int(orientation));
                    }
                }
                Np::Index(Npi::NpExifDateTimeOriginalProp) => {
                    if let Some(date) = self.xmp_meta.creation_date() {
                        props.set_value(*prop_id, PropertyValue::Date(date));
                    }
                }
                Np::Index(Npi::NpIptcKeywordsProp) => {
                    let iter = exempi2::XmpIterator::new(
                        &self.xmp_meta.xmp,
                        NS_DC,
                        "subject",
                        exempi2::IterFlags::JUST_LEAF_NODES,
                    );
                    let mut keywords: Vec<String> = vec![];
                    for v in iter {
                        keywords.push(String::from(&v.value));
                    }
                    props.set_value(*prop_id, PropertyValue::StringArray(keywords));
                }
                Np::Index(Npi::NpFileNameProp) => {
                    props.set_value(*prop_id, PropertyValue::String(self.name.clone()));
                }
                Np::Index(Npi::NpFileTypeProp) => {
                    let file_type: &str = self.file_type.into();
                    props.set_value(*prop_id, PropertyValue::String(String::from(file_type)));
                }
                Np::Index(Npi::NpFileSizeProp) => {}
                Np::Index(Npi::NpFolderProp) => {
                    props.set_value(*prop_id, PropertyValue::String(self.folder.clone()));
                }
                Np::Index(Npi::NpSidecarsProp) => {
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

    pub fn touch(&mut self) -> bool {
        let local = chrono::Local::now();
        let xmpdate = chrono::DateTime::from(local).into_xmpdate();
        self.xmp_meta
            .xmp
            .set_property_date(NS_XAP, "MetadataDate", &xmpdate, exempi2::PropFlags::NONE)
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

#[cfg(test)]
mod test {

    use super::{LibMetadata, Np};
    use crate::catalog::NiepcePropertyIdx as Npi;
    use chrono::TimeZone;
    use npc_fwk::{PropertySet, PropertyValue, XmpMeta};

    const XMP_PACKET: &[u8] = include_bytes!("../../tests/test.xmp");

    #[test]
    fn test_libmetadata() {
        let xmp = exempi2::Xmp::from_buffer(XMP_PACKET);
        assert!(xmp.is_ok());

        let xmp = xmp.unwrap();
        let xmp_meta = XmpMeta::from(xmp);
        let libmetadata = LibMetadata::new_with_xmp(1, xmp_meta);
        let mut propset = PropertySet::new();
        propset.insert(Np::Index(Npi::NpIptcKeywordsProp));
        propset.insert(Np::Index(Npi::NpTiffOrientationProp));
        propset.insert(Np::Index(Npi::NpExifDateTimeOriginalProp));

        let bag = libmetadata.to_properties(&propset);
        assert_eq!(bag.len(), 3);

        let keywords = bag.get(&Np::Index(Npi::NpIptcKeywordsProp));
        assert!(keywords.is_some());

        if let PropertyValue::StringArray(keywords) = keywords.unwrap() {
            assert_eq!(keywords.len(), 5);
            assert_eq!(keywords[0], "choir");
            assert_eq!(keywords[1], "night");
            assert_eq!(keywords[2], "ontario");
            assert_eq!(keywords[3], "ottawa");
            assert_eq!(keywords[4], "parliament of canada");
        } else {
            unreachable!();
        }

        let orientation = bag.get(&Np::Index(Npi::NpTiffOrientationProp));
        assert!(orientation.is_some());

        if let PropertyValue::Int(orientation) = orientation.unwrap() {
            assert_eq!(orientation, &1);
        } else {
            unreachable!();
        }

        let creation_date = bag.get(&Np::Index(Npi::NpExifDateTimeOriginalProp));
        assert!(creation_date.is_some());

        if let PropertyValue::Date(creation_date) = creation_date.unwrap() {
            let date = chrono::FixedOffset::west_opt(5 * 3600)
                .and_then(|tz| tz.with_ymd_and_hms(2006, 12, 7, 23, 37, 30).single())
                .unwrap();
            assert_eq!(creation_date, &date);
        } else {
            unreachable!();
        }
    }
}
