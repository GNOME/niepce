/*
 * niepce - npc-engine/src/db/sql.rs
 *
 * Copyright (C) 2022-2023 Hubert Figuière
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

//! SQL utilities for the db library

#[cfg(test)]
use rusqlite::params;

use super::{Error, Result};

/// Get the sqlite schema version. Note: this is unrelated to the `DB_SCHEMA_VERSION`
/// const in library, and should never be relied on.
pub(super) fn pragma_schema_version(conn: &rusqlite::Connection) -> Result<i64> {
    let mut stmt = conn.prepare("PRAGMA schema_version")?;
    let mut rows = stmt.query([])?;
    match rows.next()? {
        Some(row) => Ok(row.get(0)?),
        None => Err(Error::NotFound),
    }
}

/// Get the SQL for the table.
/// Currently used only for the test.
#[cfg(test)]
pub(super) fn table_sql(conn: &rusqlite::Connection, table: &str) -> Result<String> {
    let mut stmt = conn.prepare("SELECT sql FROM sqlite_schema WHERE type='table' AND name=?1")?;
    let mut rows = stmt.query(params![table])?;
    match rows.next()? {
        Some(row) => Ok(row.get(0)?),
        None => Err(Error::NotFound),
    }
}

#[cfg(test)]
pub(super) fn trigger_sql(conn: &rusqlite::Connection, table: &str) -> Result<String> {
    let mut stmt =
        conn.prepare("SELECT sql FROM sqlite_schema WHERE type='trigger' AND name=?1")?;
    let mut rows = stmt.query(params![table])?;
    match rows.next()? {
        Some(row) => Ok(row.get(0)?),
        None => Err(Error::NotFound),
    }
}
