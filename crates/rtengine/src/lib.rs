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

use std::cell::RefCell;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::Once;

use npc_fwk::toolkit::ImageBitmap;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cannot read file")]
    CannotReadFile = 1,
    #[error("Invalid header")]
    InvalidHeader = 2,
    #[error("Header error")]
    HeaderError = 3,
    #[error("Reader error")]
    ReadError = 4,
    #[error("Variant Not Supported")]
    VariantNotSupported = 5,
    #[error("File Type Not Supported")]
    FileTypeNotSupported = 6,
    #[error("Cannot Write File")]
    CannotWriteFile = 7,

    #[error("No image to process")]
    NoImage,
    #[error("Unknow Error")]
    Unknown,
}

impl From<i32> for Error {
    fn from(v: i32) -> Error {
        use Error::*;
        match v {
            1 => CannotReadFile,
            2 => InvalidHeader,
            3 => HeaderError,
            4 => ReadError,
            5 => VariantNotSupported,
            6 => FileTypeNotSupported,
            7 => CannotWriteFile,
            _ => Unknown,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn init_engine() {
    static INIT_ENGINE: Once = Once::new();

    INIT_ENGINE.call_once(|| {
        ffi::init_();
        ffi::options_load();
    });
}

#[derive(Default)]
struct EngineState {
    image_path: PathBuf,
    initial_image: Option<cxx::UniquePtr<ffi::InitialImage>>,
}

impl Drop for EngineState {
    /// Properly drop the object.
    fn drop(&mut self) {
        if let Some(image) = self.initial_image.take() {
            // The initial image must be ref uncounted.
            unsafe { ffi::decrease_ref(image.into_raw()) };
        }
    }
}

fn into_image_bitmap(image: cxx::UniquePtr<ffi::ImageIO>) -> ImageBitmap {
    let w = ffi::image_io_width(&image);
    let h = ffi::image_io_height(&image);

    let stride = w as usize * 3;
    let mut buffer = vec![0_u8; stride * h as usize];
    let b = buffer.as_mut_slice();
    for idx in 0..h {
        let ptr = b[idx as usize * stride..].as_mut_ptr();
        //println!("row = {}, ptr {:?}", idx, ptr);
        unsafe { image.scanline(idx, ptr, 8, false) };
    }
    ImageBitmap::new(buffer, w as u32, h as u32)
}

/// RawTherapee rendering engine
pub struct RtEngine {
    state: RefCell<Option<EngineState>>,
}

impl Default for RtEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RtEngine {
    /// Create a new engine. Will init anything as needed.
    pub fn new() -> RtEngine {
        init_engine();

        RtEngine {
            state: RefCell::new(None),
        }
    }

    pub fn width(&self) -> i32 {
        0
    }

    pub fn height(&self) -> i32 {
        0
    }

    /// Set the file process
    pub fn set_file<P>(&self, path: P, is_raw: bool) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let mut state = EngineState::default();
        state.image_path = path.as_ref().to_path_buf();
        let mut error = 0_i32;
        cxx::let_cxx_string!(fname = state.image_path.as_os_str().as_bytes());
        let image = ffi::initial_image_load(&fname, is_raw, &mut error);
        if !image.is_null() {
            state.initial_image = Some(image);
            self.state.replace(Some(state));
            Ok(())
        } else {
            Err(Error::from(error))
        }
    }

    /// Process the image using rtengine and return an ImageBitmap
    /// Currently it uses the default profiles and enable `LcMode::LensFunAutoMatch`.
    pub fn process(&self) -> Result<ImageBitmap> {
        if let Some(ref mut state) = *self.state.borrow_mut() {
            if let Some(ref mut image) = state.initial_image {
                let mut proc_params = ffi::proc_params_new();
                let mut raw_params = unsafe {
                    ffi::profile_store_load_dynamic_profile(image.pin_mut().get_meta_data())
                };
                ffi::partial_profile_apply_to(&raw_params, proc_params.pin_mut(), false);
                raw_params.pin_mut().delete_instance();
                ffi::proc_params_set_lcmode(proc_params.pin_mut(), ffi::LcMode::LensFunAutoMatch);

                let job = ffi::processing_job_create(
                    image.pin_mut(),
                    proc_params.as_ref().unwrap(),
                    false,
                );
                let mut error = 0_i32;
                // Warning: unless there is an error, process_image will consume it.
                let job = job.into_raw();
                let imagefloat = unsafe { ffi::process_image(job, &mut error, false) };

                if imagefloat.is_null() {
                    // Only in case of error.
                    unsafe { ffi::processing_job_destroy(job) };
                    return Err(Error::from(error));
                }
                return Ok(into_image_bitmap(imagefloat));
            }
        }
        Err(Error::NoImage)
    }
}

#[cxx::bridge(namespace = "rtengine")]
mod ffi {
    /// Lens correction mode. Should match procparams::LcMode
    #[repr(i32)]
    enum LcMode {
        /// No correction.
        None = 0,
        /// Match automatically from LensFun.
        LensFunAutoMatch = 1,
        /// Manual select a LensFun entry.
        LensFunManual = 2,
        /// Use LCP file.
        Lcp = 3,
    }

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
