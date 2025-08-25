/*
 * niepce - npc-fwk/base.rs
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

use std::collections::BTreeSet;

#[macro_use]
pub mod debug;

pub mod date;
mod executor;
pub mod fractions;
mod geometry;
mod indexed_map;
mod moniker;
mod path_tree;
pub mod propertybag;
pub mod propertyvalue;
pub mod rgbcolour;
pub mod signals;
mod worker;

pub type PropertyIndex = u32;
pub type PropertySet<T> = BTreeSet<T>;

pub use executor::Executor;
pub use geometry::{Rect, Size};
pub use indexed_map::IndexedMap;
pub use moniker::Moniker;
pub use path_tree::{PathTree, PathTreeItem};
pub use propertyvalue::PropertyValue;
pub use rgbcolour::{ColourSpace, RgbColour};
pub use signals::Signal;
pub use worker::Status as WorkerStatus;
pub use worker::{Worker, WorkerImpl};
