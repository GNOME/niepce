/*
 * niepce - npc-engine/src/db/schema_test.rs
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

//! Schema testing fixtures for TEST ONLY

use chrono::Utc;
use num_traits::ToPrimitive;
use rusqlite::params;

use super::libfolder;
use super::{Result, sql, upgrade};

/// Create a v9 schema
fn init_schema_v9(conn: &rusqlite::Connection) -> Result<()> {
    const DB_SCHEMA_VERSION: i32 = 9;

    conn.execute("CREATE TABLE admin (key TEXT NOT NULL, value TEXT)", [])
        .unwrap();
    conn.execute(
        "INSERT INTO admin (key, value) \
         VALUES ('version', ?1)",
        params![DB_SCHEMA_VERSION],
    )
    .unwrap();
    conn.execute(
        "CREATE TABLE vaults (id INTEGER PRIMARY KEY, path TEXT)",
        [],
    )
    .unwrap();
    conn.execute(
        "CREATE TABLE folders (id INTEGER PRIMARY KEY,\
         path TEXT, name TEXT, \
         vault_id INTEGER DEFAULT 0, \
         locked INTEGER DEFAULT 0, \
         virtual INTEGER DEFAULT 0, \
         expanded INTEGER DEFAULT 0, \
         parent_id INTEGER)",
        [],
    )
    .unwrap();
    // Version 9
    conn.execute(
        "CREATE TRIGGER folder_delete_trigger AFTER DELETE ON folders \
         BEGIN \
         DELETE FROM files WHERE parent_id = old.id; \
         END",
        [],
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

    conn.execute(
        "CREATE TABLE files (id INTEGER PRIMARY KEY,\
         main_file INTEGER, name TEXT, parent_id INTEGER,\
         orientation INTEGER, file_type INTEGER,\
         file_date INTEGER, rating INTEGER DEFAULT 0, \
         label INTEGER, flag INTEGER DEFAULT 0, \
         import_date INTEGER, mod_date INTEGER, \
         xmp TEXT, xmp_date INTEGER, xmp_file INTEGER,\
         jpeg_file INTEGER)",
        [],
    )
    .unwrap();
    conn.execute(
        "CREATE TABLE fsfiles (id INTEGER PRIMARY KEY,\
         path TEXT)",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE sidecars (file_id INTEGER,\
         fsfile_id INTEGER, type INTEGER, ext TEXT NOT NULL,\
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
         END; \
         COMMIT;",
    )
    .unwrap();
    //
    conn.execute(
        "CREATE TABLE keywords (id INTEGER PRIMARY KEY,\
         keyword TEXT, parent_id INTEGER DEFAULT 0)",
        [],
    )
    .unwrap();
    conn.execute(
        "CREATE TABLE keywording (file_id INTEGER,\
         keyword_id INTEGER, UNIQUE(file_id, keyword_id))",
        [],
    )
    .unwrap();
    conn.execute(
        "CREATE TRIGGER keyword_delete_trigger AFTER DELETE ON keywords \
         BEGIN \
         DELETE FROM keywording WHERE keyword_id = old.id; \
         END;",
        [],
    )
    .unwrap();
    conn.execute(
        "CREATE TABLE labels (id INTEGER PRIMARY KEY,\
         name TEXT, color TEXT)",
        [],
    )
    .unwrap();
    conn.execute("CREATE TABLE xmp_update_queue (id INTEGER UNIQUE)", [])
        .unwrap();
    conn.execute(
        "CREATE TRIGGER file_update_trigger UPDATE ON files \
         BEGIN \
         UPDATE files SET mod_date = strftime('%s','now');\
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

    Ok(())
}

#[test]
/// This test the upgrade SQL command from v9 to v11. (there is no v10)
/// A lot of the SQL code is hardcoded and expect a specific formatting.
fn test_upgrade_9_to() {
    if let Ok(conn) = rusqlite::Connection::open_in_memory() {
        init_schema_v9(&conn).expect("Couldn't initialise schema v9");
        let schema_version = sql::pragma_schema_version(&conn).expect("pragma schema version");

        {
            let time = Utc::now().timestamp();
            conn.execute(
                "INSERT INTO files (\
                      main_file, name, parent_id, import_date, mod_date,\
                      orientation, file_date, rating, label, file_type, flag, xmp)\
                      VALUES (\
                      ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    42_i32, "Filename", 10_i32, time, time, 1_i32, time, 0_i32, 0_i32, 3_i32,
                    0_i32, "",
                ],
            )
            .expect("Failed to add a row into table 'files'");
        }

        let mut stmt = conn
            .prepare("SELECT id FROM files WHERE xmp_file = 0")
            .expect("Prepare failed");
        let mut rows = stmt.query([]).expect("Query failed");
        let row = rows.next().expect("Couldn't get row");
        assert!(row.is_none(), "Found row with file = 0");

        upgrade::perform_upgrade_11(&conn, schema_version).expect("Upgrade to 11");

        assert!(sql::pragma_schema_version(&conn).expect("pragma schema version") > schema_version);

        {
            // XXX move this to `sql`
            let mut stmt = conn
                .prepare("PRAGMA integrity_check")
                .expect("Prepare failed");
            let mut rows = stmt.query([]).expect("Query failed");
            let row = rows.next().expect("Couldn't get row");
            assert!(row.is_some(), "Integrity check returned no result");

            let result: String = row.unwrap().get(0).expect("Failed to get row value");
            assert_eq!(&result, "ok", "Integrity check not OK");

            let mut stmt = conn
                .prepare("SELECT id FROM files WHERE xmp_file = 0")
                .expect("Prepare failed");
            let mut rows = stmt.query([]).expect("Query failed");
            let row = rows.next().expect("Couldn't get row");
            assert!(row.is_some(), "NULL xmp_file not converted to 0");

            let files_v11 = sql::table_sql(&conn, "files").expect("Files sql failed");

            assert_eq!(
                files_v11,
                "CREATE TABLE files (id INTEGER PRIMARY KEY AUTOINCREMENT, \
                 main_file INTEGER, name TEXT, parent_id INTEGER, \
                 orientation INTEGER, file_type INTEGER, \
                 file_date INTEGER, rating INTEGER DEFAULT 0, \
                 label INTEGER, flag INTEGER DEFAULT 0, \
                 import_date INTEGER, mod_date INTEGER, \
                 xmp TEXT, xmp_date INTEGER, xmp_file INTEGER DEFAULT 0, \
                 jpeg_file INTEGER DEFAULT 0)"
            );
        }

        let schema_version = sql::pragma_schema_version(&conn).expect("pragma schema version");
        upgrade::perform_upgrade_12(&conn, schema_version).expect("Upgrade to 12");
        assert!(sql::pragma_schema_version(&conn).expect("pragma schema version") > schema_version);

        let trigger = sql::trigger_sql(&conn, "folders_update_parent").expect("Trigger sql failed");
        assert_eq!(
            trigger,
            "CREATE TRIGGER folders_update_parent AFTER UPDATE OF parent_id ON \"folders\" \
             BEGIN \
             UPDATE \"folders\" SET path = (SELECT f.path FROM \"folders\" AS f WHERE f.id = \"folders\".parent_id) || '/' || name WHERE id = NEW.id AND parent_id != 0; \
             END"
        );

        let schema_version = sql::pragma_schema_version(&conn).expect("pragma schema version");
        upgrade::perform_upgrade_13(&conn, schema_version).expect("Upgrade to 13");
        assert!(sql::pragma_schema_version(&conn).expect("pragma schema version") > schema_version);
    }
}
