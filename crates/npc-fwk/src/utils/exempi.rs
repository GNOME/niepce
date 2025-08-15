/*
 * niepce - fwk/utils/exempi.rs
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

use std::convert::From;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use anyhow::Context;
use exempi2::Xmp;

use super::exiv2;
use crate::toolkit::heif;
use crate::{Date, DateExt};

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
    if val { XMP_TRUE } else { XMP_FALSE }
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
    ) -> exempi2::Result<()> {
        // XXX use set_struct_field() as soon as it is available
        xmp.set_property(
            ns,
            &format!("{property}/exif:Fired"),
            bool_to_propstring(self.fired),
            exempi2::PropFlags::NONE,
        )?;
        xmp.set_property(
            ns,
            &format!("{property}/exif:Return"),
            &format!("{}", self.rturn),
            exempi2::PropFlags::NONE,
        )?;
        xmp.set_property(
            ns,
            &format!("{property}/exif:Mode"),
            &format!("{}", self.mode),
            exempi2::PropFlags::NONE,
        )?;
        xmp.set_property(
            ns,
            &format!("{property}/exif:Function"),
            bool_to_propstring(self.function),
            exempi2::PropFlags::NONE,
        )?;
        xmp.set_property(
            ns,
            &format!("{property}/exif:RedEyeMode"),
            bool_to_propstring(self.red_eye),
            exempi2::PropFlags::NONE,
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
        on_err_out!(exempi2::register_namespace(
            NIEPCE_XMP_NAMESPACE,
            NIEPCE_XMP_NS_PREFIX
        ));
        on_err_out!(exempi2::register_namespace(
            UFRAW_INTEROP_NAMESPACE,
            UFRAW_INTEROP_NS_PREFIX
        ));

        if let Some(nslist) = namespaces {
            for nsdef in nslist {
                on_err_out!(exempi2::register_namespace(
                    nsdef.ns.as_str(),
                    nsdef.prefix.as_str()
                ));
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

impl std::fmt::Debug for XmpMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("XmpMeta")
            .field("xmp", &format!("{:?}", self.xmp.as_ptr()))
            .field("keywords", &self.keywords)
            .field("keywords_fetched", &self.keywords_fetched)
            .finish()
    }
}

impl Default for XmpMeta {
    fn default() -> XmpMeta {
        XmpMeta::new()
    }
}

impl From<exempi2::Xmp> for XmpMeta {
    fn from(xmp: exempi2::Xmp) -> XmpMeta {
        XmpMeta {
            xmp,
            keywords: Vec::new(),
            keywords_fetched: false,
        }
    }
}

impl XmpMeta {
    pub fn new() -> XmpMeta {
        XmpMeta {
            xmp: exempi2::Xmp::new(),
            keywords: Vec::new(),
            keywords_fetched: false,
        }
    }

    /// Create a new XMP meta for file. If `sidecar_only` is true then only
    /// the XMP sidecar is loaded. Otherwise it use XMP to load, and fallback
    /// with Exiv2.
    ///
    // XXX change this to return a `Result`.
    pub fn new_from_file<P>(p: P, sidecar_only: bool) -> Option<XmpMeta>
    where
        P: AsRef<Path> + AsRef<OsStr>,
    {
        let file: &Path = p.as_ref();
        if !file.exists() {
            // XXX return an error.
            return None;
        }
        let mut meta: Option<XmpMeta> = None;
        if !sidecar_only {
            let is_raw = file
                .extension()
                .map(|ext| {
                    // Note: the extension should be ASCII. We should be safe here.
                    // libopenraw extensions are lowercase.
                    libopenraw::extensions()
                        .iter()
                        .any(|e| OsStr::new(e) == ext.to_ascii_lowercase())
                })
                .unwrap_or(false);
            meta = if is_raw {
                exiv2::xmp_from_exiv2(file)
            } else if heif::is_heif(file) {
                // HEIF is a special case mostly because on Fedora Exiv2 is built
                // without support for it. Since we have `libheif` we can extract
                // the Exif blob and deal with it directly.
                dbg_out!("HEIF exiv2");
                let exif = heif::get_exif(&file.to_string_lossy())
                    .ok()
                    .as_deref()
                    .and_then(exiv2::xmp_from_exif);
                let xmp = heif::get_xmp(&file.to_string_lossy())
                    .ok()
                    .and_then(|buf| exempi2::Xmp::from_buffer(buf).ok())
                    .map(XmpMeta::from);
                exif.map(|mut exif| {
                    if let Some(xmp) = &xmp {
                        xmp.merge_missing_into_xmp(&mut exif);
                    }
                    exif
                })
                .or(xmp)
            } else if let Ok(xmpfile) = {
                dbg_out!("Opening XMP for {file:?}");
                exempi2::XmpFile::new_from_file(file, exempi2::OpenFlags::READ)
            } {
                match xmpfile.get_new_xmp() {
                    Ok(xmp) => Some(Self::from(xmp)),
                    Err(err) => {
                        err_out!("Failed to get XMP from {file:?}: {err:?}");
                        exiv2::xmp_from_exiv2(file)
                    }
                }
            } else {
                dbg_out!("No XMP found");
                None
            };
        }

        let mut sidecar_meta: Option<XmpMeta> = None;
        let sidecar = file.with_extension("xmp");
        if let Ok(mut sidecarfile) = File::open(sidecar) {
            let mut sidecarcontent = String::new();
            if sidecarfile.read_to_string(&mut sidecarcontent).is_ok() {
                let mut xmp = exempi2::Xmp::new();
                if xmp.parse(sidecarcontent.into_bytes().as_slice()).is_ok() {
                    sidecar_meta = Some(Self::from(xmp));
                }
            }
        }
        #[allow(clippy::unnecessary_unwrap)]
        // XXX we can revise the logic to avoid the clippy warning.
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
            dbg_out!("source meta is more recent than sidecar {source_date:?} > {dest_date:?}");
            return false;
        }

        // Properties in source but not in destination gets copied over.
        let mut iter = exempi2::XmpIterator::new(&self.xmp, "", "", exempi2::IterFlags::PROPERTIES);
        while let Some(v) = iter.next() {
            if v.name.is_empty() {
                continue;
            }
            if v.option.contains(exempi2::PropFlags::VALUE_IS_ARRAY)
                || v.option.contains(exempi2::PropFlags::VALUE_IS_STRUCT)
            {
                exempi2::XmpIterator::skip(&mut iter, exempi2::IterSkipFlags::SUBTREE);
                continue;
            }

            let schema = v.schema.to_str().unwrap_or("");
            let name = v.name.to_str().unwrap_or("");
            if !dest.xmp.has_property(schema, name) {
                let value = v.value.to_str().unwrap_or("");
                if dest
                    .xmp
                    .set_property(schema, name, value, exempi2::PropFlags::NONE)
                    .is_err()
                {
                    err_out!("Can not set property {}", v.name);
                }
            }
        }

        true
    }

    pub fn serialize_inline(&self) -> String {
        if let Ok(xmpstr) = self.xmp.serialize_and_format(
            exempi2::SerialFlags::OMITPACKETWRAPPER | exempi2::SerialFlags::OMITALLFORMATTING,
            0,
            "",
            "",
            0,
        ) {
            let buf = String::from(&xmpstr);
            return buf;
        }
        String::new()
    }

    pub fn serialize(&self) -> String {
        if let Ok(xmpstr) =
            self.xmp
                .serialize_and_format(exempi2::SerialFlags::OMITPACKETWRAPPER, 0, "\n", "", 0)
        {
            let buf = String::from(&xmpstr);
            return buf;
        }
        String::new()
    }

    pub fn unserialize(&mut self, buf: &str) -> bool {
        self.xmp.parse(buf.as_bytes()).is_ok() // XXX actually report the error.
    }

    pub fn set_orientation(&mut self, orientation: i32) -> anyhow::Result<()> {
        self.xmp
            .set_property_i32(
                NS_TIFF,
                "Orientation",
                orientation,
                exempi2::PropFlags::default(),
            )
            .context("Failed to set Orientation property")
    }

    pub fn orientation(&self) -> Option<i32> {
        let mut flags: exempi2::PropFlags = exempi2::PropFlags::default();
        self.xmp
            .get_property_i32(NS_TIFF, "Orientation", &mut flags)
            .ok()
    }

    pub fn label(&self) -> Option<String> {
        let mut flags: exempi2::PropFlags = exempi2::PropFlags::default();
        let xmpstring = self.xmp.get_property(NS_XAP, "Label", &mut flags).ok()?;
        Some(String::from(&xmpstring))
    }

    pub fn rating(&self) -> Option<i32> {
        let mut flags: exempi2::PropFlags = exempi2::PropFlags::default();
        self.xmp.get_property_i32(NS_XAP, "Rating", &mut flags).ok()
    }

    pub fn flag(&self) -> Option<i32> {
        let mut flags: exempi2::PropFlags = exempi2::PropFlags::empty();
        self.xmp
            .get_property_i32(NIEPCE_XMP_NAMESPACE, "Flag", &mut flags)
            .ok()
    }

    /// Get the creation date. In order, Exif `DateTimeOriginal`, and
    /// then XMP `CreateDate`.
    pub fn creation_date(&self) -> Option<Date> {
        let mut flags: exempi2::PropFlags = exempi2::PropFlags::default();
        let date = self
            .xmp
            .get_property_date(NS_EXIF, "DateTimeOriginal", &mut flags)
            .or_else(|_| {
                let mut flags: exempi2::PropFlags = exempi2::PropFlags::default();
                self.xmp.get_property_date(NS_XAP, "CreateDate", &mut flags)
            })
            .ok()?;

        Some(Date::from_exempi(&date))
    }

    /// Same as `creation_date()` but the original string is returned instead.
    pub fn creation_date_str(&self) -> Option<String> {
        let mut flags: exempi2::PropFlags = exempi2::PropFlags::empty();
        let xmpstring = self
            .xmp
            .get_property(NS_EXIF, "DateTimeOriginal", &mut flags)
            .or_else(|_| {
                let mut flags: exempi2::PropFlags = exempi2::PropFlags::default();
                self.xmp.get_property(NS_XAP, "CreateDate", &mut flags)
            })
            .ok()?;
        Some(String::from(&xmpstring))
    }

    /// Get the date property and return an `Option<DateTime<Utc>>`.
    /// Uses XMP to parse the date.
    pub fn get_date_property(&self, ns: &str, propname: &str) -> Option<Date> {
        let mut flags: exempi2::PropFlags = exempi2::PropFlags::default();
        let property = self.xmp.get_property_date(ns, propname, &mut flags);
        if let Err(err) = property {
            err_out!("Error getting date property {propname} {err:?}");
            return None;
        }
        Some(Date::from_exempi(property.as_ref().unwrap()))
    }

    pub fn keywords(&mut self) -> &Vec<String> {
        if !self.keywords_fetched {
            let iter = exempi2::XmpIterator::new(
                &self.xmp,
                NS_DC,
                "subject",
                exempi2::IterFlags::JUST_LEAF_NODES,
            );
            for v in iter {
                self.keywords.push(String::from(&v.value));
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
pub fn xmp_date_from_exif(d: &str, offset: Option<&str>) -> Option<exempi2::DateTime> {
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

    let mut xmp_date = exempi2::DateTime::new();

    xmp_date.set_date(year, month, day);
    xmp_date.set_time(hour, min, sec);

    let offset = offset
        .and_then(|offset| {
            if offset.len() < 6 {
                return None;
            }
            let sign = match offset.chars().next() {
                Some('-') => exempi2::TzSign::West,
                Some('+') => exempi2::TzSign::East,
                Some(' ') => exempi2::TzSign::UTC,
                _ => return None,
            };
            let v: Vec<&str> = offset[1..].split(':').collect();
            if v.len() != 2 {
                return None;
            }
            let h = v[0].parse::<i32>().ok()?;
            let m = v[1].parse::<i32>().ok()?;
            Some((sign, h, m))
        })
        .unwrap_or((exempi2::TzSign::UTC, 0, 0));
    xmp_date.set_timezone(offset.0, offset.1, offset.2);

    Some(xmp_date)
}

#[cfg(test)]
mod tests {
    use super::ExempiManager;
    use super::xmp_date_from_exif;
    use super::{NS_EXIF, XmpMeta};
    use exempi2;
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
            unreachable!();
        }
    }

    fn test_property_value(meta: &XmpMeta, ns: &str, property: &str, expected_value: &str) {
        let mut flags: exempi2::PropFlags = exempi2::PropFlags::empty();
        let value = meta.xmp.get_property(ns, property, &mut flags);
        assert!(value.is_ok());
        assert_eq!(value.unwrap().to_str(), Ok(expected_value));
    }

    fn test_property_array_value(
        meta: &XmpMeta,
        ns: &str,
        property: &str,
        idx: i32,
        expected_value: &str,
    ) {
        let mut flags: exempi2::PropFlags = exempi2::PropFlags::empty();
        let value = meta.xmp.get_array_item(ns, property, idx, &mut flags);
        assert!(value.is_ok());
        assert_eq!(value.unwrap().to_str(), Ok(expected_value));
    }

    #[test]
    fn test_merge_missing_into_xmp() {
        let dir = get_xmp_sample_path();

        // Both these files have to exist. They are on the source
        // tree.
        let mut source = dir.clone();
        source.push("test.xmp");

        let mut dest = dir;
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
            unreachable!();
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
        assert_eq!(output.unwrap(), 45.494_469_444_445);

        // well-formed 2
        output = gps_coord_from_xmp("73,38.2871666667W");
        assert!(output.is_some());
        assert_eq!(output.unwrap(), -73.638_119_444_445);

        // well-formed 3
        output = gps_coord_from_xmp("45,29,30.45N");
        assert!(output.is_some());
        assert_eq!(output.unwrap(), 45.491_791_666_666_664);
    }

    #[test]
    fn test_xmp_date_from_exif() {
        let d = xmp_date_from_exif("2012:02:17 11:10:49", None);
        assert!(d.is_some());
        let d = d.unwrap();
        assert_eq!(d.year(), 2012);
        assert_eq!(d.month(), 2);
        assert_eq!(d.day(), 17);
        assert_eq!(d.hour(), 11);
        assert_eq!(d.minute(), 10);
        assert_eq!(d.second(), 49);
        assert_eq!(d.tz_sign(), exempi2::TzSign::UTC);

        let d = xmp_date_from_exif("2012:02:17 11:10:49", Some("-04:00"));
        assert!(d.is_some());
        let d = d.unwrap();
        assert_eq!(d.year(), 2012);
        assert_eq!(d.month(), 2);
        assert_eq!(d.day(), 17);
        assert_eq!(d.hour(), 11);
        assert_eq!(d.minute(), 10);
        assert_eq!(d.second(), 49);
        assert_eq!(d.tz_sign(), exempi2::TzSign::West);
        assert_eq!(d.tz_hours(), 4);
        assert_eq!(d.tz_minutes(), 0);

        let mut xmp = exempi2::Xmp::new();
        let r = xmp.set_property_date(NS_EXIF, "DateTimeOriginal", &d, exempi2::PropFlags::NONE);
        assert!(r.is_ok());
        let mut flags = exempi2::PropFlags::default();
        let date = xmp
            .get_property_date(NS_EXIF, "DateTimeOriginal", &mut flags)
            .unwrap();
        assert_eq!(d, date);

        let date = xmp
            .get_property(NS_EXIF, "DateTimeOriginal", &mut flags)
            .unwrap();
        assert_eq!(&date.to_string(), "2012-02-17T11:10:49-04:00");

        let d = xmp_date_from_exif("2012:02:17/11:10:49", None);
        assert!(d.is_none());

        let d = xmp_date_from_exif("2012.02.17 11.10.49", None);
        assert!(d.is_none());
    }
}
