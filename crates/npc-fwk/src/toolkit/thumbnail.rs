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

use super::gdk_utils;
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

    pub fn thumbnail_file<P: AsRef<Path>>(
        path: P,
        w: u32,
        h: u32,
        orientation: u32,
    ) -> Option<Self> {
        let filename = path.as_ref();
        let mime_type = MimeType::new(filename);

        let mut pix: Option<gdk_pixbuf::Pixbuf> = None;

        if mime_type.is_unknown() {
            dbg_out!("unknown file type {:?}", filename);
        } else if mime_type.is_movie() {
            // XXX FIXME
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
                    dbg_out!("Heif image");
                    pix = heif::extract_rotated_thumbnail(filename, cmp::min(w, h), orientation)
                        .map_err(|err| {
                            err_out!("Error {:?}", err);
                            err
                        })
                        .ok();
                }
                _ => {
                    dbg_out!("not a raw type, trying GdkPixbuf loaders");
                    match gdk_pixbuf::Pixbuf::from_file_at_size(filename, w as i32, h as i32) {
                        Ok(ref pixbuf) => {
                            pix = gdk_utils::gdkpixbuf_exif_rotate(Some(pixbuf), orientation);
                        }
                        Err(err) => err_out!("exception thumbnailing image: {}", err),
                    }
                }
            }
        } else {
            dbg_out!("trying raw loader");
            pix = gdk_utils::openraw_extract_rotated_thumbnail(filename, cmp::min(w, h) as u32)
                .and_then(|pixbuf| {
                    if (w < pixbuf.width() as u32) || (h < pixbuf.height() as u32) {
                        gdk_utils::gdkpixbuf_scale_to_fit(Some(&pixbuf), cmp::min(w, h))
                    } else {
                        Some(pixbuf)
                    }
                })
                .or_else(|| {
                    err_out!("raw loader failed");
                    None
                });
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
