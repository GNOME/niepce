/*
 * niepce - libraryclient/mod.rs
 *
 * Copyright (C) 2017-2024 Hubert Figuière
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
mod host;
mod ui_data_provider;

pub use clientinterface::{ClientCallback, ClientInterface, ClientInterfaceSync};
pub use host::LibraryClientHost;
pub use ui_data_provider::UIDataProvider;

use std::ops::Deref;
use std::path::PathBuf;
use std::sync;
use std::sync::{atomic, mpsc};
use std::thread;

use crate::db::filebundle::FileBundle;
use crate::db::props::NiepceProperties as Np;
use crate::db::{LibFolder, Library, LibraryId};
use crate::library::commands;
use crate::library::notification::LcChannel;
use crate::library::op::Op;
use crate::NiepcePropertyBag;
use npc_fwk::base::{PropertyValue, RgbColour};
use npc_fwk::on_err_out;

/// LibraryClient is in charge of creating both side of the worker:
/// the sender and the actual library on a separate thread.
pub struct LibraryClient {
    terminate: sync::Arc<atomic::AtomicBool>,
    trash_id: atomic::AtomicI64,
    /// This is what will implement the interface.
    sender: LibraryClientSender,
}

impl Drop for LibraryClient {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Deref for LibraryClient {
    type Target = LibraryClientSender;
    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

impl LibraryClient {
    pub fn new(dir: PathBuf, sender: LcChannel) -> LibraryClient {
        let (task_sender, task_receiver) = mpsc::channel::<Op>();

        let mut terminate = sync::Arc::new(atomic::AtomicBool::new(false));
        let terminate2 = terminate.clone();

        /* let thread = */
        on_err_out!(thread::Builder::new()
            .name("library client".to_string())
            .spawn(move || {
                let library = Library::new(&dir, None, sender);
                Self::main(&mut terminate, task_receiver, &library);
            }));

        LibraryClient {
            terminate: terminate2,
            sender: LibraryClientSender(task_sender),
            trash_id: atomic::AtomicI64::new(0),
        }
    }

    pub fn get_trash_id(&self) -> LibraryId {
        self.trash_id.load(atomic::Ordering::Relaxed)
    }

    pub fn set_trash_id(&self, id: LibraryId) {
        self.trash_id.store(id, atomic::Ordering::Relaxed);
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

    pub fn sender(&self) -> &LibraryClientSender {
        &self.sender
    }
}

#[derive(Clone)]
/// The sender for the library client.
/// It is all you should need to perform ops on the library
pub struct LibraryClientSender(mpsc::Sender<Op>);

impl LibraryClientSender {
    pub fn schedule_op<F>(&self, f: F)
    where
        F: FnOnce(&Library) -> bool + Send + Sync + 'static,
    {
        let op = Op::new(f);

        on_err_out!(self.0.send(op));
    }
}

impl ClientInterface for LibraryClientSender {
    /// get all the preferences
    fn get_all_preferences(&self) {
        self.schedule_op(commands::cmd_list_all_preferences);
    }

    fn set_preference(&self, key: String, value: String) {
        self.schedule_op(move |lib| commands::cmd_set_preference(lib, &key, &value))
    }

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

    /// Get the root folder.
    fn get_root_folders(&self, callback: ClientCallback<Vec<LibFolder>>) {
        self.schedule_op(move |lib| commands::cmd_list_root_folders(lib, callback));
    }

    /// get all the folders
    fn get_all_folders(&self, callback: Option<ClientCallback<Vec<LibFolder>>>) {
        self.schedule_op(move |lib| commands::cmd_list_all_folders(lib, callback));
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

    /// Delete an album
    fn delete_album(&self, id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_delete_album(lib, id));
    }

    /// Add images to an album.
    fn add_to_album(&self, images: &[LibraryId], album_id: LibraryId) {
        let images = images.to_vec();
        self.schedule_op(move |lib| {
            if commands::cmd_add_to_album(lib, images, album_id) {
                commands::cmd_count_album(lib, album_id)
            } else {
                false
            }
        });
    }

    /// Remove images to an album.
    fn remove_from_album(&self, images: &[LibraryId], album_id: LibraryId) {
        let images = images.to_vec();
        self.schedule_op(move |lib| {
            if commands::cmd_remove_from_album(lib, images, album_id) {
                commands::cmd_count_album(lib, album_id)
            } else {
                false
            }
        });
    }

    /// Rename album `album_id` to `name`.
    fn rename_album(&self, album_id: LibraryId, name: String) {
        self.schedule_op(move |lib| commands::cmd_rename_album(lib, album_id, &name));
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

    fn create_label(&self, name: String, colour: RgbColour) {
        self.schedule_op(move |lib| commands::cmd_create_label(lib, &name, &colour) != 0);
    }

    fn delete_label(&self, label_id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_delete_label(lib, label_id));
    }

    /// update a label
    fn update_label(&self, label_id: LibraryId, new_name: String, new_colour: RgbColour) {
        self.schedule_op(move |lib| {
            commands::cmd_update_label(lib, label_id, &new_name, &new_colour)
        });
    }

    /// tell to process the Xmp update Queue
    fn process_xmp_update_queue(&self, write_xmp: bool) {
        self.schedule_op(move |lib| commands::cmd_process_xmp_update_queue(lib, write_xmp));
    }

    /// Import files in place.
    fn import_files(&self, files: Vec<PathBuf>) {
        self.schedule_op(move |lib| commands::cmd_import_files(lib, &files));
    }
}

impl ClientInterfaceSync for LibraryClientSender {
    fn create_label_sync(&self, name: String, colour: RgbColour) -> LibraryId {
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

    fn upgrade_library_from_sync(&self, version: i32) -> bool {
        let (tx, rx) = mpsc::sync_channel::<bool>(1);

        self.schedule_op(move |lib| {
            tx.send(commands::cmd_upgrade_library_from(lib, version))
                .unwrap();
            true
        });

        rx.recv().unwrap()
    }
}
