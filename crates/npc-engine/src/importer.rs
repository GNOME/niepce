/*
 * niepce - engine/importer/mod.rs
 *
 * Copyright (C) 2021-2022 Hubert Figuière
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

mod camera_importer;
mod directory_importer;
mod imported_file;
pub mod libraryimporter;
pub mod lrimporter;

pub use imported_file::ImportedFile;
pub use libraryimporter::{LibraryImporter, LibraryImporterProbe};
pub use lrimporter::LrImporter;

pub mod cxx {
    pub use super::camera_importer::camera_imported_file_new;
    pub use super::directory_importer::directory_imported_file_new;
    pub use super::imported_file::WrappedImportedFile;
}

use std::path::Path;

use crate::ffi::Managed;
use npc_fwk::toolkit::thumbnail::Thumbnail;
use npc_fwk::utils::files::FileList;

pub fn find_importer(path: &std::path::Path) -> Option<Box<dyn LibraryImporter>> {
    if LrImporter::can_import_library(path) {
        Some(Box::new(LrImporter::new()))
    } else {
        None
    }
}

type SourceContentReady = Box<dyn Fn(Vec<Box<dyn ImportedFile>>)>;
type PreviewReady = Box<dyn Fn(String, Thumbnail)>;
type FileImporter = Box<dyn Fn(&Path, &FileList, Managed)>;

/// Trait for file importers.
pub trait Importer {
    /// ID of the importer.
    fn id(&self) -> &str;

    /// List the source content
    fn list_source_content(&self, source: &str, callback: SourceContentReady);
    /// Fetch the previews
    fn get_previews_for(&self, source: &str, paths: &[String], callback: PreviewReady);

    /// Do the import
    fn do_import(&self, source: &str, dest_dir: &Path, callback: FileImporter);
}
