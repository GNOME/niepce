/*
 * niepce - npc_engine/lib.rs
 *
 * Copyright (C) 2017-2026 Hubert Figuière
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
pub mod metadata;

pub use library::thumbnail_cache::ThumbnailCache;

pub use metadata::xmp::exempi_manager;

pub type NiepcePropertySet = npc_fwk::PropertySet<catalog::NiepcePropertyIdx>;
pub type NiepcePropertyBag = npc_fwk::PropertyBag<catalog::NiepcePropertyIdx>;
