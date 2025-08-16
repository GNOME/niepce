/*
 * niepce - npc-engine/library/commands/import.rs
 *
 * Copyright (C) 2025 Hubert Figuière
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

//! Logic for import commands.

use crate::catalog::db::FolderOpResult;
use crate::catalog::libfolder::LibFolder;
use crate::catalog::{CatalogDb, LibError, LibResult, LibraryId};
use crate::library::notification::LibNotification;
use npc_fwk::{err_out, err_out_line};

/// Add a folder and send the lib notification.
fn add_folder_and_notify(
    catalog: &CatalogDb,
    parent: LibraryId,
    name: &str,
    path: Option<String>,
) -> LibResult<LibFolder> {
    catalog
        .add_folder_into(name, path, parent)
        .inspect(|lf| {
            if catalog
                .notify(LibNotification::AddedFolder(lf.clone()))
                .is_err()
            {
                err_out!("Failed to notify AddedFolder");
            }
        })
        .inspect_err(|err| {
            err_out_line!("Add folder failed {:?}", err);
        })
}

// Get the folder for import. Create it if needed otherwise return the
// one that exists.
//
pub(super) fn get_folder_for_import(
    catalog: &CatalogDb,
    folder: &std::path::Path,
) -> LibResult<Vec<FolderOpResult>> {
    let folder_str = folder.to_string_lossy().to_string();
    catalog
        .get_folder(&folder_str)
        .map(|lf| vec![FolderOpResult::Existing(lf)])
        .or_else(|err| {
            if err == LibError::NotFound {
                catalog
                    .root_folder_for(&folder_str)
                    .map(FolderOpResult::Existing)
                    .or_else(|err| {
                        if !matches!(err, LibError::NotFound) {
                            return Err(err);
                        }
                        let mut parent_folder = Default::default();
                        folder
                            .parent()
                            .and_then(|parent| {
                                parent_folder = parent.to_string_lossy().to_string();
                                parent
                                    .file_name()
                                    .and_then(std::ffi::OsStr::to_str)
                                    .or(Some(""))
                            })
                            .ok_or_else(|| {
                                err_out_line!("Could't get parent folder name for '{folder:?}'.");
                                LibError::InvalidResult
                            })
                            .and_then(|parent_folder_name| {
                                catalog
                                    .add_root_folder(parent_folder_name, parent_folder)
                                    .map(FolderOpResult::Created)
                            })
                    })
                    .map(|parent_folder| {
                        let mut parent_id = parent_folder.id();
                        let children = folder
                            .strip_prefix(std::path::Path::new(parent_folder.path().unwrap()))
                            .unwrap();
                        let mut folders = vec![parent_folder];
                        for folder in children.iter() {
                            folder.to_str().inspect(|name| {
                                add_folder_and_notify(catalog, parent_id, name, None)
                                    .map(FolderOpResult::Created)
                                    .ok()
                                    .map(|folder| {
                                        let id = folder.id();
                                        folders.push(folder);
                                        id
                                    })
                                    .inspect(|id| parent_id = *id);
                            });
                        }
                        folders
                    })
            } else {
                err_out_line!("get folder failed: {:?}", err);
                Err(err)
            }
        })
}

#[cfg(test)]
mod test {
    use crate::catalog::db::FolderOpResult;
    use crate::catalog::db_test;

    use super::get_folder_for_import;

    #[test]
    fn test_folder_for_import() {
        let catalog = db_test::test_catalog();

        let root = catalog.add_root_folder("Pictures", "Pictures".into());
        assert!(root.is_ok());
        let root = root.unwrap();
        assert_eq!(root.parent(), 0);

        let folders =
            get_folder_for_import(&catalog, std::path::Path::new("Pictures/2023/20230524"))
                .expect("Folder for import failed");
        assert_eq!(root.id(), folders[0].id());

        assert_eq!(folders.len(), 3);
        assert!(matches!(folders[0], FolderOpResult::Existing(_)));
        assert!(matches!(folders[1], FolderOpResult::Created(_)));
        let folder = folders.last().unwrap();
        assert!(matches!(folder, FolderOpResult::Created(_)));
        assert_eq!(folder.name(), "20230524");
        // This should have a parent we created.
        assert!(folder.parent() != 0);
        // And its parent is the root.
        assert!(folder.parent() != root.id());
        let id = folder.id();

        let lf = catalog.root_folder_for("Pictures/2023/20230524");
        assert!(lf.is_ok());
        let lf = lf.unwrap();
        println!("lf = {lf:?}");
        assert_eq!(lf.id(), root.id());
        assert_eq!(lf.name(), "Pictures");

        let folders =
            get_folder_for_import(&catalog, std::path::Path::new("Pictures/2023/20230524"))
                .expect("Folder for import failed");
        let folder = folders.last().unwrap();
        assert_eq!(id, folder.id());

        let parent_folders = get_folder_for_import(&catalog, std::path::Path::new("Pictures/2023"))
            .expect("parent Folder for import failed");
        let parent_folder = parent_folders.last().unwrap();
        assert!(matches!(parent_folder, FolderOpResult::Existing(_)));
        assert_eq!(parent_folder.name(), "2023");
        assert_eq!(parent_folder.parent(), root.id());
        assert_eq!(parent_folder.id(), folder.parent());

        let root_folders = get_folder_for_import(&catalog, std::path::Path::new("Pictures"))
            .expect("root Folder for import failed");
        let root_folder = root_folders.last().unwrap();
        assert_eq!(root_folder.id(), parent_folder.parent());

        let roots2 =
            get_folder_for_import(&catalog, std::path::Path::new("Pictures2/2025/20250228"))
                .expect("root Folder for import failed");
        assert_eq!(roots2.len(), 2);
        let root2 = &roots2[0];
        assert!(matches!(root2, FolderOpResult::Created(_)));
        assert_eq!(root2.name(), "2025");
        assert_eq!(root2.parent(), 0);
        let leaf = roots2.last().unwrap();
        assert!(matches!(leaf, FolderOpResult::Created(_)));
        assert_eq!(leaf.parent(), root2.id());
    }
}
