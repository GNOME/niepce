/*
 * niepce - npc-fwk/base/worker.rs
 *
 * Copyright (C) 2023 Hubert Figuière
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

use std::sync::mpsc;

/// WorkerImpl trait for a worker. All you need is a message type
/// and the dispatch method to for each message.
pub trait WorkerImpl: Send {
    type Message: Send + 'static;

    /// Dispatch message, and return true to continue.
    fn dispatch(&self, msg: Self::Message) -> bool;
}

/// Generic worker.
/// ```
/// use npc_fwk::base::{Worker, WorkerImpl};
///
/// enum SomeMessage {
///     One,
/// }
///
/// struct SomeWorker {
/// }
///
/// impl WorkerImpl for SomeWorker {
///     type Message = SomeMessage;
///
///     fn dispatch(&self, msg: Self::Message) -> bool {
///         match msg {
///             One => {}
///         }
///
///         true
///     }
/// }
///
/// let worker = Worker::new(SomeWorker{});
/// worker.send(SomeMessage::One);
/// ```
pub struct Worker<I: WorkerImpl> {
    sender: mpsc::Sender<I::Message>,
}

impl<I: WorkerImpl + Default + 'static> Default for Worker<I> {
    fn default() -> Worker<I> {
        Self::new(I::default())
    }
}

impl<I: WorkerImpl + 'static> Worker<I> {
    /// Create a new worker with the implementation.
    pub fn new(worker_impl: I) -> Worker<I> {
        let (sender, receiver) = mpsc::channel();
        let worker = Self { sender };

        on_err_out!(std::thread::Builder::new()
            .name(format!("worker-{}", stringify!(I)))
            .spawn(move || {
                while let Ok(msg) = receiver.recv() {
                    if !worker_impl.dispatch(msg) {
                        break;
                    }
                }
            }));

        worker
    }

    /// Send a message to the worker.
    pub fn send(&self, msg: I::Message) -> Result<(), mpsc::SendError<I::Message>> {
        self.sender.send(msg)
    }
}
