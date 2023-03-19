/*
 * niepce - processing.rs
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

use crate::ffi;
use crate::image;
use crate::params;
use crate::{Error, Result};

/// A processing job.
pub(crate) struct ProcessingJob(cxx::UniquePtr<ffi::ProcessingJob>);

impl ProcessingJob {
    /// New processing job for the `image` with `params`.
    pub fn new(
        image: &mut image::InitialImage,
        params: &params::ProcParams,
        fast: bool,
    ) -> ProcessingJob {
        ProcessingJob(ffi::processing_job_create(
            image.0.pin_mut(),
            &params.0,
            fast,
        ))
    }

    /// Run the job to process the image.
    pub fn process_image(mut self, flush: bool) -> Result<image::ImageIO> {
        let mut error = 0_i32;
        let mut job = cxx::UniquePtr::null();
        std::mem::swap(&mut self.0, &mut job);
        let job = job.into_raw();
        let image = unsafe { ffi::process_image(job, &mut error, flush) };
        if image.is_null() {
            unsafe { ffi::processing_job_destroy(job) };
            Err(Error::from(error))
        } else {
            Ok(image::ImageIO::new(image))
        }
    }
}

impl Drop for ProcessingJob {
    /// Properly drop the object.
    fn drop(&mut self) {
        if !self.0.is_null() {
            // The initial image must be ref uncounted.
            let mut job = cxx::UniquePtr::null();
            std::mem::swap(&mut self.0, &mut job);
            unsafe { ffi::processing_job_destroy(job.into_raw()) };
        }
    }
}
