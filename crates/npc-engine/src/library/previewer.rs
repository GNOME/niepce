/*
 * niepce - npc-engine/library/previewer.rs
 *
 * Copyright (C) 2023-2025 Hubert Figuière
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

use md5::Digest;
use num_derive::{FromPrimitive, ToPrimitive};

use crate::catalog;
pub(crate) use cache::{Cache, DbMessage};
use npc_fwk::base::Size;
use npc_fwk::err_out;
use npc_fwk::toolkit::ImageBitmap;

type RenderDigest = md5::Md5;

/// Trait for digest on the render params.
trait ParamDigest {
    fn digest_update(&self, digest: &mut RenderDigest);
}

impl ParamDigest for Size {
    fn digest_update(&self, digest: &mut RenderDigest) {
        digest.update(self.w.to_le_bytes());
        digest.update(self.h.to_le_bytes());
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// Render Engine.
pub enum RenderEngine {
    /// Thumbnail extractor
    Thumbnailer,
    #[default]
    /// Niepce Camera Raw
    Ncr,
    /// RT Engine
    Rt,
}

impl RenderEngine {
    /// The key is used for the cache, and for the XMP.
    pub fn key(&self) -> &'static str {
        match self {
            Self::Thumbnailer => "tnail",
            Self::Ncr => "ncr",
            Self::Rt => "rt",
        }
    }

    pub fn from_key(key: &str) -> Option<RenderEngine> {
        match key {
            "tnail" => Some(Self::Thumbnailer),
            "ncr" => Some(Self::Ncr),
            "rt" => Some(Self::Rt),
            _ => None,
        }
    }
}

impl ParamDigest for RenderEngine {
    fn digest_update(&self, digest: &mut RenderDigest) {
        digest.update(self.key().as_bytes());
    }
}

/// The message for the renderers.
pub enum RenderMsg {
    /// The the image for the processors.
    SetImage(Option<Box<catalog::LibFile>>),
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

impl ParamDigest for RenderType {
    fn digest_update(&self, digest: &mut RenderDigest) {
        digest.update(self.key().as_bytes());
    }
}

/// The rendering parameters.
#[derive(Clone)]
pub struct RenderParams {
    pub(super) type_: RenderType,
    engine: RenderEngine,
    /// Output dimensions
    /// Note for consistency if type is `RenderingType::Thumbnail` then
    /// dimensions should be a square.
    pub(super) dimensions: Size,
    id: catalog::LibraryId,
}

impl RenderParams {
    pub fn new_thumbnail(id: catalog::LibraryId, dimensions: Size) -> RenderParams {
        RenderParams {
            type_: RenderType::Thumbnail,
            engine: RenderEngine::Thumbnailer,
            dimensions,
            id,
        }
    }

    pub fn new_preview(
        file: &catalog::LibFile,
        engine: RenderEngine,
        dimensions: Size,
    ) -> RenderParams {
        let id = file.id();
        if file.metadata.is_none() {
            err_out!("new preview, metadata is none");
        }
        RenderParams {
            type_: RenderType::Preview,
            engine,
            dimensions,
            id,
        }
    }

    pub fn set_engine(&mut self, engine: RenderEngine) {
        self.engine = engine;
    }

    pub fn engine(&self) -> RenderEngine {
        self.engine
    }

    pub fn key(&self) -> String {
        self.digest()
    }

    pub fn digest(&self) -> String {
        let mut hasher = RenderDigest::new();
        self.type_.digest_update(&mut hasher);
        self.engine.digest_update(&mut hasher);
        self.dimensions.digest_update(&mut hasher);
        hasher.update(self.id.to_le_bytes());

        let result = hasher.finalize();
        format!("{result:x}")
    }
}

#[cfg(test)]
mod test {
    use super::{RenderEngine, RenderParams};
    use crate::catalog::LibFile;
    use npc_fwk::base::Size;

    #[test]
    fn test_digest() {
        let file = LibFile::new(
            1,
            1,
            1,
            std::path::PathBuf::from("/tmp/image.jpg"),
            "image.jpg",
        );
        let preview1 =
            RenderParams::new_preview(&file, RenderEngine::Ncr, Size { w: 1600, h: 1200 });
        let preview2 =
            RenderParams::new_preview(&file, RenderEngine::Rt, Size { w: 1600, h: 1200 });

        assert_ne!(preview1.digest(), preview2.digest());
    }
}
