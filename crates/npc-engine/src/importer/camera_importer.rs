/*
 * niepce - npc-engine/src/importer/camera_importer.rs
 *
 * Copyright (C) 2022 Hubert Figuière
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

use super::imported_file::WrappedImportedFile;
use super::ImportedFile;

#[derive(Clone, Default)]
pub struct CameraImportedFile {
    name: String,
    path: String,
    folder: String,
}

impl CameraImportedFile {
    pub fn new(folder: &str, name: &str) -> Box<Self> {
        Box::new(CameraImportedFile {
            folder: folder.to_string(),
            name: name.to_string(),
            path: folder.to_string() + "/" + name,
        })
    }
}

impl ImportedFile for CameraImportedFile {
    fn name(&self) -> &str {
        &self.name
    }

    fn path(&self) -> &str {
        &self.path
    }

    fn folder(&self) -> &str {
        &self.folder
    }

    fn clone_(&self) -> Box<dyn ImportedFile> {
        Box::new(self.clone())
    }
}

pub fn camera_imported_file_new(folder: &str, name: &str) -> Box<WrappedImportedFile> {
    Box::new(WrappedImportedFile(
        CameraImportedFile::new(folder, name),
        true,
    ))
}
