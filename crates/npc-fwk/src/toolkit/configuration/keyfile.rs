/*
 * niepce - fwk/toolkit/configuration/keyfile.rs
 *
 * Copyright (C) 2022-2024 Hubert Figuière
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

use super::ConfigBackend;

/// A configuration, backed by a `glib::Keyfile`
pub(super) struct KeyFile {
    filename: std::path::PathBuf,
    keyfile: glib::KeyFile,
    root: String,
}

impl ConfigBackend for KeyFile {
    /// Return true if it has `key`.
    fn has(&self, key: &str) -> bool {
        self.keyfile.has_group(&self.root) && self.keyfile.has_key(&self.root, key).unwrap_or(false)
    }

    /// Return the string value for `key` or `None` if not found.
    fn value_opt(&self, key: &str) -> Option<String> {
        if !self.has(key) {
            return None;
        }
        self.keyfile
            .string(&self.root, key)
            .map(|v| v.as_str().to_string())
            .ok()
    }

    /// Return string value for `key`, or `def` if not found.
    fn value(&self, key: &str, def: &str) -> String {
        self.value_opt(key).unwrap_or_else(|| def.to_string())
    }

    /// Set `value` for `key`
    fn set_value(&self, key: &str, value: &str) {
        self.keyfile.set_string(&self.root, key, value);
        on_err_out!(self.save());
    }
}

impl KeyFile {
    pub fn new<P: AsRef<std::path::Path>>(filename: P, root: String) -> KeyFile {
        let keyfile = glib::KeyFile::new();
        on_err_out!(keyfile.load_from_file(&filename, glib::KeyFileFlags::NONE));
        KeyFile {
            filename: filename.as_ref().to_path_buf(),
            keyfile,
            root,
        }
    }

    /// Save
    fn save(&self) -> Result<(), glib::Error> {
        glib::file_set_contents(&self.filename, self.keyfile.to_data().as_gstr().as_bytes())
    }
}
