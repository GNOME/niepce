/*
 * niepce - niepce/ui/library_module.rs
 *
 * Copyright (C) 2022-2024 Hubert Figuière
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

use npc_fwk::{gio, gtk4};

/// Trait for Library modules.
pub trait LibraryModule {
    /// Called when it is activated / deactivated.
    fn set_active(&self, _active: bool) {}

    /// Get the menu for the modules.
    fn menu(&self) -> Option<&gio::Menu> {
        None
    }

    /// Get the widget.
    fn widget(&self) -> &gtk4::Widget;
}
