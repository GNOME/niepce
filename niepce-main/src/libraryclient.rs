/*
 * niepce - libraryclient/mod.rs
 *
 * Copyright (C) 2017-2021 Hubert Figuière
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

pub mod clientimpl;
pub mod clientinterface;

pub use self::clientinterface::{ClientInterface, ClientInterfaceSync};

use libc::{c_char, c_void};
use std::ffi::CStr;
use std::path::PathBuf;
use std::sync::Arc;

use self::clientimpl::ClientImpl;
use npc_engine::db::library::Managed;
use npc_engine::db::props::NiepceProperties as Np;
use npc_engine::db::LibraryId;
use npc_engine::db::{NiepceProperties, NiepcePropertyIdx};
use npc_engine::library::notification::{LcChannel, LibNotification};
use npc_fwk::base::PropertyValue;
use npc_fwk::toolkit::PortableChannel;
use npc_fwk::utils::files::FileList;

/// Wrap the libclient Arc so that it can be passed around
/// Used in the ffi for example.
pub struct LibraryClientWrapper {
    client: Arc<LibraryClient>,
}

impl LibraryClientWrapper {
    pub fn new(
        dir: PathBuf,
        sender: async_channel::Sender<LibNotification>,
    ) -> LibraryClientWrapper {
        LibraryClientWrapper {
            client: Arc::new(LibraryClient::new(dir, sender)),
        }
    }

    /// unwrap the mutable client Arc
    /// XXX we need to unsure this is thread safe.
    /// Don't hold this reference more than you need.
    pub fn unwrap_mut(&mut self) -> &mut LibraryClient {
        Arc::get_mut(&mut self.client).unwrap()
    }
}

pub struct LibraryClient {
    pimpl: ClientImpl,

    trash_id: LibraryId,
}

impl LibraryClient {
    pub fn new(dir: PathBuf, sender: async_channel::Sender<LibNotification>) -> LibraryClient {
        LibraryClient {
            pimpl: ClientImpl::new(dir, sender),
            trash_id: 0,
        }
    }

    pub fn get_trash_id(&self) -> LibraryId {
        self.trash_id
    }

    pub fn set_trash_id(&mut self, id: LibraryId) {
        self.trash_id = id;
    }
}

impl ClientInterface for LibraryClient {
    /// get all the keywords
    fn get_all_keywords(&mut self) {
        self.pimpl.get_all_keywords();
    }
    fn query_keyword_content(&mut self, id: LibraryId) {
        self.pimpl.query_keyword_content(id);
    }
    fn count_keyword(&mut self, id: LibraryId) {
        self.pimpl.count_keyword(id);
    }

    /// get all the folder
    fn get_all_folders(&mut self) {
        self.pimpl.get_all_folders();
    }
    fn query_folder_content(&mut self, id: LibraryId) {
        self.pimpl.query_folder_content(id);
    }
    fn count_folder(&mut self, id: LibraryId) {
        self.pimpl.count_folder(id);
    }

    fn create_folder(&mut self, name: String, path: Option<String>) {
        self.pimpl.create_folder(name, path);
    }

    fn delete_folder(&mut self, id: LibraryId) {
        self.pimpl.delete_folder(id);
    }

    fn request_metadata(&mut self, id: LibraryId) {
        self.pimpl.request_metadata(id);
    }
    /// set the metadata
    fn set_metadata(&mut self, id: LibraryId, meta: Np, value: &PropertyValue) {
        self.pimpl.set_metadata(id, meta, value);
    }
    fn write_metadata(&mut self, id: LibraryId) {
        self.pimpl.write_metadata(id);
    }

    fn move_file_to_folder(&mut self, file_id: LibraryId, from: LibraryId, to: LibraryId) {
        self.pimpl.move_file_to_folder(file_id, from, to);
    }
    /// get all the labels
    fn get_all_labels(&mut self) {
        self.pimpl.get_all_labels();
    }
    fn create_label(&mut self, label: String, colour: String) {
        self.pimpl.create_label(label, colour);
    }
    fn delete_label(&mut self, id: LibraryId) {
        self.pimpl.delete_label(id);
    }
    /// update a label
    fn update_label(&mut self, id: LibraryId, new_name: String, new_colour: String) {
        self.pimpl.update_label(id, new_name, new_colour);
    }

    /// tell to process the Xmp update Queue
    fn process_xmp_update_queue(&mut self, write_xmp: bool) {
        self.pimpl.process_xmp_update_queue(write_xmp);
    }

    /// Import files from a directory
    /// @param dir the directory
    /// @param manage true if imports have to be managed
    fn import_files(&mut self, dir: String, files: Vec<PathBuf>, manage: Managed) {
        self.pimpl.import_files(dir, files, manage);
    }
}

impl ClientInterfaceSync for LibraryClient {
    fn create_keyword_sync(&mut self, keyword: String) -> LibraryId {
        self.pimpl.create_keyword_sync(keyword)
    }

    fn create_label_sync(&mut self, name: String, colour: String) -> LibraryId {
        self.pimpl.create_label_sync(name, colour)
    }

    fn create_folder_sync(&mut self, name: String, path: Option<String>) -> LibraryId {
        self.pimpl.create_folder_sync(name, path)
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

#[no_mangle]
pub unsafe extern "C" fn lcchannel_delete(obj: *mut LcChannel) {
    Box::from_raw(obj);
}

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
    client.unwrap_mut().get_trash_id()
}

#[no_mangle]
pub extern "C" fn libraryclient_get_all_keywords(client: &mut LibraryClientWrapper) {
    client.unwrap_mut().get_all_keywords();
}

#[no_mangle]
pub extern "C" fn libraryclient_get_all_folders(client: &mut LibraryClientWrapper) {
    client.unwrap_mut().get_all_folders();
}

#[no_mangle]
pub extern "C" fn libraryclient_query_folder_content(
    client: &mut LibraryClientWrapper,
    folder_id: LibraryId,
) {
    client.unwrap_mut().query_folder_content(folder_id);
}

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
    client.unwrap_mut().create_folder_sync(name, path)
}

#[no_mangle]
pub extern "C" fn libraryclient_delete_folder(client: &mut LibraryClientWrapper, id: LibraryId) {
    client.unwrap_mut().delete_folder(id);
}

#[no_mangle]
pub extern "C" fn libraryclient_count_folder(
    client: &mut LibraryClientWrapper,
    folder_id: LibraryId,
) {
    client.unwrap_mut().count_folder(folder_id);
}

#[no_mangle]
pub extern "C" fn libraryclient_query_keyword_content(
    client: &mut LibraryClientWrapper,
    keyword_id: LibraryId,
) {
    client.unwrap_mut().query_keyword_content(keyword_id);
}

#[no_mangle]
pub extern "C" fn libraryclient_count_keyword(client: &mut LibraryClientWrapper, id: LibraryId) {
    client.unwrap_mut().count_keyword(id);
}

#[no_mangle]
pub extern "C" fn libraryclient_request_metadata(
    client: &mut LibraryClientWrapper,
    file_id: LibraryId,
) {
    client.unwrap_mut().request_metadata(file_id);
}

#[no_mangle]
pub extern "C" fn libraryclient_set_metadata(
    client: &mut LibraryClientWrapper,
    file_id: LibraryId,
    meta: NiepcePropertyIdx,
    value: &PropertyValue,
) {
    client
        .unwrap_mut()
        .set_metadata(file_id, NiepceProperties::Index(meta), value);
}

#[no_mangle]
pub extern "C" fn libraryclient_write_metadata(
    client: &mut LibraryClientWrapper,
    file_id: LibraryId,
) {
    client.unwrap_mut().write_metadata(file_id);
}

#[no_mangle]
pub extern "C" fn libraryclient_move_file_to_folder(
    client: &mut LibraryClientWrapper,
    file_id: LibraryId,
    from: LibraryId,
    to: LibraryId,
) {
    client.unwrap_mut().move_file_to_folder(file_id, from, to);
}

#[no_mangle]
pub extern "C" fn libraryclient_get_all_labels(client: &mut LibraryClientWrapper) {
    client.unwrap_mut().get_all_labels();
}

#[no_mangle]
pub unsafe extern "C" fn libraryclient_create_label(
    client: &mut LibraryClientWrapper,
    s: *const c_char,
    c: *const c_char,
) {
    let name = CStr::from_ptr(s).to_string_lossy();
    let colour = CStr::from_ptr(c).to_string_lossy();
    client
        .unwrap_mut()
        .create_label(String::from(name), String::from(colour));
}

#[no_mangle]
pub unsafe extern "C" fn libraryclient_create_label_sync(
    client: &mut LibraryClientWrapper,
    s: *const c_char,
    c: *const c_char,
) -> LibraryId {
    let name = CStr::from_ptr(s).to_string_lossy();
    let colour = CStr::from_ptr(c).to_string_lossy();
    client
        .unwrap_mut()
        .create_label_sync(String::from(name), String::from(colour))
}

#[no_mangle]
pub extern "C" fn libraryclient_delete_label(
    client: &mut LibraryClientWrapper,
    label_id: LibraryId,
) {
    client.unwrap_mut().delete_label(label_id);
}

#[no_mangle]
pub unsafe extern "C" fn libraryclient_update_label(
    client: &mut LibraryClientWrapper,
    label_id: LibraryId,
    s: *const c_char,
    c: *const c_char,
) {
    let name = CStr::from_ptr(s).to_string_lossy();
    let colour = CStr::from_ptr(c).to_string_lossy();
    client
        .unwrap_mut()
        .update_label(label_id, String::from(name), String::from(colour));
}

#[no_mangle]
pub extern "C" fn libraryclient_process_xmp_update_queue(
    client: &mut LibraryClientWrapper,
    write_xmp: bool,
) {
    client.unwrap_mut().process_xmp_update_queue(write_xmp);
}

#[no_mangle]
pub unsafe extern "C" fn libraryclient_import_files(
    client: &mut LibraryClientWrapper,
    dir: *const c_char,
    files: &FileList,
    manage: Managed,
) {
    let folder = CStr::from_ptr(dir).to_string_lossy();
    client
        .unwrap_mut()
        .import_files(String::from(folder), files.0.clone(), manage);
}
