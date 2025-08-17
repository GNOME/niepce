/*
 * niepce - libraryclient/clientinterface.rs
 *
 * Copyright (C) 2017-2025 Hubert Figuière
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

use crate::NiepcePropertyBag;
use crate::catalog::filebundle::FileBundle;
use crate::catalog::props::NiepceProperties as Np;
use crate::catalog::{LibFolder, LibraryId};
use npc_fwk::base::{PropertyValue, RgbColour};

/// Callback for a local library request.
pub type ClientCallback<T> = Box<dyn Fn(T) + Sync + Send>;

/// Client interface.
pub trait ClientInterface {
    /// Preferences
    fn get_all_preferences(&self);
    fn set_preference(&self, key: String, value: String);

    /// get all the keywords
    fn get_all_keywords(&self);
    fn query_keyword_content(&self, id: LibraryId);
    fn count_keyword(&self, id: LibraryId);

    /// Get the root folders.
    fn get_root_folders(&self, callback: ClientCallback<Vec<LibFolder>>);
    /// Get all the folders.
    fn get_all_folders(&self, callback: Option<ClientCallback<Vec<LibFolder>>>);
    fn query_folder_content(&self, id: LibraryId);
    fn count_folder(&self, id: LibraryId);
    fn create_folder(&self, name: String, path: Option<String>);
    fn delete_folder(&self, id: LibraryId);

    /// get all the albums
    fn get_all_albums(&self);
    /// Count album content.
    fn count_album(&self, album_id: LibraryId);
    /// Create an album (async)
    fn create_album(&self, name: String, parent: LibraryId);
    fn delete_album(&self, id: LibraryId);
    /// Add images to an album.
    fn add_to_album(&self, images: &[LibraryId], album: LibraryId);
    /// Remove images from an album.
    fn remove_from_album(&self, images: &[LibraryId], album: LibraryId);
    /// Rename album `album_id` to `name`.
    fn rename_album(&self, album_id: LibraryId, name: String);
    /// Query content for album.
    fn query_album_content(&self, album_id: LibraryId);

    fn request_metadata(&self, id: LibraryId);
    /// set the metadata
    fn set_metadata(&self, id: LibraryId, meta: Np, value: &PropertyValue);
    /// set some properties for an image.
    fn set_image_properties(&self, id: LibraryId, props: &NiepcePropertyBag);
    fn write_metadata(&self, id: LibraryId);

    fn move_file_to_folder(&self, file_id: LibraryId, from: LibraryId, to: LibraryId);
    /// get all the labels
    fn get_all_labels(&self);
    fn create_label(&self, label: String, colour: RgbColour);
    fn delete_label(&self, id: LibraryId);
    /// update a label
    fn update_label(&self, id: LibraryId, new_name: String, new_colour: RgbColour);

    /// tell to process the Xmp update Queue
    fn process_xmp_update_queue(&self, write_xmp: bool);

    /// Import files in place.
    /// @param files the files to import
    fn import_files(&self, base: PathBuf, files: Vec<PathBuf>);
}

/// Sync client interface
pub trait ClientInterfaceSync {
    /// Create a keyword. Return the id for the keyword.
    /// If the keyword already exists, return its `LibraryId`.
    fn create_keyword_sync(&self, keyword: String) -> LibraryId;

    /// Create a label. Return the id of the newly created label.
    fn create_label_sync(&self, name: String, colour: RgbColour) -> LibraryId;

    /// Create a folder. Return the id of the newly created folder.
    fn create_folder_sync(&self, name: String, path: Option<String>) -> LibraryId;

    /// Create an album. Return the id to the newly created album.
    fn create_album_sync(&self, name: String, parent: LibraryId) -> LibraryId;

    /// Add a bundle.
    fn add_bundle_sync(&self, bundle: &FileBundle, folder: LibraryId) -> LibraryId;

    /// Upgrade the library from `version`. Note that the version is just a suggestion.
    /// Return true if successful.
    fn upgrade_catalog_from_sync(&self, version: i32) -> bool;
}
