/*
 * niepce - npc-engine/importer.rs
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

mod camera_importer;
mod directory_importer;
mod imported_file;
pub mod libraryimporter;
pub mod lrimporter;

pub use camera_importer::CameraImporter;
pub use directory_importer::DirectoryImporter;
pub use imported_file::ImportedFile;
pub use libraryimporter::{LibraryImporter, LibraryImporterProbe};
pub use lrimporter::LrImporter;

use std::path::{Path, PathBuf};
use std::rc::Rc;

use num_derive::{FromPrimitive, ToPrimitive};

use crate::catalog::filebundle::FileBundle;
use npc_fwk::base::Executor;
use npc_fwk::glib;
use npc_fwk::toolkit::thumbnail::Thumbnail;
use npc_fwk::utils::FileList;
use npc_fwk::{Date, DateExt, XmpMeta, dbg_out};

pub fn find_importer(path: &std::path::Path) -> Option<Box<dyn LibraryImporter>> {
    if LrImporter::can_import_library(path) {
        Some(Box::new(LrImporter::new()))
    } else {
        None
    }
}

/// Get the default import destdir. That's if there is none
/// saved in the catalog. Will always return a PathBuf
pub fn default_import_destdir() -> PathBuf {
    glib::user_special_dir(glib::UserDirectory::Pictures)
        .unwrap_or_else(|| PathBuf::from("~/Pictures"))
}

/// An import request
#[derive(Clone)]
pub struct ImportRequest {
    source: String,
    recursive: bool,
    dest: PathBuf,
    /// Which way to sort the pictures.
    sorting: DatePathFormat,
    importer: Rc<dyn ImportBackend>,
}

impl ImportRequest {
    pub fn new<P: AsRef<Path>>(source: String, dest: P, importer: Rc<dyn ImportBackend>) -> Self {
        Self {
            source,
            recursive: false,
            dest: dest.as_ref().to_path_buf(),
            sorting: DatePathFormat::default(),
            importer,
        }
    }

    /// Builder: set the import in recursive mode.
    pub fn set_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Builder: set the import sorting.
    pub fn set_sorting(mut self, sorting: DatePathFormat) -> Self {
        self.sorting = sorting;
        self
    }

    pub fn sorting(&self) -> DatePathFormat {
        self.sorting
    }

    pub fn set_source(mut self, source: &str) -> Self {
        self.source = source.into();
        self
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn dest_dir(&self) -> &Path {
        &self.dest
    }

    pub fn importer(&self) -> &Rc<dyn ImportBackend> {
        &self.importer
    }
}

type SourceContentReady = Box<dyn Fn(Vec<Box<dyn ImportedFile>>) + Send>;
/// Called when a preview is ready. Passing None for the path finishes.
type PreviewReady = Box<dyn Fn(Option<String>, Option<Thumbnail>, Option<Date>) + Send>;
type FileImporter = Box<dyn Fn(&Path, &FileList) + Send>;

/// Trait for file importers backends.
pub trait ImportBackend {
    /// ID of the importer backend.
    fn id(&self) -> &'static str;

    /// List the source content. If possible this should be spawning a
    /// thread. `callback` well be run on that thread.
    fn list_source_content(&self, executor: &Executor, source: &str, callback: SourceContentReady);
    /// Fetch the previews. If possible this should be spawning a thread. `callback`
    /// will be run on that thread.
    fn get_previews_for(
        &self,
        executor: &Executor,
        source: &str,
        paths: Vec<String>,
        callback: PreviewReady,
    );

    /// Do the import. This just copy (if needed) the files to the destination
    /// and call `callback` that should perform the import into the library.
    fn do_import(&self, request: &ImportRequest, callback: FileImporter);
}

/// Date path format for import destination
/// The values are stored in the catalog so it's important to keep
/// them on a compatible matter.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, FromPrimitive, ToPrimitive, PartialEq)]
pub enum DatePathFormat {
    #[default]
    NoPath = 0,
    /// YYYYMMDD
    YearMonthDay = 1,
    /// YYYY/MMDD
    YearSlashMonthDay = 2,
    /// YYYY/MM/DD
    YearSlashMonthSlashDay = 3,
    /// YYYY/YYYYMMDD
    YearSlashYearMonthDay = 4,
}

/// The importer.
pub struct Importer {}

impl Importer {
    /// Determine the destination dir based on the date format.
    /// If format is none, return `base`
    pub fn dest_dir_for_date(base: &Path, date: Option<&Date>, format: DatePathFormat) -> PathBuf {
        use DatePathFormat::*;

        if let Some(date) = date
            && let Some(d) = match format {
                NoPath => None,
                YearMonthDay => Some(date.format("%Y%m%d")),
                YearSlashMonthDay => Some(date.format("%Y/%m%d")),
                YearSlashMonthSlashDay => Some(date.format("%Y/%m/%d")),
                YearSlashYearMonthDay => Some(date.format("%Y/%Y%m%d")),
            }
        {
            base.join(d.to_string())
        } else {
            base.to_path_buf()
        }
    }

    /// Get the date from the `source`.
    fn date_from(source: &Path) -> Option<npc_fwk::Date> {
        XmpMeta::new_from_file(source, false)
            .and_then(|xmp| xmp.creation_date())
            .or_else(|| {
                std::fs::metadata(source)
                    .ok()?
                    .modified()
                    .map(|modified| {
                        let date = Date::from_system_time(modified);
                        dbg_out!("Use the FS date {date:?} for {source:?}.");
                        date
                    })
                    .ok()
            })
    }

    /// Get the imports from `source`. It will create the bundles.  It
    /// will list the files to import recursively if the imorter is
    /// recursive and the `dest` path. They will be sorted out
    /// according to `format`.
    pub fn get_imports(
        source: &Path,
        dest: &Path,
        format: DatePathFormat,
        recursive: bool,
    ) -> Vec<(PathBuf, PathBuf)> {
        let entries =
            FileList::files_from_directory(source, FileList::file_is_media, recursive, None);
        let bundles = FileBundle::filter_bundles(&entries);
        bundles
            .iter()
            .flat_map(|bundle| {
                //
                let date = Self::date_from(bundle.main()).or_else(|| Some(Date::now()));
                let dest_dir = Self::dest_dir_for_date(dest, date.as_ref(), format);
                bundle
                    .all_files()
                    .iter()
                    .filter_map(|file| {
                        let file_dest = dest_dir.clone().join(file.file_name()?);
                        Some((file.clone(), file_dest))
                    })
                    .collect::<Vec<(PathBuf, PathBuf)>>()
            })
            .collect()
    }
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use chrono;
    use chrono::{Offset, TimeZone};

    use super::{DatePathFormat, Importer};

    #[test]
    fn test_dest_dir_for_date() {
        use DatePathFormat::*;

        let date = chrono::Utc
            .fix()
            .with_ymd_and_hms(2021, 1, 6, 12, 12, 12)
            .single()
            .expect("Date no constructed");

        let base_dir = PathBuf::from("/var/home/user/Pictures");

        assert_eq!(
            Importer::dest_dir_for_date(&base_dir, Some(&date), NoPath),
            base_dir
        );
        assert_eq!(
            Importer::dest_dir_for_date(&base_dir, None, YearMonthDay),
            base_dir
        );

        let mut expected_dir = base_dir.clone();
        expected_dir.push("20210106");
        assert_eq!(
            Importer::dest_dir_for_date(&base_dir, Some(&date), YearMonthDay),
            expected_dir
        );

        let mut expected_dir = base_dir.clone();
        expected_dir.push("2021/0106");
        assert_eq!(
            Importer::dest_dir_for_date(&base_dir, Some(&date), YearSlashMonthDay),
            expected_dir
        );

        let mut expected_dir = base_dir.clone();
        expected_dir.push("2021/01/06");
        assert_eq!(
            Importer::dest_dir_for_date(&base_dir, Some(&date), YearSlashMonthSlashDay),
            expected_dir
        );
        let mut expected_dir = base_dir.clone();
        expected_dir.push("2021/20210106");
        assert_eq!(
            Importer::dest_dir_for_date(&base_dir, Some(&date), YearSlashYearMonthDay),
            expected_dir
        );
    }
}
