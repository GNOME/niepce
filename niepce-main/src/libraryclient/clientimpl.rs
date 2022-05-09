/*
 * niepce - libraryclient/clientimpl.rs
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

use std::path::PathBuf;
use std::sync;
use std::sync::atomic;
use std::sync::mpsc;
use std::thread;

use super::clientinterface::{ClientInterface, ClientInterfaceSync};
use npc_engine::db::library::Managed;
use npc_engine::db::props::NiepceProperties as Np;
use npc_engine::db::{Library, LibraryId};
use npc_engine::library::commands;
use npc_engine::library::notification::LibNotification;
use npc_engine::library::op::Op;
use npc_fwk::base::PropertyValue;
use npc_fwk::{err_out, on_err_out};

pub struct ClientImpl {
    terminate: sync::Arc<atomic::AtomicBool>,
    sender: mpsc::Sender<Op>,
}

impl Drop for ClientImpl {
    fn drop(&mut self) {
        self.stop();
    }
}

impl ClientImpl {
    pub fn new(dir: PathBuf, sender: npc_fwk::toolkit::Sender<LibNotification>) -> ClientImpl {
        let (task_sender, task_receiver) = mpsc::channel::<Op>();

        let mut terminate = sync::Arc::new(atomic::AtomicBool::new(false));
        let terminate2 = terminate.clone();

        /* let thread = */
        thread::spawn(move || {
            let library = Library::new(&dir, None, sender);
            Self::main(&mut terminate, task_receiver, &library);
        });

        ClientImpl {
            terminate: terminate2,
            sender: task_sender,
        }
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

impl ClientInterface for ClientImpl {
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

    /// get all the folder
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

    fn request_metadata(&self, file_id: LibraryId) {
        self.schedule_op(move |lib| commands::cmd_request_metadata(lib, file_id));
    }

    /// set the metadata
    fn set_metadata(&self, file_id: LibraryId, meta: Np, value: &PropertyValue) {
        let value2 = value.clone();
        self.schedule_op(move |lib| commands::cmd_set_metadata(lib, file_id, meta, &value2));
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

impl ClientInterfaceSync for ClientImpl {
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
}
