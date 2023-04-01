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

use nix::sys::stat::{stat, utimensat, UtimensatFlags};
use nix::sys::time::TimeSpec;
use walkdir::WalkDir;

use crate::toolkit::mimetype::{guess_type_for_file, MType};

/// Copy file `from` to `to`. Return the number of bytes copied which
/// is the size of both `from` and `to`. See also [`std::fs::copy`] as
/// this is called underneath. In addition to calling `copy`, it will
/// set access and modification time of the destination to those of
/// the source.
///
/// # Portability
///
/// This currently use the crate `nix`.
///
/// # Error
///
/// Will return an error if [`std::fs::copy`] fails with the same
/// parameters, or any of the call to get and set the times fail.
pub fn copy<P, Q>(from: P, to: Q) -> std::io::Result<u64>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let length = std::fs::copy(from.as_ref(), to.as_ref())?;

    //let created = to.metadata().and_then(|m| m.created());
    // XXX we do nothing with the created date.
    let file_stat = stat(from.as_ref())?;
    utimensat(
        None,
        to.as_ref(),
        &TimeSpec::new(file_stat.st_atime, file_stat.st_atime_nsec),
        &TimeSpec::new(file_stat.st_mtime, file_stat.st_mtime_nsec),
        UtimensatFlags::NoFollowSymlink,
    )?;

    Ok(length)
}

#[derive(Clone, Debug, Default)]
pub struct FileList(pub Vec<PathBuf>);

impl std::ops::Deref for FileList {
    type Target = Vec<PathBuf>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for FileList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FileList {
    /// Get the files matching `filter` from `dir`, sorted alphabetically.
    ///
    /// `filter` is a function that will return `true` for files to keep
    /// `recursive` list the content recursively.
    pub fn files_from_directory<P, F>(dir: P, filter: F, recursive: bool) -> Self
    where
        P: AsRef<Path>,
        F: Fn(&Path) -> bool + 'static,
    {
        if !dir.as_ref().is_dir() {
            err_out!("Not a directory: {:?}", dir.as_ref());
            return FileList::default();
        }

        let entries = if recursive {
            WalkDir::new(&dir)
        } else {
            WalkDir::new(&dir).max_depth(1)
        }
        .into_iter()
        // ignore everything that starts with a '.'
        .filter_entry(|entry| {
            !entry
                .file_name()
                .to_str()
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
        })
        .flatten()
        .filter_map(|entry| {
            let ftype = entry.file_type();
            if (ftype.is_file() || ftype.is_symlink()) && filter(entry.path()) {
                Some(entry.path().to_path_buf())
            } else {
                dbg_out!("Filtered out {:?}", entry);
                None
            }
        })
        .collect::<Vec<PathBuf>>();
        let mut l = FileList(entries);
        l.sort();
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
        let p = root_p.join("sub");
        assert!(fs::create_dir_all(p).is_ok());
        let file1 = root_p.join("1");
        assert!(fs::write(file1, "one").is_ok());
        let file2 = root_p.join("2");
        assert!(fs::write(file2, "two").is_ok());
        let file3 = root_p.join("3");
        assert!(fs::write(file3, "three").is_ok());

        let files = FileList::files_from_directory("foo", |_| true, false);

        assert_eq!(files.len(), 0);

        let files = FileList::files_from_directory(&root_p, |_| true, false);
        println!("files {files:?}");
        assert_eq!(files.len(), 3);

        assert!(fs::remove_dir_all(&root_p).is_ok());
    }
}
