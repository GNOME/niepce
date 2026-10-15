/*
 * exiv2-rs - src/lib.rs
 *
 * Copyright (C) 2026 Hubert Figuière
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

mod exifdata;
mod image;

use cxx::{ExternType, type_id};
use thiserror::Error;

pub use image::Image;

mod generated {
    #![allow(non_camel_case_types)]
    #![allow(non_upper_case_globals)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("C++ Exception {0}")]
    Exception(#[from] cxx::Exception),
    #[error("UTF8 conversion {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub type ImageType = generated::Exiv2_ImageType;
unsafe impl ExternType for ImageType {
    type Id = type_id!("Exiv2::ImageType");
    type Kind = cxx::kind::Trivial;
}

pub type TypeId = generated::Exiv2_TypeId;
unsafe impl ExternType for TypeId {
    type Id = type_id!("Exiv2::TypeId");
    type Kind = cxx::kind::Trivial;
}

pub type ByteOrder = generated::Exiv2_ByteOrder;
unsafe impl ExternType for ByteOrder {
    type Id = type_id!("Exiv2::ByteOrder");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge(namespace = "Exiv2")]
mod ffi {

    extern "C++" {
        type BasicIo;
    }

    extern "C++" {
        type ByteOrder = super::ByteOrder;
        type ImageType = super::ImageType;
        type TypeId = super::TypeId;
    }

    unsafe extern "C++" {
        include!("exiv2/image.hpp");
        include!("npc_exiv2.h");
        type ImageFactory;

        #[cxx_name = "ImageFactory_open"]
        fn image_factory_open(path: &str, use_curl: bool) -> Result<UniquePtr<Image>>;
        #[cxx_name = "ImageFactory_create"]
        fn image_factory_create(t: ImageType) -> Result<UniquePtr<Image>>;
    }

    unsafe extern "C++" {
        type XmpProperties;

        #[cxx_name = "XmpProperties_type"]
        fn xmp_properties_type(key: &str) -> Result<TypeId>;
    }

    unsafe extern "C++" {
        type IptcDataSets;

        #[cxx_name = "IptcDataSets_type"]
        fn iptc_data_sets_type(key: &str) -> Result<TypeId>;
    }

    unsafe extern "C++" {
        type ExifKey;

        #[cxx_name = "ExifKey_new"]
        fn exifkey_new(key: &str) -> Result<UniquePtr<ExifKey>>;
        #[cxx_name = "defaultTypeId"]
        fn default_type_id(&self) -> TypeId;
    }

    unsafe extern "C++" {
        type Metadatum;

        #[cxx_name = "toInt64"]
        fn as_int64(&self, n: usize) -> i64;
        #[cxx_name = "Metadatum_toString"]
        fn metadatum_as_string(datum: &Metadatum) -> String;
        #[cxx_name = "Metadatum_toRational"]
        fn metadatum_as_rational(datum: &Metadatum, num: &mut i32, den: &mut i32);
        #[cxx_name = "Metadatum_convertToString"]
        fn metadatum_convert_to_string(datum: &Metadatum) -> String;
    }

    unsafe extern "C++" {
        type IptcData;
        #[cxx_name = "IptcData_get"]
        fn iptc_data_get<'a>(data: &'a IptcData, key: &str) -> Result<&'a Metadatum>;
        #[cxx_name = "IptcData_tags"]
        fn iptc_data_tags(data: &IptcData) -> Result<Vec<String>>;
    }

    unsafe extern "C++" {
        type XmpData;
        #[cxx_name = "XmpData_get"]
        fn xmp_data_get<'a>(data: &'a XmpData, key: &str) -> Result<&'a Metadatum>;
        #[cxx_name = "XmpData_tags"]
        fn xmp_data_tags(data: &XmpData) -> Result<Vec<String>>;
    }

    unsafe extern "C++" {
        type ExifData;

        #[cxx_name = "ExifData_get"]
        fn exif_data_get<'a>(data: &'a ExifData, key: &str) -> Result<&'a Metadatum>;
        #[cxx_name = "ExifData_tags"]
        fn exif_data_tags(data: &ExifData) -> Result<Vec<String>>;
    }

    unsafe extern "C++" {
        type ExifParser;

        #[cxx_name = "ExifParser_decode"]
        unsafe fn exif_parser_decode(
            exif_data: Pin<&mut ExifData>,
            data: *const u8,
            size: usize,
        ) -> ByteOrder;
    }

    unsafe extern "C++" {
        type Image;

        #[cxx_name = "readMetadata"]
        fn read_metadata(self: Pin<&mut Image>) -> Result<()>;
        #[cxx_name = "exifData"]
        fn exif_data(&self) -> &ExifData;
        #[cxx_name = "exifData"]
        fn exif_data_mut(self: Pin<&mut Image>) -> Pin<&mut ExifData>;
        #[cxx_name = "iptcData"]
        fn iptc_data(&self) -> &IptcData;
        #[cxx_name = "xmpData"]
        fn xmp_data(&self) -> &XmpData;
        #[cxx_name = "xmpPacket"]
        fn xmp_packet(&self) -> &CxxString;
    }
}
