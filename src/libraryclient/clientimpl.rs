/*
 * niepce - libraryclient/clientimpl.rs
 *
 * Copyright (C) 2017-2019 Hubert Figuière
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

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync;
use std::sync::mpsc;
use std::sync::atomic;
use std::thread;

use fwk::base::PropertyValue;
use engine::db::{Library, LibraryId};
use engine::db::library::Managed;
use engine::library::op::Op;
use engine::library::commands;
use engine::library::notification::LibNotification;
use super::clientinterface::{ClientInterface,ClientInterfaceSync};
use root::eng::NiepceProperties as Np;

pub struct ClientImpl {
    terminate: sync::Arc<atomic::AtomicBool>,
    tasks: sync::Arc<(sync::Mutex<VecDeque<Op>>, sync::Condvar)>,
}

impl Drop for ClientImpl {
    fn drop(&mut self) {
        self.stop();
    }
}

impl ClientImpl {

    pub fn new(dir: PathBuf, sender: glib::Sender<LibNotification>) -> ClientImpl {
        let tasks = sync::Arc::new((sync::Mutex::new(VecDeque::new()), sync::Condvar::new()));
        let mut terminate = sync::Arc::new(atomic::AtomicBool::new(false));
        let tasks2 = tasks.clone();
        let terminate2 = terminate.clone();

        /* let thread = */ thread::spawn(move || {
            let library = Library::new(&dir, None, sender);
            Self::main(&mut terminate, &tasks, &library);
        });


        ClientImpl {
            terminate: terminate2,
            tasks: tasks2,
        }
    }

    fn stop(&mut self) {
        self.terminate.store(true, atomic::Ordering::Relaxed);
        self.tasks.1.notify_one();
    }

    fn main(terminate: &mut sync::Arc<atomic::AtomicBool>,
            tasks: &sync::Arc<(sync::Mutex<VecDeque<Op>>, sync::Condvar)>,
            library: &Library) {

        while !terminate.load(atomic::Ordering::Relaxed) {

            let elem: Option<Op>;
            {
                let mut queue = tasks.0.lock().unwrap();
                if !queue.is_empty() {
                    elem = queue.pop_front();
                } else {
                    elem = None;
                }
            }

            if let Some(op) = elem {
                op.execute(library);
            }

            let queue = tasks.0.lock().unwrap();
            if queue.is_empty() {
                if let Err(err) = tasks.1.wait(queue) {
                    err_out!("queue lock poisonned: {}", err);
                    break;
                }
            }
        }
    }

    pub fn schedule_op<F>(&mut self, f: F)
        where F: Fn(&Library) -> bool + Send + Sync + 'static {

        let op = Op::new(f);

        let mut queue = self.tasks.0.lock().unwrap();
        queue.push_back(op);
        self.tasks.1.notify_all();
    }

}

impl ClientInterface for ClientImpl {
    /// get all the keywords
    fn get_all_keywords(&mut self) {
        self.schedule_op(move |lib| {
            commands::cmd_list_all_keywords(&lib)
        });
    }

    fn query_keyword_content(&mut self, keyword_id: LibraryId) {
        self.schedule_op(move |lib| {
            commands::cmd_query_keyword_content(&lib, keyword_id)
        });
    }

    fn count_keyword(&mut self, id: LibraryId) {
        self.schedule_op(move |lib| {
            commands::cmd_count_keyword(&lib, id)
        });
    }

    /// get all the folder
    fn get_all_folders(&mut self) {
        self.schedule_op(move |lib| {
            commands::cmd_list_all_folders(&lib)
        });
    }

    fn query_folder_content(&mut self, folder_id: LibraryId) {
        self.schedule_op(move |lib| {
            commands::cmd_query_folder_content(&lib, folder_id)
        });
    }

    fn count_folder(&mut self, folder_id: LibraryId) {
        self.schedule_op(move |lib| {
            commands::cmd_count_folder(&lib, folder_id)
        });
    }

