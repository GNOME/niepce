/*
 * niepce - params.rs
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

use crate::LcMode;
use crate::ffi;
use crate::image;

use std::ffi::OsStr;

/// Processing parameters.
pub(crate) struct ProcParams(pub cxx::UniquePtr<ffi::ProcParams>);

impl ProcParams {
    /// New empty ProcParams
    pub fn new() -> ProcParams {
        ProcParams(ffi::proc_params_new())
    }

    /// Set the lens correction mode.
    pub fn set_lcmode(&mut self, mode: LcMode) {
        ffi::proc_params_set_lcmode(self.0.pin_mut(), mode)
    }
}

/// Partial process parameters from a profile.
pub(crate) struct PartialProfile(cxx::UniquePtr<ffi::PartialProfile>);

impl PartialProfile {
    /// Apply the partial profile to `params`.
    pub fn apply_to(&self, params: &mut ProcParams, from_last_saved: bool) {
        ffi::partial_profile_apply_to(&self.0, params.0.pin_mut(), from_last_saved);
    }
}

impl Drop for PartialProfile {
    fn drop(&mut self) {
        // delete_instance must be called to free the internal data.
        self.0.pin_mut().delete_instance();
    }
}

/// Access the profile store singleton
pub(crate) struct ProfileStore {}

impl ProfileStore {
    /// Load a dynamic profile based on the metadata.
    pub fn load_dynamic_profile(
        metadata: &image::FramesMetaData,
        filename: &OsStr,
    ) -> PartialProfile {
        cxx::let_cxx_string!(input_file = filename.as_encoded_bytes());
        PartialProfile(unsafe { ffi::profile_store_load_dynamic_profile(metadata.0, &input_file) })
    }
}
