/*
 * niepce - libraryclient/mod.rs
 *
 * Copyright (C) 2017-2022 Hubert Figuière
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

pub mod clientinterface;

pub use self::clientinterface::{ClientInterface, ClientInterfaceSync};

use libc::{c_char, c_void};
use std::ffi::CStr;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync;
use std::sync::mpsc;
use std::sync::{atomic, Arc};
use std::thread;

use crate::db::filebundle::FileBundle;
use crate::db::library::Managed;
use crate::db::props::NiepceProperties as Np;
use crate::db::{Library, LibraryId};
use crate::db::{NiepceProperties, NiepcePropertyIdx};
use crate::library::commands;
use crate::library::notification::{LcChannel, LibNotification};
use crate::library::op::Op;
use crate::NiepcePropertyBag;
use npc_fwk::base::PropertyValue;
use npc_fwk::toolkit::PortableChannel;
use npc_fwk::utils::files::FileList;
use npc_fwk::{err_out, on_err_out};

/// Wrap the libclient Arc so that it can be passed around
/// Used in the ffi for example.
/// Implement `Deref` to `LibraryClient`
pub struct LibraryClientWrapper {
    client: sync::Arc<LibraryClient>,
}

impl Deref for LibraryClientWrapper {
    type Target = LibraryClient;
    fn deref(&self) -> &Self::Target {
        self.client.deref()
    }
}

impl LibraryClientWrapper {
    pub fn new(
        dir: PathBuf,
        sender: async_channel::Sender<LibNotification>,
    ) -> LibraryClientWrapper {
        LibraryClientWrapper {
            client: sync::Arc::new(LibraryClient::new(dir, sender)),
        }
    }

    #[inline]
    pub fn client(&self) -> Arc<LibraryClient> {
        self.client.clone()
    }

    /// unwrap the mutable client Arc
    /// XXX we need to unsure this is thread safe.
    /// Don't hold this reference more than you need.
    #[inline]
    fn unwrap_mut(&mut self) -> &mut LibraryClient {
        Arc::get_mut(&mut self.client).unwrap()
    }
}

pub struct LibraryClient {
    terminate: sync::Arc<atomic::AtomicBool>,
    sender: mpsc::Sender<Op>,
    trash_id: LibraryId,
}

impl Drop for LibraryClient {
    fn drop(&mut self) {
        self.stop();
    }
}

impl LibraryClient {
    pub fn new(dir: PathBuf, sender: async_channel::Sender<LibNotification>) -> LibraryClient {
        let (task_sender, task_receiver) = mpsc::channel::<Op>();

        let mut terminate = sync::Arc::new(atomic::AtomicBool::new(false));
        let terminate2 = terminate.clone();

        /* let thread = */
        thread::spawn(move || {
            let library = Library::new(&dir, None, sender);
            Self::main(&mut terminate, task_receiver, &library);
        });

        LibraryClient {
            terminate: terminate2,
            sender: task_sender,
            trash_id: 0,
        }
    }

    pub fn get_trash_id(&self) -> LibraryId {
        self.trash_id
    }

    pub fn set_trash_id(&mut self, id: LibraryId) {
        self.trash_id = id;
    }

    fn stop(&mut self) {
        self.terminate.store(true, atomic::Ordering::Relaxed);
    }

    fn main(
        terminate: &mut sync::Arc<atomic::AtomicBool>,
        tasks: mpsc::Receiver<Op>,
        library: &Library,
    ) {
        while !terminate.load(atomic::Ordering::Relaxed) {
            let elem: Option<Op> = tasks.recv().ok();

            if let Some(op) = elem {
                op.execute(library);
            }
        }
    }

    pub fn schedule_op<F>(&self, f: F)
    where
        F: Fn(&Library) -> bool + Send + Sync + 'static,
    {
        let op = Op::new(f);

        on_err_out!(self.sender.send(op));
    }
}

impl ClientInterface for LibraryClient {
    /// get all the keywords
    fn get_all_keywords(&self) {
        self.schedule_op(commands::cmd_list_all_keywords);
    }

