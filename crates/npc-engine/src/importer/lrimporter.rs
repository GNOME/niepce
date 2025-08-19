/*
 * niepce - npc-engine/importer/lrimporter.rs
 *
 * Copyright (C) 2021-2025 Hubert Figuière
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

use gettextrs::gettext as i18n;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;

use lrcat::{
    Catalog, CatalogVersion, Collection, Folder, Folders, Image, Keyword, KeywordTree, LibraryFile,
    LrId, LrObject,
};

use npc_fwk::{dbg_out, err_out};

use super::libraryimporter::{Error, LibraryImporter, LibraryImporterProbe, Result};
use crate::NiepcePropertyBag;
use crate::catalog::LibraryId;
use crate::catalog::filebundle::FileBundle;
use crate::catalog::props::NiepceProperties as Np;
use crate::catalog::props::NiepcePropertyIdx as NpI;
use crate::libraryclient::{ClientInterface, ClientInterfaceSync, LibraryClient};

/// Library importer for Lightroom™
#[derive(Default)]
pub struct LrImporter {
    /// The Lr Catalog.
    catalog: Option<Catalog>,
    /// map keyword LrId to LibraryId
    folder_map: RefCell<BTreeMap<LrId, (LibraryId, String)>>,
    /// map keyword LrId to LibraryId
    keyword_map: RefCell<BTreeMap<LrId, LibraryId>>,
    /// map collection LrId to album LibraryId
    collection_map: RefCell<BTreeMap<LrId, LibraryId>>,
    /// map files LrId to file LibraryId
    file_map: RefCell<BTreeMap<LrId, LibraryId>>,
    /// map image LrId to file LibraryId
    ///
    /// XXX longer term is to have an image table.
    image_map: RefCell<BTreeMap<LrId, LibraryId>>,

    /// The root folder mapping table
    root_folder_map: BTreeMap<String, String>,
}

impl LrImporter {
    pub fn new() -> LrImporter {
        LrImporter::default()
    }

    /// Import keyword with `id`. `keywords` is all the Lr keywords, `tree`
    /// is the hierarchy tree as returned by `Catalog::load_keywords_tree()`
    fn import_keyword(
        &self,
        id: LrId,
        libclient: &LibraryClient,
        keywords: &BTreeMap<LrId, Keyword>,
        tree: &KeywordTree,
    ) {
        if let Some(keyword) = keywords.get(&id) {
            let nid = libclient.create_keyword_sync(keyword.name.clone());
            self.keyword_map.borrow_mut().insert(id, nid);
            tree.children_for(id).iter().for_each(|child| {
                self.import_keyword(*child, libclient, keywords, tree);
            });
        }
    }

    fn import_folder(&self, folder_id: LrId, path: &str, libclient: &LibraryClient) {
        let folder_name = Path::new(&path)
            .file_name()
            .map(|name| String::from(name.to_string_lossy()))
            .unwrap_or_else(|| i18n("Untitled"));
        let nid = libclient.create_folder_sync(folder_name, Some(path.into()));
        self.folder_map
            .borrow_mut()
            .insert(folder_id, (nid, path.into()));
    }

    fn import_collection(
        &self,
        collection: &Collection,
        images: Option<&Vec<LrId>>,
        libclient: &LibraryClient,
    ) {
        let parent = *self
            .collection_map
            .borrow()
            .get(&collection.parent)
            .unwrap_or(&-1);
        let nid = libclient.create_album_sync(collection.name.clone(), parent);
        self.collection_map
            .borrow_mut()
            .insert(collection.id(), nid);

        if let Some(images) = images {
            dbg_out!("Has images");
            let npc_ids: Vec<LibraryId> = images
                .iter()
                .filter_map(|id| self.image_map.borrow().get(id).cloned())
                .collect();

            dbg_out!("adding {:?} to album {}", npc_ids, nid);
            libclient.add_to_album(&npc_ids, nid);
        }
    }

    fn populate_bundle(file: &LibraryFile, folder_path: &str, bundle: &mut FileBundle) {
        let mut xmp_file: Option<String> = None;
        let mut jpeg_file: Option<String> = None;
        let sidecar_exts = file.sidecar_extensions.split(',');
        sidecar_exts.for_each(|ext| {
            if !ext.is_empty() {
                return;
            }
            if ext.to_lowercase() == "xmp" {
                xmp_file = Some(format!("{}/{}.{}", &folder_path, &file.basename, &ext));
            } else if jpeg_file.is_some() {
                err_out!("JPEG sidecar already set: {}", ext);
            } else {
                jpeg_file = Some(format!("{}/{}.{}", &folder_path, &file.basename, &ext));
            }
        });

        if let Some(jpeg_file) = jpeg_file {
            dbg_out!("Adding JPEG {}", &jpeg_file);
            bundle.add(jpeg_file);
        }
        if let Some(xmp_file) = xmp_file {
            dbg_out!("Adding XMP {}", &xmp_file);
            bundle.add(xmp_file);
        }
    }

    /// Import a library file. `image` is the optional imager from Lr, includes
    /// metadata.
    fn import_library_file(
        &self,
        file: &LibraryFile,
        image: Option<&Image>,
        libclient: &LibraryClient,
    ) {
        if let Some(folder_id) = self.folder_map.borrow().get(&file.folder) {
            let main_file = format!("{}/{}.{}", &folder_id.1, &file.basename, &file.extension);
            let mut bundle = FileBundle::new();
            dbg_out!("Adding {}", &main_file);
            bundle.add(main_file);

            if !file.sidecar_extensions.is_empty() {
                Self::populate_bundle(file, &folder_id.1, &mut bundle);
            }

            let metadata = if let Some(image) = image {
                let mut metadata = NiepcePropertyBag::default();
                metadata.set_value(
                    Np::Index(NpI::NpTiffOrientationProp),
                    image.exif_orientation().into(),
                );
                metadata.set_value(Np::Index(NpI::NpNiepceFlagProp), (image.pick as i32).into());
                metadata.set_value(Np::Index(NpI::NpNiepceXmpPacket), image.xmp.as_str().into());
                Some(metadata)
            } else {
                None
            };

            let nid = libclient.add_bundle_sync(&bundle, folder_id.0);
            if let Some(ref props) = metadata {
                libclient.set_image_properties(nid, props);
            }
            self.file_map.borrow_mut().insert(file.id(), nid);
            if let Some(image) = image {
                self.image_map.borrow_mut().insert(image.id(), nid);
            }
        }
    }

    /// Remap a folder path based on the root folder remapping
    /// If the folder isnt't found, it's the equivalent of
    /// `Folders.resolve_folder_path()`
    fn remap_folder_path(&self, folders: &Folders, folder: &Folder) -> Option<String> {
        folders
            .find_root_folder(folder.root_folder)
            .map(|root_folder| {
                let absolute_path = &root_folder.absolute_path;
                let mut absolute_path = self
                    .root_folder_map
                    .get(absolute_path)
                    .unwrap_or(absolute_path)
                    .to_string();
                absolute_path += &folder.path_from_root;
                absolute_path
            })
    }
}

impl LibraryImporter for LrImporter {
    fn name(&self) -> &'static str {
        "Adobe Lightroom™"
    }

    fn init_importer(&mut self, path: &Path) -> Result<()> {
        let mut catalog = Catalog::new(path);
        if catalog.open().is_err() {
            return Err(Error::UnsupportedFormat);
        }

        self.catalog = Some(catalog);
        Ok(())
    }

    fn import_library(&mut self, libclient: &LibraryClient) -> Result<()> {
        if let Some(ref mut catalog) = self.catalog {
            catalog.load_version();
            if catalog.catalog_version != CatalogVersion::Lr4 {
                return Err(Error::UnsupportedFormat);
            }
            catalog.load_folders();
            catalog.load_keywords_tree();
            catalog.load_keywords();
            catalog.load_library_files();
            catalog.load_images();
            catalog.load_collections();
        } else {
            return Err(Error::NoInput);
        }

        if let Some(ref catalog) = self.catalog {
            let folders = catalog.folders();
            folders
                .folders
                .iter()
                .map(|f| (f.id(), self.remap_folder_path(folders, f)))
                .filter(|p| p.1.is_some())
                .for_each(|path| self.import_folder(path.0, path.1.as_ref().unwrap(), libclient));

            let root_keyword_id = catalog.root_keyword_id;
            let keywords = catalog.keywords();
            let mut keywordtree = KeywordTree::new();
            keywordtree.add_children(keywords);
            self.import_keyword(root_keyword_id, libclient, keywords, &keywordtree);

            let images = catalog.images();
            let image_to_libfile: BTreeMap<LrId, &Image> = images
                .iter()
                .map(|image| (image.root_file, image))
                .collect();
            let library_files = catalog.libfiles();
            library_files.iter().for_each(|library_file| {
                let image = image_to_libfile.get(&library_file.id());
                self.import_library_file(library_file, image.copied(), libclient);
            });

            let collections = catalog.collections();
            collections.iter().for_each(|collection| {
                if !collection.system_only {
                    dbg_out!("Found collection {}", &collection.name);
                    let images = catalog.images_for_collection(collection.id()).ok();
                    dbg_out!(
                        "Found {} images in collection",
                        images.as_ref().map(Vec::len).unwrap_or(0)
                    );
                    self.import_collection(collection, images.as_ref(), libclient);
                }
            });

            Ok(())
        } else {
            Err(Error::NoInput)
        }
    }

    fn root_folders(&mut self) -> Vec<String> {
        if let Some(ref mut catalog) = self.catalog {
            catalog.load_folders();
        }
        if let Some(ref catalog) = self.catalog {
            catalog
                .folders()
                .roots
                .iter()
                .map(|r| {
                    let absolute_path = &r.absolute_path;
                    self.root_folder_map
                        .get(absolute_path)
                        .unwrap_or(absolute_path)
                })
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }

    fn map_root_folder(&mut self, orig: &str, dest: &str) {
        self.root_folder_map
            .insert(orig.to_string(), dest.to_string());
    }
}

impl LibraryImporterProbe for LrImporter {
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

#[cfg(test)]
mod test {
    use super::LrImporter;
    use crate::importer::{LibraryImporter, LibraryImporterProbe};

    #[test]
    fn test_lrimporter() {
        assert!(!LrImporter::can_import_library(std::path::Path::new(
            "/tmp/catalog.aplib"
        )));
        assert!(LrImporter::can_import_library(std::path::Path::new(
            "/tmp/catalog.lrcat"
        )));

        let importer = LrImporter::new();

        assert_eq!(importer.name(), "Adobe Lightroom™");
    }
}
