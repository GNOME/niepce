/*
 * niepce - library/previewer/cache.rs
 *
 * Copyright (C) 2023-2025 Hubert Figuière
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

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc::SyncSender;

use super::RenderParams;
use crate::catalog;
use npc_fwk::base::{Worker, WorkerImpl, WorkerStatus};
use npc_fwk::{dbg_out, err_out, on_err_out};

/// Schema version for the cache
const DB_SCHEMA_VERSION: i32 = 1;
/// DB Schema
const SCHEMA_CREATION: &str = "BEGIN;\
CREATE TABLE admin (key TEXT NOT NULL, value TEXT);\
CREATE TABLE cache_items (\
       id INTEGER PRIMARY KEY AUTOINCREMENT,\
       path TEXT NOT NULL,\
       last_access INTEGER,\
       created INTEGER,\
       dimension INTEGER,\
       render TEXT NOT NULL,\
       target TEXT NOT NULL,\
       UNIQUE (path, dimension, render) \
       ON CONFLICT REPLACE);\
COMMIT;";
/// Initial values: `kind` is 'cache' and 'version' is 1.
const ADMIN_TABLE_INIT: &str = "INSERT INTO admin (key, value) \
                                 VALUES ('version', ?1), ('kind', 'cache');";

#[derive(Debug)]
pub struct CacheItem {
    id: i64,
    _path: PathBuf,
    _last_access: i64,
    _created: i64,
    _dimension: i32,
    _render: String,
    pub target: PathBuf,
}

pub(crate) enum DbMessage {
    Init(PathBuf),
    Put(String, u32, RenderParams, String),
    Get(String, String, SyncSender<catalog::LibResult<CacheItem>>),
    Hit(String, String),
}

#[derive(Default)]
struct DbWorker {
    /// sqlite3 connection handle
    dbconn: RefCell<Option<rusqlite::Connection>>,
}

impl DbWorker {
    /// Check the database version as stored in the admin table.
    /// A version of 0 mean the database is empty.
    fn check_database_version(&self, conn: &rusqlite::Connection) -> catalog::LibResult<i32> {
        dbg_out!("Checking version");
        let result = conn.prepare("SELECT value FROM admin WHERE key='kind'");
        if let Ok(mut stmt) = result {
            let mut rows = stmt.query([])?;
            if let Ok(Some(row)) = rows.next() {
                let value: String = row.get(0)?;
                if value != "cache" {
                    return Err(catalog::LibError::InvalidResult);
                }
            }
        }
        let result = conn.prepare("SELECT value FROM admin WHERE key='version'");
        if let Ok(mut stmt) = result {
            let mut rows = stmt.query([])?;
            if let Ok(Some(row)) = rows.next() {
                let value: String = row.get(0)?;
                return value
                    .parse::<i32>()
                    .map_err(|_| catalog::LibError::InvalidResult);
            }
        }

        // if query fail we assume 0 to create the database.
        dbg_out!("Check version: seems to be new");
        Ok(0)
    }

    /// Initialize the database. Will create the schema if needed.
    fn initialize(&self, cache_dir: &Path) -> catalog::LibResult<()> {
        let db_file = cache_dir.join("cache.db");
        dbg_out!("Opening database at {:?}", db_file);
        on_err_out!(cachedir::ensure_tag(cache_dir));

        if let Ok(conn) = rusqlite::Connection::open(&db_file) {
            dbg_out!("db is open");
            let v = self.check_database_version(&conn);
            match v {
                Ok(0) => {
                    conn.execute_batch(SCHEMA_CREATION)?;
                    conn.execute(ADMIN_TABLE_INIT, rusqlite::params![&DB_SCHEMA_VERSION])?;
                }
                Ok(v) => {
                    if v != DB_SCHEMA_VERSION {
                        dbg_out!("Chache version check incorrect value {}", v);
                        return Err(catalog::LibError::IncorrectDbVersion);
                    }
                }
                Err(err) => return Err(err),
            }

            self.dbconn.replace(Some(conn));
            return Ok(());
        }

        err_out!("Couldn't open database");
        Err(catalog::LibError::NoSqlDb)
    }

    /// Put a new entry in the cache.
    fn put(
        &self,
        file: &str,
        size: u32,
        render: &RenderParams,
        dest: &str,
    ) -> catalog::LibResult<()> {
        let now = chrono::Utc::now().timestamp();
        if let Some(conn) = &*self.dbconn.borrow() {
            let mut stmt = conn.prepare(
                "INSERT INTO cache_items (path, last_access, created, dimension, render, target) \
                                         VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            )?;
            stmt.execute(rusqlite::params![
                &file,
                now,
                now,
                size,
                &render.key(),
                &dest,
            ])?;

            return Ok(());
        }

        Err(catalog::LibError::NoSqlDb)
    }

    /// "Hit" the cache, ie update the access date.
    fn hit(&self, file: &str, digest: &str) -> catalog::LibResult<()> {
        if let Some(conn) = &*self.dbconn.borrow() {
            let mut stmt = conn.prepare(
                "UPDATE cache_items SET last_access = ?1 \
                                         WHERE path = ?2 AND render = ?3;",
            )?;
            let now = chrono::Utc::now().timestamp();
            stmt.execute(rusqlite::params![now, file, digest])?;
            return Ok(());
        }

        Err(catalog::LibError::NoSqlDb)
    }

    /// Update the access for the row id.
    fn update_access(&self, conn: &rusqlite::Connection, id: i64) -> catalog::LibResult<usize> {
        let mut stmt = conn.prepare("UPDATE cache_items SET last_access = ?1 WHERE id = ?2;")?;
        let now = chrono::Utc::now().timestamp();
        let count = stmt.execute(rusqlite::params![now, id])?;
        Ok(count)
    }

    /// Get the preview for `file` with `size` from the cache, and return its path.
    ///
    /// This will cause `last_access` to be updated to now, however the new value is
    /// not returned.
    fn get(&self, file: &str, digest: &str) -> catalog::LibResult<CacheItem> {
        if let Some(conn) = &*self.dbconn.borrow() {
            let mut stmt = conn.prepare(
                "SELECT id, path, last_access, created, dimension, render, target \
                                         FROM cache_items WHERE path = ?1 AND render = ?2;",
            )?;
            let mut results = stmt.query_map(rusqlite::params![file, digest], |row| {
                Ok(CacheItem {
                    id: row.get(0)?,
                    _path: PathBuf::from(row.get::<usize, String>(1)?),
                    _last_access: row.get(2)?,
                    _created: row.get(3)?,
                    _dimension: row.get(4)?,
                    _render: row.get(5)?,
                    target: PathBuf::from(row.get::<usize, String>(6)?),
                })
            })?;

            let item = results
                .next()
                .map(|r| r.map_err(catalog::LibError::SqlError))
                .unwrap_or(Err(catalog::LibError::NotFound));

            dbg_out!("Found item {:?}", item);
            let item = item?;
            let r = self.update_access(conn, item.id);
            dbg_out!("access updated {:?}", r);
            return Ok(item);
        }

        Err(catalog::LibError::NoSqlDb)
    }
}

impl WorkerImpl for DbWorker {
    type Message = DbMessage;
    type State = Option<()>;

    fn dispatch(&self, msg: Self::Message, _: &mut Self::State) -> WorkerStatus {
        match msg {
            DbMessage::Init(p) => {
                on_err_out!(self.initialize(&p));
            }
            DbMessage::Hit(p, d) => {
                on_err_out!(self.hit(&p, &d));
            }
            DbMessage::Put(p, d, r, dest) => {
                on_err_out!(self.put(&p, d, &r, &dest));
            }
            DbMessage::Get(p, d, r) => {
                on_err_out!(r.send(self.get(&p, &d)));
            }
        };

        WorkerStatus::Continue
    }
}

/// The cache for previews
pub(crate) struct Cache {
    /// Directory for the cache
    cache_dir: PathBuf,
    /// Database worker
    worker: Mutex<Worker<DbWorker>>,
}

impl Cache {
    /// Create a new cache. Will create the directories.
    /// Call `initialize` for the database to be ready to use.
    pub fn new(cache_dir: PathBuf) -> Self {
        let worker = Mutex::new(Worker::<DbWorker>::default());
        // Ensure that the cache directory exists.
        on_err_out!(std::fs::create_dir_all(&cache_dir));
        Self { cache_dir, worker }
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn sender(&self) -> std::sync::mpsc::Sender<DbMessage> {
        self.worker.lock().unwrap().sender().clone()
    }

    pub fn initialize(&self) {
        on_err_out!(
            self.worker
                .lock()
                .unwrap()
                .send(DbMessage::Init(self.cache_dir.to_path_buf()))
        );
    }

    pub fn hit(&self, file: &str, digest: &str) {
        on_err_out!(
            self.worker
                .lock()
                .unwrap()
                .send(DbMessage::Hit(file.to_string(), digest.to_string()))
        );
    }

    pub fn get(&self, file: &str, digest: &str) -> catalog::LibResult<CacheItem> {
        let (sender, receiver) = std::sync::mpsc::sync_channel::<catalog::LibResult<CacheItem>>(1);
        on_err_out!(self.worker.lock().unwrap().send(DbMessage::Get(
            file.to_string(),
            digest.to_string(),
            sender
        )));
        receiver.recv().unwrap()
    }

    pub fn put(&self, file: &str, dimension: u32, render: RenderParams, dest: &str) {
        assert_ne!(dest.chars().next(), Some('/'));
        on_err_out!(self.worker.lock().unwrap().send(DbMessage::Put(
            file.to_string(),
            dimension,
            render,
            dest.to_string(),
        )));
    }

    /// For a thumbnail get a file system path, relative to the
    /// `cache_dir`.
    pub fn path_for_thumbnail(
        &self,
        filename: &Path,
        id: catalog::LibraryId,
        digest: &str,
    ) -> Option<PathBuf> {
        // XXX properly report the error
        let base_name = filename.file_name().and_then(|f| f.to_str())?;
        let thumb_name = format!("{digest}-{id}-{base_name}.png");

        Some(Self::dir_for_thumbnail(digest).join(thumb_name))
    }

    /// Relative directory for the thumbnail. Note that currently we
    /// use the digest hash value.
    fn dir_for_thumbnail(digest: &str) -> PathBuf {
        let mut dir = PathBuf::from("files");
        if digest.len() < 4 {
            err_out!("Invalid digest {digest}");
            dir.push("error");
        } else {
            dir.push(&digest[0..2]);
            dir.push(&digest[2..4]);
        }

        dir
    }
}

#[cfg(test)]
mod test {

    use npc_fwk::base::Size;

    use super::super::RenderParams;
    use super::Cache;
    use crate::catalog;

    #[test]
    fn the_cache_works() {
        let tmpdir = tempfile::tempdir().expect("Couldn't create test temp dir for the cache");

        let cache_dir = tmpdir.path().join("preview_cache");

        let cache = Cache::new(cache_dir);
        cache.initialize();

        let file_name = "test-image1.jpg";
        let file_path = tmpdir.path().join("images").join(file_name);
        let libfile = catalog::LibFile::new(15, 14, 13, file_path.clone(), file_name);

        let rendering = RenderParams::new_thumbnail(libfile.id(), Size { w: 160, h: 120 });
        let digest = rendering.digest();
        assert!(cache.get(&file_path.to_string_lossy(), &digest).is_err());
        let thumb_path = cache
            .path_for_thumbnail(&file_path, libfile.id(), &digest)
            .expect("Couldn't build thumbnail path");
        cache.put(
            &file_path.to_string_lossy(),
            160,
            rendering,
            &thumb_path.to_string_lossy(),
        );

        let cache_item = cache
            .get(&file_path.to_string_lossy(), &digest)
            .expect("Cache entry not found");
        assert_eq!(thumb_path, cache_item.target);
        assert_eq!(cache_item._last_access, cache_item._created);
        let last_used = cache_item._last_access;

        std::thread::sleep(std::time::Duration::from_secs(2));

        // Get it to update the access date. About two second after the retained one.
        cache
            .get(&file_path.to_string_lossy(), &digest)
            .expect("Cache entry not found");
        // Check the stored date.
        let cache_item = cache
            .get(&file_path.to_string_lossy(), &digest)
            .expect("Cache entry not found");
        assert!(last_used < cache_item._last_access);
    }
}
