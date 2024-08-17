/*
 * niepce - npc_craw/pipeline/rt.rs
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

/*! RawTherapee engine pipeline */

use npc_fwk::gdk_pixbuf;

use npc_fwk::toolkit::ImageBitmap;
use npc_fwk::{dbg_out, err_out, on_err_out};

pub(crate) struct RtPipeline(rtengine::RtEngine);

impl RtPipeline {
    pub(crate) fn new() -> Self {
        Self(rtengine::RtEngine::new())
    }
}

impl super::Pipeline for RtPipeline {
    fn output_width(&self) -> u32 {
        self.0.width() as u32
    }

    fn output_height(&self) -> u32 {
        self.0.height() as u32
    }

    fn rendered_image(&self) -> Option<ImageBitmap> {
        dbg_out!("Rt: rendering");
        self.0
            .process()
            .map_err(|err| {
                err_out!("Rt processing error {err}");
                err
            })
            .ok()
    }

    // Rt doesn't care about orientation.
    fn reload(&self, path: &str, is_raw: bool, _: u32) {
        on_err_out!(self.0.set_file(path, is_raw));
    }

    fn set_placeholder(&self, _placeholder: gdk_pixbuf::Pixbuf) {}
}
