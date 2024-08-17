/*
 * niepce - crates/npc-fwk/src/toolkit/map_controller.rs
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

use crate::gtk4;
use shumate::prelude::*;

use super::{Controller, ControllerImplCell, UiController};

pub struct MapController {
    imp_: ControllerImplCell<(), ()>,
    _registry: shumate::MapSourceRegistry,
    map: shumate::SimpleMap,
}

impl Default for MapController {
    fn default() -> Self {
        Self::new()
    }
}

impl Controller for MapController {
    type InMsg = ();
    type OutMsg = ();

    controller_imp_imp!(imp_);
}

impl UiController for MapController {
    fn widget(&self) -> &gtk4::Widget {
        self.map.upcast_ref::<gtk4::Widget>()
    }
}

impl MapController {
    pub fn new() -> MapController {
        let map = shumate::SimpleMap::new();
        map.set_vexpand(true);
        let registry = shumate::MapSourceRegistry::with_defaults();
        map.set_map_source(registry.item(0).and_downcast_ref::<shumate::MapSource>());

        let ctrl = MapController {
            imp_: ControllerImplCell::default(),
            _registry: registry,
            map,
        };

        // Default position. Somewhere over Montréal, QC
        ctrl.set_zoom_level(10.0);
        ctrl.center_on(45.5030854, -73.5698944);

        ctrl
    }

    pub fn center_on(&self, lat: f64, longitude: f64) {
        if let Some(viewport) = self.map.viewport() {
            viewport.set_location(lat, longitude);
        }
    }

    pub fn zoom_in(&self) {
        if let Some(map) = self.map.map() {
            map.zoom_in();
        }
    }

    pub fn zoom_out(&self) {
        if let Some(map) = self.map.map() {
            map.zoom_out();
        }
    }

    pub fn set_zoom_level(&self, level: f64) {
        if let Some(viewport) = self.map.viewport() {
            viewport.set_zoom_level(level);
        }
    }
}
