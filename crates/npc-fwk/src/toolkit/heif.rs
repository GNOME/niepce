/*
 * niepce - npc-fwk/toolkit/heif.rs
 *
 * Copyright (C) 2024-2025 Hubert Figuière
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

use crate::gdk_pixbuf;
use crate::glib;
use anyhow::{Context, Result, anyhow};
use libheif_rs::{
    Channel, ColorSpace, DecodingOptions, HeifContext, ImageHandle, ItemId, LibHeif, RgbChroma,
};

use crate::toolkit::Thumbnail;

/// Return a rotated thumbnail from an HEIF file.
pub fn extract_rotated_thumbnail<P: AsRef<Path>>(
    filename: P,
    w: u32,
    h: u32,
    orientation: Option<u32>,
) -> Result<Thumbnail> {
    dbg_out!("HEIF thumbnail size = {w}x{h} <> {orientation:?}");
    // This always returns a rotated thumbnail
    let ctx = HeifContext::read_from_file(filename.as_ref().to_str().ok_or(anyhow!("filename"))?)?;
    let handle = ctx.primary_image_handle()?;

    thumbnail_heif(&handle, w, h, orientation.is_none())
        .map(|buf| buf.thumbnail(w, h))
        .map(|mut buf| {
            let orientation = orientation
                .and_then(|orientation| image::metadata::Orientation::from_exif(orientation as u8))
                .unwrap_or(image::metadata::Orientation::NoTransforms);
            buf.apply_orientation(orientation);
            buf
        })
        .map(Thumbnail::from)
}

fn thumbnail_heif(
    handle: &ImageHandle,
    w: u32,
    h: u32,
    rotate: bool,
) -> Result<image::DynamicImage> {
    let size = std::cmp::max(w, h);

    let count = handle.number_of_thumbnails();
    dbg_out!("num of thumbnails {count}");
    if count > 0 {
        let mut ids = vec![0; count];
        let _count = handle.thumbnail_ids(&mut ids);
        let mut thumbnails = ids
            .iter()
            .filter_map(|id| {
                dbg_out!("thumb id {id}");
                handle.thumbnail(*id).ok().and_then(|thumbnail| {
                    let w = thumbnail.width();
                    let h = thumbnail.height();
                    dbg_out!("found thumbnail {}x{}", w, h);
                    let dim = cmp::max(w, h);
                    if dim >= size {
                        Some((dim, thumbnail))
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();
        thumbnails.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for thumbnail in thumbnails {
            let t = image_from_heif_handle(&thumbnail.1, rotate);
            err_out!("thumbnail {} failed to decode.", thumbnail.0);
            if t.is_ok() {
                return t;
            }
        }
    }

    image_from_heif_handle(handle, rotate)
}

/// Return the main image from an HEIF as Pixbuf.
pub fn gdkpixbuf_from_heif<P: AsRef<Path>>(filename: P) -> Result<gdk_pixbuf::Pixbuf> {
    let ctx = HeifContext::read_from_file(filename.as_ref().to_str().ok_or(anyhow!("filename"))?)?;
    let handle = ctx.primary_image_handle()?;

    gdkpixbuf_from_heif_handle(&handle)
}

/// Return if a HEVC decoder is found at runtime
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

/// Is the file HEIF ? Currently only check the extension.
pub fn is_heif<P: AsRef<Path>>(file: P) -> bool {
    file.as_ref()
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("heic"))
        .unwrap_or(false)
}

/// Get the XMP from the HEIF file.
pub fn get_xmp(file: &str) -> Result<Vec<u8>> {
    let ctx = HeifContext::read_from_file(file)?;
    let handle = ctx.primary_image_handle()?;

    let mut meta_ids: Vec<ItemId> = vec![0; 1];
    let count = handle.metadata_block_ids(&mut meta_ids, b"mime");
    if count == 1 {
        handle
            .metadata(meta_ids[0])
            .context("Failed to read metadata for XMP")
    } else {
        Err(anyhow!("HEIF XMP metadata not found"))
    }
}

/// Get the Exif blob from the HEIF file.
pub fn get_exif(file: &str) -> Result<Vec<u8>> {
    let ctx = HeifContext::read_from_file(file)?;
    let handle = ctx.primary_image_handle()?;

    let mut meta_ids: Vec<ItemId> = vec![0; 1];
    let count = handle.metadata_block_ids(&mut meta_ids, b"Exif");
    if count == 1 {
        handle
            .metadata(meta_ids[0])
            .context("Failed to read metadata for Exif")
    } else {
        Err(anyhow!("HEIF Exif metadata not found"))
    }
}

/// Return the GdkPibuf from the handle
fn image_from_heif_handle(handle: &ImageHandle, rotate: bool) -> Result<image::DynamicImage> {
    let lib_heif = LibHeif::new();

    let decoding_options = DecodingOptions::new().map(|mut options| {
        options.set_ignore_transformations(!rotate);
        options
    });
    if !rotate {
        dbg_out!("Image decoded without transformations");
    }
    let image = lib_heif
        .decode(handle, ColorSpace::Rgb(RgbChroma::Rgb), decoding_options)
        .context("failed decoding")?;

    if image.has_channel(Channel::Interleaved) {
        let w = image.width();
        let h = image.height();

        if let Some(plane) = image.planes().interleaved {
            let stride = plane
                .stride
                .try_into()
                .map(|stride: u32| stride / 3)
                .unwrap_or(w);
            return image::RgbImage::from_raw(stride, h, plane.data.to_vec())
                .map(image::DynamicImage::ImageRgb8)
                .map(|mut image| image.crop(0, 0, w, h))
                .ok_or_else(|| anyhow!("Failed to load buffer"));
        }
    }

    Err(anyhow!("Failed to decode HEIF"))
}

/// Return the GdkPibuf from the handle
fn gdkpixbuf_from_heif_handle(handle: &ImageHandle) -> Result<gdk_pixbuf::Pixbuf> {
    let lib_heif = LibHeif::new();

    let decoding_options = DecodingOptions::new().map(|mut options| {
        options.set_ignore_transformations(true);
        options
    });
    dbg_out!("Image decoded without transformations");
    let image = lib_heif
        .decode(handle, ColorSpace::Rgb(RgbChroma::Rgb), decoding_options)
        .context("failed decoding")?;

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
