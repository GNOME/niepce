/*
 * niepce - npc_craw/pipeline.rs
 *
 * Copyright (C) 2023-2024 Hubert Figuière
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

/*! Pipeline trait. */

mod ncr;
mod rt;

use npc_fwk::gdk_pixbuf;

use npc_engine::library::RenderEngine;
use npc_fwk::toolkit::ImageBitmap;

/// Pipeline trait.
pub(crate) trait Pipeline {
    fn output_width(&self) -> u32;
    fn output_height(&self) -> u32;
    fn rendered_image(&self) -> Option<ImageBitmap>;
    fn reload(&self, path: &str, is_raw: bool, orientation: u32);
    /// Set a placeholder to display.
    fn set_placeholder(&self, placeholder: gdk_pixbuf::Pixbuf);
}

pub(crate) fn create(engine: RenderEngine) -> Option<Box<dyn Pipeline>> {
    match engine {
        RenderEngine::Thumbnailer => None,
        RenderEngine::Ncr => Some(Box::new(ncr::NcrPipeline::new())),
        RenderEngine::Rt => Some(Box::new(rt::RtPipeline::new())),
    }
}
