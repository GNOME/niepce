/*
 * niepce - crates/npc-engine/src/library/preferences.rs
 *
 * Copyright (C) 2024 Hubert Figuière
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

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::libraryclient::{ClientInterface, LibraryClient};
use npc_fwk::dbg_out;
use npc_fwk::toolkit::ConfigBackend;

/// A backend to handle catalog preferences.
/// It is read once and abd write many.
pub struct CatalogPreferences {
    client: Arc<LibraryClient>,
    store: RwLock<HashMap<String, String>>,
}

impl CatalogPreferences {
    pub fn new(client: Arc<LibraryClient>) -> Self {
        Self {
            client,
            store: RwLock::default(),
        }
    }

    /// Normalise the key to always have `prefs.`
    fn normalise_key(key: &str) -> String {
        if key.starts_with("prefs.") {
            key.into()
        } else {
            String::from("prefs.") + key
        }
    }
}

impl ConfigBackend for CatalogPreferences {
    fn start(&self) {
        self.client.get_all_preferences();
    }

    fn initialise(&self, prefs: &[(String, String)]) {
        let mut store = self.store.write().unwrap();
        for pref in prefs {
            store.insert(pref.0.clone(), pref.1.clone());
        }
        dbg_out!("Catalog Preferences initialised");
    }

    /// Return true if it has `key`.
    fn has(&self, key: &str) -> bool {
        self.store
            .read()
            .unwrap()
            .contains_key(&Self::normalise_key(key))
    }

    /// Return the string value for `key` or `None` if not found.
    fn value_opt(&self, key: &str) -> Option<String> {
        self.store
            .read()
            .unwrap()
            .get(&Self::normalise_key(key))
            .cloned()
    }

    /// Return string value for `key`, or `def` if not found.
    fn value(&self, key: &str, def: &str) -> String {
        self.store
            .read()
            .unwrap()
            .get(&Self::normalise_key(key))
            .cloned()
            .unwrap_or_else(|| def.to_string())
    }

    /// Set `value` for `key`
    fn set_value(&self, key: &str, value: &str) {
        let key = Self::normalise_key(key);
        self.store
            .write()
            .unwrap()
            .insert(key.clone(), value.to_string());
        self.client.set_preference(key, value.to_string());
    }
}
