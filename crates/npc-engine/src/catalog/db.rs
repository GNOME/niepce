/*
 * niepce - engine/catalog/db.rs
 *
 * Copyright (C) 2017-2025 Hubert Figuière
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

//! The database for the catalog
#![doc = include_str!("../../../../doc/database.md")]

#[cfg(test)]
mod schema_test;
mod sql;
mod upgrade;

use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::result;

use chrono::Utc;
use num_traits::ToPrimitive;
use rusqlite::{functions::FunctionFlags, params};
use thiserror::Error;

use super::{FromDb, LibraryId};
use crate::NiepcePropertyBag;
use crate::catalog::NiepcePropertyIdx as Npi;
use crate::catalog::album::Album;
use crate::catalog::filebundle::{FileBundle, Sidecar};
use crate::catalog::keyword::Keyword;
use crate::catalog::label::Label;
use crate::catalog::libfile;
use crate::catalog::libfile::LibFile;
use crate::catalog::libfolder;
use crate::catalog::libfolder::LibFolder;
use crate::catalog::libmetadata::LibMetadata;
use crate::catalog::props::NiepceProperties as Np;
use crate::library::notification::{FolderReparent, LibNotification};
use npc_fwk::PropertyValue;
use npc_fwk::base::RgbColour;
use npc_fwk::toolkit;
use npc_fwk::{dbg_assert, dbg_out, err_out, on_err_out};

const DB_SCHEMA_VERSION: i32 = 13;

/// Error from the library database
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    /// Operation is unimplemented
    #[error("Unimplemented")]
    Unimplemented,
    /// Item was not found
    #[error("Not found")]
    NotFound,
    /// No SQL database
    #[error("No SQL database")]
    NoSqlDb,
    /// Database schema version is incorrect
    #[error("Incorrect DB version")]
    IncorrectDbVersion,
    /// Argument is invalid
    #[error("Invalid argument")]
    InvalidArg,
    /// Result is invalid
    #[error("Invalid result")]
    InvalidResult,
    /// Database isn't backed by a file.
    #[error("No DB file")]
    NoDbFile,
    /// SQL Error
    #[error("rusqlite error: {0}")]
    SqlError(#[from] rusqlite::Error),
    /// Strip Prefix
    #[error("Strip prefix error: {0}")]
    StripPrefix(#[from] std::path::StripPrefixError),
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
/// Result from a folder operaton.
/// Can be dereferenced as a `LibFolder`
pub enum FolderOpResult {
    /// Folder already exists
    Existing(LibFolder),
    /// Folder created.
    Created(LibFolder),
    // XXX missing Removed()
}

impl std::ops::Deref for FolderOpResult {
    type Target = LibFolder;

    fn deref(&self) -> &Self::Target {
        match self {
            FolderOpResult::Created(lf) | FolderOpResult::Existing(lf) => lf,
        }
    }
}

/// The Catalog database
pub struct CatalogDb {
    /// Sqlite3 connection handle.
    dbconn: Option<rusqlite::Connection>,
    /// The file backing the DB, if any.
    dbfile: Option<PathBuf>,
    /// True if initialized.
    inited: bool,
    /// Sender for notifications.
    sender: npc_fwk::toolkit::Sender<LibNotification>,
}

impl CatalogDb {
    /// New database library in memory (testing only)
    #[cfg(test)]
    pub(crate) fn new_in_memory(sender: npc_fwk::toolkit::Sender<LibNotification>) -> CatalogDb {
        let mut catalog = CatalogDb {
            // maindir: dir,
            dbconn: None,
            dbfile: None,
            inited: false,
            sender,
        };

        if let Ok(conn) = rusqlite::Connection::open_in_memory() {
            catalog.dbconn = Some(conn);
            catalog.inited = catalog.init().is_ok();
        }

        catalog
    }

    pub fn new(filename: &Path, sender: npc_fwk::toolkit::Sender<LibNotification>) -> CatalogDb {
        let mut catalog = CatalogDb {
            // maindir: dir,
            dbconn: None,
            dbfile: Some(filename.to_path_buf()),
            inited: false,
            sender,
        };

        match rusqlite::Connection::open(filename) {
            Ok(conn) => {
                catalog.dbconn = Some(conn);
                catalog.inited = catalog.init().is_ok();
            }
            Err(err) => {
                err_out!("open failed {:?}", err);
            }
        };

        catalog
    }

    /// Load the database and perform some sanity checking
    /// If the database is empty, it will call `init_db()`
    fn init(&mut self) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let sender = self.sender.clone();
            if let Err(err) = conn.create_scalar_function(
                "rewrite_xmp",
                0,
                FunctionFlags::SQLITE_UTF8,
                move |_| {
                    if let Err(err) = toolkit::thread_context()
                        .block_on(sender.send(LibNotification::XmpNeedsUpdate))
                    {
                        // This not fatal, at least the data should be saved.
                        // But still, it's not good.
                        err_out!("Error sending XmpNeedsUpdate notification: {}", err);
                    }
                    Ok(true)
                },
            ) {
                err_out!("failed to create scalar function.");
                return Err(Error::SqlError(err));
            }
        } else {
            return Err(Error::NoSqlDb);
        }

        match self.check_database_version() {
            Err(Error::InvalidResult) => {
                // error
                dbg_out!("version check incorrect value");
                Err(Error::IncorrectDbVersion)
            }
            Ok(0) => {
                // let's create our DB
                dbg_out!("version == 0");
                self.init_db().map(|_| {
                    on_err_out!(self.notify(LibNotification::DatabaseReady));
                })
            }
            Ok(version) => {
                if version != DB_SCHEMA_VERSION {
                    // WAT?
                    err_out!(
                        "Version mismatch, found {} expected {}",
                        version,
                        DB_SCHEMA_VERSION
                    );
                    on_err_out!(self.notify(LibNotification::DatabaseNeedUpgrade(version)));
                    Err(Error::IncorrectDbVersion)
                } else {
                    on_err_out!(self.notify(LibNotification::DatabaseReady));
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    /// Backup the backing file for the database and return the path.
    /// Will return `Error::NoDbFile` if it's a memory backed database,
    /// `Error::InvalidResult` if it can't get the new file name, or
    /// simply an error from `rusqlite`.
    fn backup_database_file(&self, suffix: &str) -> Result<PathBuf> {
        if self.dbfile.is_none() {
            // No backup if there is no backing file.
            return Err(Error::NoDbFile);
        }
        self.dbfile
            .as_ref()
            .and_then(|file| {
                let mut dest_file = file.clone();
                file.file_name()
                    .map(|name| {
                        let mut new_name = name.to_os_string();
                        new_name.push("-");
                        new_name.push(suffix);
                        dbg_out!("new name: {:?}", new_name);
                        new_name
                    })
                    .map(|new_name| {
                        dest_file.set_file_name(new_name);
                        dbg_out!("dest_file: {:?}", dest_file);
                        dest_file
                    })
            })
            .ok_or(Error::InvalidResult)
            .and_then(|dest_file| {
                dbg_out!("backing up database to {:?}", dest_file);
                if let Some(ref conn) = self.dbconn {
                    conn.backup(rusqlite::MAIN_DB, &dest_file, None)?;
                }
                Ok(dest_file)
            })
    }

    /// Perform the upgrade. This is called in response to a DatabaseNeedUpgrade
    ///
    /// It will perform a backup of the database.
    pub fn perform_upgrade(&self, from_version: i32) -> Result<()> {
        dbg_out!("Upgrading...");
        let suffix = format!("version_{from_version}");
        self.backup_database_file(&suffix)?;
        upgrade::catalog_to(self, from_version, DB_SCHEMA_VERSION)?;
        on_err_out!(self.notify(LibNotification::DatabaseReady));

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn is_ok(&self) -> bool {
        self.inited
    }

    /// Check the database version as stored in the admin table.
    /// A version of 0 mean the database is empty.
    fn check_database_version(&self) -> Result<i32> {
        if let Some(ref conn) = self.dbconn {
            if let Ok(mut stmt) = conn.prepare("SELECT value FROM admin WHERE key='version'") {
                let mut rows = stmt.query([])?;
                if let Ok(Some(row)) = rows.next() {
                    let value: String = row.get(0)?;
                    return value.parse::<i32>().map_err(|_| Error::InvalidResult);
                }
            } else {
                // if query fail we assume 0 to create the database.
                return Ok(0);
            }
        }

        Err(Error::NoSqlDb)
    }

    /// Initialise the database schema.
    fn init_db(&mut self) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            conn.execute(
                "CREATE TABLE admin (key TEXT NOT NULL PRIMARY KEY, value TEXT)",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO admin (key, value) \
                 VALUES ('version', ?1)",
                params![DB_SCHEMA_VERSION],
            )
            .unwrap();
            conn.execute(
                "CREATE TABLE vaults (id INTEGER PRIMARY KEY AUTOINCREMENT, path TEXT)",
                [],
            )
            .unwrap();
            // UNIQUE added in version 12.
            // delete trigger added in version 9
            conn.execute_batch(
                "CREATE TABLE folders (id INTEGER PRIMARY KEY AUTOINCREMENT, \
                 path TEXT, name TEXT, \
                 vault_id INTEGER DEFAULT 0, \
                 locked INTEGER DEFAULT 0, \
                 virtual INTEGER DEFAULT 0, \
                 expanded INTEGER DEFAULT 0, \
                 parent_id INTEGER, UNIQUE(name, parent_id)); \
                 CREATE TRIGGER folder_delete_trigger AFTER DELETE ON folders \
                 BEGIN \
                 DELETE FROM files WHERE parent_id = old.id; \
                 END; \
                 CREATE TRIGGER folders_insert AFTER INSERT ON folders \
                 BEGIN \
                 UPDATE folders SET path = (SELECT f.path FROM folders AS f WHERE f.id = folders.parent_id) || '/' || name WHERE id = new.id AND parent_id != 0; \
                 END; \
                 CREATE TRIGGER folders_update_parent AFTER UPDATE OF parent_id ON folders \
                 BEGIN \
                 UPDATE folders SET path = (SELECT f.path FROM folders AS f WHERE f.id = folders.parent_id) || '/' || name WHERE id = NEW.id AND parent_id != 0; \
                 END;",
            )
            .unwrap();
            //
            let trash_type = libfolder::FolderVirtualType::Trash.to_i32().unwrap_or(0);
            conn.execute(
                "insert into folders (name, locked, virtual, parent_id, path) \
                 values (?1, 1, ?2, 0, '')",
                params!["Trash", trash_type],
            )
            .unwrap();

            // version 10
            conn.execute(
                "CREATE TABLE albums (id INTEGER PRIMARY KEY AUTOINCREMENT, \
                 name TEXT, \
                 parent_id INTEGER)",
                [],
            )
            .unwrap();
            conn.execute(
                "CREATE TABLE albuming (\
                 file_id INTEGER, album_id INTEGER, UNIQUE(file_id, album_id))",
                [],
            )
            .unwrap();
            //

            conn.execute(
                "CREATE TABLE files (id INTEGER PRIMARY KEY AUTOINCREMENT,\
                 main_file INTEGER, name TEXT, parent_id INTEGER, \
                 orientation INTEGER, file_type INTEGER, \
                 file_date INTEGER, rating INTEGER DEFAULT 0, \
                 label INTEGER, flag INTEGER DEFAULT 0, \
                 import_date INTEGER, mod_date INTEGER, \
                 xmp TEXT, xmp_date INTEGER, xmp_file INTEGER DEFAULT 0, \
                 jpeg_file INTEGER DEFAULT 0)",
                [],
            )
            .unwrap();
            conn.execute(
                "CREATE TABLE fsfiles (id INTEGER PRIMARY KEY AUTOINCREMENT, \
                 path TEXT)",
                [],
            )
            .unwrap();
            // version = 7
            conn.execute(
                "CREATE TABLE sidecars (file_id INTEGER,\
                 fsfile_id INTEGER, type INTEGER, ext TEXT NOT NULL, \
                 UNIQUE(file_id, fsfile_id))",
                [],
            )
            .unwrap();
            conn.execute_batch(
                "BEGIN; \
                 CREATE TRIGGER pre_file_delete_trigger BEFORE DELETE ON files \
                 BEGIN \
                 DELETE FROM fsfiles WHERE id = old.main_file \
                 OR id = old.xmp_file OR id = old.jpeg_file; \
                 END; \
                 CREATE TRIGGER file_delete_trigger AFTER DELETE ON files \
                 BEGIN \
                 DELETE FROM sidecars WHERE file_id = old.id; \
                 DELETE FROM keywording WHERE file_id = old.id; \
                 DELETE FROM albuming WHERE file_id = old.id; \
                 END; \
                 CREATE TRIGGER album_delete_trigger AFTER DELETE ON albums \
                 BEGIN \
                 DELETE FROM albuming WHERE album_id = old.id; \
                 END; \
                 COMMIT;",
            )
            .unwrap();
            // UNIQUE added in version 12.
            conn.execute_batch(
                "CREATE TABLE keywords (id INTEGER PRIMARY KEY AUTOINCREMENT, \
                 keyword TEXT, parent_id INTEGER DEFAULT 0, \
                 UNIQUE(keyword, parent_id)); \
                 CREATE TABLE keywording (file_id INTEGER, \
                 keyword_id INTEGER, UNIQUE(file_id, keyword_id)); \
                 CREATE TRIGGER keyword_delete_trigger AFTER DELETE ON keywords \
                 BEGIN \
                 DELETE FROM keywording WHERE keyword_id = old.id; \
                 END;",
            )
            .unwrap();
            conn.execute(
                "CREATE TABLE labels (id INTEGER PRIMARY KEY AUTOINCREMENT, \
                 name TEXT, color TEXT)",
                [],
            )
            .unwrap();
            conn.execute("CREATE TABLE xmp_update_queue (id INTEGER UNIQUE)", [])
                .unwrap();
            conn.execute(
                "CREATE TRIGGER file_update_trigger UPDATE ON files \
                 BEGIN \
                 UPDATE files SET mod_date = strftime('%s','now'); \
                 END",
                [],
            )
            .unwrap();
            conn.execute(
                "CREATE TRIGGER xmp_update_trigger UPDATE OF xmp ON files \
                 BEGIN \
                 INSERT OR IGNORE INTO xmp_update_queue (id) VALUES(new.id);\
                 SELECT rewrite_xmp();\
                 END",
                [],
            )
            .unwrap();

            if self.notify(LibNotification::LibCreated).is_err() {
                err_out!("Error sending LibCreated notification");
            }
            return Ok(());
        }
        Err(Error::NoSqlDb)
    }

    /// Set the DB version
    pub(crate) fn set_db_version(&self, version: i32) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            conn.execute(
                "UPDATE admin SET value=?1 WHERE key='version'",
                params![version],
            )?;
            Ok(())
        } else {
            Err(Error::NoSqlDb)
        }
    }

    ///
    /// Send a `LibNotification`.
    /// @returns the result (nothing or an error)
    ///
    pub(crate) fn notify(
        &self,
        notif: LibNotification,
    ) -> std::result::Result<(), async_channel::SendError<LibNotification>> {
        toolkit::thread_context().block_on(self.sender.send(notif))
    }

    /// Set preference value
    ///
    /// It is a programing error to have an empty `key`.
    pub fn set_pref(&self, key: &str, value: &str) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let c = conn.execute(
                "INSERT OR REPLACE INTO admin (key, value) VALUES (?1, ?2);",
                params![key, value],
            )?;
            if c == 1 {
                return Ok(());
            }
            return Err(Error::InvalidResult);
        }
        Err(Error::NoSqlDb)
    }

    /// Get the preference value. As a conveniance, if it doesn't exist,
    /// then the `default` value is returned, if provided.
    ///
    /// Return `Err(Error::NotFound)` if the key doesn't exist AND
    /// `default` is `None`.
    pub fn get_pref(&self, key: &str, default: Option<&str>) -> Result<String> {
        if let Some(ref conn) = self.dbconn {
            let mut stmt = conn.prepare("SELECT value FROM admin WHERE key = ?1")?;
            let mut rows = stmt.query(params![key])?;
            return if let Some(row) = rows.next()? {
                Ok(row.get::<usize, String>(0)?)
            } else if let Some(default) = default {
                Ok(default.into())
            } else {
                Err(Error::NotFound)
            };
        }
        Err(Error::NoSqlDb)
    }

    pub fn get_all_preferences(&self) -> Result<Vec<(String, String)>> {
        if let Some(ref conn) = self.dbconn {
            let mut stmt = conn.prepare("SELECT key, value FROM admin")?;
            let mut rows = stmt.query(params![])?;
            let mut prefs = vec![];
            while let Ok(Some(row)) = rows.next() {
                prefs.push((row.get::<usize, String>(0)?, row.get::<usize, String>(1)?))
            }
            return Ok(prefs);
        }
        Err(Error::NoSqlDb)
    }

    fn add_jpeg_file_to_bundle(&self, file_id: LibraryId, fsfile_id: LibraryId) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let filetype: i32 = libfile::FileType::RawJpeg.into();
            let c = conn.execute(
                "UPDATE files SET jpeg_file=?1, file_type=?3 WHERE id=?2;",
                params![fsfile_id, file_id, filetype],
            )?;
            if c == 1 {
                return Ok(());
            }
            return Err(Error::InvalidResult);
        }
        Err(Error::NoSqlDb)
    }

    fn add_xmp_sidecar_to_bundle(&self, file_id: LibraryId, fsfile_id: LibraryId) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let c = conn.execute(
                "UPDATE files SET xmp_file=?1 WHERE id=?2;",
                params![fsfile_id, file_id],
            )?;
            if c == 1 {
                return Ok(());
            }
            return Err(Error::InvalidResult);
        }
        Err(Error::NoSqlDb)
    }

    fn add_sidecar_file_to_bundle(&self, file_id: LibraryId, sidecar: &Sidecar) -> Result<()> {
        let sidecar_t: (i32, &PathBuf) = match *sidecar {
            Sidecar::Live(ref p)
            | Sidecar::Thumbnail(ref p)
            | Sidecar::Xmp(ref p)
            | Sidecar::Jpeg(ref p) => (sidecar.to_int(), p),
            _ => return Err(Error::InvalidArg),
        };
        let p = Path::new(sidecar_t.1);
        let ext = match p.extension() {
            Some(ext2) => ext2.to_string_lossy(),
            _ => return Err(Error::InvalidArg),
        };
        let fsfile_id = self.add_fs_file(sidecar_t.1)?;
        self.add_sidecar_fsfile_to_bundle(file_id, fsfile_id, sidecar_t.0, &ext)
    }

    fn add_sidecar_fsfile_to_bundle(
        &self,
        file_id: LibraryId,
        fsfile_id: LibraryId,
        sidecar_type: i32,
        ext: &str,
    ) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let c = conn.execute(
                "INSERT INTO sidecars (file_id, fsfile_id, type, ext) VALUES(?1, ?2, ?3, ?4)",
                params![file_id, fsfile_id, sidecar_type, ext],
            )?;
            if c == 1 {
                return Ok(());
            }
            return Err(Error::InvalidResult);
        }
        Err(Error::NoSqlDb)
    }

    fn get_content(&self, id: LibraryId, sql_where: &str) -> Result<Vec<LibFile>> {
        if let Some(ref conn) = self.dbconn {
            let sql = format!(
                "SELECT {} FROM {} \
                 WHERE {} \
                 AND files.main_file=fsfiles.id",
                LibFile::read_db_columns(),
                LibFile::read_db_tables(),
                sql_where
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query(params![id])?;
            let mut files: Vec<LibFile> = vec![];
            while let Ok(Some(row)) = rows.next() {
                files.push(LibFile::read_from(row)?);
            }
            return Ok(files);
        }
        Err(Error::NoSqlDb)
    }

    /// Add folder with name and optional path into parent whose id is
    /// `into`.  A value of 0 means root.
    ///
    /// Returns a `Result<LibFolder>`.
    pub(crate) fn add_folder_into(
        &self,
        name: &str,
        path: Option<String>,
        into: LibraryId,
    ) -> Result<LibFolder> {
        if let Some(ref conn) = self.dbconn {
            let c = conn.execute(
                "INSERT INTO folders (path,name,vault_id,parent_id) VALUES(?1, ?2, '0', ?3) ON CONFLICT DO NOTHING",
                params![path, name, into],
            )?;
            let id = if c != 1 {
                if c != 0 {
                    return Err(Error::InvalidResult);
                }
                let mut stmt =
                    conn.prepare("SELECT id FROM folders WHERE name = ?1 AND parent_id = ?2")?;
                let mut rows = stmt.query(params![name, into])?;
                if let Some(row) = rows.next()? {
                    row.get(0)?
                } else {
                    return Err(Error::NotFound);
                }
            } else {
                conn.last_insert_rowid()
            };
            dbg_out!("last row inserted {}", id);
            let mut lf = LibFolder::new(id, name, path);
            lf.set_parent(into);
            return Ok(lf);
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn delete_folder(&self, id: LibraryId) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let c = conn.execute("DELETE FROM folders WHERE id=?1", params![id])?;
            if c == 1 {
                return Ok(());
            }
            return Err(Error::InvalidResult);
        }
        Err(Error::NoSqlDb)
    }

    /// Get the folder from its path
    ///
    /// Return the LibFolder or None
    pub(crate) fn get_folder(&self, path: &str) -> Result<LibFolder> {
        if let Some(ref conn) = self.dbconn {
            let sql = format!(
                "SELECT {} FROM {} WHERE path=?1",
                LibFolder::read_db_columns(),
                LibFolder::read_db_tables()
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query(params![path])?;
            return match rows.next() {
                Ok(None) => Err(Error::NotFound),
                Err(err) => {
                    err_out!("Error {:?}", err);
                    Err(Error::from(err))
                }
                Ok(Some(row)) => Ok(LibFolder::read_from(row)?),
            };
        }
        Err(Error::NoSqlDb)
    }

    /// Reparent a `folder` to the `new_parent`.
    fn reparent_folder(&self, id: LibraryId, new_parent: LibraryId) -> Result<()> {
        let conn = self.dbconn.as_ref().ok_or(Error::NoSqlDb)?;
        let c = conn.execute(
            "UPDATE folders SET parent_id=?2 WHERE id=?1",
            params![id, new_parent],
        )?;
        if c == 1 {
            return Ok(());
        }
        Err(Error::InvalidResult)
    }

    /// Get the subfolders for id. Only the direct descendents.
    ///
    /// Return a `Ok(Vec<LibFolder>)` (may be empty) or `Err(_)`.
    pub(crate) fn get_subfolders(&self, parent: LibraryId) -> Result<Vec<LibFolder>> {
        if let Some(ref conn) = self.dbconn {
            let sql = format!(
                "SELECT {} FROM {} WHERE parent_id=?1",
                LibFolder::read_db_columns(),
                LibFolder::read_db_tables()
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query(params![parent])?;
            let mut subfolders = vec![];
            loop {
                let row = rows.next();
                match row {
                    Ok(None) => break,
                    Err(err) => {
                        err_out!("Error {:?}", err);
                        return Err(Error::from(err));
                    }
                    Ok(Some(row)) => subfolders.push(LibFolder::read_from(row)?),
                }
            }
            return Ok(subfolders);
        }
        Err(Error::NoSqlDb)
    }

    /// Get the root folders, ie those whose parent_id is 0.
    pub(crate) fn get_root_folders(&self) -> Result<Vec<LibFolder>> {
        let conn = self.dbconn.as_ref().ok_or(Error::NoSqlDb)?;
        let sql = format!(
            "SELECT {} FROM {} WHERE parent_id = 0 AND virtual = false",
            LibFolder::read_db_columns(),
            LibFolder::read_db_tables()
        );
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query([])?;
        let mut folders: Vec<LibFolder> = vec![];
        while let Ok(Some(row)) = rows.next() {
            folders.push(LibFolder::read_from(row)?);
        }

        Ok(folders)
    }

    pub(crate) fn get_all_folders(&self) -> Result<Vec<LibFolder>> {
        if let Some(ref conn) = self.dbconn {
            let sql = format!(
                "SELECT {} FROM {} ORDER BY parent_id",
                LibFolder::read_db_columns(),
                LibFolder::read_db_tables()
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query([])?;
            let mut folders: Vec<LibFolder> = vec![];
            while let Ok(Some(row)) = rows.next() {
                folders.push(LibFolder::read_from(row)?);
            }
            return Ok(folders);
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn get_folder_content(&self, folder_id: LibraryId) -> Result<Vec<LibFile>> {
        self.get_content(folder_id, "parent_id = ?1")
    }

    pub(crate) fn count_folder(&self, folder_id: LibraryId) -> Result<i64> {
        if let Some(ref conn) = self.dbconn {
            let mut stmt = conn.prepare(
                "SELECT COUNT(id) FROM files \
                 WHERE parent_id=?1;",
            )?;
            let mut rows = stmt.query(params![folder_id])?;
            return match rows.next() {
                Ok(Some(row)) => Ok(row.get(0)?),
                Err(err) => Err(Error::from(err)),
                Ok(None) => Err(Error::NotFound),
            };
        }
        Err(Error::NoSqlDb)
    }

    /// Reparent all the root folders.
    pub(crate) fn reparent_roots_for(&self, id: LibraryId, folder_path: &str) -> Result<()> {
        let conn = self.dbconn.as_ref().ok_or(Error::NoSqlDb)?;
        let sql = format!(
            "SELECT {} FROM {} WHERE INSTR(path, ?1) = 1 AND id != {id} AND parent_id = 0 ORDER BY path;",
            LibFolder::read_db_columns(),
            LibFolder::read_db_tables(),
        );
        let mut stmt = conn.prepare(&sql)?;
        let base = std::path::Path::new(folder_path);
        let rows = stmt.query_map(params![folder_path], LibFolder::read_from)?;
        for folder in rows {
            let folder = folder.unwrap();
            let folder_path = std::path::Path::new(folder.path().ok_or(Error::InvalidArg)?);
            let trail_path = folder_path.strip_prefix(base)?;
            let mut current = base.to_path_buf();
            let mut current_id = id;
            for p in trail_path.components() {
                current = current.join(p);
                match self.get_folder(&current.to_string_lossy()) {
                    Err(Error::NotFound) => {
                        let name: &std::path::Path = p.as_ref();
                        let name = name.to_string_lossy();
                        let lf = self.add_folder_into(
                            &name,
                            Some(current.to_string_lossy().to_string()),
                            current_id,
                        )?;
                        // XXX make sure to notify
                        current_id = lf.id();
                        on_err_out!(self.notify(LibNotification::AddedFolder(lf)));
                    }
                    Ok(f) => {
                        if f.parent() != current_id {
                            self.reparent_folder(f.id(), current_id)?;
                            on_err_out!(self.notify(LibNotification::FolderReparented(
                                FolderReparent {
                                    id: f.id(),
                                    dest: current_id
                                }
                            )));
                        }
                        current_id = f.id()
                    }
                    _ => break,
                }
            }
        }

        Ok(())
    }

    /// Find the root folder for `folder`
    pub(crate) fn root_folder_for(&self, folder: &str) -> Result<LibFolder> {
        let conn = self.dbconn.as_ref().ok_or(Error::NoSqlDb)?;
        let folder = std::path::PathBuf::from(folder);
        let sql = format!(
            "SELECT {} FROM {} WHERE parent_id = 0 AND path = ?1;",
            LibFolder::read_db_columns(),
            LibFolder::read_db_tables()
        );
        let mut stmt = conn.prepare(&sql)?;
        for p in folder.ancestors() {
            if p.as_os_str().is_empty() {
                break;
            }
            let mut rows = stmt.query(params![&p.to_string_lossy()])?;
            if let Some(row) = rows.next()? {
                return LibFolder::read_from(row).map_err(|err| err.into());
            }
        }
        Err(Error::NotFound)
    }

    pub(crate) fn get_all_keywords(&self) -> Result<Vec<Keyword>> {
        if let Some(ref conn) = self.dbconn {
            let sql = format!(
                "SELECT {} FROM {}",
                Keyword::read_db_columns(),
                Keyword::read_db_tables()
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query([])?;
            let mut keywords: Vec<Keyword> = vec![];
            while let Ok(Some(row)) = rows.next() {
                keywords.push(Keyword::read_from(row)?);
            }
            return Ok(keywords);
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn count_keyword(&self, id: LibraryId) -> Result<i64> {
        if let Some(ref conn) = self.dbconn {
            let mut stmt = conn.prepare(
                "SELECT COUNT(keyword_id) FROM keywording \
                 WHERE keyword_id=?1;",
            )?;
            let mut rows = stmt.query(params![id])?;
            return match rows.next() {
                Ok(Some(row)) => Ok(row.get(0)?),
                Err(err) => Err(Error::from(err)),
                Ok(None) => Err(Error::NotFound),
            };
        }
        Err(Error::NoSqlDb)
    }

    /// Add an album to the library
    pub(crate) fn add_album(&self, name: &str, parent: LibraryId) -> Result<Album> {
        if let Some(ref conn) = self.dbconn {
            let c = conn.execute(
                "INSERT INTO albums (name,parent_id) VALUES(?1, ?2)",
                params![name, parent],
            )?;
            if c != 1 {
                return Err(Error::InvalidResult);
            }
            let id = conn.last_insert_rowid();
            return Ok(Album::new(id, name, parent));
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn delete_album(&self, id: LibraryId) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let c = conn.execute("DELETE FROM albums WHERE id=?1", params![id])?;
            if c == 1 {
                return Ok(());
            }
            return Err(Error::InvalidResult);
        }
        Err(Error::NoSqlDb)
    }

    /// Rename album `id` to `name`.
    pub(crate) fn rename_album(&self, id: LibraryId, name: &str) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let c = conn.execute("UPDATE albums SET name=?2 WHERE id=?1", params![id, name])?;
            if c == 1 {
                return Ok(());
            }
            return Err(Error::InvalidResult);
        }
        Err(Error::NoSqlDb)
    }

    /// Get all the albums.
    pub(crate) fn get_all_albums(&self) -> Result<Vec<Album>> {
        if let Some(ref conn) = self.dbconn {
            let sql = format!(
                "SELECT {} FROM {}",
                Album::read_db_columns(),
                Album::read_db_tables()
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query([])?;
            let mut albums: Vec<Album> = vec![];
            while let Ok(Some(row)) = rows.next() {
                albums.push(Album::read_from(row)?);
            }
            return Ok(albums);
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn count_album(&self, id: LibraryId) -> Result<i64> {
        if let Some(ref conn) = self.dbconn {
            let mut stmt = conn.prepare(
                "SELECT COUNT(album_id) FROM albuming \
                 WHERE album_id=?1;",
            )?;
            let mut rows = stmt.query(params![id])?;
            return match rows.next() {
                Ok(Some(row)) => Ok(row.get(0)?),
                Err(err) => Err(Error::from(err)),
                Ok(None) => Err(Error::NotFound),
            };
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn get_album_content(&self, album_id: LibraryId) -> Result<Vec<LibFile>> {
        self.get_content(
            album_id,
            "files.id IN \
             (SELECT file_id FROM albuming \
             WHERE album_id=?1) ",
        )
    }

    /// Add `images` to an `album`.
    pub(crate) fn add_to_album(&self, images: &[LibraryId], album: LibraryId) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let mut stmt =
                conn.prepare("INSERT INTO albuming (file_id, album_id) VALUES(?1, ?2)")?;
            for image in images {
                let c = stmt.execute(params![image, album])?;
                if c != 1 {
                    return Err(Error::InvalidResult);
                }
            }
            return Ok(());
        }
        Err(Error::NoSqlDb)
    }

    /// Remove images from an album.
    pub(crate) fn remove_from_album(&self, images: &[LibraryId], album: LibraryId) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let mut stmt =
                conn.prepare("DELETE FROM albuming WHERE (file_id=?1 AND album_id=?2)")?;
            for image in images {
                let c = stmt.execute(params![image, album])?;
                if c != 1 {
                    return Err(Error::InvalidResult);
                }
            }
            return Ok(());
        }
        Err(Error::NoSqlDb)
    }

    fn add_fs_file<P: AsRef<Path>>(&self, f: P) -> Result<LibraryId> {
        if let Some(ref conn) = self.dbconn {
            let file = f.as_ref().to_string_lossy();
            let c = conn.execute("INSERT INTO fsfiles (path) VALUES(?1)", params![file])?;
            if c != 1 {
                return Err(Error::InvalidResult);
            }
            return Ok(conn.last_insert_rowid());
        }

        Err(Error::NoSqlDb)
    }

    fn get_fs_file(&self, id: LibraryId) -> Result<String> {
        if let Some(ref conn) = self.dbconn {
            let mut stmt = conn.prepare("SELECT path FROM fsfiles WHERE id=?1")?;
            let mut rows = stmt.query(params![id])?;
            return match rows.next() {
                Ok(Some(row)) => Ok(row.get(0)?),
                Err(err) => Err(Error::from(err)),
                Ok(None) => Err(Error::NotFound),
            };
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn add_bundle(
        &self,
        folder_id: LibraryId,
        bundle: &FileBundle,
    ) -> Result<LibraryId> {
        let file_id = self.add_file(folder_id, bundle.main(), Some(bundle))?;
        if file_id <= 0 {
            err_out!("add_file returned {}", file_id);
            return Err(Error::InvalidResult);
        }
        if !bundle.xmp_sidecar().as_os_str().is_empty() {
            let fsfile_id = self.add_fs_file(bundle.xmp_sidecar())?;
            if fsfile_id > 0 {
                self.add_xmp_sidecar_to_bundle(file_id, fsfile_id)?;
                self.add_sidecar_fsfile_to_bundle(
                    file_id,
                    fsfile_id,
                    Sidecar::Xmp(PathBuf::new()).to_int(),
                    "xmp",
                )?;
            }
        }
        if !bundle.jpeg().as_os_str().is_empty() {
            let fsfile_id = self.add_fs_file(bundle.jpeg())?;
            if fsfile_id > 0 {
                self.add_jpeg_file_to_bundle(file_id, fsfile_id)?;
                self.add_sidecar_fsfile_to_bundle(
                    file_id,
                    fsfile_id,
                    Sidecar::Jpeg(PathBuf::new()).to_int(),
                    "jpg",
                )?;
            }
        }

        Ok(file_id)
    }

    fn add_file<P: AsRef<Path> + AsRef<OsStr>>(
        &self,
        folder_id: LibraryId,
        file: P,
        bundle: Option<&FileBundle>,
    ) -> Result<LibraryId> {
        dbg_assert!(folder_id != -1, "invalid folder ID");
        let file_path: &Path = file.as_ref();
        let mime = npc_fwk::MimeType::new(file_path);
        let file_type = libfile::mimetype_to_filetype(&mime);
        let label_id: LibraryId = 0;
        let orientation: i32;
        let rating: i32;
        //let label: String; // XXX fixme
        let flag: i32;
        let creation_date: npc_fwk::Time;
        let xmp: String;

        // Until we get better metadata support for RAW files, we use the Exif reconcile
        // from the sidecar JPEG to get the initial metadata.
        let meta = if let Some(bundle) = bundle {
            if bundle.bundle_type() == libfile::FileType::RawJpeg {
                npc_fwk::XmpMeta::new_from_file(bundle.jpeg(), false)
            } else {
                npc_fwk::XmpMeta::new_from_file(file_path, false)
            }
        } else {
            npc_fwk::XmpMeta::new_from_file(file_path, false)
        };

        if let Some(ref meta) = meta {
            orientation = meta.orientation().unwrap_or(0);
            rating = meta.rating().unwrap_or(0);
            //label = meta.label().unwrap_or(String::from(""));
            flag = meta.flag().unwrap_or(0);
            if let Some(ref date) = meta.creation_date() {
                creation_date = date.timestamp();
            } else {
                creation_date = 0
            }
            xmp = meta.serialize_inline();
        } else {
            orientation = 0;
            rating = 0;
            //label = String::from("");
            flag = 0;
            creation_date = 0;
            xmp = String::from("");
        }

        let filename = file_path
            .file_name()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default();
        let fs_file_id = self.add_fs_file(file_path)?;
        if fs_file_id <= 0 {
            err_out!("add fsfile failed");
            return Err(Error::InvalidResult);
        }

        if let Some(ref conn) = self.dbconn {
            let ifile_type = i32::from(file_type);
            let time = Utc::now().timestamp();
            let c = conn.execute(
                "INSERT INTO files (\
                 main_file, name, parent_id, \
                 import_date, mod_date, \
                 orientation, file_date, rating, label, \
                 file_type, flag, xmp) \
                 VALUES (\
                 ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    fs_file_id,
                    filename,
                    folder_id,
                    time,
                    time,
                    orientation,
                    creation_date,
                    rating,
                    label_id,
                    ifile_type,
                    flag,
                    xmp,
                ],
            )?;

            if c == 1 {
                let id = conn.last_insert_rowid();
                if let Some(mut meta) = meta {
                    let keywords = meta.keywords();
                    for k in keywords {
                        let kwid = self.make_keyword(k)?;
                        if kwid != -1 {
                            self.assign_keyword(kwid, id)?;
                        }
                    }
                }
                return Ok(id);
            }
            return Err(Error::InvalidResult);
        }

        Err(Error::NoSqlDb)
    }

    pub(crate) fn make_keyword(&self, keyword: &str) -> Result<LibraryId> {
        if let Some(ref conn) = self.dbconn {
            let mut stmt = conn.prepare(
                "SELECT id FROM keywords WHERE \
                 keyword=?1;",
            )?;
            let mut rows = stmt.query(params![keyword])?;
            if let Ok(Some(row)) = rows.next() {
                let keyword_id = row.get(0)?;
                if keyword_id > 0 {
                    return Ok(keyword_id);
                }
            }

            let c = conn.execute(
                "INSERT INTO keywords (keyword, parent_id) VALUES(?1, 0);",
                params![keyword],
            )?;
            if c != 1 {
                return Err(Error::InvalidResult);
            }
            let keyword_id = conn.last_insert_rowid();
            if self
                .notify(LibNotification::AddedKeyword(Keyword::new(
                    keyword_id, keyword,
                )))
                .is_err()
            {
                err_out!("Failed to send AddedKeyword notification");
            }
            return Ok(keyword_id);
        }
        Err(Error::NoSqlDb)
    }

    fn assign_keyword(&self, kw_id: LibraryId, file_id: LibraryId) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            conn.execute(
                "INSERT OR IGNORE INTO keywording\
                 (file_id, keyword_id) \
                 VALUES(?1, ?2)",
                params![kw_id, file_id],
            )?;
            Ok(())
        } else {
            Err(Error::NoSqlDb)
        }
    }

    pub(crate) fn get_keyword_content(&self, keyword_id: LibraryId) -> Result<Vec<LibFile>> {
        self.get_content(
            keyword_id,
            "files.id IN \
             (SELECT file_id FROM keywording \
             WHERE keyword_id=?1) ",
        )
    }

    pub(crate) fn get_metadata(&self, file_id: LibraryId) -> Result<Box<LibMetadata>> {
        if let Some(ref conn) = self.dbconn {
            let sql = format!(
                "SELECT {} FROM {} WHERE {}=?1",
                LibMetadata::read_db_columns(),
                LibMetadata::read_db_tables(),
                LibMetadata::read_db_where_id()
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query(params![file_id])?;
            return match rows.next() {
                Err(err) => Err(Error::from(err)),
                Ok(None) => Err(Error::NotFound),
                Ok(Some(row)) => {
                    let mut metadata = Box::new(LibMetadata::read_from(row)?);

                    let sql = "SELECT ext FROM sidecars WHERE file_id=?1";
                    let mut stmt = conn.prepare(sql)?;
                    let mut rows = stmt.query(params![file_id])?;
                    while let Ok(Some(row)) = rows.next() {
                        metadata.sidecars.push(row.get(0)?);
                    }
                    Ok(metadata)
                }
            };
        }
        Err(Error::NoSqlDb)
    }

    fn unassign_all_keywords_for_file(&self, file_id: LibraryId) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            conn.execute(
                "DELETE FROM keywording \
                 WHERE file_id=?1;",
                params![file_id],
            )?;
            // we don't really know how many rows are supposed to be impacted
            // even 0 is valid.
            return Ok(());
        }
        Err(Error::NoSqlDb)
    }

    /// Set properties for an image.
    ///
    /// XXX only the XMP Packet is currently supported.
    pub fn set_image_properties(
        &self,
        image_id: LibraryId,
        props: &NiepcePropertyBag,
    ) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            if let Some(PropertyValue::String(xmp)) = props.get(&Np::Index(Npi::NpNiepceXmpPacket))
            {
                let mut stmt = conn.prepare("UPDATE files SET xmp=?1 WHERE id=?2;")?;
                stmt.execute(params![xmp, image_id])?;
            }
            return Ok(());
        }
        Err(Error::NoSqlDb)
    }

    fn set_internal_metadata(&self, file_id: LibraryId, column: &str, value: i32) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let c = conn.execute(
                format!("UPDATE files SET {column}=?1 WHERE id=?2;").as_ref(),
                params![value, file_id],
            )?;
            if c != 1 {
                err_out!("error setting internal metadata");
                return Err(Error::InvalidResult);
            }
            return Ok(());
        }
        Err(Error::NoSqlDb)
    }

    fn set_metadata_block(&self, file_id: LibraryId, metablock: &LibMetadata) -> Result<()> {
        let xmp = metablock.serialize_inline();
        if let Some(ref conn) = self.dbconn {
            let c = conn.execute(
                "UPDATE files SET xmp=?1 WHERE id=?2;",
                params![xmp, file_id],
            )?;
            if c != 1 {
                err_out!("error setting metadatablock");
                return Err(Error::InvalidResult);
            }
            return Ok(());
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn set_metadata(
        &self,
        file_id: LibraryId,
        meta: Np,
        value: &PropertyValue,
    ) -> Result<()> {
        #[allow(non_upper_case_globals)]
        match meta {
            Np::Index(Npi::NpXmpRatingProp)
            | Np::Index(Npi::NpXmpLabelProp)
            | Np::Index(Npi::NpTiffOrientationProp)
            | Np::Index(Npi::NpNiepceFlagProp) => {
                match *value {
                    PropertyValue::Int(i) => {
                        // internal
                        // make the column mapping more generic.
                        let column = match meta {
                            Np::Index(Npi::NpXmpRatingProp) => "rating",
                            Np::Index(Npi::NpXmpLabelProp) => "label",
                            Np::Index(Npi::NpTiffOrientationProp) => "orientation",
                            Np::Index(Npi::NpNiepceFlagProp) => "flag",
                            _ => unreachable!(),
                        };
                        if !column.is_empty() {
                            self.set_internal_metadata(file_id, column, i)?;
                        }
                    }
                    _ => err_out!("improper value type for {:?}", meta),
                }
            }
            Np::Index(Npi::NpIptcKeywordsProp) => {
                self.unassign_all_keywords_for_file(file_id)?;

                match *value {
                    PropertyValue::StringArray(ref keywords) => {
                        for kw in keywords {
                            let id = self.make_keyword(kw)?;
                            if id != -1 {
                                self.assign_keyword(id, file_id)?;
                            }
                        }
                    }
                    _ => err_out!("improper value_type for {:?} : {:?}", meta, value),
                }
            }
            _ =>
            // XXX TODO
            {
                err_out!("unhandled meta {:?}", meta)
            }
        }
        let mut metablock = self.get_metadata(file_id)?;
        metablock.set_metadata(meta, value);
        metablock.touch();
        self.set_metadata_block(file_id, &metablock)?;

        Ok(())
    }

    pub(crate) fn move_file_to_folder(
        &self,
        file_id: LibraryId,
        folder_id: LibraryId,
    ) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let mut stmt = conn.prepare("SELECT id FROM folders WHERE id=?1;")?;
            let mut rows = stmt.query(params![folder_id])?;
            if let Ok(Some(_)) = rows.next() {
                conn.execute(
                    "UPDATE files SET parent_id = ?1 WHERE id = ?2;",
                    params![folder_id, file_id],
                )?;
                return Ok(());
            } else {
                return Err(Error::NotFound);
            }
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn get_all_labels(&self) -> Result<Vec<Label>> {
        if let Some(ref conn) = self.dbconn {
            let sql = format!(
                "SELECT {} FROM {} ORDER BY id;",
                Label::read_db_columns(),
                Label::read_db_tables()
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query([])?;
            let mut labels: Vec<Label> = vec![];
            while let Ok(Some(row)) = rows.next() {
                labels.push(Label::read_from(row)?);
            }
            return Ok(labels);
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn add_label(&self, name: &str, colour: &RgbColour) -> Result<LibraryId> {
        if let Some(ref conn) = self.dbconn {
            let colour = colour.to_string();
            let c = conn.execute(
                "INSERT INTO  labels (name,color) VALUES (?1, ?2);",
                params![name, &colour],
            )?;
            if c != 1 {
                return Err(Error::InvalidResult);
            }
            let label_id = conn.last_insert_rowid();
            dbg_out!("last row inserted {}", label_id);
            return Ok(label_id);
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn update_label(
        &self,
        label_id: LibraryId,
        name: &str,
        colour: &RgbColour,
    ) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let colour = colour.to_string();
            let c = conn.execute(
                "UPDATE labels SET name=?2, color=?3 WHERE id=?1;",
                params![label_id, name, &colour],
            )?;
            if c != 1 {
                return Err(Error::InvalidResult);
            }
            return Ok(());
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn delete_label(&self, label_id: LibraryId) -> Result<()> {
        if let Some(ref conn) = self.dbconn {
            let c = conn.execute("DELETE FROM labels WHERE id=?1;", [&label_id])?;
            if c != 1 {
                return Err(Error::InvalidResult);
            }
            return Ok(());
        }
        Err(Error::NoSqlDb)
    }

    fn get_xmp_ids_in_queue(&self) -> Result<Vec<LibraryId>> {
        if let Some(ref conn) = self.dbconn {
            let mut stmt = conn.prepare("SELECT id FROM xmp_update_queue;")?;
            let mut rows = stmt.query([])?;
            let mut ids = Vec::<LibraryId>::new();
            while let Ok(Some(row)) = rows.next() {
                let id: i64 = row.get(0)?;
                ids.push(id);
            }
            return Ok(ids);
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn write_metadata(&self, id: LibraryId) -> Result<()> {
        self.rewrite_xmp_for_id(id, true)
    }

    fn rewrite_xmp_for_id(&self, id: LibraryId, write_xmp: bool) -> Result<()> {
        // XXX
        // Rework this so that:
        // 1. it returns a Err<>
        // 2. it only delete if the xmp file has been updated properly
        // 3. make sure the update happened correctly, possibly ensure we don't
        // clobber the xmp.
        if let Some(ref conn) = self.dbconn {
            if conn
                .execute("DELETE FROM xmp_update_queue WHERE id=?1;", [&id])
                .is_ok()
            {
                // we don't want to write the XMP so we don't need to list them.
                if !write_xmp {
                    return Ok(());
                }
                if let Ok(mut stmt) = conn.prepare(
                    "SELECT xmp, main_file, xmp_file FROM files \
                     WHERE id=?1;",
                ) {
                    let mut rows = stmt.query([&id])?;
                    while let Ok(Some(row)) = rows.next() {
                        let xmp_buffer: String = row.get(0)?;
                        let main_file_id: LibraryId = row.get(1)?;
                        // In case of error we assume 0.
                        let xmp_file_id: LibraryId = row.get(2).unwrap_or(0);
                        let p = self.get_fs_file(main_file_id);
                        let spath = if let Ok(ref p) = p {
                            PathBuf::from(p)
                        } else {
                            // XXX we should report that error.
                            err_out!("couldn't find the main file {:?}", p);
                            dbg_assert!(false, "couldn't find the main file");
                            continue;
                        };
                        let mut p: Option<PathBuf> = None;
                        if xmp_file_id > 0 {
                            if let Ok(p2) = self.get_fs_file(xmp_file_id) {
                                p = Some(PathBuf::from(p2));
                            }
                            dbg_assert!(p.is_some(), "couldn't find the xmp file path");
                        }
                        if p.is_none() {
                            p = Some(spath.with_extension("xmp"));
                            dbg_assert!(
                                *p.as_ref().unwrap() != spath,
                                "path must have been changed"
                            );
                        }
                        let p = p.unwrap();
                        if p.exists() {
                            dbg_out!("{:?} already exist", p);
                        }
                        let mut xmppacket = npc_fwk::XmpMeta::new();
                        xmppacket.unserialize(&xmp_buffer);
                        if let Ok(mut f) = File::create(p.clone()) {
                            let sidecar = xmppacket.serialize();
                            if f.write(sidecar.as_bytes()).is_ok() && (xmp_file_id <= 0) {
                                let xmp_file_id = self.add_fs_file(&p)?;
                                dbg_assert!(xmp_file_id > 0, "couldn't add xmp_file");
                                // XXX handle error
                                let res = self.add_xmp_sidecar_to_bundle(id, xmp_file_id);
                                dbg_assert!(res.is_ok(), "add_xmp_sidecar_to_bundle failed");
                                let res = self.add_sidecar_fsfile_to_bundle(
                                    id,
                                    xmp_file_id,
                                    Sidecar::Xmp(PathBuf::new()).to_int(),
                                    "xmp",
                                );
                                dbg_assert!(res.is_ok(), "add_sidecar_fsfile_to_bundle failed");
                            }
                        }
                    }
                    return Ok(());
                }
            }
        }
        Err(Error::NoSqlDb)
    }

    pub(crate) fn process_xmp_update_queue(&self, write_xmp: bool) -> Result<()> {
        let ids = self.get_xmp_ids_in_queue()?;
        for id in ids {
            self.rewrite_xmp_for_id(id, write_xmp)?;
        }

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::NiepcePropertyBag;
    use crate::catalog::NiepceProperties as Np;
    use crate::catalog::NiepcePropertyIdx as Npi;
    use crate::catalog::filebundle::FileBundle;
    use crate::library::notification::LibNotification;

    use super::{CatalogDb, DB_SCHEMA_VERSION, Error};

    /// Create a test library. Call this to create an in memory
    /// library.  The optional `path` is to open the catalog on disk
    /// for debugging purpose. This require deleting the file for the
    /// subsequent runs.
    pub(crate) fn test_catalog(path: Option<&std::path::Path>) -> CatalogDb {
        let (sender, receiver) = async_channel::unbounded();
        let catalog = if let Some(path) = path {
            CatalogDb::new(path, sender)
        } else {
            CatalogDb::new_in_memory(sender)
        };
        assert_eq!(catalog.dbfile, path.map(std::path::Path::to_path_buf));
        let msg = receiver
            .try_recv()
            .expect("Didn't receive LibCreated message");
        match msg {
            LibNotification::LibCreated => {}
            _ => panic!("Wrong message type, expected LibCreated"),
        }

        let msg = receiver
            .try_recv()
            .expect("Didn't receive DatabaseReady message");
        match msg {
            LibNotification::DatabaseReady => {}
            _ => panic!("Wrong message type, expected DatabaseReady"),
        }

        assert!(catalog.is_ok());
        catalog
    }

    #[test]
    fn catalog_works() {
        let catalog = test_catalog(None);
        let version = catalog.check_database_version();
        assert!(version.is_ok());
        assert!(version.ok().unwrap() == super::DB_SCHEMA_VERSION);

        assert!(std::mem::size_of::<LibNotification>() <= 80);

        println!(
            "sizeof libnotification {} thumbnail {} imagerendered {} metadatachange {} label {} album {} libfolder {} keyword {} label {} libfile {} libmetadata {}",
            std::mem::size_of::<LibNotification>(),
            std::mem::size_of::<crate::library::notification::Thumbnail>(),
            std::mem::size_of::<crate::library::notification::ImageRendered>(),
            std::mem::size_of::<crate::library::notification::MetadataChange>(),
            std::mem::size_of::<crate::catalog::Label>(),
            std::mem::size_of::<crate::catalog::Album>(),
            std::mem::size_of::<crate::catalog::LibFolder>(),
            std::mem::size_of::<crate::catalog::Keyword>(),
            std::mem::size_of::<crate::catalog::Label>(),
            std::mem::size_of::<crate::catalog::LibFile>(),
            std::mem::size_of::<crate::catalog::LibMetadata>(),
        );

        // Backup should return an error.
        assert!(matches!(
            catalog.backup_database_file("backup"),
            Err(Error::NoDbFile)
        ));

        let folder_added = catalog.add_folder_into("foo", Some("/bar/foo".to_string()), 0);
        assert!(folder_added.is_ok());
        let folder_added = folder_added.unwrap();
        let parent_id = folder_added.id();
        assert!(parent_id > 0);

        let f = catalog.get_folder("/bar/foo");
        assert!(f.is_ok());
        let f = f.unwrap();
        assert_eq!(folder_added.id(), f.id());

        let id = f.id();
        let f = catalog.add_folder_into("bar", Some(String::from("/bar/bar")), id);
        assert!(f.is_ok());
        // The triggers have changed the path to match the parent's.
        let f = catalog.get_folder("/bar/foo/bar");
        assert!(f.is_ok());
        let f = f.unwrap();
        assert_eq!(f.parent(), id);

        let folders = catalog.get_all_folders();
        assert!(folders.is_ok());
        let folders = folders.unwrap();
        assert_eq!(folders.len(), 3);
        println!("folder {folders:?}");

        let root_folders = catalog.get_root_folders();
        assert!(root_folders.is_ok());
        let root_folders = root_folders.unwrap();
        println!("root folder {root_folders:?}");
        assert_eq!(root_folders.len(), 1);
        assert_eq!(root_folders[0].path(), Some("/bar/foo"));

        let file_id = catalog.add_file(folder_added.id(), "foo/myfile", None);
        assert!(file_id.is_ok());
        let file_id = file_id.unwrap();
        assert!(file_id > 0);

        assert!(catalog.move_file_to_folder(file_id, 100).is_err());
        assert!(
            catalog
                .move_file_to_folder(file_id, folder_added.id())
                .is_ok()
        );
        let count = catalog.count_folder(folder_added.id());
        assert!(count.is_ok());
        let count = count.ok().unwrap();
        assert_eq!(count, 1);

        let fl = catalog.get_folder_content(folder_added.id());
        assert!(fl.is_ok());
        let fl = fl.ok().unwrap();
        assert_eq!(fl.len(), count as usize);
        assert_eq!(fl[0].id(), file_id);

        let kwid1 = catalog.make_keyword("foo");
        assert!(kwid1.is_ok());
        let kwid1 = kwid1.ok().unwrap();
        assert!(kwid1 > 0);
        let kwid2 = catalog.make_keyword("bar");
        assert!(kwid2.is_ok());
        let kwid2 = kwid2.ok().unwrap();
        assert!(kwid2 > 0);

        // duplicate keyword
        let kwid3 = catalog.make_keyword("foo");
        assert!(kwid3.is_ok());
        let kwid3 = kwid3.ok().unwrap();
        // should return kwid1 because it already exists.
        assert_eq!(kwid3, kwid1);

        assert!(catalog.assign_keyword(kwid1, file_id).is_ok());
        assert!(catalog.assign_keyword(kwid2, file_id).is_ok());

        let fl2 = catalog.get_keyword_content(kwid1);
        assert!(fl2.is_ok());
        let fl2 = fl2.ok().unwrap();
        assert_eq!(fl2.len(), 1);
        assert_eq!(fl2[0].id(), file_id);

        let kl = catalog.get_all_keywords();
        assert!(kl.is_ok());
        let kl = kl.ok().unwrap();
        assert_eq!(kl.len(), 2);

        // Testing bundles
        let mut bundle = FileBundle::new();
        assert!(bundle.add("img_0123.crw"));
        assert!(bundle.add("img_0123.jpg"));
        assert!(bundle.add("img_0123.thm"));
        assert!(bundle.add("img_0123.xmp"));

        let bundle_id = catalog.add_bundle(folder_added.id(), &bundle);
        assert!(bundle_id.is_ok());
        assert!(bundle_id.unwrap() > 0);
    }

    const XMP_PACKET: &str = "<x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"Exempi + XMP Core 5.1.2\"> \
 <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"> \
 </rdf:RDF> \
</x:xmpmeta>";

    #[test]
    fn file_bundle_import() {
        use npc_fwk::utils::exempi::XmpMeta;

        let catalog = test_catalog(None);

        let folder_added = catalog.add_folder_into("foo", Some("/bar/foo".to_string()), 0);
        assert!(folder_added.is_ok());
        let folder_added = folder_added.unwrap();

        let mut bundle0 = FileBundle::new();
        assert!(bundle0.add("img_0123.jpg"));
        assert!(bundle0.add("img_0123.raf"));

        let bundle_id = catalog.add_bundle(folder_added.id(), &bundle0);
        assert!(bundle_id.is_ok());
        assert!(bundle_id.ok().unwrap() > 0);

        let mut bundle = FileBundle::new();
        assert!(bundle.add("img_0124.jpg"));
        assert!(bundle.add("img_0124.raf"));

        let bundle_id = catalog.add_bundle(folder_added.id(), &bundle);
        assert!(bundle_id.is_ok());
        let bundle_id = bundle_id.unwrap();
        assert!(bundle_id > 0);

        // Test setting properties

        let mut props = NiepcePropertyBag::default();
        props.set_value(Np::Index(Npi::NpNiepceXmpPacket), XMP_PACKET.into());
        // one of the problem with XMP packet serialisation is that the version
        // of the XMP SDK is written in the header so we can do comparisons
        // byte by byte
        let original_xmp_packet =
            exempi2::Xmp::from_buffer(XMP_PACKET.as_bytes()).expect("XMP packet created");
        let original_xmp_packet = XmpMeta::from(original_xmp_packet);
        let result = catalog.set_image_properties(bundle_id, &props);
        result.expect("Setting the XMP works");

        let result = catalog.get_metadata(bundle_id);
        let metadata = result.expect("Have retrieved metadata");
        let xmp_packet = metadata.serialize_inline();
        assert_eq!(
            xmp_packet.as_str(),
            original_xmp_packet.serialize_inline().as_str()
        );
    }

    #[test]
    fn root_folders() {
        let catalog = test_catalog(None);

        // Check for a root folder for the folder: it doesn't exist.
        let lf = catalog.root_folder_for("/home/USER/Pictures/20230619");
        assert!(matches!(lf, Err(Error::NotFound)));

        // Add a root folder.
        let f = catalog.add_folder_into("Pictures", Some("/home/USER/Pictures".to_string()), 0);
        assert!(f.is_ok());
        let root_id = f.unwrap().id();

        // Check for a root folder for the folder: we just created it.
        let lf = catalog.root_folder_for("/home/USER/Pictures/20230619");
        assert!(lf.is_ok());
        let lf = lf.unwrap();
        // Checking its name and its id.
        assert_eq!(lf.id(), root_id);
        assert_eq!(lf.name(), "Pictures");

        // Check for a root folder for the sub folder: we just created it.
        let lf = catalog.root_folder_for("/home/USER/Pictures/20230619/phone");
        assert!(lf.is_ok());
        let lf = lf.unwrap();
        // Checking its name and its id.
        assert_eq!(lf.id(), root_id);
        assert_eq!(lf.name(), "Pictures");

        // Add a folder into
        let lf = catalog.add_folder_into("20230619", None, root_id);
        assert!(lf.is_ok());
        let lf = lf.unwrap();
        assert_eq!(lf.parent(), root_id);
        assert_eq!(lf.name(), "20230619");
        let folder_id = lf.id();

        // Add same folder into
        let lf = catalog.add_folder_into("20230619", None, root_id);
        assert!(lf.is_ok());
        let lf = lf.unwrap();
        assert_eq!(lf.parent(), root_id);
        assert_eq!(lf.name(), "20230619");
        assert_eq!(lf.id(), folder_id);

        let subfolders = catalog.get_subfolders(root_id);
        assert!(subfolders.is_ok());
        let subfolders = subfolders.unwrap();
        assert_eq!(subfolders.len(), 1);
    }

    #[test]
    fn reparent_folder() {
        let catalog = test_catalog(None);

        // Create 2 root folders.
        let f = catalog
            .add_folder_into("Pictures", Some("/home/USER/Pictures".to_string()), 0)
            .expect("First root couldn't be created");
        let root_id = f.id();
        let f = catalog
            .add_folder_into("Pictures2", Some("/home/USER/Pictures2".to_string()), 0)
            .expect("Second root couldn't be created");
        let root_id2 = f.id();

        // Create a new folder as a descendent of first root.
        let f = catalog
            .add_folder_into(
                "2025",
                Some("/home/USER/Pictures/2025".to_string()),
                root_id,
            )
            .expect("Couldn't create folder.");
        assert_eq!(f.parent(), root_id, "Added folder has wrong parent");

        // Reparent the folder to the second root.
        catalog
            .reparent_folder(f.id(), root_id2)
            .expect("Reparent failed");
        // Check the folder is now into the second root hierarchy.
        let f = catalog
            .get_folder("/home/USER/Pictures2/2025")
            .expect("Reparented folder should have been found");
        assert_eq!(f.parent(), root_id2, "Reparented folder has wrong parent");
    }

    #[test]
    fn reparent_roots() {
        let catalog = test_catalog(None);

        // Add a root folder.
        let f = catalog
            .add_folder_into(
                "20250816",
                Some("/home/USER/Pictures/2025/20250816".to_string()),
                0,
            )
            .expect("First root couldn't be created");
        let root_id = f.id();

        // Add a second root folder.
        let f = catalog
            .add_folder_into("Pictures", Some("/home/USER/Pictures".to_string()), 0)
            .expect("Second root couldn't be created");
        let root_id2 = f.id();

        // Sanity check: the first root folder exist.
        let root1 = catalog
            .get_folder("/home/USER/Pictures/2025/20250816")
            .expect("Folder (first root) missing");
        assert_eq!(root1.parent(), 0, "The folder isn't a root");

        // Reparent so that the first root folder is actually a lead
        // from the second root folder.
        catalog
            .reparent_roots_for(root_id2, "/home/USER/Pictures")
            .expect("Reparent failed");
        // Need to get the folder to get new values.
        let root1 = catalog
            .get_folder("/home/USER/Pictures/2025/20250816")
            .expect("Folder missing");
        assert_eq!(root1.id(), root_id, "Root1 ID changed");
        assert_ne!(root1.parent(), 0, "Root1 is still root");
        // Check we actually created the intermediate folder and the
        // hierarchy is correct.
        let folder1 = catalog
            .get_folder("/home/USER/Pictures/2025")
            .expect("Folder missing");
        assert_eq!(folder1.parent(), root_id2, "Folder 1 parent isn't root2");
        assert_eq!(root1.parent(), folder1.id(), "Root1 parent isn't folder1");
    }

    #[test]
    fn preferences() {
        let catalog = test_catalog(None);

        // Sanity testing.
        let dbversion = catalog.get_pref("version", None);
        let verstr = format!("{DB_SCHEMA_VERSION}");
        assert_eq!(dbversion, Ok(verstr));

        // Test not found and default.
        let pref1 = catalog.get_pref("prefs.pref1", None);
        assert_eq!(pref1, Err(Error::NotFound));
        let pref1 = catalog.get_pref("prefs.pref1", Some("default"));
        assert_eq!(pref1, Ok("default".to_string()));
        // Ensure default didn't set it.
        let pref1 = catalog.get_pref("prefs.pref1", None);
        assert_eq!(pref1, Err(Error::NotFound));

        // Test setting values
        let pref1 = catalog.set_pref("prefs.pref1", "value1");
        assert!(pref1.is_ok());
        let pref1 = catalog.get_pref("prefs.pref1", None);
        assert_eq!(pref1, Ok("value1".to_string()));
        let pref1 = catalog.set_pref("prefs.pref1", "value2");
        assert!(pref1.is_ok());
        let pref1 = catalog.get_pref("prefs.pref1", None);
        assert_eq!(pref1, Ok("value2".to_string()));

        let prefs = catalog.get_all_preferences();
        assert!(prefs.is_ok());
        let prefs = prefs.unwrap();
        assert_eq!(prefs.len(), 2);
    }
}
