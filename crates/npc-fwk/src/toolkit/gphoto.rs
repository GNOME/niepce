/*
 * niepce - npc_fwk/toolkit/gphoto.rs
 *
 * Copyright (C) 2009-2025 Hubert Figuière
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

use crate::gio;
use crate::glib;
use gio::prelude::*;

use crate::toolkit::Thumbnail;
use crate::{Date, DateExt};

/// Describe the camera.
pub type GpDevice = gphoto2::list::CameraDescriptor;

/// Camera content entry
pub struct CameraContent {
    /// Folder path
    pub folder: String,
    /// Filename
    pub name: String,
}

lazy_static::lazy_static! {
    static ref DEVICE_LIST: GpDeviceList = GpDeviceList::default();
}

pub struct GpDeviceList {
    context: Mutex<gphoto2::Context>,
    list: RwLock<Vec<GpDevice>>,
}

impl Default for GpDeviceList {
    /// # Panic
    /// Will panic if the `gphoto2::Context` can't be created.
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

    /// Return the read lock for the list.
    pub fn list(&self) -> RwLockReadGuard<'_, Vec<GpDevice>> {
        self.list.read().unwrap()
    }

    /// Rescan the devices and update the list.
    pub fn detect(&self) {
        dbg_out!("Detecting cameras");
        let task = self.context.lock().unwrap().list_cameras();
        *self.list.write().unwrap() = if let Ok(camera_list) = task.wait() {
            camera_list.collect()
        } else {
            err_out!("error detecting cameras");
            vec![]
        };
    }

    /// Find a device by its port.
    pub fn device(&self, source: &str) -> Option<GpDevice> {
        self.list
            .read()
            .unwrap()
            .iter()
            .find(|d| d.port == source)
            .cloned()
    }
}

pub struct GpCamera {
    device: GpDevice,
    camera: Option<gphoto2::Camera>,
}

impl GpCamera {
    pub fn new(device: GpDevice) -> GpCamera {
        GpCamera {
            device,
            camera: None,
        }
    }

    /// Return true if it's a `disk:` device.
    pub fn port_is_disk(port: &str) -> bool {
        port.starts_with("disk:")
    }

    /// Open the device as per the descriptor.
    pub fn open(&mut self) -> bool {
        dbg_out!("opening camera {}", self.device.port);
        if self.camera.is_some() {
            self.close();
        }

        let task = DEVICE_LIST.context.lock().unwrap().get_camera(&self.device);
        let camera = task.wait();
        // XXX handle errors
        match camera {
            Ok(c) => {
                // This is used to check if the camera isn't busy.
                // XXX Maybe there is a better call...
                let task = c.fs().file_info("/", "DCIM");
                if let Err(err) = task.wait() {
                    dbg_out!("file info returned an error, of course {:?}", err);
                    if err.kind() == gphoto2::error::ErrorKind::IoUsbClaim {
                        dbg_out!("Trying to unmount the camera...");
                        self.try_unmount_camera();
                    }
                }
                self.camera = Some(c);
                dbg_out!("Camera open and initialized");
                true
            }
            Err(err) => {
                err_out!("Error initializing camera {:?}", err);
                false
            }
        }
    }

    /// Return the port of the camera. This is the unique device at the time.
    /// Unplugging the device is likely to change this.
    pub fn path(&self) -> &str {
        &self.device.port
    }

    /// Get te folder content.
    fn process_folder(&self, fs: &gphoto2::filesys::CameraFS, folder: &str) -> Vec<CameraContent> {
        let task = fs.list_files(folder);
        dbg_out!("processing folder '{}'", folder);
        task.wait()
            .map(|iter| {
                iter.map(|name| CameraContent {
                    folder: folder.to_string(),
                    name,
                })
                .collect()
            })
            .unwrap_or_default()
    }

    /// List the content of the camera (DCIM only).
    pub fn list_content<F>(&self, terminate: F) -> Vec<CameraContent>
    where
        F: Fn() -> bool,
    {
        self.camera
            .as_ref()
            .map(|camera| {
                let storages = camera.storages().wait();
                dbg_out!("Storages {:?}", storages);
                // XXX a '/' is always added, leading to '//DCIM'
                // Map the storages to refer the DCIM folder.
                let root_folders = storages
                    .ok()
                    .map(|storages| {
                        storages
                            .iter()
                            .filter_map(|info| info.base_directory().map(|s| s.into_owned()))
                            .collect()
                    })
                    .unwrap_or_else(|| vec!["/DCIM".to_owned()]);
                println!("root_folders {root_folders:?}");
                // List the content of each root folders, and flatten.
                root_folders
                    .iter()
                    .take_while(|_| !terminate())
                    .flat_map(|root_folder| {
                        println!("processing root folder {root_folder}");
                        let fs = camera.fs();
                        self.get_folders(&fs, root_folder, &terminate)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Recursively get the folders from `root_folder`
    fn get_folders<F>(
        &self,
        fs: &gphoto2::filesys::CameraFS,
        root_folder: &str,
        terminate: &F,
    ) -> Vec<CameraContent>
    where
        F: Fn() -> bool,
    {
        trace_out!("get_folders {root_folder}");
        fs.list_folders(root_folder)
            .wait()
            .map(|iter| {
                iter.map(|name| {
                    dbg_out!("Found folder '{}'", &name);
                    root_folder.to_owned() + "/" + &name
                })
                .take_while(|_| !terminate())
                .flat_map(|folder| {
                    let mut content = self.process_folder(fs, &folder);
                    let sub_content = self.get_folders(fs, &folder, terminate);
                    content.extend(sub_content);
                    content
                })
                .collect()
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
        let mut device_path = self.device.port.to_string();
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

    pub fn close(&mut self) {
        self.camera = None;
    }

    pub fn get_exif(&self, folder: &str, name: &str) -> Option<Vec<u8>> {
        let camera = self.camera.as_ref()?;
        let task = camera.fs().download_exif(folder, name);
        let file = task.wait().ok()?;

        let task = file.get_data(&*DEVICE_LIST.context.lock().unwrap());
        let data = task.wait().ok()?;

        Some(data.to_vec())
    }

    pub fn get_preview(
        &self,
        folder: &str,
        name: &str,
        orientation: Option<i32>,
    ) -> Option<(Thumbnail, Option<Date>)> {
        let camera = self.camera.as_ref()?;
        let task = camera.fs().download_preview(folder, name);
        let file = task.wait().ok()?;

        let task = file.get_data(&*DEVICE_LIST.context.lock().unwrap());
        let data = task.wait().ok()?;

        let mtime = file.mtime() as u64;
        let time = std::time::UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs(mtime))
            .map(Date::from_system_time);

        image::load_from_memory(&data)
            .inspect_err(|err| err_out!("Error decoding image for thumbnail: {err}"))
            .map(|mut buf| {
                let orientation = orientation
                    .and_then(|orientation| {
                        image::metadata::Orientation::from_exif(orientation as u8)
                    })
                    .unwrap_or(image::metadata::Orientation::NoTransforms);
                buf.apply_orientation(orientation);
                buf
            })
            .map(|t| (Thumbnail::from(t), time))
            .ok()
    }

    pub fn download_file(&self, folder: &str, name: &str, dest: &str) -> bool {
        let destination = std::path::PathBuf::from(dest);
        dbg_out!("Downloading '{}/{}' into {}", folder, name, &dest);
        self.camera
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

#[cfg(test)]
mod test {
    use super::GpCamera;

    #[test]
    fn test_is_disk() {
        assert!(GpCamera::port_is_disk("disk:/run/media/hub/CANON_DC"));
    }
}
