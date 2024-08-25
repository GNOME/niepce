/*
 * niepce - bridge.rs
 *
 * Copyright (C) 2023-2024 Hubert Figuière
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

#[cxx::bridge(namespace = "rtengine")]
pub(crate) mod ffi {
    #[namespace = "rtengine::procparams"]
    #[repr(i32)]
    enum LcMode {
        /// No correction.
        #[cxx_name = "NONE"]
        None,
        /// Match automatically from LensFun.
        #[cxx_name = "LENSFUNAUTOMATCH"]
        LensFunAutoMatch,
        /// Manual select a LensFun entry.
        #[cxx_name = "LENSFUNMANUAL"]
        LensFunManual,
        /// Use LCP file.
        #[cxx_name = "LCP"]
        Lcp,
    }

    #[namespace = "rtengine::procparams"]
    extern "C++" {
        include!("npc_rtengine.h");
        type LcMode;
    }

    extern "C++" {
        type FramesMetaData;
    }

    unsafe extern "C++" {
        include!("npc_rtengine.h");
        type InitialImage;

        fn init_();
        #[cxx_name = "Options_load"]
        fn options_load();
        #[cxx_name = "InitialImage_load"]
        fn initial_image_load(
            input_file: &CxxString,
            is_raw: bool,
            error_code: &mut i32,
        ) -> UniquePtr<InitialImage>;
        #[cxx_name = "getMetaData"]
        fn get_meta_data(&self) -> *const FramesMetaData;
        /// Takes ownership
        unsafe fn decrease_ref(image: *mut InitialImage);
    }

    unsafe extern "C++" {
        type ProcessingJob;
        type ImageIO;

        #[cxx_name = "ProcessingJob_create"]
        fn processing_job_create(
            image: Pin<&mut InitialImage>,
            procparams: &ProcParams,
            fast: bool,
        ) -> UniquePtr<ProcessingJob>;
        #[cxx_name = "ProcessingJob_destroy"]
        /// # Safety
        /// Dereference a pointer.
        unsafe fn processing_job_destroy(job: *mut ProcessingJob);
        /// Processs the inage. Takes ownership of `job`.
        /// Returns null in case of error.
        /// # Safety
        /// Dereference a pointer.
        unsafe fn process_image(
            job: *mut ProcessingJob,
            error_code: &mut i32,
            flush: bool,
        ) -> UniquePtr<ImageIO>;
        #[cxx_name = "ProfileStore_load_dynamic_profile"]
        /// # Safety
        /// Dereference a pointer.
        unsafe fn profile_store_load_dynamic_profile(
            metadata: *const FramesMetaData,
            input_file: &CxxString,
        ) -> UniquePtr<PartialProfile>;
        fn image_io_width(image: &ImageIO) -> i32;
        fn image_io_height(image: &ImageIO) -> i32;
        #[cxx_name = "getScanline"]
        unsafe fn scanline(self: &ImageIO, idx: i32, row: *mut u8, bps: i32, is_float: bool);
    }

    #[namespace = "rtengine::procparams"]
    unsafe extern "C++" {
        type PartialProfile;

        fn partial_profile_apply_to(
            profile: &UniquePtr<PartialProfile>,
            procparams: Pin<&mut ProcParams>,
            from_last_saved: bool,
        );
        #[cxx_name = "deleteInstance"]
        /// Delete the inner data before dropping the object.
        /// Otherwise things will leak.
        fn delete_instance(self: Pin<&mut PartialProfile>);
    }

    #[namespace = "rtengine::procparams"]
    unsafe extern "C++" {
        type ProcParams;

        #[cxx_name = "ProcParams_new"]
        fn proc_params_new() -> UniquePtr<ProcParams>;
        #[cxx_name = "ProcParams_set_lcmode"]
        /// Set the lens correction mode.
        fn proc_params_set_lcmode(params: Pin<&mut ProcParams>, mode: LcMode);
    }
}
