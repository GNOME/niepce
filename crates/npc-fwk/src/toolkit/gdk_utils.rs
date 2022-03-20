/*
 * niepce - npc-fwk/toolkit/gdk_utils.rs
 *
 * Copyright (C) 2020-2022 Hubert Figuière
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

use gdk_pixbuf::prelude::*;
use libopenraw_rs as or;
use libopenraw_rs::DataType;

/// Scale the pixbuf to fit in a square of %dim pixels
pub fn gdkpixbuf_scale_to_fit(
    pix: Option<&gdk_pixbuf::Pixbuf>,
    dim: i32,
) -> Option<gdk_pixbuf::Pixbuf> {
    pix.and_then(|pix| {
        let orig_h = pix.height();
        let orig_w = pix.width();
        let orig_dim = cmp::max(orig_h, orig_w);
        let ratio: f64 = dim as f64 / orig_dim as f64;
        let width = ratio * orig_w as f64;
        let height = ratio * orig_h as f64;
        pix.scale_simple(
            width as i32,
            height as i32,
            gdk_pixbuf::InterpType::Bilinear,
        )
    })
}

/// Rotate the pixbuf according to the Exif orientation value
pub fn gdkpixbuf_exif_rotate(
    pix: Option<&gdk_pixbuf::Pixbuf>,
    orientation: i32,
) -> Option<gdk_pixbuf::Pixbuf> {
    match orientation {
        0 | 1 => pix.cloned(),
        2 => pix.and_then(|p| p.flip(true)),
        3 => pix.and_then(|p| p.rotate_simple(gdk_pixbuf::PixbufRotation::Upsidedown)),
        4 => pix
            .and_then(|p| p.rotate_simple(gdk_pixbuf::PixbufRotation::Upsidedown))
            .and_then(|p| p.flip(true)),
        5 => pix
            .and_then(|p| p.rotate_simple(gdk_pixbuf::PixbufRotation::Clockwise))
            .and_then(|p| p.flip(false)),
        6 => pix.and_then(|p| p.rotate_simple(gdk_pixbuf::PixbufRotation::Clockwise)),
        7 => pix
            .and_then(|p| p.rotate_simple(gdk_pixbuf::PixbufRotation::Counterclockwise))
            .and_then(|p| p.flip(false)),
        8 => pix.and_then(|p| p.rotate_simple(gdk_pixbuf::PixbufRotation::Counterclockwise)),
        _ => None,
    }
}

fn thumbnail_to_pixbuf(thumbnail: &or::Thumbnail, orientation: i32) -> Option<gdk_pixbuf::Pixbuf> {
    let format = thumbnail.get_format();
    let buf = thumbnail.get_data().ok()?;

    let pixbuf = match format {
        DataType::Pixmap8Rgb => {
            let (x, y) = thumbnail.get_dimensions();

            let bytes = glib::Bytes::from(buf);
            Some(gdk_pixbuf::Pixbuf::from_bytes(
                &bytes,
                gdk_pixbuf::Colorspace::Rgb,
                false,
                8,
                x as i32,
                y as i32,
                x as i32 * 3,
            ))
        }
        DataType::Jpeg | DataType::Tiff | DataType::Png => {
            let loader = gdk_pixbuf::PixbufLoader::new();

            if let Err(err) = loader.write(&buf) {
                err_out!("loader write error: {}", err);
            }

            if let Err(err) = loader.close() {
                err_out!("loader close error: {}", err);
            }
            loader.pixbuf()
        }
        _ => None,
    };

    gdkpixbuf_exif_rotate(pixbuf.as_ref(), orientation)
}

/// Extract the thumbnail from libopenraw
pub fn openraw_extract_rotated_thumbnail<P: AsRef<Path>>(
    path: P,
    dim: u32,
) -> Option<gdk_pixbuf::Pixbuf> {
    let mut orientation: i32 = 0;
    or::RawFile::from_file(path, or::RawFileType::Unknown)
        .and_then(|r| {
            orientation = r.get_orientation();
            r.get_thumbnail(dim)
        })
        .map_err(|err| {
            err_out!("or_get_extract_thumbnail() failed with {:?}.", err);
            err
        })
        .ok()
        .and_then(|t| thumbnail_to_pixbuf(&t, orientation))
}
