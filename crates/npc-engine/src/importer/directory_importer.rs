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

use npc_fwk::utils::FileList;
use npc_fwk::{dbg_out, on_err_out, Date, XmpMeta};

use super::ImportedFile;
use crate::db::Managed;
use crate::importer::{FileImporter, ImportBackend, PreviewReady, SourceContentReady};

#[derive(Clone)]
pub struct DirectoryImportedFile {
    name: String,
    path: String,
    date: Option<Date>,
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
            date: None,
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

    fn date(&self) -> Option<&Date> {
        self.date.as_ref()
    }

    fn folder(&self) -> &str {
        unreachable!()
    }
}

#[derive(Default)]
pub struct DirectoryImporter {}

impl ImportBackend for DirectoryImporter {
    fn id(&self) -> &'static str {
        "DirectoryImporter"
    }

    /// List the source content
    fn list_source_content(&self, source: &str, callback: SourceContentReady) {
        let source = source.to_string();
        on_err_out!(std::thread::Builder::new()
            .name("dir import list source".to_string())
            .spawn(move || {
                let files = FileList::get_files_from_directory(source, FileList::file_is_media);
                dbg_out!("files size: {}", files.0.len());
                let content = files
                    .0
                    .iter()
                    .map(|path| DirectoryImportedFile::new_dyn(path))
                    .collect();

                callback(content);
            }));
    }

    /// Fetch the previews
    fn get_previews_for(&self, _source: &str, paths: Vec<String>, callback: PreviewReady) {
        on_err_out!(std::thread::Builder::new()
            .name("dir import get previews".to_string())
            .spawn(move || {
                for path in paths {
                    dbg_out!("path {}", path);
                    let thumbnail = npc_fwk::toolkit::Thumbnail::thumbnail_file(&path, 160, 160, 0);
                    let date =
                        XmpMeta::new_from_file(&path, false).and_then(|xmp| xmp.creation_date());
                    callback(path.to_string(), Some(thumbnail), date);
                }
            }));
    }

    /// Do the import
    fn do_import(&self, source: &str, _dest_dir: &Path, callback: FileImporter) {
        let files = FileList::get_files_from_directory(source, |_| true);
        callback(&std::path::PathBuf::from(source), &files, Managed::No);
    }
}