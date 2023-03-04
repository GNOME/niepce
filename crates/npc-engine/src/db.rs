/*
 * niepce - npc-engine/src/db.rs
 *
 * Copyright (C) 2017-2023 Hubert Figuière
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

pub mod album;
pub mod filebundle;
pub mod fsfile;
pub mod keyword;
pub mod label;
pub mod libfile;
pub mod libfolder;
pub mod libmetadata;
mod library;
pub mod props;

pub type LibraryId = i64;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SortOrder {
    NoSorting,
    Ascending,
    Descending,
}

// flatten namespace a bit.
pub use self::album::Album;
pub use self::keyword::Keyword;
pub use self::label::Label;
pub use self::libfile::{FileType, LibFile};
pub use self::libfolder::LibFolder;
pub use self::libmetadata::LibMetadata;
pub use self::library::{Error as LibError, Library, Managed, Result as LibResult};
pub use self::props::NiepceProperties;
pub use self::props::NiepcePropertyIdx;

pub trait FromDb: Sized {
    /// return the columns for reading from the DB.
    fn read_db_columns() -> &'static str;
    /// return the tables for reading from the DB.
    fn read_db_tables() -> &'static str;
    /// return the column for the where clause on the id for the DB.
    fn read_db_where_id() -> &'static str;
    /// read a new object from the DB row.
    fn read_from(row: &rusqlite::Row) -> rusqlite::Result<Self>;
}
