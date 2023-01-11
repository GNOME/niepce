/*
 * niepce - import/mod.rs
 *
 * Copyright (C) 2022-2023 Hubert Figuière
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
use std::rc::Rc;

use npc_engine::importer::ImportBackend;

/// An import request
pub struct ImportRequest {
    source: String,
    dest: PathBuf,
    importer: Rc<dyn ImportBackend>,
}

impl ImportRequest {
    pub fn new<P: AsRef<Path>>(source: String, dest: P, importer: Rc<dyn ImportBackend>) -> Self {
        Self {
            source,
            dest: dest.as_ref().to_path_buf(),
            importer,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn dest_dir(&self) -> &Path {
        &self.dest
    }

    pub fn importer(&self) -> &Rc<dyn ImportBackend> {
        &self.importer
    }
}
