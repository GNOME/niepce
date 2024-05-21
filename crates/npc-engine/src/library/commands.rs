/*
 * niepce - npc-engine/library/commands.rs
 *
 * Copyright (C) 2017-2024 Hubert Figuière
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

use super::notification::LibNotification;
use super::notification::{Count, FileMove, MetadataChange};
use super::queriedcontent::QueriedContent;
use crate::db::filebundle::FileBundle;
use crate::db::keyword::Keyword;
use crate::db::label::Label;
use crate::db::libfolder::LibFolder;
use crate::db::props::NiepceProperties as Np;
use crate::db::LibraryId;
use crate::db::{LibError, LibResult, Library};
use crate::libraryclient::ClientCallback;
use crate::NiepcePropertyBag;
use npc_fwk::base::RgbColour;
use npc_fwk::PropertyValue;
use npc_fwk::{err_out, err_out_line};

pub fn cmd_list_all_preferences(lib: &Library) -> bool {
    match lib.get_all_preferences() {
        Ok(prefs) => {
            if let Err(err) = lib.notify(LibNotification::Prefs(prefs)) {
                err_out!("Failed to notify Prefs {:?}", err);
                return false;
            }
            true
        }
        Err(err) => {
            err_out_line!("get all preferences failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_set_preference(lib: &Library, key: &str, value: &str) -> bool {
    match lib.set_pref(key, value) {
        Ok(_) => {
            if let Err(err) = lib.notify(LibNotification::PrefChanged(
                key.to_string(),
                value.to_string(),
            )) {
                err_out!("Failed to notify PrefChanged {:?}", err);
                return false;
            }
            true
        }
        Err(err) => {
            err_out_line!("set preference failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_list_all_keywords(lib: &Library) -> bool {
    match lib.get_all_keywords() {
        Ok(list) => {
            // XXX change this to "LoadKeywords"
            for kw in list {
                if let Err(err) = lib.notify(LibNotification::AddedKeyword(kw)) {
                    err_out!("Failed to notify AddedKeyword {:?}", err);
                    return false;
                }
            }
            true
        }
        Err(err) => {
            err_out_line!("get all keywords failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_list_root_folders(lib: &Library, callback: ClientCallback<Vec<LibFolder>>) -> bool {
    match lib.get_root_folders() {
        Ok(list) => {
            callback(list);
            true
        }
        Err(err) => {
            err_out_line!("get_root_folders failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_list_all_folders(
    lib: &Library,
    callback: Option<ClientCallback<Vec<LibFolder>>>,
) -> bool {
    match lib.get_all_folders() {
        Ok(list) => {
            if let Some(callback) = callback {
                callback(list);
            } else {
                // XXX change this to "LoadedFolders"
                for folder in list {
                    if let Err(err) = lib.notify(LibNotification::AddedFolder(folder)) {
                        err_out!("Failed to notify AddedFolder {:?}", err);
                        return false;
                    }
                }
            }
            true
        }
        Err(err) => {
            err_out_line!("get_all_folders failed: {:?}", err);
            false
        }
    }
}

fn add_folder_and_notify(
    lib: &Library,
    parent: LibraryId,
    name: &str,
    path: Option<String>,
) -> LibResult<LibFolder> {
    match lib.add_folder_into(name, path, parent) {
        Ok(lf) => {
            let libfolder = lf.clone();
            if lib.notify(LibNotification::AddedFolder(lf)).is_err() {
                err_out!("Failed to notify AddedFolder");
            }
            Ok(libfolder)
        }
        Err(err) => {
            err_out_line!("Add folder failed {:?}", err);
            Err(err)
        }
    }
}

// Get the folder for import. Create it if needed otherwise return the
// one that exists.
//
fn get_folder_for_import(lib: &Library, folder: &std::path::Path) -> LibResult<LibFolder> {
    err_out!("get folder for import for '{folder:?}'");
    let folder_str = folder.to_string_lossy().to_string();
    match lib.get_folder(&folder_str) {
        Ok(lf) => Ok(lf),
        Err(LibError::NotFound) => lib
            .root_folder_for(&folder_str)
            .or_else(|err| {
                if !matches!(err, LibError::NotFound) {
                    return Err(err);
                }
                let mut parent_folder = Default::default();
                if let Some(parent_folder_name) = folder.parent().and_then(|parent| {
                    parent_folder = parent.to_string_lossy().to_string();
                    parent
                        .file_name()
                        .and_then(std::ffi::OsStr::to_str)
                        .or(Some(""))
                }) {
                    lib.add_folder(parent_folder_name, parent_folder)
                } else {
                    err_out_line!("Could't get parent folder name for '{folder:?}'.");
                    Err(LibError::InvalidResult)
                }
            })
            .and_then(|lf| {
                if let Some(name) = folder.file_name().and_then(std::ffi::OsStr::to_str) {
                    add_folder_and_notify(lib, lf.id(), name, None)
                } else {
                    err_out_line!("Couldn't get folder name for '{folder:?}'.");
                    Err(LibError::InvalidResult)
                }
            }),
        Err(err) => {
            err_out_line!("get folder failed: {:?}", err);
            Err(err)
        }
    }
}

/// Import a list of files into the library.
/// It will build the bundles. If you already have the bundles,
/// call `cmd_import_bundles`
pub fn cmd_import_files(lib: &Library, files: &[PathBuf]) -> bool {
    let bundles = FileBundle::filter_bundles(files);

    cmd_import_bundles(lib, &bundles)
}

/// Import a list of bundles into the library.
pub fn cmd_import_bundles(lib: &Library, bundles: &[FileBundle]) -> bool {
    for bundle in bundles {
        match bundle
            .main()
            .parent()
            .ok_or(LibError::InvalidResult)
            .and_then(|folder| get_folder_for_import(lib, folder))
        {
            Ok(libfolder) => {
                let folder_id = libfolder.id();
                // XXX properly handle this error. Should be a failure.
                if let Err(err) = lib.add_bundle(folder_id, bundle) {
                    err_out!("Add bundle failed: {:?}", err);
                }
                if lib.notify(LibNotification::AddedFiles).is_err() {
                    err_out!("Failed to notify AddedFiles");
                }
            }
            Err(err) => err_out_line!("Get folder for import {err:?}"),
        }
    }
    true
}

pub fn cmd_add_bundle(lib: &Library, bundle: &FileBundle, folder: LibraryId) -> LibraryId {
    match lib.add_bundle(folder, bundle) {
        Ok(id) => {
            if lib.notify(LibNotification::AddedFiles).is_err() {
                err_out!("Failed to notify AddedFiles");
            }
            id
        }
        Err(err) => {
            err_out_line!("Bundle creation failed {:?}", err);
            -1
        }
    }
}

pub fn cmd_create_folder(lib: &Library, name: &str, path: Option<String>) -> LibraryId {
    // XXX create folder doesn't allow creating folder inside another.
    match lib.add_folder_into(name, path, 0) {
        Ok(lf) => {
            let id = lf.id();
            if lib.notify(LibNotification::AddedFolder(lf)).is_err() {
                err_out!("Failed to notify AddedFolder");
            }
            id
        }
        Err(err) => {
            err_out_line!("Folder creation failed {:?}", err);
            -1
        }
    }
}

pub fn cmd_delete_folder(lib: &Library, id: LibraryId) -> bool {
    match lib.delete_folder(id) {
        Ok(_) => {
            if lib.notify(LibNotification::FolderDeleted(id)).is_err() {
                err_out!("Failed to notify FolderDeleted");
            }
            true
        }
        Err(err) => {
            err_out_line!("Delete folder failed {:?}", err);
            false
        }
    }
}

pub fn cmd_list_all_albums(lib: &Library) -> bool {
    match lib.get_all_albums() {
        Ok(albums) => {
            // XXX change this notification type
            for album in albums {
                if let Err(err) = lib.notify(LibNotification::AddedAlbum(album)) {
                    err_out!("Failed to notify AddedAlbum {:?}", err);
                    return false;
                }
            }
            true
        }
        Err(err) => {
            err_out_line!("get_all_albums failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_count_album(lib: &Library, id: LibraryId) -> bool {
    match lib.count_album(id) {
        Ok(count) => {
            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match lib.notify(LibNotification::AlbumCounted(Count { id, count })) {
                Err(err) => {
                    err_out!("Failed to notify AlbumCounted {:?}", err);
                    false
                }
                Ok(_) => true,
            }
        }
        Err(err) => {
            err_out_line!("count_album failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_create_album(lib: &Library, name: &str, parent: LibraryId) -> LibraryId {
    match lib.add_album(name, parent) {
        Ok(album) => {
            let id = album.id();
            if lib.notify(LibNotification::AddedAlbum(album)).is_err() {
                err_out!("Failed to notify AddedAlbum");
            }
            id
        }
        Err(err) => {
            err_out_line!("Album creation failed {:?}", err);
            -1
        }
    }
}

pub fn cmd_delete_album(lib: &Library, id: LibraryId) -> bool {
    match lib.delete_album(id) {
        Ok(_) => {
            if lib.notify(LibNotification::AlbumDeleted(id)).is_err() {
                err_out!("Failed to notify AlbumDeleted");
            }
            true
        }
        Err(err) => {
            err_out_line!("Delete album failed {:?}", err);
            false
        }
    }
}

/// Command to add `images` to an `album`.
pub fn cmd_add_to_album(lib: &Library, images: Vec<LibraryId>, album: LibraryId) -> bool {
    match lib.add_to_album(&images, album) {
        Ok(_) => {
            if lib
                .notify(LibNotification::AddedToAlbum(images, album))
                .is_err()
            {
                err_out!("Failed to notify AddedToAlbum");
            }
            true
        }
        Err(err) => {
            err_out_line!(
                "Adding images {:?} to album {} failed {:?}",
                images,
                album,
                err
            );
            false
        }
    }
}

/// Command to remove `images` from an `album`.
pub fn cmd_remove_from_album(lib: &Library, images: Vec<LibraryId>, album: LibraryId) -> bool {
    match lib.remove_from_album(&images, album) {
        Ok(_) => {
            if lib
                .notify(LibNotification::RemovedFromAlbum(images, album))
                .is_err()
            {
                err_out!("Failed to notify RemovedFromAlbum");
            }
            true
        }
        Err(err) => {
            err_out_line!(
                "Removing images {:?} from album {} failed {:?}",
                images,
                album,
                err
            );
            false
        }
    }
}

pub fn cmd_rename_album(lib: &Library, album: LibraryId, name: &str) -> bool {
    match lib.rename_album(album, name) {
        Ok(_) => {
            if lib
                .notify(LibNotification::AlbumRenamed(album, name.to_string()))
                .is_err()
            {
                err_out!("Failed to notify RenamedAlbum");
            }
            true
        }
        Err(err) => {
            err_out_line!("Renaming album {} to {} failed {:?}", album, name, err);
            false
        }
    }
}

pub fn cmd_query_album_content(lib: &Library, album_id: LibraryId) -> bool {
    match lib.get_album_content(album_id) {
        Ok(fl) => {
            let mut content = QueriedContent::new(album_id);
            for f in fl {
                content.push(f);
            }
            let value = LibNotification::AlbumContentQueried(content);

            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match lib.notify(value) {
                Err(err) => {
                    err_out!("Failed to notify AlbumContent {:?}", err);
                    false
                }
                Ok(_) => true,
            }
        }
        Err(err) => {
            err_out_line!("Get album content failed {:?}", err);
            false
        }
    }
}

pub fn cmd_request_metadata(lib: &Library, file_id: LibraryId) -> bool {
    match lib.get_metadata(file_id) {
        Ok(lm) => {
            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match lib.notify(LibNotification::MetadataQueried(lm)) {
                Err(err) => {
                    err_out!("Failed to notify Metadata {:?}", err);
                    false
                }
                Ok(_) => true,
            }
        }
        Err(err) => {
            err_out_line!("Get metadata failed {:?}", err);
            false
        }
    }
}

/// Command to set image properties.
pub fn cmd_set_image_properties(
    lib: &Library,
    image_id: LibraryId,
    props: &NiepcePropertyBag,
) -> bool {
    match lib.set_image_properties(image_id, props) {
        Ok(_) => {
            // XXX set the image properties.
            true
        }
        Err(err) => {
            err_out_line!("Setting image metadata failed {:?}", err);
            false
        }
    }
}

pub fn cmd_query_folder_content(lib: &Library, folder_id: LibraryId) -> bool {
    match lib.get_folder_content(folder_id) {
        Ok(fl) => {
            let mut content = QueriedContent::new(folder_id);
            for f in fl {
                content.push(f);
            }
            let value = LibNotification::FolderContentQueried(content);

            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match lib.notify(value) {
                Err(err) => {
                    err_out!("Failed to notify FolderContent {:?}", err);
                    false
                }
                Ok(_) => true,
            }
        }
        Err(err) => {
            err_out_line!("Get folder content failed {:?}", err);
            false
        }
    }
}

pub fn cmd_set_metadata(lib: &Library, id: LibraryId, meta: Np, value: &PropertyValue) -> bool {
    match lib.set_metadata(id, meta, value) {
        Ok(_) => {
            if lib
                .notify(LibNotification::MetadataChanged(MetadataChange::new(
                    id,
                    meta,
                    value.clone(),
                )))
                .is_err()
            {
                err_out!("Failed to notify MetadataChange");
            }
            true
        }
        Err(err) => {
            err_out_line!("set_matadata failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_count_folder(lib: &Library, id: LibraryId) -> bool {
    match lib.count_folder(id) {
        Ok(count) => {
            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match lib.notify(LibNotification::FolderCounted(Count { id, count })) {
                Err(err) => {
                    err_out!("Failed to notify FolderCounted {:?}", err);
                    false
                }
                Ok(_) => true,
            }
        }
        Err(err) => {
            err_out_line!("count_folder failed: {:?}", err);
            false
        }
    }
}

/// Add a keyword. Return `LibraryId` of the keyword, already existing
/// or created.
pub fn cmd_add_keyword(lib: &Library, keyword: &str) -> LibraryId {
    match lib.make_keyword(keyword) {
        Ok(id) => {
            if lib
                .notify(LibNotification::AddedKeyword(Keyword::new(id, keyword)))
                .is_err()
            {
                err_out!("Failed to notify AddedKeyword");
            }
            id
        }
        Err(err) => {
            err_out_line!("make_keyword failed: {:?}", err);
            -1
        }
    }
}

pub fn cmd_query_keyword_content(lib: &Library, keyword_id: LibraryId) -> bool {
    match lib.get_keyword_content(keyword_id) {
        Ok(fl) => {
            let mut content = QueriedContent::new(keyword_id);
            for f in fl {
                content.push(f);
            }
            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match lib.notify(LibNotification::KeywordContentQueried(content)) {
                Err(err) => {
                    err_out!("Failed to notify KeywordContentQueried {:?}", err);
                    false
                }
                Ok(_) => true,
            }
        }
        Err(err) => {
            err_out_line!("get_keyword_content failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_count_keyword(lib: &Library, id: LibraryId) -> bool {
    match lib.count_keyword(id) {
        Ok(count) => {
            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match lib.notify(LibNotification::KeywordCounted(Count { id, count })) {
                Err(err) => {
                    err_out!("Failed to notify KeywordCounted {:?}", err);
                    false
                }
                Ok(_) => true,
            }
        }
        Err(err) => {
            err_out_line!("count_keyword failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_write_metadata(lib: &Library, file_id: LibraryId) -> bool {
    match lib.write_metadata(file_id) {
        Ok(_) => true,
        Err(err) => {
            err_out_line!("write_metadata failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_move_file_to_folder(
    lib: &Library,
    file_id: LibraryId,
    from: LibraryId,
    to: LibraryId,
) -> bool {
    match lib.move_file_to_folder(file_id, to) {
        Ok(_) => {
            if lib
                .notify(LibNotification::FileMoved(FileMove {
                    file: file_id,
                    from,
                    to,
                }))
                .is_err()
            {
                err_out!("Failed to notify FileMoved");
            }
            if lib
                .notify(LibNotification::FolderCountChanged(Count {
                    id: from,
                    count: -1,
                }))
                .is_err()
            {
                err_out!("Failed to notify FileMoved");
            }
            if lib
                .notify(LibNotification::FolderCountChanged(Count {
                    id: to,
                    count: 1,
                }))
                .is_err()
            {
                err_out!("Failed to notify FileMoved");
            }
            true
        }
        Err(err) => {
            err_out_line!("move file to folder failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_list_all_labels(lib: &Library) -> bool {
    match lib.get_all_labels() {
        Ok(l) => {
            // XXX change this notification type
            for label in l {
                if let Err(err) = lib.notify(LibNotification::AddedLabel(label)) {
                    err_out!("Failed to notify AddedLabel {:?}", err);
                    return false;
                }
            }
            true
        }
        Err(err) => {
            err_out_line!("get_all_labels failed: {:?}", err);
            false
        }
    }
}

/// This command will create a label, with `name` and `colour`.
/// Returns id of the label. Or 0 on error.
pub fn cmd_create_label(lib: &Library, name: &str, colour: &RgbColour) -> LibraryId {
    match lib.add_label(name, colour) {
        Ok(id) => {
            let l = Label::new(id, name, colour.clone());
            if lib.notify(LibNotification::AddedLabel(l)).is_err() {
                err_out!("Failed to notify AddedLabel");
            }
            id
        }
        Err(err) => {
            err_out_line!("add_label failed: {:?}", err);
            -1
        }
    }
}

pub fn cmd_delete_label(lib: &Library, label_id: LibraryId) -> bool {
    match lib.delete_label(label_id) {
        Ok(_) => {
            if lib.notify(LibNotification::LabelDeleted(label_id)).is_err() {
                err_out!("Failed to notify LabelDeleted");
            }
            true
        }
        Err(err) => {
            err_out_line!("delete label failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_update_label(
    lib: &Library,
    label_id: LibraryId,
    name: &str,
    colour: &RgbColour,
) -> bool {
    match lib.update_label(label_id, name, colour) {
        Ok(_) => {
            let label = Label::new(label_id, name, colour.clone());
            if lib.notify(LibNotification::LabelChanged(label)).is_err() {
                err_out!("Failed to notify LabelChanged");
            }
            true
        }
        Err(err) => {
            err_out_line!("update label failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_process_xmp_update_queue(lib: &Library, write_xmp: bool) -> bool {
    match lib.process_xmp_update_queue(write_xmp) {
        Ok(_) => true,
        Err(err) => {
            err_out_line!("process_xmp_update_queue failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_upgrade_library_from(lib: &Library, version: i32) -> bool {
    match lib.perform_upgrade(version) {
        Ok(_) => true,
        Err(err) => {
            err_out_line!("upgrade library: {:?}", err);
            false
        }
    }
}

#[cfg(test)]
mod test {

    use crate::db::library_test;

    use super::get_folder_for_import;

    #[test]
    fn test_folder_for_import() {
        let lib = library_test::test_library();

        let folder = get_folder_for_import(&lib, std::path::Path::new("Pictures/2023/20230524"))
            .expect("Folder for import failed");
        assert_eq!(folder.name(), "20230524");
        // This should have a parent we created.
        assert!(folder.parent() != 0);
        let id = folder.id();

        let lf = lib.root_folder_for("Pictures/2023/20230524");
        assert!(lf.is_ok());
        let lf = lf.unwrap();
        println!("lf = {lf:?}");
        assert_eq!(lf.name(), "2023");

        let folder = get_folder_for_import(&lib, std::path::Path::new("Pictures/2023/20230524"))
            .expect("Folder for import failed");
        assert_eq!(id, folder.id());
    }
}
