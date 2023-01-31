/*
 * niepce - fwk/utils/files.rs
 *
 * Copyright (C) 2018-2023 Hubert Figuière
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

use crate::toolkit::mimetype::{guess_type_for_file, MType};

#[derive(Clone, Default)]
pub struct FileList(pub Vec<PathBuf>);

impl FileList {
    /// Get the files matching `filter` from `dir`.
    ///
    /// `filter` is a function that will return `true` for files to keep
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
                            dbg_out!("Filtered out {:?}", entry);
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
        matches!(t, MType::Image(_) | MType::Movie)
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
