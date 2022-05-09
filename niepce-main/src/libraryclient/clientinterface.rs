/*
 * niepce - libraryclient/clientinterface.rs
 *
 * Copyright (C) 2017-2021 Hubert Figuière
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

use std::path::PathBuf;

use npc_engine::db::library::Managed;
use npc_engine::db::props::NiepceProperties as Np;
use npc_engine::db::LibraryId;
use npc_fwk::base::PropertyValue;

/// Client interface.
pub trait ClientInterface {
    /// get all the keywords
    fn get_all_keywords(&self);
    fn query_keyword_content(&self, id: LibraryId);
    fn count_keyword(&self, id: LibraryId);

    /// get all the folder
    fn get_all_folders(&self);
    fn query_folder_content(&self, id: LibraryId);
    fn count_folder(&self, id: LibraryId);
    fn create_folder(&self, name: String, path: Option<String>);
    fn delete_folder(&self, id: LibraryId);

    fn request_metadata(&self, id: LibraryId);
    /// set the metadata
    fn set_metadata(&self, id: LibraryId, meta: Np, value: &PropertyValue);
    fn write_metadata(&self, id: LibraryId);

    fn move_file_to_folder(&self, file_id: LibraryId, from: LibraryId, to: LibraryId);
    /// get all the labels
    fn get_all_labels(&self);
    fn create_label(&self, label: String, colour: String);
    fn delete_label(&self, id: LibraryId);
    /// update a label
    fn update_label(&self, id: LibraryId, new_name: String, new_colour: String);

    /// tell to process the Xmp update Queue
    fn process_xmp_update_queue(&self, write_xmp: bool);

    /// Import files from a directory
    /// @param dir the directory
    /// @param files the files to import
    /// @param manage true if imports have to be managed
    fn import_files(&self, dir: String, files: Vec<PathBuf>, manage: Managed);
}

/// Sync client interface
pub trait ClientInterfaceSync {
    /// Create a keyword. Return the id for the keyword.
    /// If the keyword already exists, return its `LibraryId`.
    fn create_keyword_sync(&self, keyword: String) -> LibraryId;

    /// Create a label. Return the id of the newly created label.
    fn create_label_sync(&self, name: String, colour: String) -> LibraryId;

    /// Create a folder. Return the id of the newly created folder.
    fn create_folder_sync(&self, name: String, path: Option<String>) -> LibraryId;
}
