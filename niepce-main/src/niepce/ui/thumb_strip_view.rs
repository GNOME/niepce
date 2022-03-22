/*
 * niepce - niepce/ui/thumbstripview.rs
 *
 * Copyright (C) 2020-2022 Hubert Figuière
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

use once_cell::unsync::OnceCell;

use glib::translate::*;
use gtk4::prelude::*;

use crate::niepce::ui::library_cell_renderer::LibraryCellRenderer;

const THUMB_STRIP_VIEW_DEFAULT_ITEM_HEIGHT: i32 = 0;
const THUMB_STRIP_VIEW_SPACING: i32 = 0;

#[repr(i32)]
pub enum ImageListStoreColIndex {
    ThumbIndex = 0,
    FileIndex = 1,
    StripThumbIndex = 2,
    FileStatusIndex = 3,
}

#[derive(Default)]
struct Signals {
    model_add: Option<glib::SignalHandlerId>,
    model_remove: Option<glib::SignalHandlerId>,
}

struct ItemCount {
    count: Cell<i32>,
}

impl ItemCount {
    fn set(&self, count: i32) {
        self.count.set(count);
    }

    fn row_added(&self, view: &gtk4::IconView) {
        self.count.replace(self.count.get() + 1);
        self.update(view);
    }

    fn row_deleted(&self, view: &gtk4::IconView) {
        let count = self.count.get();
        if count > 0 {
            self.count.replace(count + 1);
        }
        self.update(view);
    }

    fn update(&self, view: &gtk4::IconView) {
        view.set_columns(self.count.get());
    }
}

pub struct ThumbStripView {
    item_height: Cell<i32>,
    item_count: Rc<ItemCount>,
    icon_view: gtk4::IconView,
    renderer: OnceCell<LibraryCellRenderer>,
    store: RefCell<Option<gtk4::TreeModel>>,
    signals: RefCell<Signals>,
}

impl std::ops::Deref for ThumbStripView {
    type Target = gtk4::IconView;

    fn deref(&self) -> &gtk4::IconView {
        &self.icon_view
    }
}

impl ThumbStripView {
    pub fn new(store: &gtk4::TreeModel) -> Self {
        let tsv = Self {
            item_height: Cell::new(THUMB_STRIP_VIEW_DEFAULT_ITEM_HEIGHT),
            item_count: Rc::new(ItemCount {
                count: Cell::new(0),
            }),
            icon_view: gtk4::IconView::with_model(store),
            renderer: OnceCell::new(),
            store: RefCell::new(Some(store.clone())),
            signals: RefCell::new(Signals::default()),
        };

        let cell_renderer = LibraryCellRenderer::new_thumb_renderer();

        tsv.icon_view.pack_start(&cell_renderer, false);
        cell_renderer.set_height(100);
        cell_renderer.set_yalign(0.5);
        cell_renderer.set_xalign(0.5);

        tsv.icon_view
            .set_selection_mode(gtk4::SelectionMode::Multiple);
        tsv.icon_view.set_column_spacing(THUMB_STRIP_VIEW_SPACING);
        tsv.icon_view.set_row_spacing(THUMB_STRIP_VIEW_SPACING);
        tsv.icon_view.set_margin(0);
        tsv.icon_view.add_attribute(
            &cell_renderer,
            "pixbuf",
            ImageListStoreColIndex::StripThumbIndex as i32,
        );
        tsv.icon_view.add_attribute(
            &cell_renderer,
            "libfile",
            ImageListStoreColIndex::FileIndex as i32,
        );
        tsv.icon_view.add_attribute(
            &cell_renderer,
            "status",
            ImageListStoreColIndex::FileStatusIndex as i32,
        );
        tsv.renderer
            .set(cell_renderer)
            .expect("ThumbStripView::constructed set cell render failed.");

        tsv.setup_model();

        tsv
    }

    fn set_item_height(&self, height: i32) {
        self.item_height.set(height);
        if let Some(renderer) = self.renderer.get() {
            renderer.set_height(height);
        }
    }

    fn set_model(&self, model: Option<gtk4::TreeModel>) {
        if let Some(store) = &*self.store.borrow() {
            let mut signals = self.signals.borrow_mut();
            if signals.model_add.is_some() {
                glib::signal_handler_disconnect(store, signals.model_add.take().unwrap());
            }
            if signals.model_remove.is_some() {
                glib::signal_handler_disconnect(store, signals.model_remove.take().unwrap());
            }
        }

        self.store.replace(model.clone());
        self.setup_model();
    }

    fn setup_model(&self) {
        if let Some(store) = &*self.store.borrow() {
            // model item count
            let iter = store.iter_first();
            let count = if let Some(ref iter) = iter {
                let mut c = 0;
                while store.iter_next(iter) {
                    c += 1;
                }
                c
            } else {
                0
            };
            self.item_count.set(count);

            // update item count
            self.item_count.update(self);

            let mut signals = self.signals.borrow_mut();
            let item_count = self.item_count.clone();
            let view = self.icon_view.clone();
            signals.model_add = Some(store.connect_row_inserted(
                glib::clone!(@strong item_count, @weak view => move |_,_,_| {
                    item_count.row_added(&view);
                }),
            ));
            signals.model_remove = Some(store.connect_row_deleted(
                glib::clone!(@strong item_count, @weak view => move |_,_| {
                    item_count.row_deleted(&view);
                }),
            ));
        }
    }
}

/// # Safety
/// Use raw pointers
#[no_mangle]
pub unsafe extern "C" fn npc_thumb_strip_view_new(
    store: *mut gtk4_sys::GtkTreeModel,
) -> *mut ThumbStripView {
    Box::into_raw(Box::new(ThumbStripView::new(
        &gtk4::TreeModel::from_glib_full(store),
    )))
}

/// # Safety
/// Use raw pointers
#[no_mangle]
pub unsafe extern "C" fn npc_thumb_strip_view_get_icon_view(
    stripview: &ThumbStripView,
) -> *mut gtk4_sys::GtkIconView {
    stripview.icon_view.to_glib_none().0
}

/// # Safety
/// Use raw pointers
#[no_mangle]
pub unsafe extern "C" fn npc_thumb_strip_view_release(stripview: *mut ThumbStripView) {
    Box::from_raw(stripview);
}
