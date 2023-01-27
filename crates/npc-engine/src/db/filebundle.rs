/*
 * niepce - engine/db/filebundle.rs
 *
 * Copyright (C) 2017-2023 Hubert Figuière
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

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::db::libfile::FileType;
use npc_fwk::toolkit::mimetype::{IsRaw, MType};
use npc_fwk::MimeType;
use npc_fwk::{dbg_out, err_out};

/// Sidecar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Sidecar {
    Invalid,
    /// Sidecar for Live image (MOV file form iPhone)
    Live(PathBuf),
    /// Thumbnail file (THM from Canon)
    Thumbnail(PathBuf),
    /// XMP Sidecar
    Xmp(PathBuf),
    /// JPEG Sidecar (RAW + JPEG)
    Jpeg(PathBuf),
}

impl Sidecar {
    pub fn to_int(&self) -> i32 {
        match *self {
            Sidecar::Live(_) => 1,
            Sidecar::Thumbnail(_) => 2,
            Sidecar::Xmp(_) => 3,
            Sidecar::Jpeg(_) => 4,
            Sidecar::Invalid => 0,
        }
    }
}

impl From<(i32, PathBuf)> for Sidecar {
    fn from(t: (i32, PathBuf)) -> Self {
        match t.0 {
            1 => Sidecar::Live(t.1),
            2 => Sidecar::Thumbnail(t.1),
            3 => Sidecar::Xmp(t.1),
            4 => Sidecar::Jpeg(t.1),
            _ => Sidecar::Invalid,
        }
    }
}

/// FileBundle is a set of physical files group as one item.
/// Mostly sticking to the DCF specification.
#[derive(Clone)]
pub struct FileBundle {
    /// Type of bundle
    bundle_type: FileType,
    /// Main file.
    main: PathBuf,
    /// XMP sidecar if it exists.
    xmp_sidecar: PathBuf,
    /// JPEG alternate for RAW_JPEG
    jpeg: PathBuf,
    /// Other sidecars: Live, Thumbnail
    sidecars: Vec<Sidecar>,
}

impl Default for FileBundle {
    fn default() -> Self {
        Self::new()
    }
}

/// A file bundle represent files that are together based on their
/// basename.
impl FileBundle {
    pub fn new() -> FileBundle {
        FileBundle {
            bundle_type: FileType::Unknown,
            main: PathBuf::new(),
            xmp_sidecar: PathBuf::new(),
            jpeg: PathBuf::new(),
            sidecars: vec![],
        }
    }

    /// Filter the file list and turn them to bundles
    ///
    pub fn filter_bundles(files: &[PathBuf]) -> Vec<FileBundle> {
        let mut bundles: Vec<FileBundle> = vec![];
        let mut sorted_files: Vec<&PathBuf> = files.iter().collect();
        sorted_files.sort();
        let mut current_base = OsString::new();
        let mut current_bundle: Option<FileBundle> = None;

        for path in sorted_files {
            if let Some(basename) = path.file_stem() {
                let mut basename = basename.to_os_string();
                while basename != current_base {
                    let path2 = PathBuf::from(&basename);
                    match path2.file_stem() {
                        None => break,
                        Some(b) => {
                            if basename == b {
                                break;
                            }
                            basename = b.to_os_string();
                        }
                    }
                }
                if basename == current_base {
                    if !current_bundle.as_mut().unwrap().add(path) {
                        err_out!("FileBundle add to existing bundle failed for {:?}", path);
                    }
                    continue;
                }
                if let Some(current_bundle) = current_bundle {
                    bundles.push(current_bundle);
                }
                let mut bundle = FileBundle::new();
                if bundle.add(path) {
                    current_base = basename;
                    current_bundle = Some(bundle);
                } else {
                    err_out!("FileBundle add to new bundle failed for {:?}", path);
                    // adding to the bundle failed, we'll skip this.
                    current_bundle = None;
                }
            }
        }
        if let Some(current_bundle) = current_bundle {
            bundles.push(current_bundle);
        }
        bundles
    }

    pub fn add<P: AsRef<Path> + std::fmt::Debug>(&mut self, p: P) -> bool {
        let path = p.as_ref();
        dbg_out!("FileBundle::add path {:?}", path);
        let mime_type = MimeType::new(path);
        let mut added = true;

        match mime_type.mime_type() {
            MType::Image(is_raw) => match is_raw {
                IsRaw::Yes => {
                    if !self.main.as_os_str().is_empty() && self.jpeg.as_os_str().is_empty() {
                        self.jpeg = self.main.clone();
                        self.bundle_type = FileType::RawJpeg;
                    } else {
                        self.bundle_type = FileType::Raw;
                    }
                    self.main = path.to_path_buf();
                }
                IsRaw::No => {
                    if !self.main.as_os_str().is_empty() {
                        self.jpeg = path.to_path_buf();
                        self.bundle_type = FileType::RawJpeg;
                    } else {
                        self.main = path.to_path_buf();
                        self.bundle_type = FileType::Image;
                    }
                }
            },
            MType::Xmp => self.xmp_sidecar = path.to_path_buf(),
            MType::Movie => match self.bundle_type {
                FileType::Unknown => {
                    self.main = path.to_path_buf();
                    self.bundle_type = FileType::Video;
                }
                FileType::Image => {
                    self.sidecars.push(Sidecar::Live(path.to_path_buf()));
                }
                _ => {
                    dbg_out!("Ignoring movie file {:?}", path);
                    added = false;
                }
            },
            MType::Thumbnail => self.sidecars.push(Sidecar::Thumbnail(path.to_path_buf())),
            _ => {
                dbg_out!("Unknown file {:?} of type {:?}", path, mime_type);
                added = false;
            }
        }
        added
    }

    pub fn bundle_type(&self) -> FileType {
        self.bundle_type
    }

    pub fn main(&self) -> &Path {
        &self.main
    }

    pub fn jpeg(&self) -> &Path {
        &self.jpeg
    }

    pub fn xmp_sidecar(&self) -> &Path {
        &self.xmp_sidecar
    }

    pub fn sidecars(&self) -> &Vec<Sidecar> {
        &self.sidecars
    }
}

#[cfg(test)]
mod test {
    use super::{FileBundle, Sidecar};
    use crate::db::libfile::FileType;
    use std::path::PathBuf;

    #[test]
    fn test_filebundle() {
        let thelist: Vec<PathBuf> = vec![
            PathBuf::from("/foo/bar/img_0001.cr2"),
            PathBuf::from("/foo/bar/img_0001.jpg"),
            PathBuf::from("/foo/bar/img_0001.xmp"),
            PathBuf::from("/foo/bar/dcs_0001.jpg"),
            PathBuf::from("/foo/bar/dcs_0001.nef"),
            PathBuf::from("/foo/bar/dcs_0001.xmp"),
            PathBuf::from("/foo/bar/img_0142.jpg"),
            PathBuf::from("/foo/bar/img_0142.mov"),
            PathBuf::from("/foo/bar/img_0143.mov"),
            PathBuf::from("/foo/bar/img_0143.jpg"),
            PathBuf::from("/foo/bar/img_0144.crw"),
            PathBuf::from("/foo/bar/img_0144.thm"),
            PathBuf::from("/foo/bar/mvi_0145.mov"),
            PathBuf::from("/foo/bar/mvi_0145.thm"),
            PathBuf::from("/foo/bar/scs_3445.jpg"),
            PathBuf::from("/foo/bar/scs_3445.raf"),
            PathBuf::from("/foo/bar/scs_3445.jpg.xmp"),
            PathBuf::from("/foo/bar/scs_3446.jpg"),
            PathBuf::from("/foo/bar/scs_3446.raf"),
            PathBuf::from("/foo/bar/scs_3446.raf.pp3"),
            // This file is invalid and should cause the bundle to be rejected.
            // This would be number 9.
            // Case occur when the mime type detection returns None.
            PathBuf::from("/foo/bar/some_file.invalid"),
        ];
        let bundles_list = FileBundle::filter_bundles(&thelist);

        assert_eq!(bundles_list.len(), 8);

        let mut iter = bundles_list.iter();
        if let Some(b) = iter.next() {
            assert_eq!(b.bundle_type(), FileType::RawJpeg);
            assert_eq!(b.main(), PathBuf::from("/foo/bar/dcs_0001.nef"));
            assert_eq!(b.jpeg(), PathBuf::from("/foo/bar/dcs_0001.jpg"));
            assert_eq!(b.xmp_sidecar(), PathBuf::from("/foo/bar/dcs_0001.xmp"));
        } else {
            unreachable!();
        }

        if let Some(b) = iter.next() {
            assert_eq!(b.bundle_type(), FileType::RawJpeg);
            assert_eq!(b.main(), PathBuf::from("/foo/bar/img_0001.cr2"));
            assert_eq!(b.jpeg(), PathBuf::from("/foo/bar/img_0001.jpg"));
            assert_eq!(b.xmp_sidecar(), PathBuf::from("/foo/bar/img_0001.xmp"));
        } else {
            unreachable!();
        }

        if let Some(b) = iter.next() {
            assert_eq!(b.bundle_type(), FileType::Image);
            assert_eq!(b.main(), PathBuf::from("/foo/bar/img_0142.jpg"));
            assert!(b.jpeg().as_os_str().is_empty());
            assert!(b.xmp_sidecar().as_os_str().is_empty());
            assert_eq!(b.sidecars.len(), 1);
            assert_eq!(
                b.sidecars[0],
                Sidecar::Live(PathBuf::from("/foo/bar/img_0142.mov"))
            );
        } else {
            unreachable!();
        }

        if let Some(b) = iter.next() {
            assert_eq!(b.bundle_type(), FileType::Image);
            assert_eq!(b.main(), PathBuf::from("/foo/bar/img_0143.jpg"));
            assert!(b.jpeg().as_os_str().is_empty());
            assert!(b.xmp_sidecar().as_os_str().is_empty());
            assert_eq!(b.sidecars.len(), 1);
            assert_eq!(
                b.sidecars[0],
                Sidecar::Live(PathBuf::from("/foo/bar/img_0143.mov"))
            );
        } else {
            unreachable!();
        }

        if let Some(b) = iter.next() {
            assert_eq!(b.bundle_type(), FileType::Raw);
            assert_eq!(b.main(), PathBuf::from("/foo/bar/img_0144.crw"));
            assert!(b.jpeg().as_os_str().is_empty());
            assert!(b.xmp_sidecar().as_os_str().is_empty());
            assert_eq!(b.sidecars.len(), 1);
            assert_eq!(
                b.sidecars[0],
                Sidecar::Thumbnail(PathBuf::from("/foo/bar/img_0144.thm"))
            );
        } else {
            unreachable!();
        }

        if let Some(b) = iter.next() {
            assert_eq!(b.bundle_type(), FileType::Video);
            assert_eq!(b.main(), PathBuf::from("/foo/bar/mvi_0145.mov"));
            assert!(b.jpeg().as_os_str().is_empty());
            assert!(b.xmp_sidecar().as_os_str().is_empty());
            assert_eq!(b.sidecars.len(), 1);
            assert_eq!(
                b.sidecars[0],
                Sidecar::Thumbnail(PathBuf::from("/foo/bar/mvi_0145.thm"))
            );
        } else {
            unreachable!();
        }

        if let Some(b) = iter.next() {
            println!(
                "main = {:?}, jpeg = {:?}, xmp_sidecar = {:?}, sidecars =",
                b.main(),
                b.jpeg(),
                b.xmp_sidecar() /*, b.sidecars()*/
            );
            assert_eq!(b.bundle_type(), FileType::RawJpeg);
            assert_eq!(b.main(), PathBuf::from("/foo/bar/scs_3445.raf"));
            assert_eq!(b.jpeg(), PathBuf::from("/foo/bar/scs_3445.jpg"));
            assert_eq!(b.xmp_sidecar(), PathBuf::from("/foo/bar/scs_3445.jpg.xmp"));
            assert_eq!(b.sidecars.len(), 0);
        } else {
            unreachable!();
        }

        if let Some(b) = iter.next() {
            println!(
                "main = {:?}, jpeg = {:?}, xmp_sidecar = {:?}, sidecars =",
                b.main(),
                b.jpeg(),
                b.xmp_sidecar() /*, b.sidecars()*/
            );
            assert_eq!(b.bundle_type(), FileType::RawJpeg);
            assert_eq!(b.main(), PathBuf::from("/foo/bar/scs_3446.raf"));
            assert_eq!(b.jpeg(), PathBuf::from("/foo/bar/scs_3446.jpg"));
            assert!(b.xmp_sidecar().as_os_str().is_empty());
            assert_eq!(b.sidecars.len(), 0);
        } else {
            unreachable!();
        }
    }
}
