/*
 * niepce - ncr/image_bitmap.rs
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

use npc_fwk::base::Size;

use crate::ImageStatus;

#[derive(Default)]
pub struct ImageBitmap {
    size: Size,
    buffer: Vec<u8>,
}

impl ImageBitmap {
    pub fn new(buffer: Vec<u8>, w: u32, h: u32) -> Self {
        Self {
            size: Size { w, h },
            buffer,
        }
    }

    pub fn status(&self) -> ImageStatus {
        ImageStatus::LOADED
    }

    pub fn original_width(&self) -> u32 {
        self.size.w
    }

    pub fn original_height(&self) -> u32 {
        self.size.h
    }

    pub fn to_gdk_texture(&self) -> gdk4::Texture {
        let bytes = glib::Bytes::from_owned(self.buffer.clone());
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