    fn query_keyword_content(&self, keyword_id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_query_keyword_content(lib, keyword_id));
    }

    fn count_keyword(&self, id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_count_keyword(lib, id));
    }

    /// get all the folders
    fn get_all_folders(&self) {
        self.schedule_op(commands::cmd_list_all_folders);
    }

    fn query_folder_content(&self, folder_id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_query_folder_content(lib, folder_id));
    }

    fn count_folder(&self, folder_id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_count_folder(lib, folder_id));
    }

    fn create_folder(&self, name: String, path: Option<String>) {
        self.schedule_op(move |lib| commands::cmd_create_folder(lib, &name, path.clone()) != 0);
    }

    fn delete_folder(&self, id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_delete_folder(lib, id));
    }

    /// get all the albums
    fn get_all_albums(&self) {
        self.schedule_op(commands::cmd_list_all_albums);
    }

    /// Count album
    fn count_album(&self, album_id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_count_album(lib, album_id));
    }

    /// Create an album (async)
    fn create_album(&self, name: String, parent: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_create_album(lib, &name, parent) != 0);
    }

    /// Add an image to an album.
    fn add_to_album(&self, image_id: LibraryId, album_id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_add_to_album(lib, image_id, album_id));
    }

    /// Query content for album.
    fn query_album_content(&self, album_id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_query_album_content(lib, album_id));
    }

    fn request_metadata(&self, file_id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_request_metadata(lib, file_id));
    }

    /// set the metadata
    fn set_metadata(&self, file_id: LibraryId, meta: Np, value: &PropertyValue) {
        let value2 = value.clone();
        self.schedule_op(move |lib| commands::cmd_set_metadata(lib, file_id, meta, &value2));
    }

    fn set_image_properties(&self, image_id: LibraryId, props: &NiepcePropertyBag) {
        let props = props.clone();
        self.schedule_op(move |lib| commands::cmd_set_image_properties(lib, image_id, &props));
    }

    fn write_metadata(&self, file_id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_write_metadata(lib, file_id));
    }

    fn move_file_to_folder(&self, file_id: LibraryId, from: LibraryId, to: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_move_file_to_folder(lib, file_id, from, to));
    }

    /// get all the labels
    fn get_all_labels(&self) {
        self.schedule_op(commands::cmd_list_all_labels);
    }

    fn create_label(&self, name: String, colour: String) {
        self.schedule_op(move |lib| commands::cmd_create_label(lib, &name, &colour) != 0);
    }

    fn delete_label(&self, label_id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_delete_label(lib, label_id));
    }

    /// update a label
    fn update_label(&self, label_id: LibraryId, new_name: String, new_colour: String) {
        self.schedule_op(move |lib| {
            commands::cmd_update_label(lib, label_id, &new_name, &new_colour)
        });
    }

    /// tell to process the Xmp update Queue
    fn process_xmp_update_queue(&self, write_xmp: bool) {
        self.schedule_op(move |lib| commands::cmd_process_xmp_update_queue(lib, write_xmp));
    }

    /// Import files from a directory
    /// @param dir the directory
    /// @param manage true if imports have to be managed
    fn import_files(&self, dir: String, files: Vec<PathBuf>, manage: Managed) {
        self.schedule_op(move |lib| commands::cmd_import_files(lib, &dir, &files, manage));
    }
}

impl ClientInterfaceSync for LibraryClient {
    fn create_label_sync(&self, name: String, colour: String) -> LibraryId {
        // can't use futures::sync::oneshot
        let (tx, rx) = mpsc::sync_channel::<LibraryId>(1);

        self.schedule_op(move |lib| {
            tx.send(commands::cmd_create_label(lib, &name, &colour))
                .unwrap();
            true
        });

        rx.recv().unwrap()
    }

    fn create_keyword_sync(&self, keyword: String) -> LibraryId {
        // can't use futures::sync::oneshot
        let (tx, rx) = mpsc::sync_channel::<LibraryId>(1);

        self.schedule_op(move |lib| {
            tx.send(commands::cmd_add_keyword(lib, &keyword)).unwrap();
            true
        });

        rx.recv().unwrap()
    }

