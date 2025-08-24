/*
 * niepce - toolkit/thumbnail.rs
 *
 * Copyright (C) 2020-2025 Hubert Figuière
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

use std::cmp;
use std::convert::From;
use std::path::Path;

use crate::glib;
use crate::{gdk_pixbuf, gdk4};
use gdk_pixbuf::Colorspace;
use image::DynamicImage;
use libopenraw as or;
use libopenraw::Bitmap;

use super::heif;
use super::mimetype::{ImgFormat, MType, MimeType};
use super::movieutils;

#[derive(Clone)]
pub struct Thumbnail {
    bytes: Vec<u8>,
    width: u32,
    height: u32,
    stride: i32,
    bits_per_sample: i32,
    has_alpha: bool,
    colorspace: Colorspace,
}

impl std::fmt::Debug for Thumbnail {
    // implemented manually to skip dumping all the bytes.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("Thumbnail")
            .field("bytes.len()", &self.bytes.len())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("stride", &self.stride)
            .field("bits_per_sample", &self.bits_per_sample)
            .field("has_alpha", &self.has_alpha)
            .field("colorspace", &self.colorspace)
            .finish()
    }
}

impl Default for Thumbnail {
    fn default() -> Self {
        Self {
            bytes: vec![],
            width: 0,
            height: 0,
            stride: 0,
            bits_per_sample: 0,
            has_alpha: false,
            colorspace: Colorspace::Rgb,
        }
    }
}

impl Thumbnail {
    /// Return true if there is a pixbuf
    pub fn ok(&self) -> bool {
        !self.bytes.is_empty()
    }

    /// Get the width of the pixbuf. 0 if None
    pub fn get_width(&self) -> u32 {
        self.width
    }

    /// Get the height of the pixbuf. 0 if None
    pub fn get_height(&self) -> u32 {
        self.height
    }

    /// Make a gdk_pixbuf::Pixbuf out of the Thumbnail
    pub fn make_pixbuf(&self) -> Option<gdk_pixbuf::Pixbuf> {
        if self.ok() { Some(self.into()) } else { None }
    }

    pub fn save<P: AsRef<Path> + std::fmt::Debug>(&self, path: P, format: &str) {
        if let Some(pixbuf) = &self.make_pixbuf() {
            if let Err(err) = pixbuf.savev(&path, format, &[]) {
                err_out!("Failed to save pixbuf {:?}: {}", &path, err);
            }
        }
    }

    fn thumbnail_image<P: AsRef<Path>>(
        filename: P,
        w: u32,
        h: u32,
        orientation: Option<u32>,
    ) -> Option<Self> {
        let decoder = image::ImageReader::open(filename)
            .inspect_err(|err| err_out!("Error opening image for thumbnail: {err}"))
            .ok()?
            .into_decoder()
            .inspect_err(|err| err_out!("Error opening image for thumbnail: {err}"))
            .ok()?;
        DynamicImage::from_decoder(decoder)
            .inspect_err(|err| err_out!("Error decoding image for thumbnail: {err}"))
            .map(|buf| buf.thumbnail(w, h))
            .map(|mut buf| {
                let orientation = orientation
                    .and_then(|orientation| {
                        image::metadata::Orientation::from_exif(orientation as u8)
                    })
                    .unwrap_or(image::metadata::Orientation::NoTransforms);
                buf.apply_orientation(orientation);
                buf
            })
            .map(Thumbnail::from)
            .ok()
    }

    /// Thumbnail using libopenraw. Can work for JPEG.
    fn thumbnail_raw<P: AsRef<Path>>(
        filename: P,
        w: u32,
        h: u32,
        orientation: Option<u32>,
    ) -> Option<Self> {
        let dim = cmp::max(w, h);
        or::rawfile_from_file(filename, None)
            .and_then(|r| r.thumbnail(dim))
            .inspect_err(|err| {
                err_out!("or_get_extract_thumbnail() failed with {:?}.", err);
            })
            .ok()
            .and_then(|thumbnail| {
                let format = thumbnail.data_type();
                let buf = thumbnail.data8()?;

                let pixbuf = match format {
                    or::DataType::PixmapRgb8 => {
                        let x = thumbnail.width();
                        let y = thumbnail.height();
                        image::RgbImage::from_raw(x, y, buf.to_vec())
                            .map(image::DynamicImage::ImageRgb8)
                    }
                    or::DataType::Jpeg => {
                        let jpeg_dec =
                            image::codecs::jpeg::JpegDecoder::new(std::io::Cursor::new(buf))
                                .ok()?;
                        image::DynamicImage::from_decoder(jpeg_dec).ok()
                    }
                    _ => None,
                };
                pixbuf
                    .map(|buf| buf.thumbnail(w, h))
                    .map(|mut buf| {
                        let orientation = orientation
                            .and_then(|orientation| {
                                image::metadata::Orientation::from_exif(orientation as u8)
                            })
                            .unwrap_or(image::metadata::Orientation::NoTransforms);
                        buf.apply_orientation(orientation);
                        buf
                    })
                    .map(Thumbnail::from)
            })
    }

    /// Thumbnail a file at `path` within `w` and `h` dimensions.
    pub fn thumbnail_file<P: AsRef<Path>>(
        path: P,
        w: u32,
        h: u32,
        orientation: Option<u32>,
    ) -> Option<Self> {
        let filename = path.as_ref();
        let mime_type = MimeType::new(filename);

        let mut pix: Option<gdk_pixbuf::Pixbuf> = None;

        if mime_type.is_unknown() {
            dbg_out!("unknown file type {:?}", filename);
        } else if mime_type.is_movie() {
            dbg_out!("video thumbnail");
            pix = movieutils::thumbnail_movie(filename, w, h);
            if pix.is_none() {
                err_out!("exception thumbnailing video ");
            }
        } else if !mime_type.is_image() {
            dbg_out!("not an image type");
        } else if !mime_type.is_digicam_raw() {
            match mime_type.mime_type() {
                MType::Image(ImgFormat::Heif) => {
                    trace_out!("Heif image");
                    return heif::extract_rotated_thumbnail(filename, w, h, orientation)
                        .inspect_err(|err| {
                            err_out!("Error thumnailing HEIF {err:?}");
                        })
                        .ok();
                }
                MType::Image(ImgFormat::Jpeg) => {
                    return Self::thumbnail_raw(filename, w, h, orientation)
                        .or_else(|| Self::thumbnail_image(filename, w, h, orientation));
                }
                _ => {
                    trace_out!("not a raw type, trying image loaders");
                    return Self::thumbnail_image(filename, w, h, orientation);
                }
            }
        } else {
            dbg_out!("trying raw loader");
            return Self::thumbnail_raw(filename, w, h, orientation);
        }

        pix.map(Thumbnail::from)
    }
}

impl From<gdk_pixbuf::Pixbuf> for Thumbnail {
    fn from(pixbuf: gdk_pixbuf::Pixbuf) -> Self {
        let bytes = pixbuf.read_pixel_bytes();
        let width = pixbuf.width() as u32;
        let height = pixbuf.height() as u32;
        let stride = pixbuf.rowstride();
        let bits_per_sample = pixbuf.bits_per_sample();
        let colorspace = pixbuf.colorspace();
        let has_alpha = pixbuf.has_alpha();
        Self {
            width,
            height,
            stride,
            bits_per_sample,
            colorspace,
            has_alpha,
            bytes: Vec::from(bytes.as_ref()),
        }
    }
}

impl From<image::DynamicImage> for Thumbnail {
    fn from(image: image::DynamicImage) -> Self {
        let rgb8 = image.into_rgb8();
        Self {
            width: rgb8.width(),
            height: rgb8.height(),
            stride: rgb8.width() as i32 * 3,
            bits_per_sample: 8,
            colorspace: Colorspace::Rgb,
            has_alpha: false,
            bytes: rgb8.into_vec(),
        }
    }
}

impl From<&Thumbnail> for gdk4::Texture {
    fn from(v: &Thumbnail) -> gdk4::Texture {
        let format = match v.colorspace {
            Colorspace::Rgb => {
                if v.has_alpha {
                    gdk4::MemoryFormat::R8g8b8a8
                } else {
                    gdk4::MemoryFormat::R8g8b8
                }
            }
            // There is only one colorspace, but it's a non-exhaustive
            // enum.
            _ => unreachable!(),
        };
        gdk4::MemoryTexture::new(
            v.width as i32,
            v.height as i32,
            format,
            &glib::Bytes::from(&v.bytes),
            v.stride as usize,
        )
        .into()
    }
}

impl From<&Thumbnail> for gdk_pixbuf::Pixbuf {
    fn from(v: &Thumbnail) -> gdk_pixbuf::Pixbuf {
        gdk_pixbuf::Pixbuf::from_bytes(
            &glib::Bytes::from(&v.bytes),
            v.colorspace,
            v.has_alpha,
            v.bits_per_sample,
            v.width as i32,
            v.height as i32,
            v.stride,
        )
    }
}
