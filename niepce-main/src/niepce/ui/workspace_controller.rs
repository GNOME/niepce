/*
 * niepce - niepce/ui/workspace_controller.rs
 *
 * Copyright (C) 2021 Hubert Figuière
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
use num_traits::FromPrimitive;

use npc_engine::db::LibraryId;
use npc_engine::libraryclient::{ClientInterface, LibraryClientWrapper};
use npc_fwk::dbg_out;

#[repr(i32)]
/// XXX this must be in sync with the C++ code.
/// Until such time it's all Rust.
enum ColIndex {
    IdColumn = 1,
    TypeColumn = 3,
}

#[repr(i32)]
#[derive(Debug, num_derive::FromPrimitive)]
/// Types of items in the workspace tree view
pub enum ItemTypes {
    AlbumsItem = 0,
    FoldersItem = 1,
    ProjectsItem = 2,
    KeywordsItem = 3,
    AlbumItem = 4,
    FolderItem = 5,
    ProjectItem = 6,
    KeywordItem = 7,
}

/// Handle the selection and return the type selected or None
pub fn on_libtree_selection(
    libclient: &mut LibraryClientWrapper,
    libtree: &gtk4::TreeView,
) -> Option<ItemTypes> {
    let selection = libtree.selection();
    let selected = selection.selected();
    if let Some((model, selected)) = selected {
        // For some reason this is an ILong (on x86_64)
        // While in the C++ code it is an int64_t
        let item = model
            .get_value(&selected, ColIndex::IdColumn as i32)
            .get::<glib::ILong>()
            .map(|v| -> LibraryId { v.into() });
        if let Ok(id) = item {
            let item_type = model
                .get_value(&selected, ColIndex::TypeColumn as i32)
                .get::<i32>()
                .ok()
                .and_then(|v| ItemTypes::from_i32(v));
            match item_type {
                Some(ItemTypes::FolderItem) => {
                    libclient.query_folder_content(id);
                    Some(ItemTypes::FolderItem)
                }
                Some(ItemTypes::KeywordItem) => {
                    libclient.query_keyword_content(id);
                    Some(ItemTypes::KeywordItem)
                }
                Some(ItemTypes::AlbumItem) => {
                    libclient.query_album_content(id);
                    Some(ItemTypes::AlbumItem)
                }
                _ => {
                    dbg_out!("selected something not a container");
                    None
                }
            }
        } else {
            dbg_out!("couldn't get the id");
            None
        }
    } else {
        dbg_out!("Invalid iterator");
        None
    }
}

#[no_mangle]
/// Handle the selection and return the type selected or -1
pub unsafe extern "C" fn workspace_controller_on_libtree_selection(
    libclient: &mut LibraryClientWrapper,
    libtree: *mut gtk4_sys::GtkTreeView,
) -> i32 {
    match on_libtree_selection(libclient, &gtk4::TreeView::from_glib_none(libtree)) {
        Some(t) => t as i32,
        _ => -1,
    }
}
