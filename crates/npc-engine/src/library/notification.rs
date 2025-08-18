/*
 * niepce - engine/library/notification.rs
 *
 * Copyright (C) 2017-2025 Hubert Figuière
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
use crate::catalog::libfile::FileStatus;
use crate::catalog::{Album, Keyword, Label, LibFolder, LibMetadata, LibraryId, NiepceProperties};
use npc_fwk::PropertyValue;
use npc_fwk::toolkit::ImageBitmap;
use npc_fwk::toolkit::thumbnail;

/// Library client channel sender, to send `LibNotification`.
pub type LcChannel = async_channel::Sender<LibNotification>;

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
pub struct FolderReparent {
    pub id: LibraryId,
    pub dest: LibraryId,
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
    FolderReparented(FolderReparent),
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
    MetadataQueried(Box<LibMetadata>),
    XmpNeedsUpdate,
    ThumbnailLoaded(Box<Thumbnail>),
    ImageRendered(ImageRendered),
    Prefs(Vec<(String, String)>),
    PrefChanged(String, String),
}
