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

use glib::subclass::prelude::*;
use glib::translate::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

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

glib::wrapper! {
    pub struct ThumbStripView(
        ObjectSubclass<ThumbStripViewPriv>)
        @extends gtk::IconView, gtk::Container, gtk::Widget;
}

impl ThumbStripView {
    pub fn new(store: &gtk::TreeModel) -> Self {
        glib::Object::new(&[
            ("model", store),
            ("item-height", &THUMB_STRIP_VIEW_DEFAULT_ITEM_HEIGHT),
            ("selection-mode", &gtk::SelectionMode::Multiple),
            ("column-spacing", &THUMB_STRIP_VIEW_SPACING),
            ("row-spacing", &THUMB_STRIP_VIEW_SPACING),
            ("margin", &0),
        ])
        .expect("Failed to create ThumbStripView")
    }
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

    fn row_added(&self, view: &ThumbStripView) {
        self.count.replace(self.count.get() + 1);
        self.update(view);
    }

    fn row_deleted(&self, view: &ThumbStripView) {
        let count = self.count.get();
        if count > 0 {
            self.count.replace(count + 1);
        }
        self.update(view);
    }

    fn update(&self, view: &ThumbStripView) {
        view.set_columns(self.count.get());
    }
}

pub struct ThumbStripViewPriv {
    item_height: Cell<i32>,
    item_count: Rc<ItemCount>,
    renderer: OnceCell<LibraryCellRenderer>,
    store: RefCell<Option<gtk::TreeModel>>,
    signals: RefCell<Signals>,
}

pub trait ThumbStripViewExt {
    fn set_item_height(&self, height: i32);
    fn set_model(&self, model: Option<gtk::TreeModel>);
}

impl ThumbStripViewExt for ThumbStripView {
    fn set_item_height(&self, height: i32) {
        let priv_ = ThumbStripViewPriv::from_instance(self);
        priv_.set_item_height(height);
    }

    fn set_model(&self, model: Option<gtk::TreeModel>) {
        let priv_ = ThumbStripViewPriv::from_instance(self);
        priv_.set_model(model);
    }
}

impl ThumbStripViewPriv {
    fn set_item_height(&self, height: i32) {
        self.item_height.set(height);
        if let Some(renderer) = self.renderer.get() {
            renderer.set_height(height);
        }
    }

    fn set_model(&self, model: Option<gtk::TreeModel>) {
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
        ThumbStripViewExt::set_model(&self.instance(), model);
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

            let view = self.instance();
            // update item count
            self.item_count.update(&view);

            let mut signals = self.signals.borrow_mut();
            signals.model_add = Some(store.connect_row_inserted(
                clone!(@strong self.item_count as item_count, @weak view => move |_,_,_| {
                    item_count.row_added(&view);
                }),
            ));
            signals.model_remove = Some(store.connect_row_deleted(
                clone!(@strong self.item_count as item_count, @weak view => move |_,_| {
                    item_count.row_deleted(&view);
                }),
            ));
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ThumbStripViewPriv {
    const NAME: &'static str = "ThumbStripView";
    type Type = ThumbStripView;
    type ParentType = gtk::IconView;

    fn new() -> Self {
        Self {
            item_height: Cell::new(THUMB_STRIP_VIEW_DEFAULT_ITEM_HEIGHT),
            item_count: Rc::new(ItemCount {
                count: Cell::new(0),
            }),
            renderer: OnceCell::new(),
            store: RefCell::new(None),
            signals: RefCell::new(Signals::default()),
        }
    }
}

impl ObjectImpl for ThumbStripViewPriv {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        let cell_renderer = LibraryCellRenderer::new_thumb_renderer();

        let icon_view_self = obj.clone().upcast::<gtk::IconView>();
        icon_view_self.pack_start(&cell_renderer, false);
        cell_renderer.set_height(100);
        cell_renderer.set_yalign(0.5);
        cell_renderer.set_xalign(0.5);

        icon_view_self.add_attribute(
            &cell_renderer,
            "pixbuf",
            ImageListStoreColIndex::StripThumbIndex as i32,
        );
        icon_view_self.add_attribute(
            &cell_renderer,
            "libfile",
            ImageListStoreColIndex::FileIndex as i32,
        );
        icon_view_self.add_attribute(
            &cell_renderer,
            "status",
            ImageListStoreColIndex::FileStatusIndex as i32,
        );
        self.renderer
            .set(cell_renderer)
            .expect("ThumbStripView::constructed set cell render failed.");
        let model = icon_view_self.model();

        self.setup_model();
        self.store.replace(model);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![glib::ParamSpecInt::new(
                "item-height",
                "Item Height",
                "The Item Height",
                -1,
                100,
                THUMB_STRIP_VIEW_DEFAULT_ITEM_HEIGHT, // Default value
                glib::ParamFlags::READWRITE,
            )]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(
        &self,
        _obj: &ThumbStripView,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "item-height" => {
                let height: i32 = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.set_item_height(height);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &ThumbStripView, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "item-height" => self.item_height.get().to_value(),
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for ThumbStripViewPriv {}

impl ContainerImpl for ThumbStripViewPriv {}

impl IconViewImpl for ThumbStripViewPriv {}

/// # Safety
/// Use raw pointers
#[no_mangle]
pub unsafe extern "C" fn npc_thumb_strip_view_new(
    store: *mut gtk_sys::GtkTreeModel,
) -> *mut gtk_sys::GtkWidget {
    ThumbStripView::new(&gtk::TreeModel::from_glib_full(store))
        .upcast::<gtk::Widget>()
        .to_glib_full()
}
