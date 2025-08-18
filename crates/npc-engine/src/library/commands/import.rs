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

pub(super) trait CatalogDbImportHelper {
    fn add_root_folder_and_notify(&self, name: &str, path: String) -> LibResult<LibFolder>;
    fn add_folder_and_notify(
        &self,
        parent: LibraryId,
        name: &str,
        path: Option<String>,
    ) -> LibResult<LibFolder>;
    fn get_folder_for_import(&self, folder: &std::path::Path) -> LibResult<Vec<FolderOpResult>>;
}

impl CatalogDbImportHelper for CatalogDb {
    fn add_root_folder_and_notify(&self, name: &str, path: String) -> LibResult<LibFolder> {
        self.add_folder_and_notify(0, name, Some(path))
    }

    /// Add a folder and send the lib notification.
    fn add_folder_and_notify(
        &self,
        parent: LibraryId,
        name: &str,
        path: Option<String>,
    ) -> LibResult<LibFolder> {
        self.add_folder_into(name, path, parent)
            .inspect(|lf| {
                if self
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
    fn get_folder_for_import(&self, folder: &std::path::Path) -> LibResult<Vec<FolderOpResult>> {
        let folder_str = folder.to_string_lossy();
        self.get_folder(&folder_str)
            .map(|lf| vec![FolderOpResult::Existing(lf)])
            .or_else(|err| {
                if err == LibError::NotFound {
                    self.root_folder_for(&folder_str)
                        .map(FolderOpResult::Existing)
                        .or_else(|err| {
                            if !matches!(err, LibError::NotFound) {
                                return Err(err);
                            }
                            let folder_name = folder
                                .file_name()
                                .ok_or(LibError::InvalidResult)?
                                .to_string_lossy();
                            self.add_root_folder_and_notify(&folder_name, folder_str.to_string())
                                .and_then(|lf| {
                                    self.reparent_roots_for(lf.id(), &folder_str)?;
                                    Ok(lf)
                                })
                                .map(FolderOpResult::Created)
                        })
                        .map(|parent_folder| {
                            let mut parent_id = parent_folder.id();
                            let children = folder
                                .strip_prefix(std::path::Path::new(parent_folder.path().unwrap()))
                                .unwrap();
                            let mut folders = vec![parent_folder];
                            for folder in children.iter() {
                                folder.to_str().inspect(|name| {
                                    self.add_folder_and_notify(parent_id, name, None)
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
}

#[cfg(test)]
mod test {
    use crate::catalog::db::FolderOpResult;
    use crate::catalog::db_test;

    use super::CatalogDbImportHelper;

    #[test]
    fn test_folder_for_import() {
        let catalog = db_test::test_catalog(None);

        let root = catalog.add_folder_into("Pictures", Some("Pictures".into()), 0);
        assert!(root.is_ok());
        let root = root.unwrap();
        assert_eq!(root.parent(), 0);

        let folders = catalog
            .get_folder_for_import(std::path::Path::new("Pictures/2023/20230524"))
            .expect("Folder for import failed");
        assert_eq!(root.id(), folders[0].id());

        assert_eq!(folders.len(), 3);
        assert!(matches!(folders[0], FolderOpResult::Existing(_)));
        assert!(matches!(folders[1], FolderOpResult::Created(_)));
        let folder = folders.last().unwrap();
        assert!(matches!(folder, FolderOpResult::Created(_)));
        assert_eq!(folder.name(), "20230524");
        // This should have a parent we created.
        assert_ne!(folder.parent(), 0);
        // And its parent is the root.
        assert_ne!(folder.parent(), root.id());
        let id = folder.id();

        let lf = catalog.root_folder_for("Pictures/2023/20230524");
        assert!(lf.is_ok());
        let lf = lf.unwrap();
        assert_eq!(lf.id(), root.id());
        assert_eq!(lf.name(), "Pictures");

        let folders = catalog
            .get_folder_for_import(std::path::Path::new("Pictures/2023/20230524"))
            .expect("Folder for import failed");
        let folder = folders.last().unwrap();
        assert_eq!(id, folder.id());

        let parent_folders = catalog
            .get_folder_for_import(std::path::Path::new("Pictures/2023"))
            .expect("parent Folder for import failed");
        let parent_folder = parent_folders.last().unwrap();
        assert!(matches!(parent_folder, FolderOpResult::Existing(_)));
        assert_eq!(parent_folder.name(), "2023");
        assert_eq!(parent_folder.parent(), root.id());
        assert_eq!(parent_folder.id(), folder.parent());

        let root_folders = catalog
            .get_folder_for_import(std::path::Path::new("Pictures"))
            .expect("root Folder for import failed");
        let root_folder = root_folders.last().unwrap();
        assert_eq!(root_folder.id(), parent_folder.parent());

        // "Pictures" exist, but not "2025", so both 2025 and 20250816
        // should be created.
        let folders = catalog.get_folder_for_import(std::path::Path::new("Pictures/2025/20250816"));
        assert!(folders.is_ok());
        let folders = folders.unwrap();
        assert_eq!(root.id(), folders[0].id());

        assert_eq!(folders.len(), 3);
        assert!(matches!(folders[0], FolderOpResult::Existing(_)));
        let folder = &folders[0];
        assert_eq!(folder.name(), "Pictures");
        assert!(matches!(folders[1], FolderOpResult::Created(_)));
        let folder = &folders[1];
        assert_eq!(folder.name(), "2025");

        let folder = folders.last().unwrap();
        assert!(matches!(folder, FolderOpResult::Created(_)));
        assert_eq!(folder.name(), "20250816");
        // This should have a parent we created.
        assert!(folder.parent() != 0);
        // And its parent is the root.
        assert!(folder.parent() != root.id());

        // "Pictures2" nor "Pictures2/2025" do exist. The root created
        // in 20250228
        let roots2 = catalog
            .get_folder_for_import(std::path::Path::new("Pictures2/2025/20250228"))
            .expect("root Folder for import failed");
        assert_eq!(roots2.len(), 1);
        let root2 = &roots2[0];
        assert!(matches!(root2, FolderOpResult::Created(_)));
        assert_eq!(root2.name(), "20250228");
        assert_eq!(root2.parent(), 0);

        //
        let roots3 = catalog
            .get_folder_for_import(std::path::Path::new("Pictures2"))
            .expect("root Folder for import failed");
        assert_eq!(roots3.len(), 1);
        let root3 = &roots3[0];
        assert!(matches!(root3, FolderOpResult::Created(_)));
        assert_eq!(root3.name(), "Pictures2");
        assert_eq!(root3.parent(), 0);
        // Check that it was reparented.
        let folder = catalog.get_folder("Pictures2/2025/20250228");
        assert!(folder.is_ok());
        let folder = folder.unwrap();
        // XXX fixme
        assert_ne!(folder.parent(), 0);
    }
}
