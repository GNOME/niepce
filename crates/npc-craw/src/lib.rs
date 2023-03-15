/*
 * niepce - npc-craw/lib.rs
 *
 * Copyright (C) 2023 Hubert Figuière
 *
 * This program is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 3 of the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this program; if not, see
 * <http://www.gnu.org/licenses/>.
 */

mod pipeline;
mod render_worker;

use std::sync::Once;

pub use render_worker::{RenderImpl, RenderWorker};

fn ncr_init() {
    static START: Once = Once::new();

    START.call_once(|| {
        crate::ffi::init();
    });
}

#[cxx::bridge(namespace = "ncr")]
mod ffi {
    #[namespace = ""]
    unsafe extern "C++" {
        include!(<gdk-pixbuf/gdk-pixbuf.h>);
        include!(<gdk/gdk.h>);

        type GdkPixbuf;
        type GdkTexture;
    }

    #[rust_name = "ImageStatus"]
    #[derive(Debug, PartialOrd)]
    enum Status {
        UNSET = 0,
        LOADING,
        LOADED,
        ERROR,
        NOT_FOUND,
    }

    unsafe extern "C++" {
        include!("ncr/init.hpp");
        fn init();
    }

    unsafe extern "C++" {
        include!("ncr/image.hpp");

        #[cxx_name = Image]
        type ImagePipeline;

        #[cxx_name = "Image_new"]
        fn image_pipeline_new() -> UniquePtr<ImagePipeline>;
        #[cxx_name = "get_status"]
        fn status(&self) -> ImageStatus;
        #[cxx_name = "get_original_width"]
        fn original_width(&self) -> i32;
        #[cxx_name = "get_original_height"]
        fn original_height(&self) -> i32;

        #[cxx_name = "get_output_width"]
        fn output_width(&self) -> i32;
        #[cxx_name = "get_output_height"]
        fn output_height(&self) -> i32;

        fn set_output_scale(self: Pin<&mut ImagePipeline>, scale: f64);
        fn to_buffer(self: Pin<&mut ImagePipeline>, buffer: &mut [u8]) -> bool;

        fn reload(self: Pin<&mut ImagePipeline>, path: &CxxString, is_raw: bool, orientation: i32);
        #[cxx_name = "reload_pixbuf_"]
        /// # Safety
        /// Derefence pointers.
        unsafe fn reload_pixbuf(self: Pin<&mut ImagePipeline>, p: *mut GdkPixbuf);
        /// # Safety
        /// Derefence pointers.
        unsafe fn connect_signal_update(
            self: Pin<&mut ImagePipeline>,
            callback: unsafe fn(*const u8),
            userdata: *const u8,
        );
    }
}

pub use ffi::{ImagePipeline, ImageStatus};
