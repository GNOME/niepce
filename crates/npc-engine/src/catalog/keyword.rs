/*
 * niepce - engine/db/keyword.rs
 *
 * Copyright (C) 2017-2022 Hubert Figuière
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

#[derive(Clone, Debug)]
pub struct Keyword {
    id: LibraryId,
    keyword: String,
}

impl Keyword {
    pub fn new(id: LibraryId, keyword: &str) -> Keyword {
        Keyword {
            id,
            keyword: String::from(keyword),
        }
    }

    pub fn id(&self) -> LibraryId {
        self.id
    }

    pub fn keyword(&self) -> &str {
        &self.keyword
    }
}

impl FromDb for Keyword {
    fn read_db_columns() -> &'static str {
        "id,keyword"
    }

    fn read_db_tables() -> &'static str {
        "keywords"
    }

    fn read_db_where_id() -> &'static str {
        "id"
    }

    fn read_from(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let kw: String = row.get(1)?;
        Ok(Keyword::new(row.get(0)?, &kw))
    }
}
