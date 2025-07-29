/*
 * niepce - niepce/ui/film_strip_controller.rs
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

use std::cell::OnceCell;
use std::rc::Rc;

use gtk4::prelude::*;
use npc_fwk::{gio, gtk4};

use npc_fwk::toolkit::{Controller, ControllerImplCell, UiController};

use super::image_list_store::ImageListStore;
use super::thumb_nav::{ThumbNav, ThumbNavMode};
use super::thumb_strip_view::ThumbStripView;

struct Widgets {
    widget_: gtk4::Widget,
    _thumb_nav: ThumbNav,
    thumb_strip_view: ThumbStripView,
}

pub struct FilmStripController {
    imp_: ControllerImplCell<(), ()>,

    widgets: OnceCell<Widgets>,
    store: Rc<ImageListStore>,
}

impl Controller for FilmStripController {
    type InMsg = ();
    type OutMsg = ();

    npc_fwk::controller_imp_imp!(imp_);
}

impl UiController for FilmStripController {
    fn widget(&self) -> &gtk4::Widget {
        &self
            .widgets
            .get_or_init(|| {
                let thumb_strip_view = ThumbStripView::new(self.store.selection_model().clone());
                thumb_strip_view.set_item_height(120);

                let thumb_nav = ThumbNav::new(&thumb_strip_view, ThumbNavMode::OneRow, true);
                thumb_strip_view.set_hexpand(true);
                thumb_nav.set_size_request(-1, 134);
                thumb_nav.set_hexpand(true);

                Widgets {
                    widget_: thumb_nav.clone().upcast(),
                    _thumb_nav: thumb_nav,
                    thumb_strip_view,
                }
            })
            .widget_
    }

    fn actions(&self) -> Option<(&str, &gio::ActionGroup)> {
        None
    }
}

impl FilmStripController {
    pub fn new(store: Rc<ImageListStore>) -> Rc<FilmStripController> {
        Rc::new(FilmStripController {
            imp_: ControllerImplCell::default(),
            widgets: OnceCell::new(),
            store,
        })
    }

    pub fn grid_view(&self) -> gtk4::GridView {
        let _ = self.widget();
        self.widgets.get().unwrap().thumb_strip_view.clone()
    }
}
