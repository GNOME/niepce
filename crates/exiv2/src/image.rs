/*
 * exiv2-rs - src/image.rs
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

use std::pin::Pin;

use crate::exifdata::ExifParser;
use crate::{ImageType, ffi};

/// Image: contains metadata.
pub struct Image(cxx::UniquePtr<ffi::Image>);

impl std::ops::Deref for Image {
    type Target = ffi::Image;

    fn deref(&self) -> &ffi::Image {
        &self.0
    }
}

impl Image {
    /// Open an image. Equivalent to C++ `Exiv2::ImageFactory::open()`
    pub fn from_file<P>(path: P) -> crate::Result<Image>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(Image(ffi::image_factory_open(
            &path.as_ref().as_os_str().to_string_lossy(),
            false,
        )?))
    }

    /// Create and empty `Image` with a specific type.
    pub fn new_with_type(t: ImageType) -> crate::Result<Image> {
        Ok(Image(ffi::image_factory_create(t)?))
    }

    /// Load the image from JPEG APP1 segement
    /// This is inherited from gexiv2.
    pub fn from_app1_segment(buffer: &[u8]) -> crate::Result<Image> {
        let mut image = Self::new_with_type(ImageType::jpeg)?;
        ExifParser::decode(image.exif_data_mut(), buffer);
        Ok(image)
    }

    /// Read the metadata. This parses the file. It must be called
    /// prior to reading the metadata.
    pub fn read_metadata(&mut self) -> crate::Result<()> {
        Ok(self.0.pin_mut().read_metadata()?)
    }

    /// Return all Exif tags (not the values)
    pub fn exif_tags(&self) -> crate::Result<Vec<String>> {
        let exifdata = self.exif_data();
        let mut tags = ffi::exif_data_tags(exifdata)?;
        tags.sort();
        Ok(tags)
    }

    /// Return all IPTC tags (not the values)
    pub fn iptc_tags(&self) -> crate::Result<Vec<String>> {
        let iptcdata = self.iptc_data();
        let mut tags = ffi::iptc_data_tags(iptcdata)?;
        tags.sort();
        Ok(tags)
    }

    /// Return all XMP tags (not the values)
    pub fn xmp_tags(&self) -> crate::Result<Vec<String>> {
        let xmpdata = self.xmp_data();
        let mut tags = ffi::xmp_data_tags(xmpdata)?;
        tags.sort();
        Ok(tags)
    }

    fn exif_data_mut(&mut self) -> Pin<&mut ffi::ExifData> {
        self.0.pin_mut().exif_data_mut()
    }

    /// Get a tag type.
    pub fn tag_type(tag: &str) -> crate::Result<crate::TypeId> {
        Ok(ffi::exifkey_new(tag)
            .map(|key| key.default_type_id())
            .or_else(|_| ffi::iptc_data_sets_type(tag))
            .or_else(|_| ffi::xmp_properties_type(tag))?)
    }

    /// Get a tag string value. If it's not a string, good luck.
    pub fn tag_interpreted_string(&self, tag: &str) -> crate::Result<String> {
        Ok(ffi::exif_data_get(self.exif_data(), tag)
            .or_else(|_| ffi::iptc_data_get(self.iptc_data(), tag))
            .or_else(|_| ffi::xmp_data_get(self.xmp_data(), tag))
            .map(ffi::metadatum_convert_to_string)?)
    }

    /// Get a tag string value. If it's not a string, good luck.
    pub fn tag_string(&self, tag: &str) -> crate::Result<String> {
        Ok(ffi::exif_data_get(self.exif_data(), tag)
            .or_else(|_| ffi::iptc_data_get(self.iptc_data(), tag))
            .or_else(|_| ffi::xmp_data_get(self.xmp_data(), tag))
            .map(ffi::metadatum_as_string)?)
    }

    /// Get a tag string value. If it's not a string, good luck.
    pub fn tag_numeric(&self, tag: &str) -> i64 {
        ffi::exif_data_get(self.exif_data(), tag)
            .or_else(|_| ffi::iptc_data_get(self.iptc_data(), tag))
            .or_else(|_| ffi::xmp_data_get(self.xmp_data(), tag))
            .map(|datum| datum.as_int64(0))
            .unwrap_or(0)
    }

    /// Get a tag string value. If it's not a string, good luck.
    pub fn tag_rational(&self, tag: &str) -> crate::Result<num_rational::Ratio<i32>> {
        Ok(ffi::exif_data_get(self.exif_data(), tag)
            .or_else(|_| ffi::iptc_data_get(self.iptc_data(), tag))
            .or_else(|_| ffi::xmp_data_get(self.xmp_data(), tag))
            .map(|datum| {
                let mut num = 0_i32;
                let mut den = 0_i32;
                ffi::metadatum_as_rational(datum, &mut num, &mut den);
                num_rational::Ratio::new(num, den)
            })?)
    }

    /// Return the XMP packet
    pub fn xmp_packet(&self) -> crate::Result<&str> {
        Ok(self.0.xmp_packet().to_str()?)
    }
}
