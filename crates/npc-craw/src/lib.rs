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

#[cxx::bridge(namespace = "ncr")]
mod ffi {
    #[namespace = ""]
    unsafe extern "C++" {
        include!(<gdk/gdk.h>);

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
        include!("ncr/image.hpp");

        type Image;

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

        #[cxx_name = "set_output_scale_"]
        fn set_output_scale(&self, scale: f64);
        #[cxx_name = "to_gdk_texture_"]
        fn to_gdk_texture(&self) -> *mut GdkTexture;

        /// # Safety
        /// Derefence pointers.
        unsafe fn connect_signal_update(&self, callback: unsafe fn(*const u8), userdata: *const u8);
    }

    impl SharedPtr<Image> {}
}

pub use ffi::{Image, ImageStatus};
