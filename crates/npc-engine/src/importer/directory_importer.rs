/*
 * niepce - npc-engine/src/importer/directory_importer.rs
 *
 * Copyright (C) 2022-2023 Hubert Figuière
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

use npc_fwk::dbg_out;
use npc_fwk::utils::files::FileList;

use super::ImportedFile;
use crate::ffi::Managed;
use crate::importer::{FileImporter, Importer, PreviewReady, SourceContentReady};

#[derive(Clone)]
pub struct DirectoryImportedFile {
    name: String,
    path: String,
}

impl DirectoryImportedFile {
    pub fn new_dyn(path: &Path) -> Box<dyn ImportedFile> {
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
}

#[derive(Default)]
pub struct DirectoryImporter {}

impl Importer for DirectoryImporter {
    fn id(&self) -> &'static str {
        "DirectoryImporter"
    }

    /// List the source content
    fn list_source_content(&self, source: &str, callback: SourceContentReady) {
        let source = source.to_string();
        std::thread::spawn(move || {
            let files = FileList::get_files_from_directory(source, FileList::file_is_media);
            dbg_out!("files size: {}", files.0.len());
            let content = files
                .0
                .iter()
                .map(|path| DirectoryImportedFile::new_dyn(path))
                .collect();

            callback(content);
        });
    }

    /// Fetch the previews
    fn get_previews_for(&self, _source: &str, paths: Vec<String>, callback: PreviewReady) {
        std::thread::spawn(move || {
            for path in paths {
                dbg_out!("path {}", path);
                let thumbnail = npc_fwk::toolkit::Thumbnail::thumbnail_file(&path, 160, 160, 0);
                callback(path.to_string(), thumbnail);
            }
        });
    }

    /// Do the import
    fn do_import(&self, source: &str, _dest_dir: &Path, callback: FileImporter) {
        let files = FileList::get_files_from_directory(source, |_| true);
        callback(&std::path::PathBuf::from(source), &files, Managed::NO);
    }
}
