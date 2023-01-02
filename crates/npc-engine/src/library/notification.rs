/*
 * niepce - engine/library/notification.rs
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

use super::queriedcontent::QueriedContent;
use crate::db::libfile::FileStatus;
use crate::db::{Album, Keyword, Label, LibFolder, LibMetadata, LibraryId, NiepceProperties};
use npc_fwk::toolkit::thumbnail;
use npc_fwk::PropertyValue;

/// Library client channel sender, to send `LibNotification`.
pub type LcChannel = async_channel::Sender<LibNotification>;

use cxx::{type_id, ExternType};

// cxx
pub use crate::ffi::NotificationType;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct FileMove {
    pub file: LibraryId,
    pub from: LibraryId,
    pub to: LibraryId,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct FileStatusChange {
    pub id: LibraryId,
    pub status: FileStatus,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Count {
    pub id: LibraryId,
    pub count: i64,
}

#[derive(Clone, Debug)]
pub struct MetadataChange {
    pub id: LibraryId,
    pub meta: NiepceProperties,
    pub value: PropertyValue,
}

impl MetadataChange {
    pub fn new(id: LibraryId, meta: NiepceProperties, value: PropertyValue) -> Self {
        MetadataChange { id, meta, value }
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Thumbnail {
    pub id: LibraryId,
    pub width: i32,
    pub height: i32,
    pub pix: thumbnail::Thumbnail,
}

#[derive(Clone, Debug)]
pub enum LibNotification {
    AddedFile,
    AddedFiles,
    AddedFolder(LibFolder),
    AddedKeyword(Keyword),
    AddedLabel(Label),
    AddedAlbum(Album),
    AddedToAlbum(Vec<LibraryId>, LibraryId),
    RemovedFromAlbum(Vec<LibraryId>, LibraryId),
    AlbumContentQueried(QueriedContent),
    AlbumCounted(Count),
    AlbumCountChanged(Count),
    AlbumDeleted(LibraryId),
    AlbumRenamed(LibraryId, String),
    FileMoved(FileMove),
    FileStatusChanged(FileStatusChange),
    FolderContentQueried(QueriedContent),
    FolderCounted(Count),
    FolderCountChanged(Count),
    FolderDeleted(LibraryId),
    KeywordContentQueried(QueriedContent),
    KeywordCounted(Count),
    KeywordCountChanged(Count),
    LabelChanged(Label),
    LabelDeleted(LibraryId),
    LibCreated,
    DatabaseNeedUpgrade(i32),
    DatabaseReady,
    MetadataChanged(MetadataChange),
    MetadataQueried(LibMetadata),
    XmpNeedsUpdate,
    ThumbnailLoaded(Thumbnail),
}

unsafe impl ExternType for LibNotification {
    type Id = type_id!("eng::LibNotification");
    type Kind = cxx::kind::Opaque;
}

impl LibNotification {
    pub fn type_(&self) -> NotificationType {
        match *self {
            LibNotification::AddedFile => NotificationType::ADDED_FILE,
            LibNotification::AddedFiles => NotificationType::ADDED_FILES,
            LibNotification::AddedFolder(_) => NotificationType::ADDED_FOLDER,
            LibNotification::AddedKeyword(_) => NotificationType::ADDED_KEYWORD,
            LibNotification::AddedLabel(_) => NotificationType::ADDED_LABEL,
            LibNotification::AddedAlbum(_) => NotificationType::ADDED_ALBUM,
            LibNotification::AddedToAlbum(_, _) => NotificationType::ADDED_TO_ALBUM,
            LibNotification::RemovedFromAlbum(_, _) => NotificationType::RemovedFromAlbum,
            LibNotification::AlbumCounted(_) => NotificationType::ALBUM_COUNTED,
            LibNotification::AlbumCountChanged(_) => NotificationType::ALBUM_COUNT_CHANGE,
            LibNotification::AlbumContentQueried(_) => NotificationType::ALBUM_CONTENT_QUERIED,
            LibNotification::AlbumDeleted(_) => NotificationType::AlbumDeleted,
            LibNotification::AlbumRenamed(..) => NotificationType::AlbumRenamed,
            LibNotification::FileMoved(_) => NotificationType::FILE_MOVED,
            LibNotification::FileStatusChanged(_) => NotificationType::FILE_STATUS_CHANGED,
            LibNotification::FolderContentQueried(_) => NotificationType::FOLDER_CONTENT_QUERIED,
            LibNotification::FolderCounted(_) => NotificationType::FOLDER_COUNTED,
            LibNotification::FolderCountChanged(_) => NotificationType::FOLDER_COUNT_CHANGE,
            LibNotification::FolderDeleted(_) => NotificationType::FOLDER_DELETED,
            LibNotification::KeywordContentQueried(_) => NotificationType::KEYWORD_CONTENT_QUERIED,
            LibNotification::KeywordCounted(_) => NotificationType::KEYWORD_COUNTED,
            LibNotification::KeywordCountChanged(_) => NotificationType::KEYWORD_COUNT_CHANGE,
            LibNotification::LabelChanged(_) => NotificationType::LABEL_CHANGED,
            LibNotification::LabelDeleted(_) => NotificationType::LABEL_DELETED,
            LibNotification::LibCreated => NotificationType::NEW_LIBRARY_CREATED,
            LibNotification::DatabaseNeedUpgrade(_) => NotificationType::DatabaseNeedUpgrade,
            LibNotification::DatabaseReady => NotificationType::DatabaseReady,
            LibNotification::MetadataChanged(_) => NotificationType::METADATA_CHANGED,
            LibNotification::MetadataQueried(_) => NotificationType::METADATA_QUERIED,
            LibNotification::XmpNeedsUpdate => NotificationType::XMP_NEEDS_UPDATE,
            LibNotification::ThumbnailLoaded(_) => NotificationType::ThumbnailLoaded,
        }
    }

    pub fn id(&self) -> i64 {
        match *self {
            LibNotification::MetadataChanged(ref changed) => changed.id,
            LibNotification::AlbumDeleted(id) => id,
            LibNotification::FolderDeleted(id) => id,
            LibNotification::LabelDeleted(id) => id,
            LibNotification::FileStatusChanged(ref changed) => changed.id,
            LibNotification::ThumbnailLoaded(ref thumbnail) => thumbnail.id,
            _ => unreachable!(),
        }
    }

    pub fn get_libmetadata(&self) -> &LibMetadata {
        match *self {
            LibNotification::MetadataQueried(ref m) => m,
            _ => unreachable!(),
        }
    }
}
