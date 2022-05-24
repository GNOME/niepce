/*
 * niepce - fwk/utils/exempi.rs
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

use std::convert::From;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use chrono::{DateTime, Utc};
use exempi::Xmp;

use super::exiv2;

pub const NIEPCE_XMP_NAMESPACE: &str = "http://xmlns.figuiere.net/ns/niepce/1.0";
pub const NIEPCE_XMP_NS_PREFIX: &str = "niepce";
const UFRAW_INTEROP_NAMESPACE: &str = "http://xmlns.figuiere.net/ns/ufraw_interop/1.0";
const UFRAW_INTEROP_NS_PREFIX: &str = "ufrint";

pub const NS_TIFF: &str = "http://ns.adobe.com/tiff/1.0/";
pub const NS_XAP: &str = "http://ns.adobe.com/xap/1.0/";
pub const NS_EXIF: &str = "http://ns.adobe.com/exif/1.0/";
pub const NS_EXIF_EX: &str = "http://cipa.jp/exif/1.0/";
pub const NS_DC: &str = "http://purl.org/dc/elements/1.1/";
pub const NS_AUX: &str = "http://ns.adobe.com/exif/1.0/aux/";
pub const NS_PHOTOSHOP: &str = "http://ns.adobe.com/photoshop/1.0/";

const XMP_TRUE: &str = "True";
const XMP_FALSE: &str = "False";

/// Convert a bool to a propstring
fn bool_to_propstring(val: bool) -> &'static str {
    if val {
        XMP_TRUE
    } else {
        XMP_FALSE
    }
}

/// The Flash property, decoded
#[derive(Clone, Default, Debug)]
pub struct Flash {
    fired: bool,
    rturn: u8,
    mode: u8,
    function: bool,
    red_eye: bool,
}

impl From<i32> for Flash {
    /// Interpret the exif value and make it a Flash struct
    fn from(flash: i32) -> Flash {
        let fired = (flash & 0x1) != 0;
        let rturn = ((flash >> 1) & 0x3) as u8;
        let mode = ((flash >> 3) & 0x3) as u8;
        let function = ((flash >> 5) & 0x1) != 0;
        let red_eye = ((flash >> 6) & 0x10) != 0;
        Flash {
            fired,
            rturn,
            mode,
            function,
            red_eye,
        }
    }
}

impl Flash {
    pub fn set_as_xmp_property(
        &self,
        xmp: &mut Xmp,
        ns: &str,
        property: &str,
    ) -> exempi::Result<()> {
        // XXX use set_struct_field() as soon as it is available
        xmp.set_property(
            ns,
            &format!("{}/exif:Fired", property),
            bool_to_propstring(self.fired),
            exempi::PropFlags::NONE,
        )?;
        xmp.set_property(
            ns,
            &format!("{}/exif:Return", property),
            &format!("{}", self.rturn),
            exempi::PropFlags::NONE,
        )?;
        xmp.set_property(
            ns,
            &format!("{}/exif:Mode", property),
            &format!("{}", self.mode),
            exempi::PropFlags::NONE,
        )?;
        xmp.set_property(
            ns,
            &format!("{}/exif:Function", property),
            bool_to_propstring(self.function),
            exempi::PropFlags::NONE,
        )?;
        xmp.set_property(
            ns,
            &format!("{}/exif:RedEyeMode", property),
            bool_to_propstring(self.red_eye),
            exempi::PropFlags::NONE,
        )?;

        Ok(())
    }
}

pub struct NsDef {
    ns: String,
    prefix: String,
}

pub struct ExempiManager {}

impl ExempiManager {
    pub fn new(namespaces: Option<Vec<NsDef>>) -> ExempiManager {
        if exempi::init() {
            exempi::register_namespace(NIEPCE_XMP_NAMESPACE, NIEPCE_XMP_NS_PREFIX);
            exempi::register_namespace(UFRAW_INTEROP_NAMESPACE, UFRAW_INTEROP_NS_PREFIX);
        }
        if let Some(nslist) = namespaces {
            for nsdef in nslist {
                // TOOD check the return value
                exempi::register_namespace(nsdef.ns.as_str(), nsdef.prefix.as_str());
            }
        }
        ExempiManager {}
    }
}

#[derive(Clone)]
pub struct XmpMeta {
    pub xmp: Xmp,
    keywords: Vec<String>,
    keywords_fetched: bool,
}

impl Default for XmpMeta {
    fn default() -> XmpMeta {
        XmpMeta::new()
    }
}

impl XmpMeta {
    pub fn new() -> XmpMeta {
        XmpMeta {
            xmp: exempi::Xmp::new(),
            keywords: Vec::new(),
            keywords_fetched: false,
        }
    }

    pub fn new_with_xmp(xmp: exempi::Xmp) -> XmpMeta {
        XmpMeta {
            xmp,
            keywords: Vec::new(),
            keywords_fetched: false,
        }
    }

    pub fn new_from_file<P>(p: P, sidecar_only: bool) -> Option<XmpMeta>
    where
        P: AsRef<Path> + AsRef<OsStr>,
    {
        let file: &Path = p.as_ref();
        let mut meta: Option<XmpMeta> = None;
        if !sidecar_only {
            if let Ok(xmpfile) =
                exempi::XmpFile::open_new(&*file.to_string_lossy(), exempi::OpenFlags::READ)
            {
                meta = match xmpfile.get_new_xmp() {
                    Ok(xmp) => Some(Self::new_with_xmp(xmp)),
                    _ => exiv2::xmp_from_exiv2(file),
                }
            }
        }

        let mut sidecar_meta: Option<XmpMeta> = None;
        let sidecar = file.with_extension("xmp");
        let sidecaropen = File::open(sidecar);
        if let Ok(mut sidecarfile) = sidecaropen {
            let mut sidecarcontent = String::new();
            if sidecarfile.read_to_string(&mut sidecarcontent).is_ok() {
                let mut xmp = exempi::Xmp::new();
                if xmp.parse(sidecarcontent.into_bytes().as_slice()).is_ok() {
                    sidecar_meta = Some(Self::new_with_xmp(xmp));
                }
            }
        }
        if meta.is_none() || sidecar_meta.is_none() {
            if meta.is_some() {
                return meta;
            }
            sidecar_meta
        } else if let Some(mut final_meta) = sidecar_meta {
            if !meta
                .as_ref()
                .unwrap()
                .merge_missing_into_xmp(&mut final_meta)
            {
                err_out!("xmp merge failed");
                // XXX with the current heuristics, it is probably safe to just
                // keep the source metadata.
                meta
            } else {
                Some(final_meta)
            }
        } else {
            unreachable!("sidecar_meta was None");
        }
    }

    ///
    /// Merge missing properties from self (source) to destination
    /// struct and array are considerd missing as a whole, not their content.
    ///
    pub fn merge_missing_into_xmp(&self, dest: &mut XmpMeta) -> bool {
        // Merge XMP self into the dest that has more recent.
        let source_date = self.get_date_property(NS_XAP, "MetadataDate");
        let dest_date = dest.get_date_property(NS_XAP, "MetadataDate");

        if source_date.is_none() || dest_date.is_none() {
            dbg_out!(
                "missing metadata date {} {}",
                source_date.is_some(),
                dest_date.is_some()
            );
            return false;
        }
        if source_date > dest_date {
            dbg_out!("source meta is more recent than sidecar");
            return false;
        }

        // Properties in source but not in destination gets copied over.
        let mut iter = exempi::XmpIterator::new(&self.xmp, "", "", exempi::IterFlags::PROPERTIES);
        {
            use exempi::XmpString;
            let mut schema = XmpString::new();
            let mut name = XmpString::new();
            let mut value = XmpString::new();
            let mut option = exempi::PropFlags::default();
            while iter.next(&mut schema, &mut name, &mut value, &mut option) {
                if name.to_str().is_empty() {
                    continue;
                }
                if option.contains(exempi::PropFlags::VALUE_IS_ARRAY)
                    || option.contains(exempi::PropFlags::VALUE_IS_STRUCT)
                {
                    iter.skip(exempi::IterSkipFlags::SUBTREE);
                    continue;
                }

                if !dest.xmp.has_property(schema.to_str(), name.to_str())
                    && dest
                        .xmp
                        .set_property(
                            schema.to_str(),
                            name.to_str(),
                            value.to_str(),
                            exempi::PropFlags::NONE,
                        )
                        .is_err()
                {
                    err_out!("Can set property {}", name);
                }
            }
        }

        true
    }

    pub fn serialize_inline(&self) -> String {
        if let Ok(xmpstr) = self.xmp.serialize_and_format(
            exempi::SerialFlags::OMITPACKETWRAPPER | exempi::SerialFlags::OMITALLFORMATTING,
            0,
            "",
            "",
            0,
        ) {
            let buf = String::from(xmpstr.to_str());
            return buf;
        }
        String::new()
    }

    pub fn serialize(&self) -> String {
        if let Ok(xmpstr) =
            self.xmp
                .serialize_and_format(exempi::SerialFlags::OMITPACKETWRAPPER, 0, "\n", "", 0)
        {
            let buf = String::from(xmpstr.to_str());
            return buf;
        }
        String::new()
    }

    pub fn unserialize(&mut self, buf: &str) -> bool {
        self.xmp.parse(buf.as_bytes()).is_ok() // XXX actually report the error.
    }

    pub fn orientation(&self) -> Option<i32> {
        let mut flags: exempi::PropFlags = exempi::PropFlags::default();
        self.xmp
            .get_property_i32(NS_TIFF, "Orientation", &mut flags)
            .ok()
    }

    pub fn label(&self) -> Option<String> {
        let mut flags: exempi::PropFlags = exempi::PropFlags::default();
        let xmpstring = self.xmp.get_property(NS_XAP, "Label", &mut flags).ok()?;
        Some(String::from(xmpstring.to_str()))
    }

    pub fn rating(&self) -> Option<i32> {
        let mut flags: exempi::PropFlags = exempi::PropFlags::default();
        self.xmp.get_property_i32(NS_XAP, "Rating", &mut flags).ok()
    }

    pub fn flag(&self) -> Option<i32> {
        let mut flags: exempi::PropFlags = exempi::PropFlags::empty();
        self.xmp
            .get_property_i32(NIEPCE_XMP_NAMESPACE, "Flag", &mut flags)
            .ok()
    }

    pub fn creation_date(&self) -> Option<DateTime<Utc>> {
        let mut flags: exempi::PropFlags = exempi::PropFlags::default();
        let xmpstring = self
            .xmp
            .get_property(NS_EXIF, "DateTimeOriginal", &mut flags)
            .ok()?;
        let date = DateTime::parse_from_rfc3339(xmpstring.to_str()).ok()?;

        Some(date.with_timezone(&Utc))
    }

    pub fn creation_date_str(&self) -> Option<String> {
        let mut flags: exempi::PropFlags = exempi::PropFlags::empty();
        let xmpstring = self
            .xmp
            .get_property(NS_EXIF, "DateTimeOriginal", &mut flags)
            .ok()?;
        Some(String::from(xmpstring.to_str()))
    }

    /// Get the date property and return a Option<DateTime<Utc>> parsed
    /// from the string value.
    pub fn get_date_property(&self, ns: &str, property: &str) -> Option<DateTime<Utc>> {
        let mut flags: exempi::PropFlags = exempi::PropFlags::default();
        let property = self.xmp.get_property(ns, property, &mut flags);
        if property.is_err() {
            err_out!("Error getting date property {:?}", property.err());
            return None;
        }
        let xmpstring = property.unwrap();
        let parsed = DateTime::parse_from_rfc3339(xmpstring.to_str());
        if parsed.is_err() {
            err_out!(
                "Error parsing property value '{}': {:?}",
                xmpstring,
                parsed.err()
            );
            return None;
        }
        let date = parsed.unwrap();
        Some(date.with_timezone(&Utc))
    }

    pub fn keywords(&mut self) -> &Vec<String> {
        if !self.keywords_fetched {
            use exempi::XmpString;

            let mut iter = exempi::XmpIterator::new(
                &self.xmp,
                NS_DC,
                "subject",
                exempi::IterFlags::JUST_LEAF_NODES,
            );
            let mut schema = XmpString::new();
            let mut name = XmpString::new();
            let mut value = XmpString::new();
            let mut option = exempi::PropFlags::default();
            while iter.next(&mut schema, &mut name, &mut value, &mut option) {
                self.keywords.push(String::from(value.to_str()));
            }
            self.keywords_fetched = true;
        }
        &self.keywords
    }
}

pub fn gps_coord_from_xmp(xmps: &str) -> Option<f64> {
    let mut current: &str = xmps;

    // step 1 - degrees
    let sep = current.find(',')?;
    let (d, remainder) = current.split_at(sep);
    current = remainder;
    let degs = d;

    // step 2 - minutes
    if current.is_empty() {
        return None;
    }
    // get rid of the comma
    let (_, current) = current.split_at(1);
    let orientation = match current.chars().last() {
        Some('N') | Some('E') => 1.0f64,
        Some('S') | Some('W') => -1.0f64,
        _ => return None,
    };

    // extract minutes. There are two formats
    let fminutes = if let Some(sep) = current.find(',') {
        // DD,mm,ss format
        if sep >= (current.len() - 1) {
            return None;
        }
        if current.len() <= 2 {
            return None;
        }
        let (minutes, seconds) = current.split_at(sep);
        let (_, seconds) = seconds.split_at(1);
        let (seconds, _) = seconds.split_at(seconds.len() - 1);
        let m = minutes.parse::<f64>().ok()?;
        let s = seconds.parse::<f64>().ok()?;
        m + (s / 60_f64)
    } else {
        // DD,mm.mm format
        let (minutes, _) = current.split_at(current.len() - 1);
        minutes.parse::<f64>().ok()?
    };

    let mut deg = degs.parse::<f64>().ok()?;
    if deg > 180.0 {
        return None;
    }
    deg += fminutes / 60.0;
    deg *= orientation;

    Some(deg)
}

/// Get and XMP date from an Exif date string
/// XXX Currently assume it is UTC.
pub fn xmp_date_from_exif(d: &str) -> Option<exempi::DateTime> {
    let v: Vec<&str> = d.split(' ').collect();
    if v.len() != 2 {
        err_out!("Space split failed {:?}", v);
        return None;
    }

    let ymd: Vec<&str> = v[0].split(':').collect();
    if ymd.len() != 3 {
        err_out!("ymd split failed {:?}", ymd);
        return None;
    }
    let year = ymd[0].parse::<i32>().ok()?;
    let month = ymd[1].parse::<i32>().ok()?;
    if !(1..=12).contains(&month) {
        return None;
    }
    let day = ymd[2].parse::<i32>().ok()?;
    if !(1..=31).contains(&day) {
        return None;
    }
    let hms: Vec<&str> = v[1].split(':').collect();
    if hms.len() != 3 {
        err_out!("hms split failed {:?}", hms);
        return None;
    }
    let hour = hms[0].parse::<i32>().ok()?;
    if !(0..=23).contains(&hour) {
        return None;
    }
    let min = hms[1].parse::<i32>().ok()?;
    if !(0..=59).contains(&min) {
        return None;
    }
    let sec = hms[2].parse::<i32>().ok()?;
    if !(0..=59).contains(&sec) {
        return None;
    }

    let mut xmp_date = exempi::DateTime::new();

    xmp_date.set_date(year, month, day);
    xmp_date.set_time(hour, min, sec);
    // XXX use an actual timezone
    xmp_date.set_timezone(exempi::XmpTzSign::UTC, 0, 0);

    Some(xmp_date)
}

#[no_mangle]
pub extern "C" fn fwk_exempi_manager_new() -> *mut ExempiManager {
    Box::into_raw(Box::new(ExempiManager::new(None)))
}

/// Delete the ExempiManager
///
/// # Safety
/// Dereference the pointer.
#[no_mangle]
pub unsafe extern "C" fn fwk_exempi_manager_delete(em: *mut ExempiManager) {
    Box::from_raw(em);
}

#[cfg(test)]
mod tests {
    use super::xmp_date_from_exif;
    use super::ExempiManager;
    use super::XmpMeta;
    use exempi;
    use std::path::PathBuf;

    fn get_xmp_sample_path() -> PathBuf {
        use std::env;

        let mut dir: PathBuf;
        if let Ok(pdir) = env::var("CARGO_MANIFEST_DIR") {
            dir = PathBuf::from(pdir);
            dir.push("src");
            dir.push("utils");
        } else {
            dir = PathBuf::from(".");
        }
        dir
    }

    #[test]
    fn xmp_meta_works() {
        let mut dir = get_xmp_sample_path();
        dir.push("test.xmp");
        let _xmp_manager = ExempiManager::new(None);

        if let Some(xmpfile) = dir.to_str() {
            let meta = XmpMeta::new_from_file(xmpfile, true);

            assert!(meta.is_some());
            let mut meta = meta.unwrap();
            assert_eq!(meta.orientation().unwrap_or(0), 1);
            // test keywords()
            let keywords = meta.keywords();
            assert_eq!(keywords.len(), 5);
            assert_eq!(keywords[0], "choir");
            assert_eq!(keywords[1], "night");
            assert_eq!(keywords[2], "ontario");
            assert_eq!(keywords[3], "ottawa");
            assert_eq!(keywords[4], "parliament of canada");
        } else {
            assert!(false);
        }
    }

    fn test_property_value(meta: &XmpMeta, ns: &str, property: &str, expected_value: &str) {
        let mut flags: exempi::PropFlags = exempi::PropFlags::empty();
        let value = meta.xmp.get_property(ns, property, &mut flags);
        assert!(value.is_ok());
        assert_eq!(value.unwrap().to_str(), expected_value);
    }

    fn test_property_array_value(
        meta: &XmpMeta,
        ns: &str,
        property: &str,
        idx: i32,
        expected_value: &str,
    ) {
        let mut flags: exempi::PropFlags = exempi::PropFlags::empty();
        let value = meta.xmp.get_array_item(ns, property, idx, &mut flags);
        assert!(value.is_ok());
        assert_eq!(value.unwrap().to_str(), expected_value);
    }

    #[test]
    fn test_merge_missing_into_xmp() {
        let dir = get_xmp_sample_path();

        // Both these files have to exist. They are on the source
        // tree.
        let mut source = dir.clone();
        source.push("test.xmp");

        let mut dest = dir.clone();
        dest.push("test2.xmp");
        let _xmp_manager = ExempiManager::new(None);

        if let Some(xmpfile) = source.to_str() {
            let meta = XmpMeta::new_from_file(xmpfile, true);
            assert!(meta.is_some());
            let meta = meta.unwrap();

            if let Some(xmpfile) = dest.to_str() {
                let dstmeta = XmpMeta::new_from_file(xmpfile, true);
                assert!(dstmeta.is_some());
                let mut dstmeta = dstmeta.unwrap();

                let result = meta.merge_missing_into_xmp(&mut dstmeta);
                assert!(result);
                // properties that were missing
                test_property_value(&dstmeta, super::NS_TIFF, "Model", "Canon EOS 20D");
                test_property_value(&dstmeta, super::NS_AUX, "Lens", "24.0-85.0 mm");

                // Array property that contain less in destination
                // Shouldn't have changed.
                test_property_array_value(&dstmeta, super::NS_DC, "subject", 1, "night");
                test_property_array_value(&dstmeta, super::NS_DC, "subject", 2, "ontario");
                test_property_array_value(&dstmeta, super::NS_DC, "subject", 3, "ottawa");
                test_property_array_value(
                    &dstmeta,
                    super::NS_DC,
                    "subject",
                    4,
                    "parliament of canada",
                );
                assert!(!dstmeta.xmp.has_property(super::NS_DC, "dc:subject[5]"));
            }
        } else {
            assert!(false);
        }
    }

    #[test]
    fn gps_coord_from_works() {
        use super::gps_coord_from_xmp;

        let mut output = gps_coord_from_xmp("foobar");
        assert!(output.is_none());

        // malformed 1
        output = gps_coord_from_xmp("45,29.6681666667");
        assert!(output.is_none());

        // malformed 2
        output = gps_coord_from_xmp("45,W");
        assert!(output.is_none());

        // malformed 3
        output = gps_coord_from_xmp("45,29,N");
        assert!(output.is_none());

        // out of bounds
        output = gps_coord_from_xmp("200,29.6681666667N");
        assert!(output.is_none());

        // well-formed 1
        output = gps_coord_from_xmp("45,29.6681666667N");
        assert!(output.is_some());
        assert_eq!(
            output.unwrap(),
            45.494469444445002181964810006320476531982421875
        );

        // well-formed 2
        output = gps_coord_from_xmp("73,38.2871666667W");
        assert!(output.is_some());
        assert_eq!(
            output.unwrap(),
            -73.6381194444449960201382054947316646575927734375
        );

        // well-formed 3
        output = gps_coord_from_xmp("45,29,30.45N");
        assert!(output.is_some());
        assert_eq!(
            output.unwrap(),
            45.49179166666666418450404307805001735687255859375
        );
    }

    #[test]
    fn test_xmp_date_from_exif() {
        let d = xmp_date_from_exif("2012:02:17 11:10:49");
        assert!(d.is_some());
        let d = d.unwrap();
        assert_eq!(d.year(), 2012);
        assert_eq!(d.month(), 02);
        assert_eq!(d.day(), 17);
        assert_eq!(d.hour(), 11);
        assert_eq!(d.minute(), 10);
        assert_eq!(d.second(), 49);

        let d = xmp_date_from_exif("2012:02:17/11:10:49");
        assert!(d.is_none());

        let d = xmp_date_from_exif("2012.02.17 11.10.49");
        assert!(d.is_none());
    }
}
