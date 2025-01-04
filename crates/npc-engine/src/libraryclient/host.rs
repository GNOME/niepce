/*
 * niepce - libraryclient/host.rs
 *
 * Copyright (C) 2022-2025 Hubert Figuière
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

use std::rc::Rc;
use std::sync::Arc;

use crate::ThumbnailCache;
use npc_fwk::base::Moniker;

use super::{LcChannel, LibraryClient, UIDataProvider};

const LEGACY_DATABASENAME: &str = "niepcelibrary.db";

/// This host of the element of the library client.
pub struct LibraryClientHost {
    notif_sender: LcChannel,
    client: Arc<LibraryClient>,
    thumbnail_cache: ThumbnailCache,
    ui_provider: std::rc::Rc<UIDataProvider>,
}

impl LibraryClientHost {
    pub fn new(moniker: &Moniker, channel: &LcChannel) -> LibraryClientHost {
        let mut path = std::path::PathBuf::from(moniker.path());
        // This can't be a directory. Try to get the legacy path.
        if path.is_dir() {
            path = path.join(LEGACY_DATABASENAME);
        }
        // XXX have a more recoverable check.
        // XXX what if path doesn't exist
        assert!(!path.is_dir());

        // XXX these unwrap should not be fatal.
        let cache_path = ThumbnailCache::path_from_catalog(&path).unwrap();

        LibraryClientHost {
            notif_sender: channel.clone(),
            client: Arc::new(LibraryClient::new(path, channel.clone())),
            thumbnail_cache: ThumbnailCache::new(&cache_path, channel.clone()),
            ui_provider: Rc::new(UIDataProvider::default()),
        }
    }

    pub fn close(&self) {
        self.thumbnail_cache.close();
        self.client.close();
    }

    pub fn notif_sender(&self) -> &LcChannel {
        &self.notif_sender
    }

    pub fn client(&self) -> &Arc<LibraryClient> {
        &self.client
    }

    pub fn thumbnail_cache(&self) -> &ThumbnailCache {
        &self.thumbnail_cache
    }

    pub fn ui_provider(&self) -> &UIDataProvider {
        self.ui_provider.as_ref()
    }

    /// If you need a Rc.
    // XXX figure out which one should be prefered.
    pub fn shared_ui_provider(&self) -> Rc<UIDataProvider> {
        self.ui_provider.clone()
    }
}
