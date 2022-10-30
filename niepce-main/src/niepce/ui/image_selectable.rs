/*
 * niepce - niepce/ui/image_selectable.rs
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

use uuid::Uuid;

use npc_engine::db;

/// Trait of selectables to keep in sync with the `SelectionController`
pub trait ImageSelectable {
    /// uuid of the selectable
    /// Create it with `Uuid::new_v4()`
    fn id(&self) -> &Uuid;

    /// Return the widget of the image list
    fn image_list(&self) -> &gtk4::IconView;

    /// Return the selected image ID
    fn get_selected(&self) -> Option<db::LibraryId>;

    /// Select the image by ID.
    fn select_image(&self, id: db::LibraryId);
}

/// Tell if two selectables are the same. Because we can't compare pointers
/// or Rc.
pub fn are_same_selectable(s1: &dyn ImageSelectable, s2: &dyn ImageSelectable) -> bool {
    s1.id() == s2.id()
}
