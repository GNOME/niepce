/*
 * niepce - engine/importer/mod.rs
 *
 * Copyright (C) 2021-2023 Hubert Figuière
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

pub use camera_importer::CameraImporter;
pub use directory_importer::DirectoryImporter;
pub use imported_file::ImportedFile;
pub use libraryimporter::{LibraryImporter, LibraryImporterProbe};
pub use lrimporter::LrImporter;

use std::path::Path;

use crate::db::Managed;
use npc_fwk::toolkit::thumbnail::Thumbnail;
use npc_fwk::utils::files::FileList;

pub fn find_importer(path: &std::path::Path) -> Option<Box<dyn LibraryImporter>> {
    if LrImporter::can_import_library(path) {
        Some(Box::new(LrImporter::new()))
    } else {
        None
    }
}

type SourceContentReady = Box<dyn Fn(Vec<Box<dyn ImportedFile>>) + Send>;
type PreviewReady = Box<dyn Fn(String, Thumbnail) + Send>;
type FileImporter = Box<dyn Fn(&Path, &FileList, Managed) + Send>;

/// Trait for file importers.
pub trait Importer {
    /// ID of the importer.
    fn id(&self) -> &'static str;

    /// List the source content. If possible this should be spawning a thread. `callback`
    /// well be run on that thread.
    fn list_source_content(&self, source: &str, callback: SourceContentReady);
    /// Fetch the previews. If possible this should be spawning a thread. `callback`
    /// well be run on that thread.
    fn get_previews_for(&self, source: &str, paths: Vec<String>, callback: PreviewReady);

    /// Do the import
    fn do_import(&self, source: &str, dest_dir: &Path, callback: FileImporter);
}
