/*
 * niepce - import/mod.rs
 *
 * Copyright (C) 2022 Hubert Figuière
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

/// An import request
pub struct ImportRequest {
    source: String,
    dest: String,
    importer: cxx::SharedPtr<crate::ffi::IImporter>,
}

impl ImportRequest {
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn dest_dir(&self) -> &str {
        &self.dest
    }

    pub fn importer(&self) -> &cxx::SharedPtr<crate::ffi::IImporter> {
        &self.importer
    }
}

pub fn import_request_new(
    source: &str,
    dest: &str,
    importer: cxx::SharedPtr<crate::ffi::IImporter>,
) -> Box<ImportRequest> {
    Box::new(ImportRequest {
        source: source.to_string(),
        dest: dest.to_string(),
        importer,
    })
}
