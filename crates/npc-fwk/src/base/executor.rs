/*
 * niepce - npc-fwk/base/worker.rs
 *
 * Copyright (C) 2025 Hubert Figuière
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

//! Executing a cancellable task in a thread.

use std::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

pub use super::worker::Status;

pub struct Executor {
    name: String,
    is_running: Arc<AtomicBool>,
    thread: Cell<Option<thread::JoinHandle<()>>>,
    terminate: Arc<AtomicBool>,
}

impl Drop for Executor {
    fn drop(&mut self) {
        self.cancel();
    }
}

impl Executor {
    /// New executor with `name`. Name is used for the thread name if
    /// applicable (works fine on Linux).
    pub fn new(name: String) -> Self {
        Executor {
            name,
            is_running: Arc::default(),
            thread: Cell::default(),
            terminate: Arc::default(),
        }
    }

    /// Return if the executor is still running.
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Acquire)
    }

    /// Return a function that callers can check to see if the task
    /// needs cancellation.
    pub fn terminator(&self) -> impl Fn() -> bool + 'static {
        let terminate = self.terminate.clone();
        move || terminate.load(Ordering::Acquire)
    }

    /// Run `f`.
    pub fn run<F>(&self, f: F)
    where
        F: Fn() -> Status + Send + 'static,
    {
        if self.is_running() {
            self.cancel();
        }
        if !self.is_running() {
            self.is_running.store(true, Ordering::Release);
            self.terminate.store(false, Ordering::Release);

            let main = self.create_main(f);
            let thread: Result<thread::JoinHandle<()>, _> =
                thread::Builder::new().name(self.name.clone()).spawn(main);
            on_err_out!(thread);
            self.thread.set(thread.ok());
        }
    }

    /// Cancel the executor.
    pub fn cancel(&self) {
        self.terminate.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            on_err_out!(thread.join());
        }
    }

    /// Create the higher order thread main function.
    fn create_main<F>(&self, f: F) -> impl Fn() + 'static
    where
        F: Fn() -> Status + Send + 'static,
    {
        let is_running: Arc<AtomicBool> = self.is_running.clone();
        let terminate: Arc<AtomicBool> = self.terminate.clone();

        move || {
            while !terminate.load(Ordering::Acquire) {
                let status = f();
                if status == Status::Stop {
                    break;
                }
            }
            is_running.store(false, Ordering::Release);
            terminate.store(false, Ordering::Release);
        }
    }
}