    fn create_folder_sync(&self, name: String, path: Option<String>) -> LibraryId {
        // can't use futures::sync::oneshot
        let (tx, rx) = mpsc::sync_channel::<LibraryId>(1);

        self.schedule_op(move |lib| {
            tx.send(commands::cmd_create_folder(lib, &name, path.clone()))
                .unwrap();
            true
        });

        rx.recv().unwrap()
    }

    fn create_album_sync(&self, name: String, parent: LibraryId) -> LibraryId {
        // can't use futures::sync::oneshot
        let (tx, rx) = mpsc::sync_channel::<LibraryId>(1);

        self.schedule_op(move |lib| {
            tx.send(commands::cmd_create_album(lib, &name, parent))
                .unwrap();
            true
        });

        rx.recv().unwrap()
    }

    fn add_bundle_sync(&self, bundle: &FileBundle, folder: LibraryId) -> LibraryId {
        let (tx, rx) = mpsc::sync_channel::<LibraryId>(1);

        let bundle = bundle.clone();
        self.schedule_op(move |lib| {
            tx.send(commands::cmd_add_bundle(lib, &bundle, folder))
                .unwrap();
            true
        });

        rx.recv().unwrap()
    }
}

#[no_mangle]
pub extern "C" fn lcchannel_new(
    cb: extern "C" fn(n: *const LibNotification, data: *mut c_void) -> i32,
    data: *mut c_void,
) -> *mut LcChannel {
    let (sender, receiver) = async_channel::unbounded();
    let event_handler = async move {
        while let Ok(n) = receiver.recv().await {
            if cb(&n, data) == 0 {
                receiver.close();
                break;
            }
        }
    };
    glib::MainContext::default().spawn_local(event_handler);
    Box::into_raw(Box::new(PortableChannel::<LibNotification>(sender)))
}

/// # Safety
/// Dereference a pointer.
#[no_mangle]
pub unsafe extern "C" fn lcchannel_delete(obj: *mut LcChannel) {
    Box::from_raw(obj);
}

/// # Safety
/// Dereference a pointer.
#[no_mangle]
pub unsafe extern "C" fn libraryclient_new(
    path: *const c_char,
    channel: *const LcChannel,
) -> *mut LibraryClientWrapper {
    let dir = PathBuf::from(&*CStr::from_ptr(path).to_string_lossy());
    Box::into_raw(Box::new(LibraryClientWrapper::new(
        dir,
        (*channel).0.clone(),
    )))
}

/// # Safety
/// Dereference a pointer.
#[no_mangle]
pub unsafe extern "C" fn libraryclient_delete(obj: *mut LibraryClientWrapper) {
    Box::from_raw(obj);
}

#[no_mangle]
pub extern "C" fn libraryclient_set_trash_id(client: &mut LibraryClientWrapper, id: LibraryId) {
    client.unwrap_mut().set_trash_id(id);
}

#[no_mangle]
pub extern "C" fn libraryclient_get_trash_id(client: &mut LibraryClientWrapper) -> LibraryId {
    client.get_trash_id()
}

#[no_mangle]
pub extern "C" fn libraryclient_get_all_keywords(client: &mut LibraryClientWrapper) {
    client.get_all_keywords();
}

#[no_mangle]
pub extern "C" fn libraryclient_get_all_folders(client: &mut LibraryClientWrapper) {
    client.get_all_folders();
}

#[no_mangle]
pub extern "C" fn libraryclient_get_all_albums(client: &mut LibraryClientWrapper) {
    client.get_all_albums();
}

/// # Safety
/// Dereference a pointer.
#[no_mangle]
pub unsafe extern "C" fn libraryclient_create_folder_sync(
    client: &mut LibraryClientWrapper,
    n: *const c_char,
    p: *const c_char,
) -> LibraryId {
    let name = String::from(CStr::from_ptr(n).to_string_lossy());
    let path = if p.is_null() {
        None
    } else {
        Some(String::from(CStr::from_ptr(p).to_string_lossy()))
    };
    client.create_folder_sync(name, path)
}

