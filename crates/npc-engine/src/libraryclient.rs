/*
 * niepce - npc-engine/libraryclient.rs
 *
 * Copyright (C) 2017-2025 Hubert Figuière
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
use std::sync::{atomic, mpsc};
use std::thread;

use crate::NiepcePropertyBag;
use crate::catalog::filebundle::FileBundle;
use crate::catalog::props::NiepceProperties as Np;
use crate::catalog::{CatalogDb, LibFolder, LibraryId};
use crate::library::commands;
use crate::library::notification::LcChannel;
use crate::library::op::Op;
use npc_fwk::base::{PropertyValue, RgbColour};
use npc_fwk::on_err_out;

enum Request {
    Terminate,
    Op(Op),
}

/// LibraryClient is in charge of creating both side of the worker:
/// the sender and the actual library on a separate thread.
pub struct LibraryClient {
    thread: thread::JoinHandle<()>,
    trash_id: atomic::AtomicI64,
    /// This is what will implement the interface.
    sender: LibraryClientSender,
    catalog_file: PathBuf,
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
    pub fn new(filename: PathBuf, sender: LcChannel) -> LibraryClient {
        let (task_sender, task_receiver) = mpsc::channel::<Request>();
        let catalog_file = filename.clone();
        let thread = thread::Builder::new()
            .name("library client".to_string())
            .spawn(move || {
                let library = CatalogDb::new(&filename, sender);
                Self::main(task_receiver, &library);
            })
            .unwrap();

        LibraryClient {
            thread,
            sender: LibraryClientSender(task_sender),
            trash_id: atomic::AtomicI64::new(0),
            catalog_file,
        }
    }

    pub fn base_directory(&self) -> Option<&std::path::Path> {
        self.catalog_file.parent()
    }

    pub fn get_trash_id(&self) -> LibraryId {
        self.trash_id.load(atomic::Ordering::Relaxed)
    }

    pub fn set_trash_id(&self, id: LibraryId) {
        self.trash_id.store(id, atomic::Ordering::Relaxed);
    }

    fn stop(&self) {
        self.close();
    }

    /// Close the library client. This will wait for the thread to be
    /// terminated unless it's closed on the same thread.  This is
    /// meant to ensure the sqlite file is flushed to disk.
    fn close(&self) {
        on_err_out!(self.sender.0.send(Request::Terminate));
        if self.thread.thread().id() != std::thread::current().id() {
            // We could join the handle but this require a `&mut self`
            // which we don't really have.
            while !self.thread.is_finished() {
                thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }

    fn main(tasks: mpsc::Receiver<Request>, library: &CatalogDb) {
        loop {
            match tasks.recv() {
                Ok(Request::Terminate) => break,
                Ok(Request::Op(op)) => {
                    op.execute(library);
                }
                Err(err) => {
                    npc_fwk::err_out!("LibrayClient err: {err}");
                    break;
                }
            }
        }
        npc_fwk::dbg_out!("LibraryClient terminated");
    }

    pub fn sender(&self) -> &LibraryClientSender {
        &self.sender
    }
}

#[derive(Clone)]
/// The sender for the library client.
/// It is all you should need to perform ops on the library
pub struct LibraryClientSender(mpsc::Sender<Request>);

impl LibraryClientSender {
    pub fn schedule_op<F>(&self, f: F)
    where
        F: FnOnce(&CatalogDb) -> bool + Send + Sync + 'static,
    {
        let op = Op::new(f);

        on_err_out!(self.0.send(Request::Op(op)));
    }
}

impl ClientInterface for LibraryClientSender {
    /// get all the preferences
    fn get_all_preferences(&self) {
        self.schedule_op(commands::cmd_list_all_preferences);
    }

    fn set_preference(&self, key: String, value: String) {
        self.schedule_op(move |catalog| commands::cmd_set_preference(catalog, &key, &value))
    }

    /// get all the keywords
    fn get_all_keywords(&self) {
        self.schedule_op(commands::cmd_list_all_keywords);
    }

    fn query_keyword_content(&self, keyword_id: LibraryId) {
        self.schedule_op(move |catalog| commands::cmd_query_keyword_content(catalog, keyword_id));
    }

    fn count_keyword(&self, id: LibraryId) {
        self.schedule_op(move |catalog| commands::cmd_count_keyword(catalog, id));
    }

    /// Get the root folder.
    fn get_root_folders(&self, callback: ClientCallback<Vec<LibFolder>>) {
        self.schedule_op(move |catalog| commands::cmd_list_root_folders(catalog, callback));
    }

    /// get all the folders
    fn get_all_folders(&self, callback: Option<ClientCallback<Vec<LibFolder>>>) {
        self.schedule_op(move |catalog| commands::cmd_list_all_folders(catalog, callback));
    }

    fn query_folder_content(&self, folder_id: LibraryId) {
        self.schedule_op(move |catalog| commands::cmd_query_folder_content(catalog, folder_id));
    }

    fn count_folder(&self, folder_id: LibraryId) {
        self.schedule_op(move |catalog| commands::cmd_count_folder(catalog, folder_id));
    }

    fn create_folder(&self, name: String, path: Option<String>) {
        self.schedule_op(move |catalog| {
            commands::cmd_create_folder(catalog, &name, path.clone()) != 0
        });
    }

    fn delete_folder(&self, id: LibraryId) {
        // Delete folder, recursive.
        // XXX maybe one day we'll have a non recursive option.
        self.schedule_op(move |catalog| commands::cmd_delete_folder(catalog, id, true));
    }

    /// get all the albums
    fn get_all_albums(&self) {
        self.schedule_op(commands::cmd_list_all_albums);
    }

    /// Count album
    fn count_album(&self, album_id: LibraryId) {
        self.schedule_op(move |catalog| commands::cmd_count_album(catalog, album_id));
    }

    /// Create an album (async)
    fn create_album(&self, name: String, parent: LibraryId) {
        self.schedule_op(move |catalog| commands::cmd_create_album(catalog, &name, parent) != 0);
    }

    /// Delete an album
    fn delete_album(&self, id: LibraryId) {
        self.schedule_op(move |catalog| commands::cmd_delete_album(catalog, id));
    }

    /// Add images to an album.
    fn add_to_album(&self, images: &[LibraryId], album_id: LibraryId) {
        let images = images.to_vec();
        self.schedule_op(move |catalog| {
            if commands::cmd_add_to_album(catalog, images, album_id) {
                commands::cmd_count_album(catalog, album_id)
            } else {
                false
            }
        });
    }

    /// Remove images to an album.
    fn remove_from_album(&self, images: &[LibraryId], album_id: LibraryId) {
        let images = images.to_vec();
        self.schedule_op(move |catalog| {
            if commands::cmd_remove_from_album(catalog, images, album_id) {
                commands::cmd_count_album(catalog, album_id)
            } else {
                false
            }
        });
    }

    /// Rename album `album_id` to `name`.
    fn rename_album(&self, album_id: LibraryId, name: String) {
        self.schedule_op(move |catalog| commands::cmd_rename_album(catalog, album_id, &name));
    }

    /// Query content for album.
    fn query_album_content(&self, album_id: LibraryId) {
        self.schedule_op(move |catalog| commands::cmd_query_album_content(catalog, album_id));
    }

    fn request_metadata(&self, file_id: LibraryId) {
        self.schedule_op(move |catalog| commands::cmd_request_metadata(catalog, file_id));
    }

    /// set the metadata
    fn set_metadata(&self, file_id: LibraryId, meta: Np, value: &PropertyValue) {
        let value2 = value.clone();
        self.schedule_op(move |catalog| {
            commands::cmd_set_metadata(catalog, file_id, meta, &value2)
        });
    }

    fn set_image_properties(&self, image_id: LibraryId, props: &NiepcePropertyBag) {
        let props = props.clone();
        self.schedule_op(move |catalog| {
            commands::cmd_set_image_properties(catalog, image_id, &props)
        });
    }

    fn write_metadata(&self, file_id: LibraryId) {
        self.schedule_op(move |catalog| commands::cmd_write_metadata(catalog, file_id));
    }

    fn move_file_to_folder(&self, file_id: LibraryId, from: LibraryId, to: LibraryId) {
        self.schedule_op(move |catalog| {
            commands::cmd_move_file_to_folder(catalog, file_id, from, to)
        });
    }

    /// get all the labels
    fn get_all_labels(&self) {
        self.schedule_op(commands::cmd_list_all_labels);
    }

    fn create_label(&self, name: String, colour: RgbColour) {
        self.schedule_op(move |catalog| commands::cmd_create_label(catalog, &name, &colour) != 0);
    }

    fn delete_label(&self, label_id: LibraryId) {
        self.schedule_op(move |catalog| commands::cmd_delete_label(catalog, label_id));
    }

    /// update a label
    fn update_label(&self, label_id: LibraryId, new_name: String, new_colour: RgbColour) {
        self.schedule_op(move |catalog| {
            commands::cmd_update_label(catalog, label_id, &new_name, &new_colour)
        });
    }

    /// tell to process the Xmp update Queue
    fn process_xmp_update_queue(&self, write_xmp: bool) {
        self.schedule_op(move |catalog| commands::cmd_process_xmp_update_queue(catalog, write_xmp));
    }

    /// Import files in place.
    fn import_files(&self, base: PathBuf, files: Vec<PathBuf>) {
        self.schedule_op(move |catalog| commands::cmd_import_files(catalog, &base, &files));
    }
}

impl ClientInterfaceSync for LibraryClientSender {
    fn create_label_sync(&self, name: String, colour: RgbColour) -> LibraryId {
        // can't use futures::sync::oneshot
        let (tx, rx) = mpsc::sync_channel::<LibraryId>(1);

        self.schedule_op(move |catalog| {
            tx.send(commands::cmd_create_label(catalog, &name, &colour))
                .unwrap();
            true
        });

        rx.recv().unwrap()
    }

    fn create_keyword_sync(&self, keyword: String) -> LibraryId {
        // can't use futures::sync::oneshot
        let (tx, rx) = mpsc::sync_channel::<LibraryId>(1);

        self.schedule_op(move |catalog| {
            tx.send(commands::cmd_add_keyword(catalog, &keyword))
                .unwrap();
            true
        });

        rx.recv().unwrap()
    }

    fn create_folder_sync(&self, name: String, path: Option<String>) -> LibraryId {
        // can't use futures::sync::oneshot
        let (tx, rx) = mpsc::sync_channel::<LibraryId>(1);

        self.schedule_op(move |catalog| {
            tx.send(commands::cmd_create_folder(catalog, &name, path.clone()))
                .unwrap();
            true
        });

        rx.recv().unwrap()
    }

    fn create_album_sync(&self, name: String, parent: LibraryId) -> LibraryId {
        // can't use futures::sync::oneshot
        let (tx, rx) = mpsc::sync_channel::<LibraryId>(1);

        self.schedule_op(move |catalog| {
            tx.send(commands::cmd_create_album(catalog, &name, parent))
                .unwrap();
            true
        });

        rx.recv().unwrap()
    }

    fn add_bundle_sync(&self, bundle: &FileBundle, folder: LibraryId) -> LibraryId {
        let (tx, rx) = mpsc::sync_channel::<LibraryId>(1);

        let bundle = bundle.clone();
        self.schedule_op(move |catalog| {
            tx.send(commands::cmd_add_bundle(catalog, &bundle, folder))
                .unwrap();
            true
        });

        rx.recv().unwrap()
    }

    fn upgrade_catalog_from_sync(&self, version: i32) -> bool {
        let (tx, rx) = mpsc::sync_channel::<bool>(1);

        self.schedule_op(move |catalog| {
            tx.send(commands::cmd_upgrade_catalog_from(catalog, version))
                .unwrap();
            true
        });

        rx.recv().unwrap()
    }
}
