/*
 * niepce - niepce/ui/film_strip_controller.rs
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

use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use gtk4::prelude::*;
use once_cell::unsync::OnceCell;

use npc_engine::db;
use npc_fwk::toolkit::{Controller, ControllerImpl, UiController};

use super::image_list_store::ImageListStore;
use super::thumb_nav::{ThumbNav, ThumbNavMode};
use super::thumb_strip_view::ThumbStripView;

struct Widgets {
    widget_: gtk4::Widget,
    thumb_nav: ThumbNav,
    thumb_strip_view: ThumbStripView,
}

pub struct FilmStripController {
    imp_: RefCell<ControllerImpl>,

    widgets: OnceCell<Widgets>,
    store: Rc<ImageListStore>,
}

impl Controller for FilmStripController {
    fn imp(&self) -> Ref<'_, ControllerImpl> {
        self.imp_.borrow()
    }

    fn imp_mut(&self) -> RefMut<'_, ControllerImpl> {
        self.imp_.borrow_mut()
    }

    fn on_ready(&self) {}
}

impl UiController for FilmStripController {
    fn widget(&self) -> &gtk4::Widget {
        &self
            .widgets
            .get_or_init(|| {
                let thumb_strip_view = ThumbStripView::new(self.store.liststore().as_ref());
                thumb_strip_view.set_item_height(120);

                let thumb_nav = ThumbNav::new(&thumb_strip_view, ThumbNavMode::OneRow, true);
                thumb_strip_view.set_selection_mode(gtk4::SelectionMode::Single);
                thumb_strip_view.set_hexpand(true);
                thumb_nav.set_size_request(-1, 134);
                thumb_nav.set_hexpand(true);

                Widgets {
                    widget_: thumb_nav.clone().upcast(),
                    thumb_nav,
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
            imp_: RefCell::new(ControllerImpl::default()),
            widgets: OnceCell::new(),
            store,
        })
    }

    fn selected(&self) -> db::LibraryId {
        if let Some(widgets) = self.widgets.get() {
            let paths = widgets.thumb_strip_view.selected_items();
            if paths.is_empty() {
                return 0;
            }

            return self.store.get_file_id_at_path(&paths[0]);
        }

        0
    }
}
