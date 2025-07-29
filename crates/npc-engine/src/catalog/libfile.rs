/*
 * niepce - engine/db/libfile.rs
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

use std::path::{Path, PathBuf};

use npc_fwk::err_out;
use npc_fwk::glib;

use super::FromDb;
use super::NiepceProperties as Np;
use super::NiepcePropertyIdx as Npi;
use super::fsfile::FsFile;
use super::{LibMetadata, LibraryId};

#[repr(i32)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, glib::Enum)]
#[enum_type(name = "FileStatus")]
/// FileStatus indicate the transient status of the file on the storage.
pub enum FileStatus {
    /// File is OK
    Ok = 0,
    /// File is missing
    Missing = 1,
    /// Invalid
    #[default]
    Invalid = -1,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// The `LibFile` type.
pub enum FileType {
    /// Don't know
    Unknown = 0,
    /// Camera Raw
    Raw = 1,
    /// Bundle of RAW + processed. Don't assume JPEG.
    RawJpeg = 2,
    /// Processed Image
    Image = 3,
    /// Video
    Video = 4,
}

impl From<i32> for FileType {
    fn from(t: i32) -> Self {
        match t {
            0 => FileType::Unknown,
            1 => FileType::Raw,
            2 => FileType::RawJpeg,
            3 => FileType::Image,
            4 => FileType::Video,
            _ => FileType::Unknown,
        }
    }
}

impl From<FileType> for &'static str {
    fn from(v: FileType) -> &'static str {
        match v {
            FileType::Unknown => "Unknown",
            FileType::Raw => "RAW",
            FileType::RawJpeg => "RAW + JPEG",
            FileType::Image => "Image",
            FileType::Video => "Video",
        }
    }
}

impl From<FileType> for i32 {
    fn from(v: FileType) -> i32 {
        match v {
            FileType::Unknown => 0,
            FileType::Raw => 1,
            FileType::RawJpeg => 2,
            FileType::Image => 3,
            FileType::Video => 4,
        }
    }
}

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "LibFile", nullable)]
pub struct LibFile {
    id: LibraryId,
    folder_id: LibraryId,
    name: String,
    main_file: FsFile,
    orientation: u32,
    rating: i32,
    label: i32,
    flag: i32,
    file_type: FileType,
    pub metadata: Option<LibMetadata>,
}

impl LibFile {
    pub fn new(
        id: LibraryId,
        folder_id: LibraryId,
        fs_file_id: LibraryId,
        path: PathBuf,
        name: &str,
    ) -> LibFile {
        let main_file = FsFile::new(fs_file_id, path);
        LibFile {
            id,
            folder_id,
            name: String::from(name),
            main_file,
            orientation: 0,
            rating: 0,
            label: 0,
            flag: 0,
            file_type: FileType::Unknown,
            metadata: None,
        }
    }

    pub fn same(&self, other: &LibFile) -> bool {
        self.id() == other.id()
    }

    pub fn id(&self) -> LibraryId {
        self.id
    }

    pub fn folder_id(&self) -> LibraryId {
        self.folder_id
    }

    pub fn metadata(&self) -> Option<&LibMetadata> {
        self.metadata.as_ref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &Path {
        self.main_file.path()
    }

    pub fn orientation(&self) -> u32 {
        self.orientation
    }
    pub fn set_orientation(&mut self, o: u32) {
        self.orientation = o;
    }

    pub fn rating(&self) -> i32 {
        self.rating
    }
    pub fn set_rating(&mut self, r: i32) {
        self.rating = r;
    }

    pub fn label(&self) -> i32 {
        self.label
    }
    pub fn set_label(&mut self, l: i32) {
        self.label = l;
    }

    pub fn flag(&self) -> i32 {
        self.flag
    }
    pub fn set_flag(&mut self, f: i32) {
        self.flag = f;
    }

    pub fn file_type(&self) -> FileType {
        self.file_type.to_owned()
    }

    pub fn set_file_type(&mut self, ft: FileType) {
        self.file_type = ft;
    }

    pub fn property(&self, idx: Np) -> i32 {
        match idx {
            Np::Index(Npi::NpTiffOrientationProp) => self.orientation() as i32,
            Np::Index(Npi::NpXmpRatingProp) => self.rating(),
            Np::Index(Npi::NpXmpLabelProp) => self.label(),
            Np::Index(Npi::NpNiepceFlagProp) => self.flag(),
            _ => -1,
        }
    }

    pub fn set_property(&mut self, idx: Np, value: i32) {
        match idx {
            Np::Index(Npi::NpTiffOrientationProp) => self.set_orientation(value as u32),
            Np::Index(Npi::NpXmpRatingProp) => self.set_rating(value),
            Np::Index(Npi::NpXmpLabelProp) => self.set_label(value),
            Np::Index(Npi::NpNiepceFlagProp) => self.set_flag(value),
            _ => err_out!("invalid property {:?} - noop", idx),
        };
    }

    /// return an URI of the real path as Glib want this, oftern
    pub fn uri(&self) -> String {
        let mut s = String::from("file://");
        s.push_str(&self.main_file.path().to_string_lossy());
        s
    }
}

impl FromDb for LibFile {
    fn read_db_columns() -> &'static str {
        "files.id,parent_id,fsfiles.path,\
         name,orientation,rating,label,file_type,fsfiles.id,flag"
    }

    fn read_db_tables() -> &'static str {
        "files, fsfiles"
    }

    fn read_db_where_id() -> &'static str {
        "id"
    }

    fn read_from(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        //DBG_ASSERT(dbdrv->get_number_of_columns() == 10, "wrong number of columns");
        let id = row.get(0)?;
        let fid = row.get(1)?;
        let path: String = row.get(2)?;
        let name: String = row.get(3)?;
        let fsfid = row.get(8)?;
        let mut file = LibFile::new(id, fid, fsfid, PathBuf::from(&path), &name);

        file.set_orientation(row.get(4)?);
        file.set_rating(row.get(5)?);
        file.set_label(row.get(6)?);
        file.set_flag(row.get(9)?);
        let file_type: i32 = row.get(7)?;
        file.set_file_type(FileType::from(file_type));

        Ok(file)
    }
}

/**
 * Converts a mimetype, which is expensive to calculate, into a FileType.
 * @param mime The mimetype we want to know as a filetype
 * @return the filetype
 * @todo: add the JPEG+RAW file types.
 */
pub fn mimetype_to_filetype(mime: &npc_fwk::MimeType) -> FileType {
    if mime.is_digicam_raw() {
        FileType::Raw
    } else if mime.is_image() {
        FileType::Image
    } else if mime.is_movie() {
        FileType::Video
    } else {
        FileType::Unknown
    }
}
