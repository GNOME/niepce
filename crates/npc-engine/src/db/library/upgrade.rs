/*
 * niepce - npc-engine/src/db/upgrade.rs
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

//! The database schema upgrade

use super::{sql, Error, Library, Result};
use npc_fwk::dbg_out;

/// Upgrade library `from` version `to` version
/// Will run the step by step upgrade
///
/// A few notes:
/// Some `from` `to` combination are no-op. Like anything before version 11
/// as it's early development.
///
/// If `from` is not the current version, it will return `Error::IncorrectDbVersion`.
///
/// `ALTER TABLE` is limited in sqlite and some internal trickery is necessary.
/// See sqlite documentation https://www.sqlite.org/lang_altertable.html, section 7.
///
pub(crate) fn library_to(library: &Library, from: i32, to: i32) -> Result<()> {
    if from > to {
        return Err(Error::IncorrectDbVersion);
    }
    let version = library.check_database_version()?;
    if version != from {
        return Err(Error::IncorrectDbVersion);
    }
    dbg_out!("upgrade from {} to {}", from, to);
    for v in from..to {
        dbg_out!("handling {}", v + 1);
        // XXX remove the allow when there is one more branch.
        // We want that structure.
        #[allow(clippy::single_match)]
        match v + 1 {
            11 => {
                // This was tested from version 9. Some feature branch mess make that
                // there are two version 10.
                if let Some(conn) = &library.dbconn {
                    let schema_version = sql::pragma_schema_version(conn)?;
                    perform_upgrade_11(conn, schema_version).expect("Upgrade failed");
                    library.set_db_version(11).expect("set_db_version failed");
                }
            }
            _ => {}
        }
    }

    Ok(())
}

pub(crate) fn perform_upgrade_11(conn: &rusqlite::Connection, schema_version: i64) -> Result<()> {
    dbg_out!("schema_version {}", schema_version);
    let sql = format!(
        "BEGIN;\
         PRAGMA writable_schema=ON;\
         UPDATE sqlite_schema SET sql='CREATE TABLE files (id INTEGER PRIMARY KEY AUTOINCREMENT, main_file INTEGER, name TEXT, parent_id INTEGER, orientation INTEGER, file_type INTEGER, file_date INTEGER, rating INTEGER DEFAULT 0, label INTEGER, flag INTEGER DEFAULT 0, import_date INTEGER, mod_date INTEGER, xmp TEXT, xmp_date INTEGER, xmp_file INTEGER DEFAULT 0, jpeg_file INTEGER DEFAULT 0)' WHERE type='table' AND name='files';\
         UPDATE sqlite_schema SET sql='CREATE TABLE vaults (id INTEGER PRIMARY KEY AUTOINCREMENT, path TEXT)' WHERE type='table' AND name='vaults';\
         UPDATE sqlite_schema SET sql='CREATE TABLE folders (id INTEGER PRIMARY KEY AUTOINCREMENT, path TEXT, name TEXT, vault_id INTEGER DEFAULT 0, locked INTEGER DEFAULT 0, virtual INTEGER DEFAULT 0, expanded INTEGER DEFAULT 0, parent_id INTEGER)' WHERE type='table' AND name='folders';\
         UPDATE sqlite_schema SET sql='CREATE TRIGGER file_delete_trigger AFTER DELETE ON files BEGIN DELETE FROM sidecars WHERE file_id = old.id; DELETE FROM keywording WHERE file_id = old.id; DELETE FROM albuming WHERE file_id = old.id; END' WHERE type='trigger' AND name='file_delete_trigger';\
         PRAGMA schema_version={schema_version};\
         PRAGMA writable_schema=OFF;\
         PRAGMA integrity_check;\
         CREATE TABLE albums (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, parent_id INTEGER);\
         CREATE TABLE albuming (file_id INTEGER, album_id INTEGER, UNIQUE(file_id, album_id));\
         CREATE TRIGGER album_delete_trigger AFTER DELETE ON albums BEGIN DELETE FROM albuming WHERE album_id = old.id; END;\
         UPDATE files SET xmp_file = 0 WHERE xmp_file IS NULL;\
         UPDATE files SET jpeg_file = 0 WHERE jpeg_file IS NULL;\
         COMMIT;");
    conn.execute_batch(&sql)?;
    Ok(())
}
