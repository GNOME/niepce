/*
 * niepce - niepce/modules/map.rs
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

use std::cell::Cell;
use std::rc::Rc;

use gtk4::prelude::*;
use npc_fwk::gtk4;

use crate::niepce::ui::LibraryModule;
use npc_engine::catalog::NiepceProperties as Np;
use npc_engine::catalog::NiepcePropertyIdx as Npi;
use npc_engine::library::notification::LibNotification;
use npc_fwk::dbg_out;
use npc_fwk::toolkit::{Controller, ControllerImplCell, MapController, UiController};

pub struct MapModule {
    imp_: ControllerImplCell<(), ()>,
    map: MapController,
    active: Cell<bool>,
    widget: gtk4::Box,
}

impl Controller for MapModule {
    type InMsg = ();
    type OutMsg = ();

    npc_fwk::controller_imp_imp!(imp_);
}

impl UiController for MapModule {
    fn widget(&self) -> &gtk4::Widget {
        self.widget.upcast_ref::<gtk4::Widget>()
    }
}

impl LibraryModule for MapModule {
    fn set_active(&self, active: bool) {
        self.active.set(active);
    }

    fn widget(&self) -> &gtk4::Widget {
        UiController::widget(self)
    }
}

impl MapModule {
    pub fn new() -> Rc<Self> {
        let widget = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let mut module = Self {
            imp_: ControllerImplCell::default(),
            widget,
            map: MapController::new(),
            active: Cell::new(false),
        };

        module.build_widget();

        Rc::new(module)
    }

    fn build_widget(&mut self) {
        let map_widget = self.map.widget();
        self.widget.append(map_widget);
    }

    pub fn on_lib_notification(&self, ln: &LibNotification) {
        if !self.active.get() {
            return;
        }
        if let LibNotification::MetadataQueried(lm) = ln {
            dbg_out!("received metadata in MapModule");

            let mut propset = npc_fwk::PropertySet::new();
            propset.insert(Np::Index(Npi::NpExifGpsLongProp));
            propset.insert(Np::Index(Npi::NpExifGpsLatProp));

            let properties = lm.to_properties(&propset);
            if let Some(longitude) = properties
                .get(&Np::Index(Npi::NpExifGpsLongProp))
                .and_then(|v| v.string())
                .and_then(npc_fwk::gps_coord_from_xmp)
            {
                if let Some(latitude) = properties
                    .get(&Np::Index(Npi::NpExifGpsLatProp))
                    .and_then(|v| v.string())
                    .and_then(npc_fwk::gps_coord_from_xmp)
                {
                    self.map.center_on(latitude, longitude);
                }
            }
        }
    }
}
