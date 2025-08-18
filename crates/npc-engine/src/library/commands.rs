/*
 * niepce - npc-engine/library/commands.rs
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

mod import;

use std::path::{Path, PathBuf};

use super::notification::LibNotification;
use super::notification::{Count, FileMove, MetadataChange};
use super::queriedcontent::QueriedContent;
use crate::NiepcePropertyBag;
use crate::catalog::LibraryId;
use crate::catalog::filebundle::FileBundle;
use crate::catalog::keyword::Keyword;
use crate::catalog::label::Label;
use crate::catalog::libfolder::LibFolder;
use crate::catalog::props::NiepceProperties as Np;
use crate::catalog::{CatalogDb, LibError};
use crate::libraryclient::ClientCallback;
use import::CatalogDbImportHelper;
use npc_fwk::PropertyValue;
use npc_fwk::base::RgbColour;
use npc_fwk::{err_out, err_out_line};

pub fn cmd_list_all_preferences(catalog: &CatalogDb) -> bool {
    match catalog.get_all_preferences() {
        Ok(prefs) => {
            if let Err(err) = catalog.notify(LibNotification::Prefs(prefs)) {
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

pub fn cmd_set_preference(catalog: &CatalogDb, key: &str, value: &str) -> bool {
    match catalog.set_pref(key, value) {
        Ok(_) => {
            if let Err(err) = catalog.notify(LibNotification::PrefChanged(
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

pub fn cmd_list_all_keywords(catalog: &CatalogDb) -> bool {
    match catalog.get_all_keywords() {
        Ok(list) => {
            // XXX change this to "LoadKeywords"
            for kw in list {
                if let Err(err) = catalog.notify(LibNotification::AddedKeyword(kw)) {
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

pub fn cmd_list_root_folders(
    catalog: &CatalogDb,
    callback: ClientCallback<Vec<LibFolder>>,
) -> bool {
    match catalog.get_root_folders() {
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
    catalog: &CatalogDb,
    callback: Option<ClientCallback<Vec<LibFolder>>>,
) -> bool {
    match catalog.get_all_folders() {
        Ok(list) => {
            if let Some(callback) = callback {
                callback(list);
            } else {
                // XXX change this to "LoadedFolders"
                for folder in list {
                    if let Err(err) = catalog.notify(LibNotification::AddedFolder(folder)) {
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

/// Import a list of files into the library.
/// It will build the bundles. If you already have the bundles,
/// call `cmd_import_bundles`
pub fn cmd_import_files(catalog: &CatalogDb, base: &Path, files: &[PathBuf]) -> bool {
    let bundles = FileBundle::filter_bundles(files);

    cmd_import_bundles(catalog, base, &bundles)
}

/// Import a list of bundles into the library.
pub fn cmd_import_bundles(catalog: &CatalogDb, base: &Path, bundles: &[FileBundle]) -> bool {
    let base_folders = catalog.get_folder_for_import(base);
    if let Err(err) = base_folders {
        err_out!("Couldn't get folder for import {base:?}: {err}");
        return false;
    }

    for bundle in bundles {
        match bundle
            .main()
            .parent()
            .ok_or(LibError::InvalidResult)
            .and_then(|folder| catalog.get_folder_for_import(folder))
        {
            Ok(libfolders) => {
                let folder_id = libfolders.last().unwrap().id();
                // XXX properly handle this error. Should be a failure.
                if let Err(err) = catalog.add_bundle(folder_id, bundle) {
                    err_out!("Add bundle failed: {:?}", err);
                }
                if catalog.notify(LibNotification::AddedFiles).is_err() {
                    err_out!("Failed to notify AddedFiles");
                }
            }
            Err(err) => err_out_line!("Get folder for import {err:?}"),
        }
    }
    true
}

pub fn cmd_add_bundle(catalog: &CatalogDb, bundle: &FileBundle, folder: LibraryId) -> LibraryId {
    match catalog.add_bundle(folder, bundle) {
        Ok(id) => {
            if catalog.notify(LibNotification::AddedFiles).is_err() {
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

pub fn cmd_create_folder(catalog: &CatalogDb, name: &str, path: Option<String>) -> LibraryId {
    // XXX create folder doesn't allow creating folder inside another.
    match catalog.add_folder_into(name, path, 0) {
        Ok(lf) => {
            let id = lf.id();
            if catalog.notify(LibNotification::AddedFolder(lf)).is_err() {
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

pub fn cmd_delete_folder(catalog: &CatalogDb, id: LibraryId, recursive: bool) -> bool {
    delete_folder(catalog, id, recursive).is_ok()
}

fn delete_one_folder(catalog: &CatalogDb, id: LibraryId) -> crate::catalog::db::Result<()> {
    catalog
        .delete_folder(id)
        .inspect(|_| {
            if catalog.notify(LibNotification::FolderDeleted(id)).is_err() {
                err_out!("Failed to notify FolderDeleted");
            }
        })
        .inspect_err(|err| {
            err_out_line!("Delete folder failed {:?}", err);
        })
}

/// Delete the folder, eventually recursively. If `recursive` is false
/// the it just calls [`delete_one_folder`]
fn delete_folder(
    catalog: &CatalogDb,
    id: LibraryId,
    recursive: bool,
) -> crate::catalog::db::Result<()> {
    if recursive {
        catalog.get_subfolders(id).and_then(|subfolders| {
            if subfolders.is_empty() {
                delete_one_folder(catalog, id)
            } else {
                subfolders
                    .iter()
                    .try_for_each(|folder| delete_folder(catalog, folder.id(), recursive))
                    .and_then(|_| delete_one_folder(catalog, id))
            }
        })
    } else {
        delete_one_folder(catalog, id)
    }
}

pub fn cmd_list_all_albums(catalog: &CatalogDb) -> bool {
    match catalog.get_all_albums() {
        Ok(albums) => {
            // XXX change this notification type
            for album in albums {
                if let Err(err) = catalog.notify(LibNotification::AddedAlbum(album)) {
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

pub fn cmd_count_album(catalog: &CatalogDb, id: LibraryId) -> bool {
    match catalog.count_album(id) {
        Ok(count) => {
            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match catalog.notify(LibNotification::AlbumCounted(Count { id, count })) {
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

pub fn cmd_create_album(catalog: &CatalogDb, name: &str, parent: LibraryId) -> LibraryId {
    match catalog.add_album(name, parent) {
        Ok(album) => {
            let id = album.id();
            if catalog.notify(LibNotification::AddedAlbum(album)).is_err() {
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

pub fn cmd_delete_album(catalog: &CatalogDb, id: LibraryId) -> bool {
    match catalog.delete_album(id) {
        Ok(_) => {
            if catalog.notify(LibNotification::AlbumDeleted(id)).is_err() {
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
pub fn cmd_add_to_album(catalog: &CatalogDb, images: Vec<LibraryId>, album: LibraryId) -> bool {
    match catalog.add_to_album(&images, album) {
        Ok(_) => {
            if catalog
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
pub fn cmd_remove_from_album(
    catalog: &CatalogDb,
    images: Vec<LibraryId>,
    album: LibraryId,
) -> bool {
    match catalog.remove_from_album(&images, album) {
        Ok(_) => {
            if catalog
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

pub fn cmd_rename_album(catalog: &CatalogDb, album: LibraryId, name: &str) -> bool {
    match catalog.rename_album(album, name) {
        Ok(_) => {
            if catalog
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

pub fn cmd_query_album_content(catalog: &CatalogDb, album_id: LibraryId) -> bool {
    match catalog.get_album_content(album_id) {
        Ok(fl) => {
            let mut content = QueriedContent::new(album_id);
            for f in fl {
                content.push(f);
            }
            let value = LibNotification::AlbumContentQueried(content);

            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match catalog.notify(value) {
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

pub fn cmd_request_metadata(catalog: &CatalogDb, file_id: LibraryId) -> bool {
    match catalog.get_metadata(file_id) {
        Ok(lm) => {
            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match catalog.notify(LibNotification::MetadataQueried(lm)) {
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
    catalog: &CatalogDb,
    image_id: LibraryId,
    props: &NiepcePropertyBag,
) -> bool {
    match catalog.set_image_properties(image_id, props) {
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

pub fn cmd_query_folder_content(catalog: &CatalogDb, folder_id: LibraryId) -> bool {
    match catalog.get_folder_content(folder_id) {
        Ok(fl) => {
            let mut content = QueriedContent::new(folder_id);
            for f in fl {
                content.push(f);
            }
            let value = LibNotification::FolderContentQueried(content);

            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match catalog.notify(value) {
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

pub fn cmd_set_metadata(
    catalog: &CatalogDb,
    id: LibraryId,
    meta: Np,
    value: &PropertyValue,
) -> bool {
    match catalog.set_metadata(id, meta, value) {
        Ok(_) => {
            if catalog
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

pub fn cmd_count_folder(catalog: &CatalogDb, id: LibraryId) -> bool {
    match catalog.count_folder(id) {
        Ok(count) => {
            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match catalog.notify(LibNotification::FolderCounted(Count { id, count })) {
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
pub fn cmd_add_keyword(catalog: &CatalogDb, keyword: &str) -> LibraryId {
    match catalog.make_keyword(keyword) {
        Ok(id) => {
            if catalog
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

pub fn cmd_query_keyword_content(catalog: &CatalogDb, keyword_id: LibraryId) -> bool {
    match catalog.get_keyword_content(keyword_id) {
        Ok(fl) => {
            let mut content = QueriedContent::new(keyword_id);
            for f in fl {
                content.push(f);
            }
            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match catalog.notify(LibNotification::KeywordContentQueried(content)) {
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

pub fn cmd_count_keyword(catalog: &CatalogDb, id: LibraryId) -> bool {
    match catalog.count_keyword(id) {
        Ok(count) => {
            // This time it's a fatal error since the purpose of this comand
            // is to retrieve.
            match catalog.notify(LibNotification::KeywordCounted(Count { id, count })) {
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

pub fn cmd_write_metadata(catalog: &CatalogDb, file_id: LibraryId) -> bool {
    match catalog.write_metadata(file_id) {
        Ok(_) => true,
        Err(err) => {
            err_out_line!("write_metadata failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_move_file_to_folder(
    catalog: &CatalogDb,
    file_id: LibraryId,
    from: LibraryId,
    to: LibraryId,
) -> bool {
    match catalog.move_file_to_folder(file_id, to) {
        Ok(_) => {
            if catalog
                .notify(LibNotification::FileMoved(FileMove {
                    file: file_id,
                    from,
                    to,
                }))
                .is_err()
            {
                err_out!("Failed to notify FileMoved");
            }
            if catalog
                .notify(LibNotification::FolderCountChanged(Count {
                    id: from,
                    count: -1,
                }))
                .is_err()
            {
                err_out!("Failed to notify FileMoved");
            }
            if catalog
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

pub fn cmd_list_all_labels(catalog: &CatalogDb) -> bool {
    match catalog.get_all_labels() {
        Ok(l) => {
            // XXX change this notification type
            for label in l {
                if let Err(err) = catalog.notify(LibNotification::AddedLabel(label)) {
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
pub fn cmd_create_label(catalog: &CatalogDb, name: &str, colour: &RgbColour) -> LibraryId {
    match catalog.add_label(name, colour) {
        Ok(id) => {
            let l = Label::new(id, name, colour.clone());
            if catalog.notify(LibNotification::AddedLabel(l)).is_err() {
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

pub fn cmd_delete_label(catalog: &CatalogDb, label_id: LibraryId) -> bool {
    match catalog.delete_label(label_id) {
        Ok(_) => {
            if catalog
                .notify(LibNotification::LabelDeleted(label_id))
                .is_err()
            {
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
    catalog: &CatalogDb,
    label_id: LibraryId,
    name: &str,
    colour: &RgbColour,
) -> bool {
    match catalog.update_label(label_id, name, colour) {
        Ok(_) => {
            let label = Label::new(label_id, name, colour.clone());
            if catalog
                .notify(LibNotification::LabelChanged(label))
                .is_err()
            {
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

pub fn cmd_process_xmp_update_queue(catalog: &CatalogDb, write_xmp: bool) -> bool {
    match catalog.process_xmp_update_queue(write_xmp) {
        Ok(_) => true,
        Err(err) => {
            err_out_line!("process_xmp_update_queue failed: {:?}", err);
            false
        }
    }
}

pub fn cmd_upgrade_catalog_from(catalog: &CatalogDb, version: i32) -> bool {
    match catalog.perform_upgrade(version) {
        Ok(_) => true,
        Err(err) => {
            err_out_line!("upgrade library: {:?}", err);
            false
        }
    }
}

#[cfg(test)]
mod test {
    use crate::catalog::{db::Error, db_test};

    use super::{cmd_delete_folder, import::CatalogDbImportHelper};

    #[test]
    fn test_delete_folder() {
        let catalog = db_test::test_catalog(None);

        let root = catalog.add_folder_into("Pictures", Some("Pictures".into()), 0);
        assert!(root.is_ok());
        let root = root.unwrap();
        assert_eq!(root.parent(), 0);

        let folders = catalog
            .get_folder_for_import(std::path::Path::new("Pictures/2023"))
            .expect("Folder for import failed");
        assert_eq!(root.id(), folders[0].id());
        let parent_folder = folders.last().unwrap();

        let lf = catalog.add_folder_into(
            "20230524",
            Some("Pictures/2023/20230524".to_string()),
            parent_folder.id(),
        );
        assert!(lf.is_ok());
        let lf2 = catalog.add_folder_into(
            "20230508",
            Some("Pictures/2023/20230508".to_string()),
            parent_folder.id(),
        );
        assert!(lf2.is_ok());

        let result = cmd_delete_folder(&catalog, parent_folder.id(), true);
        assert!(result);

        let found1 = catalog.get_folder("Pictures/2023/20230524");
        assert!(matches!(found1, Err(Error::NotFound)));
        let found2 = catalog.get_folder("Pictures/2023/20230508");
        assert!(matches!(found2, Err(Error::NotFound)));
        let found3 = catalog.get_folder("Pictures/2023");
        assert!(matches!(found3, Err(Error::NotFound)));
    }
}
