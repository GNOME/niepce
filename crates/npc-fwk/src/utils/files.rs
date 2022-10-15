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

use std::path::{Path, PathBuf};

use gio::prelude::*;

use crate::toolkit::mimetype::{guess_type, MType};

#[derive(Clone, Default)]
pub struct FileList(pub Vec<PathBuf>);

impl FileList {
    // cxx
    pub fn size(&self) -> usize {
        self.0.len()
    }

    // cxx
    /// Return the path string at index %idx
    /// The resulting string must be freeed with %rust_cstring_delete
    pub fn at(&self, idx: usize) -> String {
        self.0[idx].to_string_lossy().into()
    }

    // cxx
    /// Push a file path to the list
    pub fn push_back(&mut self, value: &str) {
        self.0.push(PathBuf::from(value));
    }

    /// Get the files matching `filter` from `dir`.
    ///
    /// `filter` is a function that will return `true` for files to keep
    pub fn get_files_from_directory<P, F>(dir: P, filter: F) -> Self
    where
        P: AsRef<Path>,
        F: Fn(&gio::FileInfo) -> bool + 'static,
    {
        let mut l = FileList::default();

        let dir = gio::File::for_path(dir);
        let dir_path = dir.path();
        if dir_path.is_none() {
            err_out!("Couldn't get dir path");
            return l;
        }
        let dir_path = dir_path.unwrap();
        if let Ok(enumerator) = dir.enumerate_children(
            "*",
            gio::FileQueryInfoFlags::NONE,
            Option::<&gio::Cancellable>::None,
        ) {
            for itr in enumerator {
                if itr.is_err() {
                    err_out!("Enumeration failed: {:?}", itr.err());
                    continue;
                }
                let finfo = itr.unwrap();
                let ftype = finfo.file_type();
                if ftype == gio::FileType::Regular || ftype == gio::FileType::SymbolicLink {
                    if !filter(&finfo) {
                        err_out!("Filtered out");
                        continue;
                    }
                    let name = finfo.name();
                    let fullname: PathBuf = [&dir_path, &name].iter().collect();
                    dbg_out!("Found file {:?}", &fullname);
                    l.0.push(fullname);
                }
            }
        }

        l.0.sort();
        l
    }

    pub fn file_is_media(fileinfo: &gio::FileInfo) -> bool {
        if let Some(gmtype) = fileinfo.content_type() {
            let t = guess_type(&gmtype);
            return matches!(t, MType::Image(_) | MType::Movie);
        }

        err_out!("Coudln't get file type");
        false
    }
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
