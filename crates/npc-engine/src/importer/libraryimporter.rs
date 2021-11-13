/*
 * niepce - engine/importer/libraryimporter.rs
 *
 * Copyright (C) 2021 Hubert Figuière
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

use crate::libraryclient::LibraryClient;

/// Interface trait for a library importer.
pub trait LibraryImporter {
    /// Return a new library importer
    fn new() -> Self;

    /// import the library at path.
    /// if can_import_library returne false this should return false
    /// XXX return an actual Result<>
    fn import_library(&mut self, path: &Path, libclient: &mut LibraryClient) -> bool;

    /// Return true if or a given path the importer recognize the library
    fn can_import_library(path: &Path) -> bool;
}
