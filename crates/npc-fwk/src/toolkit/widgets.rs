/*
 * niepce - crates/npc-fwk/src/toolkit/widgets/mod.rs
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

pub mod rating_label;
mod token_text_view;
mod toolbox_item;

// Re-exports
pub use token_text_view::TokenTextView;
pub use toolbox_item::ToolboxItem;

pub mod prelude {
    pub use super::toolbox_item::ToolboxItemExt;
    pub use super::toolbox_item::ToolboxItemImpl;
}
