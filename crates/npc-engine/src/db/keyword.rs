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
use libc::c_char;
use std::ffi::CStr;
use std::ffi::CString;

#[derive(Clone)]
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

/// # Safety
/// Dereference raw pointer.
#[no_mangle]
pub unsafe extern "C" fn engine_db_keyword_new(id: i64, keyword: *const c_char) -> *mut Keyword {
    let kw = Box::new(Keyword::new(
        id,
        &*CStr::from_ptr(keyword).to_string_lossy(),
    ));
    Box::into_raw(kw)
}

#[no_mangle]
pub extern "C" fn engine_db_keyword_id(obj: &Keyword) -> i64 {
    obj.id() as i64
}

#[no_mangle]
pub extern "C" fn engine_db_keyword_keyword(obj: &Keyword) -> *mut c_char {
    let cstr = CString::new(obj.keyword()).unwrap();
    cstr.into_raw()
}

/// # Safety
/// Dereference raw pointer.
#[no_mangle]
pub unsafe extern "C" fn engine_db_keyword_delete(kw: *mut Keyword) {
    drop(Box::from_raw(kw));
}
