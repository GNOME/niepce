/*
 * niepce - image.rs
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

use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::ffi;
use crate::{Error, Result};

use npc_fwk::toolkit::ImageBitmap;

/// Image output, result of processing.
pub(crate) struct ImageIO(cxx::UniquePtr<ffi::ImageIO>);

impl ImageIO {
    /// Create a new ImageIO from the ffi.
    pub fn new(image: cxx::UniquePtr<ffi::ImageIO>) -> ImageIO {
        ImageIO(image)
    }

    /// Width
    fn width(&self) -> i32 {
        ffi::image_io_width(&self.0)
    }

    /// Height
    fn height(&self) -> i32 {
        ffi::image_io_height(&self.0)
    }

    /// Get a scanline of 8 bit per sample for row `idx` into `row`.
    fn scanline(&self, idx: i32, row: &mut [u8]) {
        unsafe { self.0.scanline(idx, row.as_mut_ptr(), 8, false) };
    }

    /// Convert to an image bitmap.
    pub(crate) fn to_image_bitmap(&self) -> ImageBitmap {
        let w = self.width();
        let h = self.height();

        let stride = w as usize * 3;
        let mut buffer = vec![0_u8; stride * h as usize];
        let b = buffer.as_mut_slice();
        for idx in 0..h {
            let b = &mut b[idx as usize * stride..];
            self.scanline(idx, b);
        }
        ImageBitmap::new(buffer, w as u32, h as u32)
    }
}

/// Metadata for the image.
pub(crate) struct FramesMetaData(pub *const ffi::FramesMetaData);

/// InitialImage is carries the information about the source image.
pub(crate) struct InitialImage(pub cxx::UniquePtr<ffi::InitialImage>);

impl InitialImage {
    /// Load an image. `is_raw` will use a RAW loader.
    pub fn load<P>(path: P, is_raw: bool) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        cxx::let_cxx_string!(fname = path.as_ref().as_os_str().as_bytes());
        let mut err = 0_i32;
        let image = ffi::initial_image_load(&fname, is_raw, &mut err);
        if !image.is_null() {
            Ok(Self(image))
        } else {
            Err(Error::from(err))
        }
    }

    /// Get the metadata for the image.
    pub fn meta_data(&mut self) -> FramesMetaData {
        FramesMetaData(self.0.pin_mut().get_meta_data())
    }
}

impl Drop for InitialImage {
    /// Properly drop the object, the ffi use ref counting
    /// We can't clone the type automatically.
    fn drop(&mut self) {
        if !self.0.is_null() {
            // The initial image must be ref uncounted.
            let mut image = cxx::UniquePtr::null();
            std::mem::swap(&mut self.0, &mut image);
            unsafe { ffi::decrease_ref(image.into_raw()) };
        }
    }
}
