/*
 * niepce - fwk/toolkit/configuration.rs
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

mod keyfile;

use crate::glib;

/// Config backend
pub trait ConfigBackend: Send + Sync {
    /// Start the backend
    fn start(&self) {}
    fn initialise(&self, _prefs: &[(String, String)]) {}
    /// Return true if it has `key`.
    fn has(&self, key: &str) -> bool;
    /// Return the string value for `key` or `None` if not found.
    fn value_opt(&self, key: &str) -> Option<String>;
    /// Return string value for `key`, or `def` if not found.
    fn value(&self, key: &str, def: &str) -> String;
    /// Set `value` for `key`
    fn set_value(&self, key: &str, value: &str);
}

/// Configuration with modular backend.
pub struct Configuration {
    backend: Box<dyn ConfigBackend>,
}

impl Configuration {
    /// New `KeyFile` configuration from a file.
    pub fn from_file<P: AsRef<std::path::Path>>(file: P) -> Configuration {
        Configuration {
            backend: Box::new(keyfile::KeyFile::new(file, "main".to_string())),
        }
    }

    /// Build a configuration from a backend.
    pub fn from_impl<T: ConfigBackend + 'static>(backend: Box<T>) -> Configuration {
        Configuration { backend }
    }

    pub fn imp(&self) -> &dyn ConfigBackend {
        &*self.backend
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
        self.backend.has(key)
    }

    /// Return string value for `key`, or `def` if not found.
    pub fn value(&self, key: &str, def: &str) -> String {
        self.backend.value(key, def)
    }

    /// Return the string value for `key` or `None` if not found.
    pub fn value_opt(&self, key: &str) -> Option<String> {
        self.backend.value_opt(key)
    }

    /// Set `value` for `key`
    pub fn set_value(&self, key: &str, value: &str) {
        self.backend.set_value(key, value)
    }

    /// Set value to a switchrow
    pub fn to_switchrow(&self, checkbox: &adw::SwitchRow, key: &str, def: &str) {
        let value = self.value(key, def);
        checkbox.set_active(value == "1");
    }

    /// Set value from a switchrow
    pub fn from_switchrow(&self, checkbox: &adw::SwitchRow, key: &str) {
        self.set_value(key, if checkbox.is_active() { "1" } else { "0" });
    }
}

#[cfg(test)]
mod tests {

    use super::Configuration;

    #[test]
    fn test_configuration_keyfile() {
        let tmpdir = tempfile::tempdir().expect("Tmp directory failed");

        let mut test_file = tmpdir.path().to_path_buf();
        test_file.push("test_file.ini");

        assert!(!test_file.exists());
        {
            let cfg = Configuration::from_file(&test_file);
            assert!(!test_file.exists());

            assert!(!cfg.has("foobar"));
            assert_eq!(cfg.value("foobar", "some_default"), "some_default");

            cfg.set_value("foobar", "some_value");
            assert!(test_file.exists());

            assert!(cfg.has("foobar"));
            assert_eq!(cfg.value("foobar", "some_default"), "some_value");
        }

        // Test we can read it back.
        assert!(test_file.exists());
        {
            let cfg = Configuration::from_file(&test_file);
            assert!(cfg.has("foobar"));
            assert_eq!(cfg.value("foobar", "some_default"), "some_value");
        }
    }
}
