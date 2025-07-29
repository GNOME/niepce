/*
 * niepce - npc-engine/library/queriedcontent.rs
 *
 * Copyright (C) 2020-2025 Hubert Figuière
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

use crate::catalog::LibraryId;
use crate::catalog::libfile::LibFile;

/// Queried content to pass a list of LibFile and the id of the container.
#[derive(Clone, Debug)]
pub struct QueriedContent {
    pub id: LibraryId,
    pub content: Vec<LibFile>,
}

impl QueriedContent {
    pub fn new(id: LibraryId) -> Self {
        QueriedContent {
            id,
            content: vec![],
        }
    }

    pub fn push(&mut self, f: LibFile) {
        self.content.push(f);
    }

    pub fn get_content(&self) -> &Vec<LibFile> {
        &self.content
    }
}
