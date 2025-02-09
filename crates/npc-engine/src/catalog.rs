/*
 * niepce - npc-engine/src/db.rs
 *
 * Copyright (C) 2017-2024 Hubert Figuière
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
pub(crate) mod db;
pub mod filebundle;
pub mod fsfile;
pub mod keyword;
pub mod label;
pub mod libfile;
pub mod libfolder;
pub mod libmetadata;
pub mod props;

pub type LibraryId = i64;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SortOrder {
    NoSorting,
    Ascending,
    Descending,
}

// flatten namespace a bit.
pub use album::Album;
#[cfg(test)]
pub(crate) use db::test as db_test;
pub use db::{CatalogDb, Error as LibError, Result as LibResult};
pub use keyword::Keyword;
pub use label::Label;
pub use libfile::{FileType, LibFile};
pub use libfolder::LibFolder;
pub use libmetadata::LibMetadata;
pub use props::NiepceProperties;
pub use props::NiepcePropertyIdx;

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
