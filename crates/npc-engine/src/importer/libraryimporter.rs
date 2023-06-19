/*
 * niepce - engine/importer/libraryimporter.rs
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

use std::path::Path;

use thiserror::Error;

use crate::libraryclient::LibraryClient;

#[derive(Error, Debug)]
pub enum Error {
    /// The format is unsupported. This is specific to the implementation.
    #[error("Unsupported format")]
    UnsupportedFormat,
    /// There is no input available: usually an error opening the
    /// input library that is not `UnsupportedFormat`. The latter
    /// can't happen with this condition.
    #[error("No input")]
    NoInput,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Interface trait for a library importer.
///
/// Once constructed, call `init_importer` with a `Path`.
/// Call `import_library` to do the import.
pub trait LibraryImporter {
    /// Initiatlize the importer.
    fn init_importer(&mut self, path: &Path) -> Result<()>;

    /// import the library at path.
    /// if can_import_library returned false this should return an error
    fn import_library(&mut self, libclient: &LibraryClient) -> Result<()>;

    /// Return the root folders. They can then me remapped using `map_root_folder`.
    fn root_folders(&mut self) -> Vec<String>;

    /// Map a root folder a new destination.
    fn map_root_folder(&mut self, orig: &str, dest: &str);

    /// The name of the importer. Should be the name of the original application.
    /// XXX see about localizing this.
    fn name(&self) -> &'static str;
}

///
/// Trait for probing importers.
///
pub trait LibraryImporterProbe: LibraryImporter {
    /// Return true if or a given path the importer recognize the library
    fn can_import_library(path: &Path) -> bool;
}
