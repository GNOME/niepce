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

pub mod libraryimporter;
pub mod lrimporter;

pub use libraryimporter::{LibraryImporter, LibraryImporterProbe};
pub use lrimporter::LrImporter;

pub fn find_importer(path: &std::path::Path) -> Option<Box<dyn LibraryImporter>> {
    if LrImporter::can_import_library(path) {
        Some(Box::new(LrImporter::new()))
    } else {
        None
    }
}
