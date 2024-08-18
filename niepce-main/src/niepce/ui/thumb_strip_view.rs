/*
 * niepce - niepce/ui/thumbstripview.rs
 *
 * Copyright (C) 2020-2024 Hubert Figuière
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

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use npc_fwk::{glib, gtk4};

use crate::niepce::ui::image_list_item::ImageListItem;
use crate::niepce::ui::library_cell_renderer::LibraryCellRenderer;
use npc_fwk::{dbg_out, err_out};

const THUMB_STRIP_VIEW_DEFAULT_ITEM_HEIGHT: i32 = 0;

#[derive(Default)]
struct Signals {
    model_changed: Option<glib::SignalHandlerId>,
}

struct ItemCount {
    count: Cell<u32>,
}

impl ItemCount {
    fn set(&self, count: u32) {
        self.count.set(count);
    }

    fn changed(&self, view: &gtk4::GridView, change: i32) {
        let mut count = self.count.get() as i64;
        count += change as i64;
        if count < 0 {
            err_out!("Count is negative");
            count = 0;
        }
        self.count.replace(count as u32);
        self.update(view);
    }

    fn update(&self, view: &gtk4::GridView) {
        view.set_min_columns(std::cmp::max(1, self.count.get()));
    }
}

pub struct ThumbStripView {
    item_height: Cell<i32>,
    item_count: Rc<ItemCount>,
    grid_view: gtk4::GridView,
    store: RefCell<Option<gtk4::SingleSelection>>,
    signals: RefCell<Signals>,
}

impl std::ops::Deref for ThumbStripView {
    type Target = gtk4::GridView;

    fn deref(&self) -> &gtk4::GridView {
        &self.grid_view
    }
}

impl ThumbStripView {
    pub fn new(store: gtk4::SingleSelection) -> Self {
        let factory = gtk4::SignalListItemFactory::new();
        factory.connect_setup(move |_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let renderer = LibraryCellRenderer::new_thumb_renderer();
            item.set_child(Some(&renderer));
        });

        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let image_item = item.item().and_downcast::<ImageListItem>().unwrap();
            let renderer = item.child().and_downcast::<LibraryCellRenderer>().unwrap();
            image_item
                .bind_property("thumbnail", &renderer, "pixbuf")
                .sync_create()
                .build();
            image_item
                .bind_property("file", &renderer, "libfile")
                .sync_create()
                .build();
            image_item
                .bind_property("file-status", &renderer, "status")
                .sync_create()
                .build();
        });

        let tsv = Self {
            item_height: Cell::new(THUMB_STRIP_VIEW_DEFAULT_ITEM_HEIGHT),
            item_count: Rc::new(ItemCount {
                count: Cell::new(0),
            }),
            grid_view: gtk4::GridView::new(Some(store.clone()), Some(factory)),
            store: RefCell::new(Some(store)),
            signals: RefCell::new(Signals::default()),
        };

        // ideally this should be the max, but `std::u32::MAX` is too much
        tsv.grid_view.set_max_columns(100000);
        tsv.setup_model();

        tsv
    }

    pub fn set_item_height(&self, height: i32) {
        self.item_height.set(height);
        dbg_out!("set_item_height {}", height);
    }

    pub fn set_model(&self, model: Option<gtk4::SingleSelection>) {
        if let Some(store) = &*self.store.borrow() {
            let mut signals = self.signals.borrow_mut();
            if signals.model_changed.is_some() {
                glib::signal_handler_disconnect(store, signals.model_changed.take().unwrap());
            }
        }

        self.store.replace(model);
        self.setup_model();
    }

    fn setup_model(&self) {
        if let Some(store) = &*self.store.borrow() {
            let count = store.n_items();
            self.item_count.set(count);

            // update item count
            self.item_count.update(self);

            let mut signals = self.signals.borrow_mut();
            let item_count = self.item_count.clone();
            let view = self.grid_view.clone();
            signals.model_changed = Some(store.connect_items_changed(glib::clone!(
                #[strong]
                item_count,
                #[weak]
                view,
                move |_, _, removed, added| {
                    let changed: i32 = added as i32 - removed as i32;
                    item_count.changed(&view, changed);
                }
            )));
        }
    }
}