#[no_mangle]
pub extern "C" fn libraryclient_delete_folder(client: &mut LibraryClientWrapper, id: LibraryId) {
    client.delete_folder(id);
}

#[no_mangle]
pub extern "C" fn libraryclient_count_folder(
    client: &mut LibraryClientWrapper,
    folder_id: LibraryId,
) {
    client.count_folder(folder_id);
}

#[no_mangle]
pub extern "C" fn libraryclient_count_keyword(client: &mut LibraryClientWrapper, id: LibraryId) {
    client.count_keyword(id);
}

#[no_mangle]
pub extern "C" fn libraryclient_count_album(
    client: &mut LibraryClientWrapper,
    album_id: LibraryId,
) {
    client.count_album(album_id);
}

#[no_mangle]
pub extern "C" fn libraryclient_request_metadata(
    client: &mut LibraryClientWrapper,
    file_id: LibraryId,
) {
    client.request_metadata(file_id);
}

#[no_mangle]
pub extern "C" fn libraryclient_set_metadata(
    client: &mut LibraryClientWrapper,
    file_id: LibraryId,
    meta: NiepcePropertyIdx,
    value: &PropertyValue,
) {
    client.set_metadata(file_id, NiepceProperties::Index(meta), value);
}

#[no_mangle]
pub extern "C" fn libraryclient_write_metadata(
    client: &mut LibraryClientWrapper,
    file_id: LibraryId,
) {
    client.write_metadata(file_id);
}

#[no_mangle]
pub extern "C" fn libraryclient_move_file_to_folder(
    client: &mut LibraryClientWrapper,
    file_id: LibraryId,
    from: LibraryId,
    to: LibraryId,
) {
    client.move_file_to_folder(file_id, from, to);
}

#[no_mangle]
pub extern "C" fn libraryclient_get_all_labels(client: &mut LibraryClientWrapper) {
    client.get_all_labels();
}

/// # Safety
/// Dereference a pointer.
#[no_mangle]
pub unsafe extern "C" fn libraryclient_create_label(
    client: &mut LibraryClientWrapper,
    s: *const c_char,
    c: *const c_char,
) {
    let name = CStr::from_ptr(s).to_string_lossy();
    let colour = CStr::from_ptr(c).to_string_lossy();
    client.create_label(String::from(name), String::from(colour));
}

/// # Safety
/// Dereference a pointer.
#[no_mangle]
pub unsafe extern "C" fn libraryclient_create_label_sync(
    client: &mut LibraryClientWrapper,
    s: *const c_char,
    c: *const c_char,
) -> LibraryId {
    let name = CStr::from_ptr(s).to_string_lossy();
    let colour = CStr::from_ptr(c).to_string_lossy();
    client.create_label_sync(String::from(name), String::from(colour))
}

#[no_mangle]
pub extern "C" fn libraryclient_delete_label(
    client: &mut LibraryClientWrapper,
    label_id: LibraryId,
) {
    client.delete_label(label_id);
}

/// # Safety
/// Dereference a pointer.
#[no_mangle]
pub unsafe extern "C" fn libraryclient_update_label(
    client: &mut LibraryClientWrapper,
    label_id: LibraryId,
    s: *const c_char,
    c: *const c_char,
) {
    let name = CStr::from_ptr(s).to_string_lossy();
    let colour = CStr::from_ptr(c).to_string_lossy();
    client.update_label(label_id, String::from(name), String::from(colour));
}

#[no_mangle]
pub extern "C" fn libraryclient_process_xmp_update_queue(
    client: &mut LibraryClientWrapper,
    write_xmp: bool,
) {
    client.process_xmp_update_queue(write_xmp);
}

/// # Safety
/// Dereference a pointer.
#[no_mangle]
pub unsafe extern "C" fn libraryclient_import_files(
    client: &mut LibraryClientWrapper,
    dir: *const c_char,
    files: &FileList,
    manage: Managed,
) {
    let folder = CStr::from_ptr(dir).to_string_lossy();
    client.import_files(String::from(folder), files.0.clone(), manage);
}
