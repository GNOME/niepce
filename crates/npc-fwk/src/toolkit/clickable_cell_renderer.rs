/*
 * niepce - npc-fwk/toolkit/clickable_cell_renderer.rs
 *
 * Copyright (C) 2020 Hubert Figuière
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

/// Trait for get clicks from cell renderer.
/// This is used to work around some bug in Gtk.
pub trait ClickableCellRenderer {
    fn hit(&self, x: i32, y: i32);
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn is_hit(&self) -> bool;
    fn reset_hit(&self);
}
