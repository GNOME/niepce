/*
 * niepce - npc-engine/src/importer/directory_importer.rs
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

#[derive(Clone)]
pub struct DirectoryImportedFile {
    name: String,
    path: String,
}

impl DirectoryImportedFile {
    pub fn new(path: &str) -> Box<Self> {
        let path = std::path::PathBuf::from(path);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        Box::new(Self {
            name,
            path: path.to_str().unwrap_or("").to_owned(),
        })
    }
}

impl ImportedFile for DirectoryImportedFile {
    fn name(&self) -> &str {
        &self.name
    }

    fn path(&self) -> &str {
        &self.path
    }

    fn folder(&self) -> &str {
        unreachable!()
    }

    fn clone_(&self) -> Box<dyn ImportedFile> {
        Box::new(self.clone())
    }
}

pub fn directory_imported_file_new(path: &str) -> Box<WrappedImportedFile> {
    Box::new(WrappedImportedFile(DirectoryImportedFile::new(path), false))
}
