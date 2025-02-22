/*
 * niepce - niepce/ui/image_grid_view.rs
 *
 * Copyright (C) 2020-2025 Hubert Figuière
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

use gtk4::prelude::*;
use npc_fwk::{gdk4, glib, gtk4};

use super::library_cell_renderer::LibraryCellRenderer;
use npc_engine::catalog;
use npc_engine::libraryclient::UIDataProvider;
use npc_fwk::base::Signal;
use npc_fwk::toolkit::ListViewRow;

pub struct ImageGridView {
    grid_view: gtk4::GridView,
    signal_rating_changed: Rc<Signal<(catalog::LibraryId, i32)>>,
}

impl ImageGridView {
    pub fn new(
        store: gtk4::SingleSelection,
        context_menu: Option<gtk4::PopoverMenu>,
        ui_provider: Option<Rc<UIDataProvider>>,
    ) -> Self {
        let factory = gtk4::SignalListItemFactory::new();
        let grid_view = gtk4::GridView::new(Some(store), Some(factory.clone()));
        let signal_rating_changed = Rc::new(Signal::default());
        let weak_signal = Rc::downgrade(&signal_rating_changed);

        let ui_provider = ui_provider.map(|v| Rc::downgrade(&v));
        factory.connect_setup(move |_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let renderer = LibraryCellRenderer::new(ui_provider.clone());
            let weak_signal = weak_signal.clone();
            renderer.connect_closure(
                "rating-changed",
                false,
                glib::closure_local!(move |_: &LibraryCellRenderer, id, rating| {
                    if let Some(signal) = weak_signal.upgrade() {
                        signal.emit((id, rating));
                    }
                }),
            );

            item.set_child(Some(&renderer));
        });

        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let image_item = item.item().and_downcast::<ImageListItem>().unwrap();
            let renderer = item.child().and_downcast::<LibraryCellRenderer>().unwrap();
            renderer.bind(&image_item, None);
        });

        factory.connect_unbind(move |_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let renderer = item.child().and_downcast::<LibraryCellRenderer>().unwrap();
            renderer.unbind();
        });

        // Context menu
        let click = gtk4::GestureClick::new();
        click.set_button(0);
        click.connect_pressed(glib::clone!(
            #[weak]
            grid_view,
            #[strong]
            context_menu,
            move |gesture, _, x, y| {
                Self::press_event(&grid_view, &context_menu, gesture, x, y);
            }
        ));
        grid_view.add_controller(click);
        grid_view.set_min_columns(1);
        grid_view.set_max_columns(1000);

        ImageGridView {
            grid_view,
            signal_rating_changed,
        }
    }

    pub fn add_rating_listener(&self, listener: Box<dyn Fn((catalog::LibraryId, i32))>) {
        self.signal_rating_changed.connect(listener);
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
                if let Some(menu) = menu {
                    menu.set_pointing_to(Some(&gdk4::Rectangle::new(x as i32, y as i32, 0, 0)));
                    menu.popup();
                }
            }
        }
    }
}
