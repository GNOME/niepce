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

use super::{
    DirectoryImporter, FileImporter, ImportBackend, ImportRequest, ImportedFile, PreviewReady,
    SourceContentReady,
};
use npc_fwk::toolkit::{GpCamera, GpDeviceList};
use npc_fwk::utils::FileList;
use npc_fwk::{Date, dbg_out, on_err_out};

#[derive(Clone, Default)]
pub struct CameraImportedFile {
    name: String,
    path: String,
    date: Option<Date>,
    folder: String,
}

impl CameraImportedFile {
    pub fn new_dyn(folder: &str, name: &str) -> Box<dyn ImportedFile> {
        Box::new(CameraImportedFile {
            folder: folder.to_string(),
            name: name.to_string(),
            date: None,
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

    fn date(&self) -> Option<&Date> {
        self.date.as_ref()
    }

    fn folder(&self) -> &str {
        &self.folder
    }
}

/// The kind of backend
enum CameraBackend {
    /// Gphoto2
    Gphoto2,
    /// Directory importer
    File,
    /// Error
    Error,
}

#[derive(Default)]
pub struct CameraImporter {
    camera: RefCell<Option<GpCamera>>,
    file_backend: RefCell<Option<DirectoryImporter>>,
}

impl CameraImporter {
    /// Ensure the camera is open. If it's a mass storage device
    /// it will create the file backend instead. It will return the
    /// type of backend.
    fn ensure_camera_open(&self, source: &str) -> CameraBackend {
        if GpCamera::port_is_disk(source) {
            dbg_out!("Using file backend");
            if self.file_backend.borrow().is_none() {
                dbg_out!("Created file backend");
                let mut backend = DirectoryImporter::default();
                backend.set_copy(true);
                backend.set_recursive(true);
                *self.file_backend.borrow_mut() = Some(backend);
            }
            return CameraBackend::File;
        }
        let need_camera = {
            let camera_lock = self.camera.borrow();
            camera_lock.is_none() || camera_lock.as_ref().unwrap().path() != source
        };
        if need_camera {
            self.camera
                .replace(GpDeviceList::instance().device(source).map(GpCamera::new));
        }
        if let Some(camera) = &mut *self.camera.borrow_mut() {
            camera.open();
            CameraBackend::Gphoto2
        } else {
            CameraBackend::Error
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

impl ImportBackend for CameraImporter {
    fn id(&self) -> &'static str {
        "CameraImporter"
    }

    fn list_source_content(&self, source: &str, callback: SourceContentReady) {
        match self.ensure_camera_open(source) {
            CameraBackend::Gphoto2 => {
                if let Some(ref camera) = *self.camera.borrow() {
                    let file_list = Self::list_content_for_camera(camera);
                    callback(file_list);
                }
            }
            CameraBackend::File => {
                if let Some(ref backend) = *self.file_backend.borrow() {
                    backend.list_source_content(&source[5..], callback);
                }
            }
            CameraBackend::Error => {}
        }
    }

    fn get_previews_for(&self, source: &str, paths: Vec<String>, callback: PreviewReady) {
        match self.ensure_camera_open(source) {
            CameraBackend::Gphoto2 => {
                paths.iter().for_each(|path| {
                    if let Some(last_slash) = path.rfind('/') {
                        let name = &path[last_slash + 1..];
                        let folder = &path[..last_slash];
                        let thumbnail = self
                            .camera
                            .borrow()
                            .as_ref()
                            .and_then(|camera| camera.get_preview(folder, name));

                        if thumbnail.is_some() {
                            callback(path.to_string(), thumbnail, None);
                        }
                    }
                });
            }
            CameraBackend::File => {
                if let Some(ref backend) = *self.file_backend.borrow() {
                    backend.get_previews_for(&source[5..], paths, callback)
                }
            }
            CameraBackend::Error => {}
        }
    }

    fn do_import(&self, request: &ImportRequest, callback: FileImporter) {
        match self.ensure_camera_open(request.source()) {
            CameraBackend::Gphoto2 => {
                if let Some(camera) = self.camera.borrow_mut().take() {
                    let dest_dir = request.dest_dir().to_path_buf();
                    on_err_out!(
                        std::thread::Builder::new()
                            .name("camera import".to_string())
                            .spawn(move || {
                                let file_list = Self::list_content_for_camera(&camera);
                                // XXX we likely need to handle this error better
                                on_err_out!(std::fs::create_dir_all(&dest_dir));
                                let files = file_list
                                    .iter()
                                    .filter_map(|file| {
                                        let name = file.name();
                                        let output_path = dest_dir.join(name);
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
                                callback(&FileList(files));
                            })
                    );
                }
            }
            CameraBackend::File => {
                if let Some(backend) = self.file_backend.borrow_mut().take() {
                    let source = &request.source()[5..];
                    let request = request.clone().set_source(source);
                    backend.do_import(&request, callback);
                }
            }
            CameraBackend::Error => {}
        }
    }
}
