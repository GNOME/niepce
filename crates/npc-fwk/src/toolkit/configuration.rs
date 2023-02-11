/*
 * niepce - fwk/toolkit/configuration.rs
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

use gtk4::glib;

/// A configuration, backed by a `glib::Keyfile`
pub struct Configuration {
    filename: std::path::PathBuf,
    keyfile: glib::KeyFile,
    root: String,
}

impl Configuration {
    /// New configuration from a file.
    pub fn from_file<P: AsRef<std::path::Path>>(file: P) -> Configuration {
        let keyfile = glib::KeyFile::new();
        on_err_out!(keyfile.load_from_file(&file, glib::KeyFileFlags::NONE));
        Configuration {
            filename: file.as_ref().to_path_buf(),
            keyfile,
            root: "main".to_string(),
        }
    }

    /// New XDG compliant config path from `app_name`.
    pub fn make_config_path(app_name: &str) -> std::path::PathBuf {
        let mut filename = glib::user_config_dir();
        filename.push(app_name);
        on_err_out!(std::fs::create_dir_all(&filename));
        filename.push("config");

        filename
    }

    /// Return true if it has `key`.
    pub fn has(&self, key: &str) -> bool {
        self.keyfile.has_group(&self.root) && self.keyfile.has_key(&self.root, key).unwrap_or(false)
    }

    /// Return string value for `key`, or `def` if not found.
    pub fn value(&self, key: &str, def: &str) -> String {
        self.value_opt(key).unwrap_or_else(|| def.to_string())
    }

    /// Return the string value for `key` or `None` if not found.
    pub fn value_opt(&self, key: &str) -> Option<String> {
        if !self.has(key) {
            return None;
        }
        self.keyfile
            .string(&self.root, key)
            .map(|v| v.as_str().to_string())
            .ok()
    }

    /// Set `value` for `key`
    pub fn set_value(&self, key: &str, value: &str) {
        self.keyfile.set_string(&self.root, key, value);
        on_err_out!(self.save());
    }

    /// Save
    fn save(&self) -> Result<(), glib::Error> {
        glib::file_set_contents(&self.filename, self.keyfile.to_data().as_gstr().as_bytes())
    }
}

#[cfg(test)]
mod tests {

    use super::Configuration;

    #[test]
    fn test_configuration() {
        let tmpdir = tempfile::tempdir().expect("Tmp directory failed");

        let mut test_file = tmpdir.path().to_path_buf();
        test_file.push("test_file.ini");

        assert!(!test_file.exists());
        let cfg = Configuration::from_file(&test_file);
        assert!(!test_file.exists());

        assert!(!cfg.has("foobar"));
        assert_eq!(cfg.value("foobar", "some_default"), "some_default");

        cfg.set_value("foobar", "some_value");
        assert!(test_file.exists());

        assert!(cfg.has("foobar"));
        assert_eq!(cfg.value("foobar", "some_default"), "some_value");
    }
}
