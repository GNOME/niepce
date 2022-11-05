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
use npc_engine::db::LibraryId;
use npc_engine::library::notification::{LcChannel, LibNotification};

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

    /// Re-wrap the LibraryClient
    // cxx
    pub fn wrap(client: &Arc<LibraryClient>) -> Self {
        LibraryClientWrapper {
            client: client.clone(),
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
pub extern "C" fn libraryclient_request_metadata(
    client: &LibraryClientWrapper,
    file_id: LibraryId,
) {
    client.request_metadata(file_id);
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
