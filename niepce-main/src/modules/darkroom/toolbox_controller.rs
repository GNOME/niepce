/*
 * niepce - modules/darkroom/toolbox_controller.rs
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

use gettextrs::gettext as i18n;
use gtk4::prelude::*;
use npc_fwk::gtk4;

use npc_fwk::toolkit::widgets::EditableHScale;
use npc_fwk::toolkit::{Controller, ControllerImplCell, UiController};

use super::dr_item::DrItem;

pub struct ToolboxController {
    imp_: ControllerImplCell<(), ()>,
    _name: &'static str,
    _long_name: String,
    _icon_name: &'static str,
    box_: gtk4::Box,
}

impl UiController for ToolboxController {
    fn widget(&self) -> &gtk4::Widget {
        self.box_.upcast_ref()
    }
}

impl Controller for ToolboxController {
    type InMsg = ();
    type OutMsg = ();

    npc_fwk::controller_imp_imp!(imp_);
}

impl ToolboxController {
    pub fn new() -> ToolboxController {
        let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 8);

        let item = DrItem::new(&i18n("Crop"));
        box_.append(&item);
        let s = EditableHScale::new(-45.0, 45.0, 0.5);
        item.add_widget(&i18n("Tilt"), &s);

        let item = DrItem::new(&i18n("White balance"));
        box_.append(&item);

        let s = EditableHScale::new(0.0, 100.0, 1.0);
        item.add_widget(&i18n("Color temperature"), &s);

        let item = DrItem::new(&i18n("Tone and colour"));
        box_.append(&item);
        let s = EditableHScale::new(-5.0, 5.0, 0.1);
        item.add_widget(&i18n("Exposure"), &s);
        let s = EditableHScale::new(0.0, 100.0, 1.0);
        item.add_widget(&i18n("Recovery"), &s);
        let s = EditableHScale::new(0.0, 100.0, 1.0);
        item.add_widget(&i18n("Fill Light"), &s);
        let s = EditableHScale::new(0.0, 100.0, 1.0);
        item.add_widget(&i18n("Blacks"), &s);
        let s = EditableHScale::new(-100.0, 100.0, 1.0);
        item.add_widget(&i18n("Brightness"), &s);
        let s = EditableHScale::new(-100.0, 100.0, 1.0);
        item.add_widget(&i18n("Contrast"), &s);
        let s = EditableHScale::new(-100.0, 100.0, 1.0);
        item.add_widget(&i18n("Saturation"), &s);
        let s = EditableHScale::new(-100.0, 100.0, 1.0);
        item.add_widget(&i18n("Vibrance"), &s);

        ToolboxController {
            imp_: ControllerImplCell::default(),
            _name: "tools",
            _long_name: i18n("Develop"),
            _icon_name: "apply",
            box_,
        }
    }
}
