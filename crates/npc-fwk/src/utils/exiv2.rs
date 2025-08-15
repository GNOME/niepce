/*
 * niepce - fwk/utils/exiv2.rs
 *
 * Copyright (C) 2018-2024 Hubert Figuière
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

use multimap::MultiMap;
use std::ffi::OsStr;

use super::exempi::{
    Flash, NS_AUX, NS_EXIF, NS_EXIF_EX, NS_TIFF, NS_XAP, XmpMeta, xmp_date_from_exif,
};

/// Property conversion rules
#[derive(Clone, Copy, Debug)]
enum Conversion {
    /// Use value as is
    None,
    /// Convert from the Exif date format to a date
    ExifDate,
    /// Convert the Exif flash tag to the struct for XMP
    Flash,
    /// Use exiv2 get_interpreted_tag_string
    Interpreted,
}

/// Converstion result
#[derive(Clone, Debug)]
enum Converted {
    /// A String
    Str(String),
    #[allow(dead_code)]
    /// An int 32
    Int(i32),
    /// an Xmp DateTime value
    Date(Option<exempi2::DateTime>),
    /// The Flash XMP structure
    Flash(Flash),
}

#[derive(Clone)]
struct XmpPropDesc(&'static str, &'static str, Conversion);

lazy_static::lazy_static! {
    static ref EXIV2_TO_XMP: MultiMap<&'static str, XmpPropDesc> =
        multimap::multimap!(
            "Exif.Image.DateTime" =>
                XmpPropDesc(NS_XAP, "ModifyDate", Conversion::ExifDate),
            "Exif.Image.ImageHeight" =>
                XmpPropDesc(NS_TIFF, "ImageHeight", Conversion::None),
            "Exif.Image.ImageWidth" =>
                XmpPropDesc(NS_TIFF, "ImageWidth", Conversion::None),
            "Exif.Image.Make" =>
                XmpPropDesc(NS_TIFF, "Make", Conversion::None),
            "Exif.Image.Model" =>
                XmpPropDesc(NS_TIFF, "Model", Conversion::None),
            "Exif.Image.Orientation" =>
                XmpPropDesc(NS_TIFF, "Orientation", Conversion::None),
            "Exif.Image.Software" =>
                XmpPropDesc(NS_TIFF, "Software", Conversion::None),
            "Exif.Photo.ApertureValue" =>
                XmpPropDesc(NS_EXIF, "ApertureValue", Conversion::None),
            "Exif.Photo.BodySerialNumber" =>
                XmpPropDesc(NS_EXIF_EX, "BodySerialNumber", Conversion::None),
            "Exif.Photo.CameraOwnerName" =>
                XmpPropDesc(NS_EXIF_EX, "CameraOwnerName", Conversion::None),
            "Exif.Photo.ColorSpace" =>
                XmpPropDesc(NS_EXIF, "ColorSpace", Conversion::None),
            "Exif.Photo.DateTimeOriginal" =>
                XmpPropDesc(NS_EXIF, "DateTimeOriginal", Conversion::ExifDate),
            "Exif.Photo.DateTimeDigitized" =>
                XmpPropDesc(NS_XAP, "CreateDate", Conversion::ExifDate),
            "Exif.Photo.ExposureBiasValue" =>
                XmpPropDesc(NS_EXIF, "ExposureBiasValue", Conversion::None),
            "Exif.Photo.ExposureMode" =>
                XmpPropDesc(NS_EXIF, "ExposureMode", Conversion::None),
            "Exif.Photo.ExposureProgram" =>
                XmpPropDesc(NS_EXIF, "ExposureProgram", Conversion::None),
            "Exif.Photo.ExposureTime" =>
                XmpPropDesc(NS_EXIF, "ExposureTime", Conversion::None),
            "Exif.Photo.FNumber" =>
                XmpPropDesc(NS_EXIF, "FNumber", Conversion::None),
            "Exif.Photo.Flash" =>
                XmpPropDesc(NS_EXIF, "Flash", Conversion::Flash),
            "Exif.Photo.FocalLength" =>
                XmpPropDesc(NS_EXIF, "FocalLength", Conversion::None),
            "Exif.Photo.FocalLengthIn35mmFilm" =>
                XmpPropDesc(NS_EXIF, "FocalLengthIn35mmFilm", Conversion::None),
            "Exif.Photo.ISOSpeedRatings" =>
                XmpPropDesc(NS_EXIF, "ISOSpeedRatings", Conversion::None),
            "Exif.Photo.LensMake" =>
                XmpPropDesc(NS_EXIF_EX, "LensMake", Conversion::None),
            "Exif.Photo.LensModel" =>
                XmpPropDesc(NS_AUX, "Lens", Conversion::None),
            "Exif.Photo.LensModel" =>
                XmpPropDesc(NS_EXIF_EX, "LensModel", Conversion::None),
            "Exif.Photo.LensSerialNumber" =>
                XmpPropDesc(NS_EXIF_EX, "LensSerialNumber", Conversion::None),
            "Exif.Photo.LensSpecification" =>
                XmpPropDesc(NS_EXIF_EX, "LensSpecification", Conversion::None),
            "Exif.Photo.LightSource" =>
                XmpPropDesc(NS_EXIF, "LightSource", Conversion::None),
            "Exif.Photo.MeteringMode" =>
                XmpPropDesc(NS_EXIF, "MeteringMode", Conversion::None),
            "Exif.Photo.SceneCaptureType" =>
                XmpPropDesc(NS_EXIF, "SceneCaptureType", Conversion::None),
            "Exif.Photo.ShutterSpeedValue" =>
                XmpPropDesc(NS_EXIF, "ShutterSpeedValue", Conversion::None),
            "Exif.Photo.UserComment" =>
                XmpPropDesc(NS_EXIF, "UserComment", Conversion::None),
            "Exif.Photo.WhiteBalance" =>
                XmpPropDesc(NS_EXIF, "WhiteBalance", Conversion::None),
            "Exif.Canon.LensModel" =>
                XmpPropDesc(NS_AUX, "Lens", Conversion::None),
            "Exif.Canon.LensModel" =>
                XmpPropDesc(NS_EXIF_EX, "LensModel", Conversion::None),
            "Exif.Minolta.LensID" =>
                XmpPropDesc(NS_AUX, "Lens", Conversion::Interpreted),
            "Exif.Minolta.LensID" =>
                XmpPropDesc(NS_EXIF_EX, "LensModel", Conversion::Interpreted),
            "Exif.Nikon3.Lens" =>
                XmpPropDesc(NS_AUX, "Lens", Conversion::Interpreted),
            "Exif.Nikon3.Lens" =>
                XmpPropDesc(NS_EXIF_EX, "LensModel", Conversion::Interpreted),
            "Exif.OlympusEq.LensModel" =>
                XmpPropDesc(NS_AUX, "Lens", Conversion::None),
            "Exif.OlympusEq.LensModel" =>
                XmpPropDesc(NS_EXIF_EX, "LensModel", Conversion::None),
            "Exif.OlympusEq.LensSerialNumber" =>
                XmpPropDesc(NS_EXIF_EX, "LensSerialNumber", Conversion::None),
            "Exif.Panasonic.LensType" =>
                XmpPropDesc(NS_AUX, "Lens", Conversion::None),
            "Exif.Panasonic.LensType" =>
                XmpPropDesc(NS_EXIF_EX, "LensModel", Conversion::None),
            "Exif.Panasonic.LensSerialNumber" =>
                XmpPropDesc(NS_EXIF_EX, "LensSerialNumber", Conversion::None),
            "Exif.Pentax.LensType" =>
                XmpPropDesc(NS_AUX, "Lens", Conversion::Interpreted),
            "Exif.Pentax.LensType" =>
                XmpPropDesc(NS_EXIF_EX, "LensModel", Conversion::Interpreted),
            "Xmp.xmp.Rating" =>
                XmpPropDesc(NS_XAP, "Rating", Conversion::None),
        );
}

fn convert(meta: &rexiv2::Metadata, tag: &str, conversion: Conversion, value: &str) -> Converted {
    match conversion {
        Conversion::None => Converted::Str(value.to_string()),
        Conversion::ExifDate => {
            // Time difference
            let offset_tag = match tag {
                "Exif.Image.DateTime" => Some("Exif.Photo.OffsetTime"),
                "Exif.Photo.DateTimeOriginal" => Some("Exif.Photo.OffsetTimeOriginal"),
                "Exif.Photo.DateTimeDigitized" => Some("Exif.Photo.OffsetTimeDigitized"),
                _ => None,
            };
            let offset = offset_tag.and_then(|offset_tag| meta.get_tag_string(offset_tag).ok());
            Converted::Date(xmp_date_from_exif(value, offset.as_deref()))
        }
        _ => {
            err_out!("Conversion error for str to {:?}", conversion);
            Converted::Str(value.to_string())
        }
    }
}

fn convert_int(conversion: Conversion, int: i32) -> Converted {
    match conversion {
        Conversion::None => Converted::Int(int),
        Conversion::Flash => Converted::Flash(Flash::from(int)),
        _ => {
            err_out!("Conversion error from int to {:?}", conversion);
            Converted::Int(int)
        }
    }
}

/// Convert ASCII tag value into the XMP.
fn ascii_tag_to_xmp(
    meta: &rexiv2::Metadata,
    tag: &str,
    xmp_prop: &XmpPropDesc,
    xmp: &mut exempi2::Xmp,
) {
    if let Ok(value) = meta.get_tag_string(tag) {
        let converted = convert(meta, tag, xmp_prop.2, &value);
        match converted {
            Converted::Str(s) => {
                if let Err(err) =
                    xmp.set_property(xmp_prop.0, xmp_prop.1, &s, exempi2::PropFlags::NONE)
                {
                    err_out!(
                        "Error setting property {} {}: {:?}",
                        &xmp_prop.0,
                        &xmp_prop.1,
                        &err
                    );
                }
            }
            Converted::Date(d) => {
                if let Some(d) = d {
                    if let Err(err) =
                        xmp.set_property_date(xmp_prop.0, xmp_prop.1, &d, exempi2::PropFlags::NONE)
                    {
                        err_out!(
                            "Error setting property {} {}: {:?}",
                            &xmp_prop.0,
                            &xmp_prop.1,
                            &err
                        );
                    }
                } else {
                    err_out!("Couldn't convert Exif date {}", &value);
                }
            }
            _ => {
                err_out!("Unknown conversion result {:?}", &converted);
            }
        }
    }
}

/// Convert one of the many Int tag values into the XMP.
fn int_tag_to_xmp(
    meta: &rexiv2::Metadata,
    tag: &str,
    tagtype: rexiv2::TagType,
    xmp_prop: &XmpPropDesc,
    xmp: &mut exempi2::Xmp,
) {
    // XXX rexiv2 returns an i32, which is a problem for UnsignedLong
    match xmp_prop.2 {
        Conversion::Flash => {
            if let Converted::Flash(flash) = convert_int(xmp_prop.2, meta.get_tag_numeric(tag)) {
                if let Err(err) = flash.set_as_xmp_property(xmp, NS_EXIF, "Flash") {
                    err_out!(
                        "Error setting property {} {}: {:?}",
                        &xmp_prop.0,
                        &xmp_prop.1,
                        &err
                    );
                }
            }
        }
        Conversion::None => {
            let value = meta.get_tag_numeric(tag);
            if let Err(err) =
                xmp.set_property_i32(xmp_prop.0, xmp_prop.1, value, exempi2::PropFlags::NONE)
            {
                err_out!(
                    "Error setting property {} {}: {:?}",
                    &xmp_prop.0,
                    &xmp_prop.1,
                    &err
                );
            }
        }
        Conversion::Interpreted => {
            if let Ok(value) = meta.get_tag_interpreted_string(tag) {
                if let Err(err) =
                    xmp.set_property(xmp_prop.0, xmp_prop.1, &value, exempi2::PropFlags::NONE)
                {
                    err_out!(
                        "Error setting property {} {}: {:?}",
                        &xmp_prop.0,
                        &xmp_prop.1,
                        &err
                    );
                }
            }
        }
        _ => {
            err_out!("Unknown conversion from {:?} to {:?}", tagtype, xmp_prop.2);
            let value = meta.get_tag_numeric(tag);
            if let Err(err) =
                xmp.set_property_i32(xmp_prop.0, xmp_prop.1, value, exempi2::PropFlags::NONE)
            {
                err_out!(
                    "Error setting property {} {}: {:?}",
                    &xmp_prop.0,
                    &xmp_prop.1,
                    &err
                );
            }
        }
    }
}

/// Build the XMP packet from a file with Exiv2.
pub fn xmp_from_exiv2<S: AsRef<OsStr>>(file: S) -> Option<XmpMeta> {
    rexiv2::Metadata::new_from_path(file)
        .ok()
        .and_then(xmp_from_exiv2meta)
}

/// Build the XMP packet from the Exif blob. Same as `xmp_from_exiv2` otherwise.
pub fn xmp_from_exif(buffer: &[u8]) -> Option<XmpMeta> {
    rexiv2::Metadata::new_from_app1_segment(buffer)
        .ok()
        .and_then(xmp_from_exiv2meta)
}

/// Build the XMP packet from the `rexiv2::Metadata`.
fn xmp_from_exiv2meta(meta: rexiv2::Metadata) -> Option<XmpMeta> {
    dbg_out!("xmp_from_exiv2meta");
    let mut xmp = exempi2::Xmp::new();
    let mut all_tags: Vec<String> = vec![];
    if let Ok(mut tags) = meta.get_exif_tags() {
        all_tags.append(&mut tags);
    }
    if let Ok(mut tags) = meta.get_iptc_tags() {
        all_tags.append(&mut tags);
    }
    if let Ok(mut tags) = meta.get_xmp_tags() {
        all_tags.append(&mut tags);
    }
    for tag in all_tags {
        if let Some(xmp_props) = EXIV2_TO_XMP.get_vec(tag.as_str()) {
            for xmp_prop in xmp_props {
                let tagtype = rexiv2::get_tag_type(&tag);
                match tagtype {
                    Ok(rexiv2::TagType::AsciiString) | Ok(rexiv2::TagType::XmpText) => {
                        ascii_tag_to_xmp(&meta, &tag, xmp_prop, &mut xmp);
                    }
                    Ok(rexiv2::TagType::UnsignedShort)
                    | Ok(rexiv2::TagType::UnsignedLong)
                    | Ok(rexiv2::TagType::SignedShort)
                    | Ok(rexiv2::TagType::SignedLong) => {
                        int_tag_to_xmp(&meta, &tag, *tagtype.as_ref().unwrap(), xmp_prop, &mut xmp);
                    }
                    Ok(rexiv2::TagType::UnsignedRational) | Ok(rexiv2::TagType::SignedRational) => {
                        match xmp_prop.2 {
                            Conversion::Interpreted => {
                                if let Ok(value) = meta.get_tag_interpreted_string(&tag) {
                                    if let Err(err) = xmp.set_property(
                                        xmp_prop.0,
                                        xmp_prop.1,
                                        &value,
                                        exempi2::PropFlags::NONE,
                                    ) {
                                        err_out!(
                                            "Error setting property {} {}: {:?}",
                                            &xmp_prop.0,
                                            &xmp_prop.1,
                                            &err
                                        );
                                    }
                                }
                            }
                            _ => {
                                if let Some(value) = meta.get_tag_rational(&tag) {
                                    let value_str = format!("{}/{}", value.numer(), value.denom());
                                    if let Err(err) = xmp.set_property(
                                        xmp_prop.0,
                                        xmp_prop.1,
                                        &value_str,
                                        exempi2::PropFlags::NONE,
                                    ) {
                                        err_out!(
                                            "Error setting property {} {}: {:?}",
                                            &xmp_prop.0,
                                            &xmp_prop.1,
                                            &err
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Ok(rexiv2::TagType::Comment) => {
                        if let Ok(value) = meta.get_tag_string(&tag) {
                            if let Err(err) = xmp.set_property(
                                xmp_prop.0,
                                xmp_prop.1,
                                &value,
                                exempi2::PropFlags::NONE,
                            ) {
                                err_out!(
                                    "Error setting property {} {}: {:?}",
                                    &xmp_prop.0,
                                    &xmp_prop.1,
                                    &err
                                );
                            }
                        }
                    }
                    Ok(rexiv2::TagType::UnsignedByte) => match xmp_prop.2 {
                        Conversion::Interpreted => {
                            if let Ok(value) = meta.get_tag_interpreted_string(&tag) {
                                if let Err(err) = xmp.set_property(
                                    xmp_prop.0,
                                    xmp_prop.1,
                                    &value,
                                    exempi2::PropFlags::NONE,
                                ) {
                                    err_out!(
                                        "Error setting property {} {}: {:?}",
                                        &xmp_prop.0,
                                        &xmp_prop.1,
                                        &err
                                    );
                                }
                            }
                        }
                        _ => err_out!("Unhandled type {:?} for {} as UnsignedByte", tagtype, &tag),
                    },
                    _ => {
                        err_out!("Unhandled type {:?} for {}", tagtype, &tag);
                    }
                }
            }
        } else {
            //                err_out!("Unknown property {}", &tag);
        }
    }
    meta.get_gps_info();

    let mut options = exempi2::PropFlags::default();
    if let Ok(date) = xmp.get_property_date(NS_XAP, "ModifyDate", &mut options) {
        if let Err(err) =
            xmp.set_property_date(NS_XAP, "MetadataDate", &date, exempi2::PropFlags::NONE)
        {
            err_out!("Error setting MetadataDate: {:?}", &err);
        }
    } else {
        err_out!("Couldn't get the ModifyDate");
    }
    Some(XmpMeta::from(xmp))
}
