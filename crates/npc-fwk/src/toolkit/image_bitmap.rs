/*
 * niepce - crates/npc-fwk/src/tookit/image_bitmap.rs
 *
 * Copyright (C) 2023 Hubert Figuière
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

use std::sync::Arc;

use crate::base::Size;

#[derive(Clone, Default)]
/// ImageBitmap represent the bitmap of the image with `size`.
/// It currently assumes BGRA.
pub struct ImageBitmap {
    /// The pixel size.
    size: Size,
    /// Buffer is shared, so that clone share the buffer.
    buffer: Arc<Vec<u8>>,
}

impl std::fmt::Debug for ImageBitmap {
    // implemented manually to skip dumping all the bytes.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("ImageBitmap")
            .field("buffer.len()", &self.buffer.len())
            .field("size", &self.size)
            .finish()
    }
}

impl ImageBitmap {
    pub fn new(buffer: Vec<u8>, w: u32, h: u32) -> Self {
        Self {
            size: Size { w, h },
            buffer: Arc::new(buffer),
        }
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
        let bytes = glib::Bytes::from_owned((*self.buffer).clone());
        gdk4::MemoryTexture::new(
            self.size.w as i32,
            self.size.h as i32,
            gdk4::MemoryFormat::B8g8r8a8,
            &bytes,
            (self.size.w * 4) as usize,
        )
        .into()
    }
}
