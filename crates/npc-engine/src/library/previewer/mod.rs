/*
 * niepce - library/previewer/mod.rs
 *
 * Copyright (C) 2023 Hubert Figuière
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

mod cache;

use num_derive::{FromPrimitive, ToPrimitive};

use crate::db;
pub(crate) use cache::{Cache, DbMessage};
use npc_fwk::base::Size;
use npc_fwk::toolkit::ImageBitmap;

/// The message for the renderers.
pub enum RenderMsg {
    /// The the image for the processors.
    SetImage(Option<db::LibFile>),
    /// Reload with processing params
    Reload(Option<RenderParams>),
    /// Get the bitmap and call the lambda with the result.
    GetBitmap(Box<dyn Fn(ImageBitmap) + Send>),
}

/// The sender type for renderers.
pub type RenderSender = std::sync::mpsc::Sender<RenderMsg>;

#[derive(Clone, Copy, FromPrimitive, PartialEq, ToPrimitive)]
/// The type of a requested rendering.
///
/// The numeric values are important for the cache backend.
pub(super) enum RenderType {
    /// Plain thumbnail as extracted from the file. Include Exif and other (RAW).
    Thumbnail = 1,
    /// Preview of the image, renderered from original. This will run the image
    /// through the pipeline.
    Preview = 2,
}

impl RenderType {
    pub fn key(&self) -> &'static str {
        match self {
            Self::Thumbnail => "THUMBNAIL",
            Self::Preview => "PREVIEW",
        }
    }
}

/// The rendering parameters.
#[derive(Clone)]
pub struct RenderParams {
    type_: RenderType,
    /// Output dimensions
    /// Note for consistency if type is `RenderingType::Thumbnail` then
    /// dimensions should be a square.
    pub(super) dimensions: Size,
    id: db::LibraryId,
}

impl RenderParams {
    pub fn new_thumbnail(id: db::LibraryId, dimensions: Size) -> RenderParams {
        RenderParams {
            type_: RenderType::Thumbnail,
            dimensions,
            id,
        }
    }

    pub fn new_preview(id: db::LibraryId, dimensions: Size) -> RenderParams {
        RenderParams {
            type_: RenderType::Preview,
            dimensions,
            id,
        }
    }

    pub fn key(&self) -> String {
        format!(
            "{}-{}-{}x{}",
            self.type_.key(),
            self.id,
            self.dimensions.w,
            self.dimensions.h
        )
    }
}
