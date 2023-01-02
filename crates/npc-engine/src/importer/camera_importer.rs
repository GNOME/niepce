/*
 * niepce - npc-engine/src/importer/camera_importer.rs
 *
 * Copyright (C) 2017-2023 Hubert Figuière
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

use std::cell::RefCell;
use std::path::Path;

use super::{FileImporter, ImportedFile, Importer, PreviewReady, SourceContentReady};
use crate::ffi::Managed;
use npc_fwk::toolkit::{GpCamera, GpDeviceList};
use npc_fwk::utils::files::FileList;
use npc_fwk::{err_out, on_err_out};

#[derive(Clone, Default)]
pub struct CameraImportedFile {
    name: String,
    path: String,
    folder: String,
}

impl CameraImportedFile {
    pub fn new_dyn(folder: &str, name: &str) -> Box<dyn ImportedFile> {
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
}

#[derive(Default)]
pub struct CameraImporter {
    camera: RefCell<Option<GpCamera>>,
}

impl CameraImporter {
    fn ensure_camera_open(&self, source: &str) -> bool {
        let need_camera = {
            let camera_lock = self.camera.borrow();
            camera_lock.is_none() || camera_lock.as_ref().unwrap().path() != source
        };
        if need_camera {
            self.camera
                .replace(GpDeviceList::instance().device(source).map(GpCamera::new));
        }
        if let Some(camera) = &*self.camera.borrow() {
            camera.open();
            true
        } else {
            false
        }
    }

    /// List the content for the `camera` and return the list.
    fn list_content_for_camera(camera: &GpCamera) -> Vec<Box<dyn ImportedFile>> {
        camera
            .list_content()
            .iter()
            .map(|item| CameraImportedFile::new_dyn(&item.folder, &item.name))
            .collect()
    }
}

impl Importer for CameraImporter {
    fn id(&self) -> &'static str {
        "CameraImporter"
    }

    fn list_source_content(&self, source: &str, callback: SourceContentReady) {
        if self.ensure_camera_open(source) {
            if let Some(camera) = &*self.camera.borrow() {
                let file_list = Self::list_content_for_camera(camera);
                callback(file_list);
            }
        }
    }

    fn get_previews_for(&self, source: &str, paths: Vec<String>, callback: PreviewReady) {
        if self.ensure_camera_open(source) {
            paths.iter().for_each(|path| {
                if let Some(last_slash) = path.rfind('/') {
                    let name = &path[last_slash + 1..];
                    let folder = &path[..last_slash];
                    if let Some(thumbnail) = self
                        .camera
                        .borrow()
                        .as_ref()
                        .and_then(|camera| camera.get_preview(folder, name))
                    {
                        callback(path.to_string(), thumbnail);
                    }
                }
            });
        }
    }

    fn do_import(&self, source: &str, dest_dir: &Path, callback: FileImporter) {
        if self.ensure_camera_open(source) {
            if let Some(camera) = self.camera.borrow_mut().take() {
                let dest_dir = dest_dir.to_path_buf();
                std::thread::spawn(move || {
                    let file_list = Self::list_content_for_camera(&camera);
                    // XXX we likely need to handle this error better
                    on_err_out!(std::fs::create_dir_all(&dest_dir));
                    let files = file_list
                        .iter()
                        .filter_map(|file| {
                            let name = file.name();
                            let mut output_path = dest_dir.clone();
                            output_path.push(name);
                            if camera.download_file(
                                file.folder(),
                                name,
                                &output_path.to_string_lossy(),
                            ) {
                                return Some(output_path);
                            }

                            None
                        })
                        .collect();
                    callback(&dest_dir, &FileList(files), Managed::NO);
                });
            }
        }
    }
}
