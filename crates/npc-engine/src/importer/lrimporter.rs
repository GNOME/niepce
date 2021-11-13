/*
 * niepce - npc-engine/importer/lrimporter.rs
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

use gettextrs::gettext;

use std::collections::BTreeMap;
use std::path::Path;

use lrcat::{Catalog, CatalogVersion, Folder, Keyword, KeywordTree, LibraryFile, LrId, LrObject};

use super::libraryimporter::LibraryImporter;
use crate::db::filebundle::FileBundle;
use crate::db::LibraryId;
use crate::libraryclient::{ClientInterfaceSync, LibraryClient};

/// Library importer for Lightroom
pub struct LrImporter {
    /// map keyword LrId to LibraryId
    folder_map: BTreeMap<LrId, (LibraryId, String)>,
    /// map keyword LrId to LibraryId
    keyword_map: BTreeMap<LrId, LibraryId>,
    /// map collection LrId to album LibraryId
    collection_map: BTreeMap<LrId, LibraryId>,
    /// map files LrId to file LibraryId
    file_map: BTreeMap<LrId, LibraryId>,
}

impl LrImporter {
    /// Import keyword with `id`. `keywords` is all the Lr keywords, `tree`
    /// is the hierarchy tree as returned by `Catalog::load_keywords_tree()`
    fn import_keyword(
        &mut self,
        id: LrId,
        mut libclient: &mut LibraryClient,
        keywords: &BTreeMap<LrId, Keyword>,
        tree: &KeywordTree,
    ) {
        if let Some(keyword) = keywords.get(&id) {
            let nid = libclient.create_keyword_sync(keyword.name.clone());
            self.keyword_map.insert(id, nid);
            tree.children_for(id).iter().for_each(|child| {
                self.import_keyword(*child, &mut libclient, keywords, tree);
            });
        }
    }

    fn import_folder(&mut self, folder: &Folder, path: &str, libclient: &mut LibraryClient) {
        let folder_name = Path::new(&path)
            .file_name()
            .map(|name| String::from(name.to_string_lossy()))
            .unwrap_or_else(|| gettext("Untitled"));
        let nid = libclient.create_folder_sync(folder_name, Some(path.into()));
        self.folder_map.insert(folder.id(), (nid, path.into()));
    }

    fn import_library_file(&mut self, file: &LibraryFile, libclient: &mut LibraryClient) {
        if let Some(folder_id) = self.folder_map.get(&file.folder) {
            let main_file = format!("{}/{}.{}", &folder_id.1, &file.basename, &file.extension);
            let mut xmp_file: Option<String> = None;
            let mut jpeg_file: Option<String> = None;
            let sidecar_exts = file.sidecar_extensions.split(",");
            sidecar_exts.for_each(|ext| {
                if ext.to_lowercase() == "xmp" {
                    xmp_file = Some(format!("{}/{}.{}", &folder_id.1, &file.basename, &ext));
                } else {
                    if jpeg_file.is_some() {
                        err_out!("JPEG sidecar already set: {}", ext);
                    } else {
                        jpeg_file = Some(format!("{}/{}.{}", &folder_id.1, &file.basename, &ext));
                    }
                }
            });

            let mut bundle = FileBundle::new();
            dbg_out!("Adding {}", &main_file);
            bundle.add(main_file);
            if let Some(jpeg_file) = jpeg_file {
                dbg_out!("Adding JPEG {}", &jpeg_file);
                bundle.add(jpeg_file);
            }
            if let Some(xmp_file) = xmp_file {
                dbg_out!("Adding XMP {}", &xmp_file);
                bundle.add(xmp_file);
            }
            let nid = libclient.add_bundle_sync(&bundle, folder_id.0);
            self.file_map.insert(file.id(), nid);
        }
    }
}

impl LibraryImporter for LrImporter {
    fn new() -> LrImporter {
        LrImporter {
            folder_map: BTreeMap::new(),
            keyword_map: BTreeMap::new(),
            collection_map: BTreeMap::new(),
            file_map: BTreeMap::new(),
        }
    }

    fn import_library(&mut self, path: &Path, libclient: &mut LibraryClient) -> bool {
        let mut catalog = Catalog::new(path);
        if !catalog.open() {
            return false;
        }

        catalog.load_version();
        if catalog.catalog_version != CatalogVersion::Lr4 {
            return false;
        }

        let folders = catalog.load_folders();
        folders.folders.iter().for_each(|folder| {
            if let Some(path) = folders.resolve_folder_path(folder) {
                self.import_folder(&folder, &path, libclient);
            }
        });

        let root_keyword_id = catalog.root_keyword_id;
        let keywordtree = catalog.load_keywords_tree();
        let keywords = catalog.load_keywords();
        self.import_keyword(root_keyword_id, libclient, keywords, &keywordtree);

        let collections = catalog.load_collections();
        collections.iter().for_each(|collection| {
            if !collection.system_only {
                dbg_out!("Found collection {}", collection.name);
            }
        });

        let library_files = catalog.load_library_files();
        library_files.iter().for_each(|library_file| {
            self.import_library_file(library_file, libclient);
        });
        true
    }

    /// Detect if this is a Lr catalog
    /// XXX improve it.
    fn can_import_library(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if ext == "lrcat" {
                return true;
            }
        }
        false
    }
}
