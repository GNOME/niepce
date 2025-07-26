/*
 * niepce - crates/npc-fwk/src/tookit/image_bitmap.rs
 *
 * Copyright (C) 2023-2025 Hubert Figuière
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

use std::io::{Read, Write};
use std::sync::Arc;

use crate::gdk4;
use crate::glib;
use image::{ImageDecoder, ImageEncoder};
use thiserror::Error;

use crate::base::Size;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
}

type Result<T> = std::result::Result<T, Error>;

enum BitmapType {
    Rgb(Vec<u8>),
    Png(Vec<u8>),
}

impl Default for BitmapType {
    fn default() -> BitmapType {
        BitmapType::Rgb(vec![])
    }
}

impl std::fmt::Debug for BitmapType {
    // implemented manually to skip dumping all the bytes.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("BitmapType")
            .field(
                "buffer",
                &match self {
                    BitmapType::Rgb(b) => format!("Rgb({})", b.len()),
                    BitmapType::Png(b) => format!("Png({})", b.len()),
                },
            )
            .finish()
    }
}

#[derive(Clone, Debug, Default)]
/// ImageBitmap represent the bitmap of the image with `size`.
/// It currently assumes BGRA.
pub struct ImageBitmap {
    /// The pixel size.
    size: Size,
    /// Buffer is shared, so that clone share the buffer.
    buffer: Arc<BitmapType>,
}

impl ImageBitmap {
    /// New from buffer.
    pub fn new(buffer: Vec<u8>, w: u32, h: u32) -> Self {
        Self {
            size: Size { w, h },
            buffer: Arc::new(BitmapType::Rgb(buffer)),
        }
    }

    /// Load from PNG file. This will not decompress the PNG
    /// stream, but will check its dimension
    pub fn from_file<P>(file: P) -> Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let mut buffer = Vec::new();
        let mut f = std::fs::File::open(&file)?;
        f.read_to_end(&mut buffer)?;
        let decoder = image::codecs::png::PngDecoder::new(std::io::Cursor::new(&buffer))?;
        let dimensions = decoder.dimensions();
        Ok(Self {
            size: Size {
                w: dimensions.0,
                h: dimensions.1,
            },
            buffer: Arc::new(BitmapType::Png(buffer)),
        })
    }

    /// Save as PNG
    pub fn save_png<P>(&self, file: P) -> Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        match *self.buffer {
            BitmapType::Png(ref buffer) => {
                let mut f = std::fs::File::create(&file)?;
                f.write_all(buffer)?;
            }
            BitmapType::Rgb(ref buffer) => {
                let f = std::fs::File::create(&file)?;
                let encoder = image::codecs::png::PngEncoder::new(f);
                encoder.write_image(
                    buffer,
                    self.size.w,
                    self.size.h,
                    image::ExtendedColorType::Rgb8,
                )?;
            }
        }
        Ok(())
    }

    /// The width of the image in pixels
    pub fn original_width(&self) -> u32 {
        self.size.w
    }

    /// The height of the image in pixels.
    pub fn original_height(&self) -> u32 {
        self.size.h
    }

    /// Create a gdk4::Texture from the image for display.
    /// Caveat: there don't seem to be a way to consume the data, so it's duplicated.
    pub fn to_gdk_texture(&self) -> gdk4::Texture {
        match &*self.buffer {
            BitmapType::Rgb(b) => {
                let bytes = glib::Bytes::from_owned(b.clone());
                gdk4::MemoryTexture::new(
                    self.size.w as i32,
                    self.size.h as i32,
                    gdk4::MemoryFormat::R8g8b8,
                    &bytes,
                    (self.size.w * 3) as usize,
                )
                .into()
            }
            BitmapType::Png(b) => {
                let bytes = glib::Bytes::from_owned(b.clone());
                gdk4::Texture::from_bytes(&bytes).expect("Couldn't load")
            }
        }
    }
}
