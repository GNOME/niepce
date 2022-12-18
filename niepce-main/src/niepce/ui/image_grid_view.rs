/*
 * niepce - niepce/ui/image_grid_view.rs
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

pub use super::image_list_item::ImageListItem;

use std::rc::Rc;

use glib::translate::*;
use gtk4::prelude::*;

use super::library_cell_renderer::LibraryCellRenderer;
use npc_engine::db;
use npc_engine::libraryclient::{LibraryClientHost, UIDataProvider};
use npc_fwk::base::Signal;

pub struct ImageGridView {
    grid_view: gtk4::GridView,
    signal_rating_changed: Rc<Signal<(db::LibraryId, i32)>>,
}

impl ImageGridView {
    pub fn new(
        store: &gtk4::SingleSelection,
        context_menu: Option<gtk4::PopoverMenu>,
        ui_provider: Option<Rc<UIDataProvider>>,
    ) -> Self {
        let factory = gtk4::SignalListItemFactory::new();
        let grid_view = gtk4::GridView::new(Some(store), Some(&factory));
        let signal_rating_changed = Rc::new(Signal::default());
        let weak_signal = Rc::downgrade(&signal_rating_changed);

        let ui_provider = ui_provider.map(|v| Rc::downgrade(&v));
        factory.connect_setup(move |_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let renderer = LibraryCellRenderer::new(ui_provider.clone());
            let weak_signal = weak_signal.clone();
            renderer.connect_local("rating-changed", false, move |values| {
                if let Some(signal) = weak_signal.upgrade() {
                    signal.emit((values[1].get().unwrap(), values[2].get().unwrap()));
                }
                None
            });

            item.set_child(Some(&renderer));
        });

        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let image_item = item.item().unwrap().downcast::<ImageListItem>().unwrap();
            let renderer = item
                .child()
                .unwrap()
                .downcast::<LibraryCellRenderer>()
                .unwrap();
            renderer.set_property("pixbuf", image_item.thumbnail());
            renderer.set_property("libfile", image_item.file());
            renderer.set_property("status", image_item.status() as i32);
        });

        // Context menu
        let click = gtk4::GestureClick::new();
        click.set_button(0);
        click.connect_pressed(
            glib::clone!(@weak grid_view, @strong context_menu => move |gesture, _, x, y| {
                Self::press_event(&grid_view, &context_menu, gesture, x, y);
            }),
        );
        grid_view.add_controller(&click);

        ImageGridView {
            grid_view,
            signal_rating_changed,
        }
    }

    pub fn add_rating_listener(&self, listener: cxx::UniquePtr<crate::ffi::RatingClickListener>) {
        self.signal_rating_changed.connect(move |(id, rating)| {
            listener.call(id, rating);
        });
    }
}

impl std::ops::Deref for ImageGridView {
    type Target = gtk4::GridView;

    fn deref(&self) -> &gtk4::GridView {
        &self.grid_view
    }
}

impl ImageGridView {
    fn press_event(
        _grid_view: &gtk4::GridView,
        menu: &Option<gtk4::PopoverMenu>,
        gesture: &gtk4::GestureClick,
        x: f64,
        y: f64,
    ) {
        if let Some(event) = gesture.last_event(None) {
            if event.triggers_context_menu() {
                if let Some(ref menu) = menu {
                    menu.set_pointing_to(Some(&gdk4::Rectangle::new(x as i32, y as i32, 0, 0)));
                    menu.popup();
                }
            }
        }
    }

    // cxx
    pub fn get_grid_view(&self) -> *mut crate::ffi::GtkGridView {
        let grid_view: *mut gtk4_sys::GtkGridView = self.grid_view.to_glib_none().0;
        grid_view as *mut crate::ffi::GtkGridView
    }
}

/// Create a new `ImageGridView`
///
/// # Safety
/// Use raw pointers.
///
/// The `store` and `context_menu` will get ref.
/// context_menu can be `nullptr`
pub unsafe fn npc_image_grid_view_new(
    store: *mut crate::ffi::GtkSingleSelection,
    context_menu: *mut crate::ffi::GtkPopoverMenu,
    libclient_host: &LibraryClientHost,
) -> Box<ImageGridView> {
    Box::new(ImageGridView::new(
        &gtk4::SingleSelection::from_glib_none(store as *mut gtk4_sys::GtkSingleSelection),
        Option::<gtk4::PopoverMenu>::from_glib_none(context_menu as *mut gtk4_sys::GtkPopoverMenu),
        Some(libclient_host.shared_ui_provider()),
    ))
}

/// Create a new `ImageGridView`
///
/// # Safety
/// Use raw pointers.
///
/// The `store` will get ref.
pub unsafe fn npc_image_grid_view_new2(
    store: *mut crate::ffi::GtkSingleSelection,
) -> Box<ImageGridView> {
    Box::new(ImageGridView::new(
        &gtk4::SingleSelection::from_glib_none(store as *mut gtk4_sys::GtkSingleSelection),
        None,
        None,
    ))
}
