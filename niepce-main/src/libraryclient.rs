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

pub mod clientimpl;
pub mod clientinterface;
mod host;
mod ui_data_provider;

pub use clientinterface::{ClientInterface, ClientInterfaceSync};
pub use host::{library_client_host_delete, library_client_host_new, LibraryClientHost};
pub use ui_data_provider::{ui_data_provider_new, UIDataProvider};

use libc::c_char;
use std::cell::Cell;
use std::ffi::CStr;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use self::clientimpl::ClientImpl;
use npc_engine::db::library::Managed;
use npc_engine::db::LibraryId;
use npc_engine::db::{NiepceProperties, NiepcePropertyIdx};
use npc_engine::library::notification::{LcChannel, LibNotification};
use npc_fwk::base::PropertyValue;
use npc_fwk::utils::files::FileList;

/// Wrap the libclient Arc so that it can be passed around
/// Used in the ffi for example.
/// Implement `Deref` to `LibraryClient`
pub struct LibraryClientWrapper {
    client: Arc<LibraryClient>,
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
            client: Arc::new(LibraryClient::new(dir, sender)),
        }
    }

    #[inline]
    pub fn client(&self) -> Arc<LibraryClient> {
        self.client.clone()
    }
}

pub struct LibraryClient {
    pimpl: ClientImpl,

    trash_id: Cell<LibraryId>,
}

impl std::ops::Deref for LibraryClient {
    type Target = ClientImpl;

    fn deref(&self) -> &Self::Target {
        &self.pimpl
    }
}

impl LibraryClient {
    pub fn new(dir: PathBuf, sender: async_channel::Sender<LibNotification>) -> LibraryClient {
        LibraryClient {
            pimpl: ClientImpl::new(dir, sender),
            trash_id: Cell::new(0),
        }
    }

    pub fn get_trash_id(&self) -> LibraryId {
        self.trash_id.get()
    }

    pub fn set_trash_id(&self, id: LibraryId) {
        self.trash_id.set(id);
    }
}

#[no_mangle]
pub extern "C" fn libraryclient_set_trash_id(client: &LibraryClientWrapper, id: LibraryId) {
    client.set_trash_id(id);
}

#[no_mangle]
pub extern "C" fn libraryclient_get_trash_id(client: &LibraryClientWrapper) -> LibraryId {
    client.get_trash_id()
}

#[no_mangle]
pub extern "C" fn libraryclient_get_all_keywords(client: &LibraryClientWrapper) {
    client.get_all_keywords();
}

#[no_mangle]
pub extern "C" fn libraryclient_get_all_folders(client: &LibraryClientWrapper) {
    client.get_all_folders();
}

#[no_mangle]
pub extern "C" fn libraryclient_query_folder_content(
    client: &LibraryClientWrapper,
    folder_id: LibraryId,
) {
    client.query_folder_content(folder_id);
}

#[no_mangle]
pub extern "C" fn libraryclient_delete_folder(client: &LibraryClientWrapper, id: LibraryId) {
    client.delete_folder(id);
}

#[no_mangle]
pub extern "C" fn libraryclient_count_folder(client: &LibraryClientWrapper, folder_id: LibraryId) {
    client.count_folder(folder_id);
}

#[no_mangle]
pub extern "C" fn libraryclient_query_keyword_content(
    client: &LibraryClientWrapper,
    keyword_id: LibraryId,
) {
    client.query_keyword_content(keyword_id);
}

#[no_mangle]
pub extern "C" fn libraryclient_count_keyword(client: &LibraryClientWrapper, id: LibraryId) {
    client.count_keyword(id);
}

#[no_mangle]
pub extern "C" fn libraryclient_request_metadata(
    client: &LibraryClientWrapper,
    file_id: LibraryId,
) {
    client.request_metadata(file_id);
}

#[no_mangle]
pub extern "C" fn libraryclient_set_metadata(
    client: &LibraryClientWrapper,
    file_id: LibraryId,
    meta: NiepcePropertyIdx,
    value: &PropertyValue,
) {
    client.set_metadata(file_id, NiepceProperties::Index(meta), value);
}

#[no_mangle]
pub extern "C" fn libraryclient_write_metadata(client: &LibraryClientWrapper, file_id: LibraryId) {
    client.write_metadata(file_id);
}

#[no_mangle]
pub extern "C" fn libraryclient_move_file_to_folder(
    client: &LibraryClientWrapper,
    file_id: LibraryId,
    from: LibraryId,
    to: LibraryId,
) {
    client.move_file_to_folder(file_id, from, to);
}

#[no_mangle]
pub extern "C" fn libraryclient_get_all_labels(client: &LibraryClientWrapper) {
    client.get_all_labels();
}

/// # Safety
/// Dereference a pointer.
#[no_mangle]
pub unsafe extern "C" fn libraryclient_create_label(
    client: &LibraryClientWrapper,
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
    client: &LibraryClientWrapper,
    s: *const c_char,
    c: *const c_char,
) -> LibraryId {
    let name = CStr::from_ptr(s).to_string_lossy();
    let colour = CStr::from_ptr(c).to_string_lossy();
    client.create_label_sync(String::from(name), String::from(colour))
}

#[no_mangle]
pub extern "C" fn libraryclient_delete_label(client: &LibraryClientWrapper, label_id: LibraryId) {
    client.delete_label(label_id);
}

/// # Safety
/// Dereference a pointer.
#[no_mangle]
pub unsafe extern "C" fn libraryclient_update_label(
    client: &LibraryClientWrapper,
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
    client: &LibraryClientWrapper,
    write_xmp: bool,
) {
    client.process_xmp_update_queue(write_xmp);
}

/// # Safety
/// Dereference a pointer.
#[no_mangle]
pub unsafe extern "C" fn libraryclient_import_files(
    client: &LibraryClientWrapper,
    dir: *const c_char,
    files: &FileList,
    manage: Managed,
) {
    let folder = CStr::from_ptr(dir).to_string_lossy();
    client.import_files(String::from(folder), files.0.clone(), manage);
}
