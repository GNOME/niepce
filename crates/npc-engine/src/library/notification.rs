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
use npc_fwk::toolkit::ImageBitmap;
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
    pub width: u32,
    pub height: u32,
    pub pix: thumbnail::Thumbnail,
}

#[derive(Clone, Debug)]
pub struct ImageRendered {
    pub id: LibraryId,
    pub image: ImageBitmap,
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
    ImageRendered(ImageRendered),
}

unsafe impl ExternType for LibNotification {
    type Id = type_id!("eng::LibNotification");
    type Kind = cxx::kind::Opaque;
}

impl LibNotification {
    pub fn type_(&self) -> NotificationType {
        match *self {
            LibNotification::AddedFile => NotificationType::AddedFile,
            LibNotification::AddedFiles => NotificationType::AddedFiles,
            LibNotification::AddedFolder(_) => NotificationType::AddedFolder,
            LibNotification::AddedKeyword(_) => NotificationType::AddedKeyword,
            LibNotification::AddedLabel(_) => NotificationType::AddedLabel,
            LibNotification::AddedAlbum(_) => NotificationType::AddedAlbum,
            LibNotification::AddedToAlbum(_, _) => NotificationType::AddedToAlbum,
            LibNotification::RemovedFromAlbum(_, _) => NotificationType::RemovedFromAlbum,
            LibNotification::AlbumCounted(_) => NotificationType::AlbumCounted,
            LibNotification::AlbumCountChanged(_) => NotificationType::AlbumCountChange,
            LibNotification::AlbumContentQueried(_) => NotificationType::AlbumContentQueried,
            LibNotification::AlbumDeleted(_) => NotificationType::AlbumDeleted,
            LibNotification::AlbumRenamed(..) => NotificationType::AlbumRenamed,
            LibNotification::FileMoved(_) => NotificationType::FileMoved,
            LibNotification::FileStatusChanged(_) => NotificationType::FileStatusChanged,
            LibNotification::FolderContentQueried(_) => NotificationType::FolderContentQueried,
            LibNotification::FolderCounted(_) => NotificationType::FolderCounted,
            LibNotification::FolderCountChanged(_) => NotificationType::FolderCountChange,
            LibNotification::FolderDeleted(_) => NotificationType::FolderDeleted,
            LibNotification::KeywordContentQueried(_) => NotificationType::KeywordContentQueried,
            LibNotification::KeywordCounted(_) => NotificationType::KeywordCounted,
            LibNotification::KeywordCountChanged(_) => NotificationType::KeywordCountChange,
            LibNotification::LabelChanged(_) => NotificationType::LabelChanged,
            LibNotification::LabelDeleted(_) => NotificationType::LabelDeleted,
            LibNotification::LibCreated => NotificationType::NewLibraryCreated,
            LibNotification::DatabaseNeedUpgrade(_) => NotificationType::DatabaseNeedUpgrade,
            LibNotification::DatabaseReady => NotificationType::DatabaseReady,
            LibNotification::MetadataChanged(_) => NotificationType::MetadataChanged,
            LibNotification::MetadataQueried(_) => NotificationType::MetadataQueried,
            LibNotification::XmpNeedsUpdate => NotificationType::XmpNeedsUpdate,
            LibNotification::ThumbnailLoaded(_) => NotificationType::ThumbnailLoaded,
            LibNotification::ImageRendered(_) => NotificationType::ImageRendered,
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