    fn create_folder(&mut self, name: String, path: Option<String>) {
        self.schedule_op(move |lib| {
            commands::cmd_create_folder(&lib, &name, path.clone()) != 0
        });
    }

    fn delete_folder(&mut self, id: LibraryId) {
        self.schedule_op(move |lib| {
            commands::cmd_delete_folder(&lib, id)
        });
    }

    fn request_metadata(&mut self, file_id: LibraryId) {
        self.schedule_op(move |lib| {
            commands::cmd_request_metadata(&lib, file_id)
        });
    }

    /// set the metadata
    fn set_metadata(&mut self, file_id: LibraryId, meta: Np, value: &PropertyValue) {
        let value2 = value.clone();
        self.schedule_op(move |lib| {
            commands::cmd_set_metadata(&lib, file_id, meta, &value2)
        });
    }
    fn write_metadata(&mut self, file_id: LibraryId) {
        self.schedule_op(move |lib| {
            commands::cmd_write_metadata(&lib, file_id)
        });
    }

    fn move_file_to_folder(&mut self, file_id: LibraryId, from: LibraryId,
                           to: LibraryId) {
        self.schedule_op(move |lib| {
            commands::cmd_move_file_to_folder(&lib, file_id, from, to)
        });
    }

    /// get all the labels
    fn get_all_labels(&mut self) {
        self.schedule_op(move |lib| {
            commands::cmd_list_all_labels(&lib)
        });
    }
    fn create_label(&mut self, name: String, colour: String) {
        self.schedule_op(move |lib| {
            commands::cmd_create_label(&lib, &name, &colour) != 0
        });
    }
    fn delete_label(&mut self, label_id: LibraryId) {
        self.schedule_op(move |lib| {
            commands::cmd_delete_label(&lib, label_id)
        });
    }
    /// update a label
    fn update_label(&mut self, label_id: LibraryId, new_name: String, new_colour: String) {
        self.schedule_op(move |lib| {
            commands::cmd_update_label(&lib, label_id, &new_name, &new_colour)
        });
    }

    /// tell to process the Xmp update Queue
    fn process_xmp_update_queue(&mut self, write_xmp: bool) {
        self.schedule_op(move |lib| {
            commands::cmd_process_xmp_update_queue(&lib, write_xmp)
        });
    }

    /// Import files from a directory
    /// @param dir the directory
    /// @param manage true if imports have to be managed
    fn import_files(&mut self, dir: String, files: Vec<String>, manage: Managed) {
        self.schedule_op(move |lib| {
            commands::cmd_import_files(&lib, &dir, &files, manage)
        });
    }
}


impl ClientInterfaceSync for ClientImpl {

    fn create_label_sync(&mut self, name: String, colour: String) -> LibraryId {
        // can't use futures::sync::oneshot
        let (tx, rx) = mpsc::sync_channel::<LibraryId>(1);

        self.schedule_op(move |lib| {
            tx.send(commands::cmd_create_label(&lib, &name, &colour)).unwrap();
            true
        });

        rx.recv().unwrap()
    }

    fn create_keyword_sync(&mut self, keyword: String) -> LibraryId {
        // can't use futures::sync::oneshot
        let (tx, rx) = mpsc::sync_channel::<LibraryId>(1);

        self.schedule_op(move |lib| {
            tx.send(commands::cmd_add_keyword(&lib, &keyword)).unwrap();
            true
        });

        rx.recv().unwrap()
    }

    fn create_folder_sync(&mut self, name: String, path: Option<String>) -> LibraryId {
        // can't use futures::sync::oneshot
        let (tx, rx) = mpsc::sync_channel::<LibraryId>(1);

        self.schedule_op(move |lib| {
            tx.send(commands::cmd_create_folder(&lib, &name, path.clone())).unwrap();
            true
        });

        rx.recv().unwrap()
    }
}
