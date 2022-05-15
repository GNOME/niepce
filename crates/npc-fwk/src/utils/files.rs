/*
 * niepce - fwk/utils/files.rs
 *
 * Copyright (C) 2018-2022 Hubert Figuière
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
use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use crate::toolkit::mimetype::{guess_type_for_file, MType};

#[derive(Clone, Default)]
pub struct FileList(pub Vec<PathBuf>);

impl FileList {
    /// Get files from directory P, possibly filtered by F.
    pub fn get_files_from_directory<P, F>(dir: P, filter: F) -> Self
    where
        P: AsRef<Path>,
        F: Fn(&Path) -> bool + 'static,
    {
        let mut l = FileList::default();
        if !dir.as_ref().is_dir() {
            err_out!("Not a directory: {:?}", dir.as_ref());
            return l;
        }
        if let Ok(read_dir) = std::fs::read_dir(dir) {
            for entry in read_dir {
                if entry.is_err() {
                    err_out!("Enumeration failed: {:?}", entry.err());
                    continue;
                }
                let entry = entry.unwrap();
                if let Ok(ftype) = entry.file_type() {
                    if ftype.is_file() || ftype.is_symlink() {
                        if !filter(&entry.path()) {
                            dbg_out!("Filtered out");
                            continue;
                        }
                        l.0.push(entry.path());
                    }
                }
            }
        }

        l.0.sort();
        l
    }

    pub fn file_is_media(fileinfo: &Path) -> bool {
        let t = guess_type_for_file(fileinfo);
        return matches!(t, MType::Image(_) | MType::Movie);
    }
}

/// Tell is the file pointed by finfo is a media file
///
/// # Safety
/// Dereference the finfo pointer.
#[no_mangle]
pub unsafe extern "C" fn fwk_file_is_media(file: *const c_char) -> bool {
    let cfile = CStr::from_ptr(file);
    let fileinfo = PathBuf::from(std::ffi::OsStr::from_bytes(cfile.to_bytes()));
    FileList::file_is_media(&fileinfo)
}

#[no_mangle]
pub extern "C" fn fwk_file_list_new() -> *mut FileList {
    Box::into_raw(Box::new(FileList::default()))
}

/// Delete the file list object from ffi code
///
/// # Safety
/// Dereference the pointer
#[no_mangle]
pub unsafe extern "C" fn fwk_file_list_delete(l: *mut FileList) {
    Box::from_raw(l);
}

/// Get the files in directory dir
///
/// # Safety
/// Dereference the dir pointer (C String)
#[no_mangle]
pub unsafe extern "C" fn fwk_file_list_get_files_from_directory(
    dir: *const c_char,
    filter: Option<extern "C" fn(*const c_char) -> bool>,
) -> *mut FileList {
    let cstr = CStr::from_ptr(dir);
    match filter {
        Some(filter) => {
            let f = Box::new(filter);
            Box::into_raw(Box::new(FileList::get_files_from_directory(
                &PathBuf::from(&*cstr.to_string_lossy()),
                move |p| {
                    if let Ok(pc) = CString::new(p.as_os_str().as_bytes()) {
                        f(pc.as_ptr())
                    } else {
                        err_out!("file path conversion failed.");
                        false
                    }
                },
            )))
        }
        None => Box::into_raw(Box::new(FileList::get_files_from_directory(
            &PathBuf::from(&*cstr.to_string_lossy()),
            move |_| true,
        ))),
    }
}

#[no_mangle]
pub extern "C" fn fwk_file_list_size(l: &FileList) -> usize {
    l.0.len()
}

/// Return the path string at index %idx
/// The resulting string must be freeed with %rust_cstring_delete
#[no_mangle]
pub extern "C" fn fwk_file_list_at(l: &FileList, idx: usize) -> *mut c_char {
    CString::new(l.0[idx].to_string_lossy().as_bytes())
        .unwrap()
        .into_raw()
}

#[no_mangle]
/// Push a file path to the list
///
/// # Safety
/// Dereference the value pointer (C string)
pub unsafe extern "C" fn fwk_file_list_push_back(l: &mut FileList, value: *const c_char) {
    assert!(!value.is_null());
    if value.is_null() {
        return;
    }
    let s = CStr::from_ptr(value);
    l.0.push(PathBuf::from(&*s.to_string_lossy()));
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    pub fn test_files_sanity() {
        let root_p = PathBuf::from("AAtest");
        let mut p = root_p.clone();
        p.push("sub");
        assert!(fs::create_dir_all(&p).is_ok());
        let mut file1 = root_p.clone();
        file1.push("1");
        assert!(fs::write(&file1, "one").is_ok());
        let mut file2 = root_p.clone();
        file2.push("2");
        assert!(fs::write(file2, "two").is_ok());
        let mut file3 = root_p.clone();
        file3.push("3");
        assert!(fs::write(file3, "three").is_ok());

        let files = FileList::get_files_from_directory("foo", |_| true);

        assert_eq!(files.0.len(), 0);

        let files = FileList::get_files_from_directory(&root_p, |_| true);
        assert_eq!(files.0.len(), 3);

        assert!(fs::remove_dir_all(&root_p).is_ok());
    }
}
