/*
 * niepce - engine/importer/mod.rs
 *
 * Copyright (C) 2021-2023 Hubert Figuière
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

use num_derive::{FromPrimitive, ToPrimitive};

use crate::db::filebundle::FileBundle;
use crate::db::Managed;
use npc_fwk::toolkit::thumbnail::Thumbnail;
use npc_fwk::utils::FileList;
use npc_fwk::{dbg_out, Date, XmpMeta};

pub fn find_importer(path: &std::path::Path) -> Option<Box<dyn LibraryImporter>> {
    if LrImporter::can_import_library(path) {
        Some(Box::new(LrImporter::new()))
    } else {
        None
    }
}

type SourceContentReady = Box<dyn Fn(Vec<Box<dyn ImportedFile>>) + Send>;
type PreviewReady = Box<dyn Fn(String, Option<Thumbnail>, Option<Date>) + Send>;
type FileImporter = Box<dyn Fn(&Path, &FileList, Managed) + Send>;

/// Trait for file importers backends.
pub trait ImportBackend {
    /// ID of the importer backend.
    fn id(&self) -> &'static str;

    /// List the source content. If possible this should be spawning a
    /// thread. `callback` well be run on that thread.
    fn list_source_content(&self, source: &str, callback: SourceContentReady);
    /// Fetch the previews. If possible this should be spawning a thread. `callback`
    /// well be run on that thread.
    fn get_previews_for(&self, source: &str, paths: Vec<String>, callback: PreviewReady);

    /// Do the import
    fn do_import(&self, source: &str, dest_dir: &Path, callback: FileImporter);
}

/// Date path format for import destination
#[repr(u32)]
#[derive(Clone, Copy, Default, FromPrimitive, ToPrimitive)]
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
pub struct Importer {
    recursive: bool,
    source: PathBuf,
}

impl Importer {
    pub fn from_dir(dir: &Path) -> Self {
        Importer {
            source: dir.into(),
            recursive: false,
        }
    }

    /// Builder: set the import in recursive mode.
    pub fn set_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Determine the destination dir based on the date format.
    pub fn dest_dir_for_date(base: &Path, date: &Date, format: DatePathFormat) -> PathBuf {
        let mut dest_dir = PathBuf::from(base);

        use DatePathFormat::*;

        if let Some(d) = match format {
            NoPath => None,
            YearMonthDay => Some(date.format("%Y%m%d")),
            YearSlashMonthDay => Some(date.format("%Y/%m%d")),
            YearSlashMonthSlashDay => Some(date.format("%Y/%m/%d")),
            YearSlashYearMonthDay => Some(date.format("%Y/%Y%m%d")),
        } {
            dest_dir.push(d.to_string());
        }

        dest_dir
    }

    /// Get the date from the `source`.
    fn date_from(source: &Path) -> Option<npc_fwk::Date> {
        XmpMeta::new_from_file(source, false)
            .and_then(|xmp| xmp.creation_date())
            .or_else(|| {
                std::fs::metadata(source)
                    .ok()?
                    .created()
                    .map(|created| {
                        dbg_out!("Use the FS date for {source:?}.");
                        Date::from_system_time(created)
                    })
                    .ok()
            })
    }

    /// Get the imports from `source`. It will create the bundles.
    /// It will list the files to import recursively if the imorter
    /// is recursive.
    pub fn get_imports(&self, dest: &Path, format: DatePathFormat) -> Vec<(PathBuf, PathBuf)> {
        let entries =
            FileList::files_from_directory(&self.source, FileList::file_is_media, self.recursive);
        let bundles = FileBundle::filter_bundles(&entries);
        bundles
            .iter()
            .flat_map(|bundle| {
                //
                let date = Self::date_from(bundle.main()).unwrap_or_else(Date::now);
                let dest_dir = Self::dest_dir_for_date(dest, &date, format);
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
    use npc_fwk::Date;

    #[test]
    fn test_dest_dir_for_date() {
        use DatePathFormat::*;

        let date = Date(
            chrono::Utc
                .fix()
                .with_ymd_and_hms(2021, 1, 6, 12, 12, 12)
                .single()
                .expect("Date no constructed"),
        );
        let base_dir = PathBuf::from("/var/home/user/Pictures");

        let expected_dir = base_dir.clone();
        assert_eq!(
            Importer::dest_dir_for_date(&base_dir, &date, NoPath),
            expected_dir
        );

        let mut expected_dir = base_dir.clone();
        expected_dir.push("20210106");
        assert_eq!(
            Importer::dest_dir_for_date(&base_dir, &date, YearMonthDay),
            expected_dir
        );

        let mut expected_dir = base_dir.clone();
        expected_dir.push("2021/0106");
        assert_eq!(
            Importer::dest_dir_for_date(&base_dir, &date, YearSlashMonthDay),
            expected_dir
        );

        let mut expected_dir = base_dir.clone();
        expected_dir.push("2021/01/06");
        assert_eq!(
            Importer::dest_dir_for_date(&base_dir, &date, YearSlashMonthSlashDay),
            expected_dir
        );
        let mut expected_dir = base_dir.clone();
        expected_dir.push("2021/20210106");
        assert_eq!(
            Importer::dest_dir_for_date(&base_dir, &date, YearSlashYearMonthDay),
            expected_dir
        );
    }
}
