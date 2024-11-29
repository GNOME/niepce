/*
 * niepce - fwk/base/signals.rs
 *
 * Copyright (C) 2022-2024 Hubert Figuière
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

//! Implement simple signals/slot. Unlike the `signals2` create this doesn't
//! require `Sync` + `Send`.

use std::sync::{Arc, RwLock};

type Slot<T> = Box<dyn Fn(T)>;

/// A Signal will dispatch a value on `emit` for any of the connected slots.
///
/// A cloned signal just clone the outer and maintain the same slots
/// through a reference count.
///
/// Note: Signal is currently unsync.
pub struct Signal<T>
where
    T: Clone,
{
    slots: Arc<RwLock<Vec<Slot<T>>>>,
}

impl<T> Signal<T>
where
    T: Clone,
{
    /// Create a new signal.
    pub fn new() -> Self {
        Signal {
            slots: Arc::new(RwLock::new(Vec::default())),
        }
    }

    /// Emit the `param`: call all the slots.
    pub fn emit(&self, param: T) {
        self.slots
            .read()
            .unwrap()
            .iter()
            .for_each(move |s| s(param.clone()));
    }

    /// Connect a slot.
    pub fn connect<F: Fn(T) + 'static>(&self, f: F) {
        self.slots.write().unwrap().push(Box::new(f))
    }
}

impl<T> Default for Signal<T>
where
    T: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for Signal<T>
where
    T: Clone,
{
    /// Cloning a signal just clone the outter.
    fn clone(&self) -> Self {
        Signal {
            slots: self.slots.clone(),
        }
    }
}
