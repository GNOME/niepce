/*
 * niepce - npc_engine/lib.rs
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

pub mod catalog;
pub mod importer;
pub mod library;
pub mod libraryclient;

pub use library::thumbnail_cache::ThumbnailCache;

pub type NiepcePropertySet = npc_fwk::PropertySet<catalog::NiepceProperties>;
pub type NiepcePropertyBag = npc_fwk::PropertyBag<catalog::NiepceProperties>;
