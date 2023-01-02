/*
 * niepce - libraryclient/host.rs
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

use std::rc::Rc;

use crate::ThumbnailCache;
use npc_fwk::base::Moniker;

use super::{LcChannel, LibraryClientWrapper, UIDataProvider};

const THUMBCACHE_DIRNAME: &str = "thumbcache";

/// This host of the element of the library client.
pub struct LibraryClientHost {
    // XXX get rid of the wrapper
    client: LibraryClientWrapper,
    thumbnail_cache: ThumbnailCache,
    ui_provider: std::rc::Rc<UIDataProvider>,
}

unsafe impl cxx::ExternType for LibraryClientHost {
    type Id = cxx::type_id!("eng::LibraryClientHost");
    type Kind = cxx::kind::Opaque;
}

impl LibraryClientHost {
    pub fn new(moniker: &Moniker, channel: &LcChannel) -> LibraryClientHost {
        let path = std::path::PathBuf::from(moniker.path());
        let mut cache_path = path.clone();
        cache_path.push(THUMBCACHE_DIRNAME);

        LibraryClientHost {
            client: LibraryClientWrapper::new(path, channel.clone()),
            thumbnail_cache: ThumbnailCache::new(&cache_path, channel.clone()),
            ui_provider: Rc::new(UIDataProvider::default()),
        }
    }

    pub fn client(&self) -> &LibraryClientWrapper {
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
