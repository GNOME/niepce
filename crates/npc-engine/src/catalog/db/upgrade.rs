/*
 * niepce - npc-engine/src/catalog/db/upgrade.rs
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

//! The database schema upgrade
#![doc = include_str!("../../../../../doc/database_upgrade.md")]

use super::{CatalogDb, Error, Result, sql};
use npc_fwk::dbg_out;

/// Upgrade catalog `from` version `to` version
/// Will run the step by step upgrade
///
/// A few notes:
/// Some `from` `to` combination are no-op. Like anything before version 11
/// as it's early development.
///
/// If `from` is not the current version, it will return `Error::IncorrectDbVersion`.
///
/// `ALTER TABLE` is limited in sqlite and some internal trickery is necessary.
/// See sqlite documentation <https://www.sqlite.org/lang_altertable.html>,
/// section 7.
///
pub(crate) fn catalog_to(catalog: &CatalogDb, from: i32, to: i32) -> Result<()> {
    if from > to {
        return Err(Error::IncorrectDbVersion);
    }
    let version = catalog.check_database_version()?;
    if version != from {
        return Err(Error::IncorrectDbVersion);
    }
    dbg_out!("upgrade from {} to {}", from, to);
    for v in from..to {
        dbg_out!("handling {}", v + 1);
        // XXX remove the allow when there is one more branch.
        // We want that structure.
        match v + 1 {
            11 => {
                // This was tested from version 9. Some feature branch mess make that
                // there are two version 10.
                if let Some(conn) = &catalog.dbconn {
                    let schema_version = sql::pragma_schema_version(conn)?;
                    perform_upgrade_11(conn, schema_version).expect("Upgrade failed");
                    catalog.set_db_version(11).expect("set_db_version failed");
                }
            }
            12 => {
                if let Some(conn) = &catalog.dbconn {
                    let schema_version = sql::pragma_schema_version(conn)?;
                    perform_upgrade_12(conn, schema_version).expect("Upgrade failed");
                    catalog.set_db_version(12).expect("set_db_version failed");
                }
            }
            13 => {
                if let Some(conn) = &catalog.dbconn {
                    let schema_version = sql::pragma_schema_version(conn)?;
                    perform_upgrade_13(conn, schema_version).expect("Upgrade failed");
                    catalog.set_db_version(13).expect("set_db_version failed");
                }
            }
            _ => {}
        }
    }

    Ok(())
}

pub(crate) fn perform_upgrade_13(conn: &rusqlite::Connection, schema_version: i64) -> Result<()> {
    dbg_out!("schema_version {}", schema_version);
    dbg_out!("upgrade 13");
    conn.execute_batch(
        "BEGIN;\
         CREATE TABLE admin_new (key TEXT NOT NULL PRIMARY KEY, value TEXT); \
         INSERT INTO admin_new SELECT * FROM admin; \
         DROP TABLE admin; \
         ALTER TABLE admin_new RENAME TO admin; \
         COMMIT;",
    )?;

    Ok(())
}

pub(crate) fn perform_upgrade_12(conn: &rusqlite::Connection, schema_version: i64) -> Result<()> {
    dbg_out!("schema_version {}", schema_version);
    dbg_out!("upgrade 12, step 1");
    conn.execute_batch(
        "BEGIN;\
         CREATE TABLE folders_new (id INTEGER PRIMARY KEY AUTOINCREMENT, \
         path TEXT, name TEXT, \
         vault_id INTEGER DEFAULT 0, \
         locked INTEGER DEFAULT 0, \
         virtual INTEGER DEFAULT 0, \
         expanded INTEGER DEFAULT 0, \
         parent_id INTEGER, UNIQUE(name, parent_id)); \
         CREATE TRIGGER folders_insert AFTER INSERT ON folders_new \
         BEGIN \
         UPDATE folders_new SET path = (SELECT f.path FROM folders_new AS f WHERE f.id = folders_new.parent_id) || '/' || name WHERE id = new.id AND parent_id != 0; \
         END; \
         CREATE TRIGGER folders_update_parent AFTER UPDATE OF parent_id ON folders_new \
         BEGIN \
         UPDATE folders_new SET path = (SELECT f.path FROM folders_new AS f WHERE f.id = folders_new.parent_id) || '/' || name WHERE id = NEW.id AND parent_id != 0; \
         END; \
         INSERT INTO folders_new SELECT * FROM folders; \
         DROP TRIGGER folder_delete_trigger; \
         CREATE TRIGGER folder_delete_trigger AFTER DELETE ON folders_new \
         BEGIN \
         DELETE FROM files WHERE parent_id = old.id; \
         END; \
         DROP TABLE folders; \
         ALTER TABLE folders_new RENAME TO folders; \
         COMMIT;",
    )?;

    dbg_out!("upgrade 12, step 2");
    conn.execute_batch(
        "BEGIN;\
         CREATE TABLE keywords_new (id INTEGER PRIMARY KEY AUTOINCREMENT, \
         keyword TEXT, parent_id INTEGER DEFAULT 0, \
         UNIQUE(keyword, parent_id)); \
         INSERT INTO keywords_new SELECT * FROM keywords; \
         DROP TRIGGER keyword_delete_trigger; \
         CREATE TRIGGER keyword_delete_trigger AFTER DELETE ON keywords \
         BEGIN \
         DELETE FROM keywording WHERE keyword_id = old.id; \
         END; \
         DROP TABLE keywords; \
         ALTER TABLE keywords_new RENAME TO keywords; \
         COMMIT;",
    )?;

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
         PRAGMA writable_schema=RESET;\
         PRAGMA integrity_check;\
         CREATE TABLE albums (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, parent_id INTEGER);\
         CREATE TABLE albuming (file_id INTEGER, album_id INTEGER, UNIQUE(file_id, album_id));\
         CREATE TRIGGER album_delete_trigger AFTER DELETE ON albums BEGIN DELETE FROM albuming WHERE album_id = old.id; END;\
         UPDATE files SET xmp_file = 0 WHERE xmp_file IS NULL;\
         UPDATE files SET jpeg_file = 0 WHERE jpeg_file IS NULL;\
         COMMIT;"
    );
    conn.execute_batch(&sql)?;
    Ok(())
}
