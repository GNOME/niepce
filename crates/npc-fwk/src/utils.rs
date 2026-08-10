/*
 * niepce - npc-fwk/utils.rs
 *
 * Copyright (C) 2017-2026 Hubert Figuière
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

pub mod exempi;
pub mod exif;
mod files;

pub use files::{FileList, copy, normalize_for_display, trim_trailing_path_sep};

#[cfg(test)]
pub(crate) mod tests {
    use std::path::PathBuf;

    /// get the test sample path for testing.
    pub(crate) fn get_test_sample_path() -> PathBuf {
        use std::env;

        let mut dir: PathBuf;
        if let Ok(pdir) = env::var("CARGO_MANIFEST_DIR") {
            dir = PathBuf::from(pdir);
            dir.push("src");
            dir.push("utils");
        } else {
            dir = PathBuf::from(".");
        }
        dir
    }
}
