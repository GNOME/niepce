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
    pub fn new(store: &gtk4::TreeModel, context_menu: Option<gtk4::PopoverMenu>) -> Self {
        let icon_view = gtk4::IconView::with_model(store);

        let click = gtk4::GestureClick::new();
        click.set_button(0);
        click.connect_pressed(
            glib::clone!(@weak icon_view, @strong context_menu => move |gesture, _, x, y| {
                Self::press_event(&icon_view, &context_menu, gesture, x, y);
            }),
        );
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
    fn press_event(
        icon_view: &gtk4::IconView,
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

        if let Some((_, cell)) = icon_view.item_at_pos(x as i32, y as i32) {
            if let Ok(mut cell) = cell.downcast::<LibraryCellRenderer>() {
                cell.hit(x as i32, y as i32);
            }
        }
    }
}

/// Create a new `ImageGridView`
///
/// # Safety
/// Use raw pointers.
///
/// The `store` and `context_menu` will get ref.
/// context_menu can be `nullptr`
#[no_mangle]
pub unsafe extern "C" fn npc_image_grid_view_new(
    store: *mut gtk4_sys::GtkTreeModel,
    context_menu: *mut gtk4_sys::GtkPopoverMenu,
) -> *mut ImageGridView {
    Box::into_raw(Box::new(ImageGridView::new(
        &gtk4::TreeModel::from_glib_none(store),
        Option::<gtk4::PopoverMenu>::from_glib_none(context_menu),
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
