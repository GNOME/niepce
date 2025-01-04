/*
 * niepce - niepce/ui/content_view.rs
 *
 * Copyright (C) 2022-2025 Hubert Figuière
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

use npc_engine::catalog::LibraryId;

/// Define the what is being viewed
#[derive(Clone, Copy, Default)]
pub enum ContentView {
    /// No content
    #[default]
    Empty,
    /// Folder with id
    Folder(LibraryId),
    /// Album with id
    Album(LibraryId),
    /// Keyword with id
    Keyword(LibraryId),
}
