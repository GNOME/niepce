/*
 * niepce - npc_craw/pipeline/ncr.rs
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

/*! Niepce Camera Raw pipeline */

use std::cell::RefCell;

use glib::translate::*;

use npc_fwk::err_out;
use npc_fwk::toolkit::ImageBitmap;

/// Wrapper for the C++ ncr pipeline
pub(crate) struct NcrPipeline(RefCell<cxx::UniquePtr<crate::ImagePipeline>>);

impl NcrPipeline {
    pub(crate) fn new() -> Self {
        Self(RefCell::new(crate::ffi::image_pipeline_new()))
    }
}

impl super::Pipeline for NcrPipeline {
    fn output_width(&self) -> u32 {
        self.0.borrow().output_width() as u32
    }

    fn output_height(&self) -> u32 {
        self.0.borrow().output_height() as u32
    }

    fn rendered_image(&self) -> Option<ImageBitmap> {
        let w = self.output_width();
        let h = self.output_height();
        let mut buffer = vec![0; (w * h * 3) as usize];
        let success = self
            .0
            .borrow_mut()
            .pin_mut()
            .to_buffer(buffer.as_mut_slice());
        if success {
            Some(ImageBitmap::new(buffer, w, h))
        } else {
            err_out!("Failed to get buffer");
            None
        }
    }

    fn reload(&self, path: &str, is_raw: bool, orientation: i32) {
        cxx::let_cxx_string!(p = path);
        self.0
            .borrow_mut()
            .pin_mut()
            .reload(&p, is_raw, orientation);
    }

    fn set_placeholder(&self, placeholder: gdk_pixbuf::Pixbuf) {
        let p: *mut gdk_pixbuf_sys::GdkPixbuf = placeholder.to_glib_none().0;
        unsafe {
            self.0
                .borrow_mut()
                .pin_mut()
                .reload_pixbuf(p as *mut crate::ffi::GdkPixbuf);
        }
    }
}
