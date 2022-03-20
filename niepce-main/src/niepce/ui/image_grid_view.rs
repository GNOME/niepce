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

use glib::translate::*;
use gtk4::prelude::*;

use super::library_cell_renderer::LibraryCellRenderer;
use npc_fwk::toolkit::clickable_cell_renderer::ClickableCellRenderer;

pub struct ImageGridView {
    icon_view: gtk4::IconView,
}

impl ImageGridView {
    pub fn new(store: &gtk4::TreeModel) -> Self {
        let icon_view = gtk4::IconView::with_model(store);

        let click = gtk4::GestureClick::new();
        //        click.connect_pressed(glib::clone!(@weak obj => move |gesture, n, x, y| {
        // XXX handle press event
        // self.press_event(x, y);
        //        }));
        icon_view.add_controller(&click);

        ImageGridView { icon_view }
    }
}

impl std::ops::Deref for ImageGridView {
    type Target = gtk4::IconView;

    fn deref(&self) -> &gtk4::IconView {
        &self.icon_view
    }
}

impl ImageGridView {
    fn press_event(&self, x: f64, y: f64) {
        // let event = gesture.last_event();

        // XXX forward to the icon_view or something
        // self.parent_button_press_event(widget, event);

        if let Some((_, cell)) = self.icon_view.item_at_pos(x as i32, y as i32) {
            if let Ok(mut cell) = cell.downcast::<LibraryCellRenderer>() {
                cell.hit(x as i32, y as i32);
            }
        }
    }
}

/// # Safety
/// Use raw pointers.
#[no_mangle]
pub unsafe extern "C" fn npc_image_grid_view_new(
    store: *mut gtk4_sys::GtkTreeModel,
) -> *mut ImageGridView {
    Box::into_raw(Box::new(ImageGridView::new(
        &gtk4::TreeModel::from_glib_full(store),
    )))
}

/// # Safety
/// Use raw pointers
#[no_mangle]
pub unsafe extern "C" fn npc_image_grid_view_get_icon_view(
    view: &ImageGridView,
) -> *mut gtk4_sys::GtkIconView {
    view.icon_view.to_glib_none().0
}

/// # Safety
/// Use raw pointers
#[no_mangle]
pub unsafe extern "C" fn npc_image_grid_view_release(view: *mut ImageGridView) {
    Box::from_raw(view);
}
