/*
 * niepce - capi.rs
 *
 * Copyright (C) 2017-2018 Hubert Figuière
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

use libc::c_char;
use std::ffi::CString;

/// Release the raw pointer from a CString.
///
/// # Safety
/// Dereference the pointer
#[no_mangle]
pub unsafe extern "C" fn rust_cstring_delete(string: *mut c_char) {
    drop(CString::from_raw(string));
}
