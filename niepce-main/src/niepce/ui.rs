/*
 * niepce - niepce/ui/mod.rs
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

pub mod dialogs;
mod film_strip_controller;
pub mod image_grid_view;
pub mod image_list_store;
mod image_selectable;
pub mod imagetoolbar;
pub mod library_cell_renderer;
pub mod metadata_pane_controller;
mod module_shell_widget;
pub mod niepce_window;
mod selection_controller;
pub mod thumb_nav;
pub mod thumb_strip_view;
mod workspace_controller;

pub use film_strip_controller::FilmStripController;
pub use image_list_store::ImageListStore;
pub use image_selectable::{are_same_selectable, ImageSelectable};
pub use module_shell_widget::ModuleShellWidget;
pub use selection_controller::{selection_controller_new, SelectionController};

pub mod cxx {
    pub use super::module_shell_widget::module_shell_widget_new;
}
