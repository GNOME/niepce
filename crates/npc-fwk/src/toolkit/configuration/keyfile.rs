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

use std::sync::RwLock;

use configparser::ini::Ini;

use super::ConfigBackend;

/// A configuration, backed by an ini file.
pub(super) struct KeyFile {
    filename: std::path::PathBuf,
    keyfile: RwLock<Ini>,
    root: String,
}

impl ConfigBackend for KeyFile {
    /// Return true if it has `key`.
    fn has(&self, key: &str) -> bool {
        self.keyfile.read().unwrap().get(&self.root, key).is_some()
    }

    /// Return the string value for `key` or `None` if not found.
    fn value_opt(&self, key: &str) -> Option<String> {
        if !self.has(key) {
            return None;
        }
        self.keyfile
            .read()
            .unwrap()
            .get(&self.root, key)
            .map(|v| v.as_str().to_string())
    }

    /// Return string value for `key`, or `def` if not found.
    fn value(&self, key: &str, def: &str) -> String {
        self.value_opt(key).unwrap_or_else(|| def.to_string())
    }

    /// Set `value` for `key`
    fn set_value(&self, key: &str, value: &str) {
        self.keyfile
            .write()
            .unwrap()
            .set(&self.root, key, Some(value.to_string()));
        on_err_out!(self.save());
    }
}

impl KeyFile {
    pub fn new<P: AsRef<std::path::Path>>(filename: P, root: String) -> KeyFile {
        let mut keyfile = Ini::new();
        on_err_out!(keyfile.load(&filename));
        KeyFile {
            filename: filename.as_ref().to_path_buf(),
            keyfile: RwLock::new(keyfile),
            root,
        }
    }

    /// Save
    fn save(&self) -> std::io::Result<()> {
        self.keyfile.read().unwrap().write(&self.filename)
    }
}
