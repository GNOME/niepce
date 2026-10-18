/*
 * niepce - lib.rs
 *
 * Copyright (C) 2023-2026 Hubert Figuière
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

mod bridge;
mod image;
mod params;
mod processing;

use std::cell::RefCell;
use std::ffi::OsString;
use std::path::Path;
use std::sync::Once;

use num_enum::FromPrimitive;

use npc_fwk::toolkit::ImageBitmap;

pub use bridge::LcMode;
use bridge::ffi;

#[derive(Debug, thiserror::Error, FromPrimitive)]
#[repr(i32)]
pub enum RtError {
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
    // Libraw specific. See libraw_const.h
    #[error("LibRaw Unspecified")]
    LibRawUnspecified = -1,
    #[error("LibRaw file unsupported")]
    LibRawFileUnsupported = -2,
    #[error("LibRaw request for non existent image")]
    LibRawReqForNonExistentImage = -3,
    #[error("LibRaw out of order call")]
    LibRawOutOfOrderCall = -4,
    #[error("LibRaw no thumbnail")]
    LibRawNoThumbnail = -5,
    #[error("LibRaw unsupported thumbnail")]
    LibRawUnsupportedThumbnail = -6,
    #[error("LibRaw inpiut closed")]
    LibRawInputClosed = -7,
    #[error("LibRaw not implemented")]
    LibRawNotImplemented = -8,
    #[error("LibRaw request for non existent thunbnail")]
    LibRawReqForNonExistentThumbnail = -9,
    #[error("LibRaw Not enough memory")]
    LibRawUnsufficientMemory = -100007,
    #[error("LibRaw data error")]
    LibRawDataError = -100008,
    #[error("LibRaw IO error")]
    LibRawIoError = -100009,
    #[error("LibRaw cancelled")]
    LibRawCancelledByCallback = -100010,
    #[error("LibRaw bad crop")]
    LibRawBadCrop = -100011,
    #[error("LibRaw too big")]
    LibRawTooBig = -100012,
    #[error("LibRaw mem pool overflow")]
    LibRawMempoolOverflow = -100013,
    #[error("Unknown")]
    #[num_enum(default)]
    Unknown = 0xffffff,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("RawTherapee error {0}")]
    RtProcessing(#[from] RtError),
    #[error("C++ Exception {0}")]
    Exception(#[from] cxx::Exception),
    #[error("No image to process")]
    NoImage,
    #[error("Unknow Error")]
    Unknown,
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
    input_file: OsString,
    initial_image: Option<image::InitialImage>,
}

impl EngineState {
    fn new(initial_image: Option<image::InitialImage>, input_file: OsString) -> EngineState {
        Self {
            initial_image,
            input_file,
        }
    }
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
        let input_file = path.as_ref().as_os_str();
        let state = EngineState::new(
            Some(image::InitialImage::load(&path, is_raw)?),
            input_file.to_os_string(),
        );
        self.state.replace(Some(state));
        Ok(())
    }

    /// Process the image using rtengine and return an ImageBitmap
    /// Currently it uses the default profiles and enable `LcMode::LensFunAutoMatch`.
    pub fn process(&self) -> Result<ImageBitmap> {
        if let Some(ref mut state) = *self.state.borrow_mut()
            && let Some(ref mut image) = state.initial_image
        {
            let mut proc_params = params::ProcParams::new();
            let raw_params =
                params::ProfileStore::load_dynamic_profile(&image.meta_data(), &state.input_file);
            raw_params.apply_to(&mut proc_params, false);
            proc_params.set_lcmode(LcMode::LENSFUNAUTOMATCH);

            let job = processing::ProcessingJob::new(image, &proc_params, false);
            return job
                .process_image(false)
                .map(|image| image.to_image_bitmap());
        }
        Err(Error::NoImage)
    }
}
