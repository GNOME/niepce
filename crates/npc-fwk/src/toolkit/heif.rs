/*
 * niepce - npc-fwk/toolkit/heif.rs
 *
 * Copyright (C) 2024 Hubert Figuière
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
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use libheif_rs::{Channel, ColorSpace, HeifContext, ImageHandle, LibHeif, RgbChroma};

use super::gdk_utils::{gdkpixbuf_exif_rotate, gdkpixbuf_scale_to_fit};

/// Return a rotated thumbnail from an HEIF file.
pub fn extract_rotated_thumbnail<P: AsRef<Path>>(
    filename: P,
    size: u32,
    orientation: u32,
) -> Result<gdk_pixbuf::Pixbuf> {
    dbg_out!("orientation heif = {} size = {}", orientation, size);
    gdkpixbuf_from_heif_thumbnail(filename, size)
        .and_then(|pixbuf| {
            gdkpixbuf_scale_to_fit(Some(&pixbuf), size).ok_or(anyhow!("scale to fit failed"))
        })
        .and_then(|pixbuf| {
            gdkpixbuf_exif_rotate(Some(&pixbuf), orientation).ok_or(anyhow!("exif rotate"))
        })
}

/// Return the thumnail image from an HEIF as Pixbuf.
/// If there is no thumnail it will return the main image
/// Size determine which image to get. It is returned as is.
fn gdkpixbuf_from_heif_thumbnail<P: AsRef<Path>>(
    filename: P,
    size: u32,
) -> Result<gdk_pixbuf::Pixbuf> {
    let ctx = HeifContext::read_from_file(filename.as_ref().to_str().ok_or(anyhow!("filename"))?)?;
    let handle = ctx.primary_image_handle()?;

    let count = handle.number_of_thumbnails();
    dbg_out!("num of thumbnails {count}");
    if count > 0 {
        let mut ids = vec![0; count];
        let _count = handle.thumbnail_ids(&mut ids);
        for id in ids {
            if let Ok(thumbnail) = handle.thumbnail(id) {
                let w = thumbnail.width();
                let h = thumbnail.height();
                dbg_out!("found thumbnail {}x{}", w, h);
                if cmp::max(w, h) >= size {
                    return gdkpixbuf_from_heif_handle(&thumbnail);
                }
            }
        }
    }

    gdkpixbuf_from_heif_handle(&handle)
}

/// Return the main image from an HEIF as Pixbuf.
pub fn gdkpixbuf_from_heif<P: AsRef<Path>>(filename: P) -> Result<gdk_pixbuf::Pixbuf> {
    let ctx = HeifContext::read_from_file(filename.as_ref().to_str().ok_or(anyhow!("filename"))?)?;
    let handle = ctx.primary_image_handle()?;

    gdkpixbuf_from_heif_handle(&handle)
}

/// Return if a HEVC decode is found at runtime
///
pub fn has_hevc_decoder() -> bool {
    let lib_heif = LibHeif::new();

    for descriptor in lib_heif.decoder_descriptors(libc::c_int::MAX as usize, None) {
        if descriptor.id() == "libde265" {
            dbg_out!("Found HEVC libde265");
            return true;
        } else if descriptor.id() == "ffmpeg" {
            dbg_out!("Found HEVC ffmpeg");
            return true;
        }
    }

    false
}

/// Return the GdkPibuf from the handle
fn gdkpixbuf_from_heif_handle(handle: &ImageHandle) -> Result<gdk_pixbuf::Pixbuf> {
    let lib_heif = LibHeif::new();

    let image = lib_heif
        .decode(handle, ColorSpace::Rgb(RgbChroma::Rgb), None)
        .context("failed decoding")?;
    // XXX see about using the thumbnails in HEIF

    if image.has_channel(Channel::Interleaved) {
        let x = image.width();
        let y = image.height();

        if let Some(plane) = image.planes().interleaved {
            let bytes = glib::Bytes::from(plane.data);
            let stride: i32 = plane.stride.try_into().unwrap_or(x as i32 * 3);

            return Ok(gdk_pixbuf::Pixbuf::from_bytes(
                &bytes,
                gdk_pixbuf::Colorspace::Rgb,
                false,
                8,
                x as i32,
                y as i32,
                stride,
            ));
        }
    }

    Err(anyhow!("Failed to decode HEIF"))
}
