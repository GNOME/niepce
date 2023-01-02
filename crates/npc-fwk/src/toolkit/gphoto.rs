/*
 * niepce - npc_fwk/toolkit/gphoto.rs
 *
 * Copyright (C) 2009-2023 Hubert Figuière
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

use std::sync::{Mutex, RwLock, RwLockReadGuard};

use gdk_pixbuf::prelude::*;

#[derive(Clone)]
/// This is like gphoto2::CameraDescriptor
pub struct GpDevice {
    model: String,
    path: String,
}

impl From<gphoto2::list::CameraDescriptor> for GpDevice {
    fn from(desc: gphoto2::list::CameraDescriptor) -> GpDevice {
        GpDevice {
            model: desc.model,
            path: desc.port,
        }
    }
}

impl GpDevice {
    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

lazy_static::lazy_static! {
    static ref DEVICE_LIST: GpDeviceList = GpDeviceList::default();
}

pub struct GpDeviceList {
    context: Mutex<gphoto2::Context>,
    list: RwLock<Vec<GpDevice>>,
}

impl Default for GpDeviceList {
    fn default() -> GpDeviceList {
        GpDeviceList {
            context: Mutex::new(gphoto2::Context::new().expect("Coudln't initialise gphoto2")),
            list: RwLock::new(vec![]),
        }
    }
}

impl GpDeviceList {
    pub fn instance() -> &'static GpDeviceList {
        &DEVICE_LIST
    }

    pub fn list(&self) -> RwLockReadGuard<'_, Vec<GpDevice>> {
        self.list.read().unwrap()
    }

    pub fn detect(&self) {
        dbg_out!("Detecting cameras");
        let task = self.context.lock().unwrap().list_cameras();
        *self.list.write().unwrap() = if let Ok(camera_list) = task.wait() {
            camera_list.map(GpDevice::from).collect()
        } else {
            err_out!("error detecting cameras");
            vec![]
        };
    }

    pub fn device(&self, source: &str) -> Option<GpDevice> {
        self.list
            .read()
            .unwrap()
            .iter()
            .find(|d| d.path() == source)
            .cloned()
    }
}

pub struct GpCamera {
    device: GpDevice,
    camera: RwLock<Option<gphoto2::Camera>>,
}

impl GpCamera {
    pub fn new(device: GpDevice) -> GpCamera {
        GpCamera {
            device,
            camera: RwLock::new(None),
        }
    }

    pub fn open(&self) -> bool {
        dbg_out!("opening camera {}", self.device.path());
        if self.camera.read().unwrap().is_some() {
            self.close();
        }

        let desc = gphoto2::list::CameraDescriptor {
            model: self.device.model().to_string(),
            port: self.device.path().to_string(),
        };
        let task = DEVICE_LIST.context.lock().unwrap().get_camera(&desc);
        let camera = task.wait();
        // XXX handle errors
        match camera {
            Ok(c) => {
                let task = c.fs().file_info("/", "DCIM");
                if let Err(err) = task.wait() {
                    dbg_out!("file info returned an error, of course {:?}", err);
                    if err.kind() == gphoto2::error::ErrorKind::IoUsbClaim {
                        dbg_out!("Trying to unmount the camera...");
                        self.try_unmount_camera();
                    }
                }
                *self.camera.write().unwrap() = Some(c);
                dbg_out!("Camera open and initialized");
                true
            }
            Err(err) => {
                err_out!("Error initializing camera {:?}", err);
                false
            }
        }
    }

    pub fn path(&self) -> &str {
        &self.device.path
    }

    fn process_folders(&self, folders: Vec<String>) -> Vec<crate::ffi::CameraContent> {
        folders
            .iter()
            .flat_map(|folder| {
                if let Some(camera) = self.camera.read().unwrap().as_ref() {
                    let task = camera.fs().list_files(folder);
                    dbg_out!("processing folder '{}'", folder);
                    task.wait()
                        .map(|iter| {
                            iter.map(|name| crate::ffi::CameraContent {
                                folder: folder.to_string(),
                                name,
                            })
                            .collect()
                        })
                        .unwrap_or_default()
                } else {
                    vec![]
                }
            })
            .collect()
    }

    pub fn list_content(&self) -> Vec<crate::ffi::CameraContent> {
        // XXX fixme this should not be hardcoded.
        // This is the path for PTP.
        // XXX use Camera::storages to get the list of the root folders.
        let root_folder_ptp = "/store_00010001/DCIM";
        // This is the path for a regular DCF.
        let mut root_folder = "/DCIM";
        self.camera
            .read()
            .unwrap()
            .as_ref()
            .and_then(|camera| {
                let storages = camera.storages().wait();
                dbg_out!("Storages {:?}", storages);
                camera
                    .fs()
                    .list_folders(root_folder)
                    .wait()
                    .or_else(|err| {
                        dbg_out!(
                            "Folder {} not found, trying {}, error {}",
                            root_folder,
                            root_folder_ptp,
                            err
                        );
                        root_folder = root_folder_ptp;
                        camera.fs().list_folders(root_folder).wait()
                    })
                    .map(|iter| {
                        iter.map(|name| {
                            dbg_out!("Found folder '{}'", &name);
                            root_folder.to_owned() + "/" + &name
                        })
                        .collect()
                    })
                    .map(|folders| self.process_folders(folders))
                    .ok()
            })
            .unwrap_or_default()
    }

    /// A hackish attempt to unmount the camera
    /// The code is specific to Linux as we assume the form of the
    /// device used for USB.
    ///
    /// A better solution should be thought.
    fn try_unmount_camera(&self) -> bool {
        // Turn the gphoto device ID into a device path.
        // This code is Linux specific at the moment.
        let mut device_path = self.device.path.to_string();
        // The device path has to start with "usb:".
        // XXX figure out non USB.
        if device_path.find("usb:") != Some(0) {
            err_out!("Device {} is not USB", &device_path);
            return false;
        }
        // Conveniently we can replace the ':' by a '/'
        device_path.replace_range(3..4, "/");
        let device_id = if let Some(comma) = device_path.find(',') {
            device_path.replace_range(comma..=comma, "/");
            // XXX this is specific to Linux
            "/dev/bus/".to_owned() + &device_path
        } else {
            err_out!("Device {} is not USB", &device_path);
            return false;
        };

        glib::MainContext::default().invoke(move || {
            let mounts = gio::VolumeMonitor::get().mounts();
            if let Some(to_unmount) = mounts.iter().find(|mount| {
                if let Some(volume) = mount.volume() {
                    if let Some(id) = volume.identifier("unix-device") {
                        if id == device_id {
                            dbg_out!("found volume {}", &device_id);
                            return true;
                        }
                    }
                }
                false
            }) {
                let mount_op = gio::MountOperation::new();
                dbg_out!("Schedule eject operation for volume");
                to_unmount.unmount_with_operation(
                    gio::MountUnmountFlags::NONE,
                    Some(&mount_op),
                    gio::Cancellable::NONE,
                    |result| {
                        if let Err(err) = result {
                            err_out!("Error unmounting {:?}", err);
                        } else {
                            dbg_out!("Unmount completed");
                        }
                    },
                );
            }
        });

        false
    }

    pub fn close(&self) {
        *self.camera.write().unwrap() = None;
    }

    pub fn get_preview(&self, folder: &str, name: &str) -> Option<crate::Thumbnail> {
        if let Some(camera) = self.camera.write().unwrap().as_ref() {
            let task = camera.fs().download_preview(folder, name);
            let file = task.wait().ok()?;

            let task = file.get_data(&DEVICE_LIST.context.lock().unwrap());
            let data = task.wait().ok()?;

            let loader = gdk_pixbuf::PixbufLoader::new();
            loader.write(&data).ok()?;
            loader.close().ok()?;
            Some(crate::Thumbnail::from(loader.pixbuf()))
        } else {
            None
        }
    }

    pub fn download_file(&self, folder: &str, name: &str, dest: &str) -> bool {
        let destination = std::path::PathBuf::from(dest);
        dbg_out!("Downloading '{}/{}' into {}", folder, name, &dest);
        self.camera
            .write()
            .unwrap()
            .as_ref()
            .and_then(|camera| {
                camera
                    .fs()
                    .download_to(folder, name, &destination)
                    .wait()
                    .map(|_| true)
                    .map_err(|err| {
                        if let Ok(attr) = std::fs::metadata(&destination) {
                            if attr.is_file() && attr.len() == 0 {
                                on_err_out!(std::fs::remove_file(&destination));
                            } else {
                                err_out!("File {:?} not deleted after error.", &destination);
                            }
                        }
                        err_out!("Camera error {:?}", err);
                        false
                    })
                    .ok()
            })
            .unwrap_or(false)
    }
}
