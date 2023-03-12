/*
 * niepce - lib.rs
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

#[cxx::bridge(namespace = "rtengine")]
pub mod ffi {
    unsafe extern "C++" {
        type ProgressListener;
    }

    unsafe extern "C++" {
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
        fn get_meta_data(self: Pin<&mut InitialImage>) -> *const FramesMetaData;
        /// Takes ownership
        unsafe fn decrease_ref(image: *mut InitialImage);
    }

    unsafe extern "C++" {
        type ProcessingJob;
        type IImagefloat;

        #[cxx_name = "ProcessingJob_create"]
        fn processing_job_create(
            image: Pin<&mut InitialImage>,
            procparams: &ProcParams,
            fast: bool,
        ) -> UniquePtr<ProcessingJob>;
        /// Processs the inage. Takes ownership of `job`.
        /// Returns null in case of error.
        /// # Safety
        /// Dereference a pointer.
        unsafe fn process_image(
            job: *mut ProcessingJob,
            error_code: &mut i32,
            flush: bool,
        ) -> UniquePtr<IImagefloat>;
        #[cxx_name = "ProfileStore_load_dynamic_profile"]
        /// # Safety
        /// Dereference a pointer.
        unsafe fn profile_store_load_dynamic_profile(
            metadata: *const FramesMetaData,
        ) -> UniquePtr<PartialProfile>;
        // Default 100, 3
        fn save_as_jpeg(
            image: &UniquePtr<IImagefloat>,
            file: &CxxString,
            compression: i32,
            subsampling: i32,
        ) -> i32;
    }

    #[namespace = "rtengine::procparams"]
    unsafe extern "C++" {
        type PartialProfile;

        fn partial_profile_apply_to(
            profile: &UniquePtr<PartialProfile>,
            procparams: Pin<&mut ProcParams>,
            from_last_saved: bool,
        );
    }
    #[namespace = "rtengine::procparams"]
    unsafe extern "C++" {
        type ProcParams;

        #[cxx_name = "ProcParams_new"]
        fn proc_params_new() -> UniquePtr<ProcParams>;
    }
}
