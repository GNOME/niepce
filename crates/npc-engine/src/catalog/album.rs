/*
 * niepce - npc-engine/db/album.rs
 *
 * Copyright (C) 2021-2022 Hubert Figuière
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

use super::FromDb;
use super::LibraryId;
use super::SortOrder;

/// Represents an album, that contains image
#[derive(Clone, Debug)]
pub struct Album {
    /// Album ID
    id: LibraryId,
    /// Album name as displayed
    name: String,
    /// Album Parent. -1 for no parent.
    parent: LibraryId,
    /// Sorting
    order: SortOrder,
    #[allow(dead_code)]
    /// Key
    order_by: String,
}

impl Album {
    pub fn new(id: LibraryId, name: &str, parent: LibraryId) -> Self {
        Album {
            id,
            name: name.to_owned(),
            parent,
            order: SortOrder::NoSorting,
            order_by: "".to_owned(),
        }
    }

    /// Get the album ID
    pub fn id(&self) -> LibraryId {
        self.id
    }

    /// Get the album name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get parent album ID.
    pub fn parent(&self) -> LibraryId {
        self.parent
    }

    pub fn order(&self) -> SortOrder {
        self.order
    }

    pub fn set_order(&mut self, order: SortOrder) {
        self.order = order;
    }
}

impl FromDb for Album {
    fn read_db_columns() -> &'static str {
        "id,name,parent_id"
    }

    fn read_db_tables() -> &'static str {
        "albums"
    }

    fn read_db_where_id() -> &'static str {
        "id"
    }

    fn read_from(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let name: String = row.get(1)?;
        Ok(Album::new(row.get(0)?, &name, row.get(2)?))
    }
}
