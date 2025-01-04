/*
 * niepce - engine/db/fsfile.rs
 *
 * Copyright (C) 2017-2020 Hubert Figuière
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

use super::LibraryId;

/// Describe a file on the file system
#[derive(Clone, Debug)]
pub struct FsFile {
    id: LibraryId,
    path: PathBuf,
}

impl FsFile {
    pub fn new(id: LibraryId, path: PathBuf) -> FsFile {
        FsFile { id, path }
    }

    pub fn id(&self) -> LibraryId {
        self.id
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}
