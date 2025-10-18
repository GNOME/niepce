/*
 * niepce - npc-python/src/editor.rs
 *
 * Copyright (C) 2025 Hubert Figuière
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

use npc_fwk::adw;
use npc_fwk::gtk4;
use sourceview5::prelude::*;

use npc_fwk::controller_imp_imp;
use npc_fwk::toolkit::{Controller, ControllerImplCell, DialogController, UiController};

pub enum Event {
    Run,
    Close,
}

pub struct Editor {
    imp_: ControllerImplCell<Event, ()>,
    window: adw::Window,
}

impl Controller for Editor {
    type InMsg = Event;
    type OutMsg = ();

    controller_imp_imp!(imp_);
}

impl UiController for Editor {
    fn widget(&self) -> &npc_fwk::gtk4::Widget {
        self.dialog().upcast_ref()
    }
}

impl DialogController for Editor {
    fn dialog(&self) -> &adw::Window {
        &self.window
    }
}

impl Editor {
    pub fn new() -> Rc<Self> {
        let builder = gtk4::Builder::from_resource("/net/figuiere/npc-python/ui/editor.ui");
        get_widget!(builder, adw::Window, window);
        get_widget!(builder, sourceview5::View, editor);
        let buffer = sourceview5::Buffer::new(None);
        buffer.set_highlight_syntax(true);
        buffer.set_language(
            sourceview5::LanguageManager::new()
                .language("python")
                .as_ref(),
        );
        buffer.set_style_scheme(
            sourceview5::StyleSchemeManager::new()
                .scheme("solarized-light")
                .as_ref(),
        );
        editor.set_buffer(Some(&buffer));
        window.set_default_size(500, 500);
        let editor = Rc::new(Self {
            imp_: ControllerImplCell::default(),
            window,
        });

        <Self as DialogController>::start(&editor);

        editor
    }
}
