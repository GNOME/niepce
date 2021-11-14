/*
 * niepce - npc-engine/library/commands.rs
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

use super::notification::LibNotification;
use super::notification::{Count, FileMove, MetadataChange};
use super::queriedcontent::QueriedContent;
use crate::db::filebundle::FileBundle;
use crate::db::keyword::Keyword;
use crate::db::label::Label;
use crate::db::libfolder::LibFolder;
use crate::db::library;
use crate::db::library::{Library, Managed};
use crate::db::props::NiepceProperties as Np;
use crate::db::LibraryId;
use npc_fwk::PropertyValue;

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

pub fn cmd_list_all_folders(lib: &Library) -> bool {
    match lib.get_all_folders() {
        Ok(list) => {
            // XXX change this to "LoadedFolders"
            for folder in list {
                if let Err(err) = lib.notify(LibNotification::AddedFolder(folder)) {
                    err_out!("Failed to notify AddedFolder {:?}", err);
                    return false;
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

//
// Get the folder for import. Create it if needed otherwise return the one that exists
//
fn get_folder_for_import(lib: &Library, folder: &str) -> library::Result<LibFolder> {
    match lib.get_folder(folder) {
        Ok(lf) => Ok(lf),
        Err(library::Error::NotFound) => {
            // folder doesn't exist, we'll create it
            if let Some(name) = Library::leaf_name_for_pathname(folder) {
                match lib.add_folder(&name, Some(String::from(folder))) {
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
            } else {
                err_out_line!("Can't get folder name.");
                Err(library::Error::InvalidResult)
            }
        }
        Err(err) => {
            err_out_line!("get folder failed: {:?}", err);
            Err(err)
        }
    }
}

pub fn cmd_import_files(lib: &Library, folder: &str, files: &[PathBuf], manage: Managed) -> bool {
    dbg_assert!(
        manage == Managed::NO,
        "managing file is currently unsupported"
    );

    let bundles = FileBundle::filter_bundles(files);
    match get_folder_for_import(lib, folder) {
        Ok(libfolder) => {
            let folder_id = libfolder.id();
            for bundle in bundles {
                // XXX properle handle this error. Should be a failure.
                if let Err(err) = lib.add_bundle(folder_id, &bundle, manage) {
                    err_out!("Add bundle failed: {:?}", err);
                }
            }
            if lib.notify(LibNotification::AddedFiles).is_err() {
                err_out!("Failed to notify AddedFiles");
            }
            true
        }
        Err(err) => {
            err_out_line!("Get folder for import {:?}", err);
            false
        }
    }
}

pub fn cmd_add_bundle(lib: &Library, bundle: &FileBundle, folder: LibraryId) -> LibraryId {
    match lib.add_bundle(folder, bundle, library::Managed::NO) {
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
    match lib.add_folder(name, path) {
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
pub fn cmd_create_label(lib: &Library, name: &str, colour: &str) -> LibraryId {
    match lib.add_label(name, colour) {
        Ok(id) => {
            let l = Label::new(id, name, colour);
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

pub fn cmd_update_label(lib: &Library, label_id: LibraryId, name: &str, colour: &str) -> bool {
    match lib.update_label(label_id, name, colour) {
        Ok(_) => {
            let label = Label::new(label_id, name, colour);
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
