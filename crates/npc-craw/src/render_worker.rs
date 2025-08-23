/*
 * niepce - ncr/render_worker.rs
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

use std::cell::RefCell;
use std::ops::Deref;

use npc_fwk::gdk_pixbuf;

use npc_engine::catalog;
use npc_engine::library::{RenderMsg, RenderParams};
use npc_fwk::base::{Worker, WorkerImpl, WorkerStatus};
use npc_fwk::dbg_out;
use npc_fwk::toolkit::ImageBitmap;

use crate::pipeline::Pipeline;

pub type RenderWorker = Worker<RenderImpl>;

#[derive(Default)]
pub struct RenderImpl {
    imagefile: RefCell<Option<catalog::LibFile>>,
}

impl RenderImpl {
    pub fn new() -> Self {
        // We are trying to ensure that gegl is inited on the right thread.
        crate::ncr_init();
        Self {
            imagefile: RefCell::new(None),
        }
    }

    fn reload(&self, pipeline: &dyn Pipeline) {
        if let Some(file) = self.imagefile.borrow().as_ref() {
            // currently we treat RAW + JPEG as RAW.
            // TODO: have a way to actually choose the JPEG.
            let file_type = file.file_type();
            let is_raw =
                (file_type == catalog::FileType::Raw) || (file_type == catalog::FileType::RawJpeg);
            let path = file.path().to_string_lossy();
            dbg_out!("pipeline reload for {path}");
            pipeline.reload(&path, is_raw, file.orientation());
        } else if let Ok(p) = gdk_pixbuf::Pixbuf::from_resource(
            "/net/figuiere/Niepce/pixmaps/niepce-image-generic.png",
        ) {
            pipeline.set_placeholder(p);
        }
    }

    fn render(&self, state: &RendererState) -> Option<ImageBitmap> {
        state.pipeline.as_ref()?.rendered_image()
    }
}

impl WorkerImpl for RenderImpl {
    type Message = RenderMsg;
    type State = RendererState;

    fn dispatch(&self, msg: Self::Message, state: &mut RendererState) -> WorkerStatus {
        use RenderMsg::*;
        match msg {
            SetImage(file) => {
                if file.is_some()
                    && self
                        .imagefile
                        .borrow()
                        .as_ref()
                        .map(|v| v.same(file.as_ref().unwrap()))
                        .unwrap_or(false)
                {
                    dbg_out!("Same image file, doing nothing");
                } else {
                    self.imagefile.replace(file.as_deref().cloned());
                }
            }
            Reload(params) => {
                if state
                    .params
                    .as_ref()
                    .is_none_or(|p2| params.as_ref().is_none_or(|p| p.engine() != p2.engine()))
                {
                    state.pipeline = params.as_ref().and_then(|params| {
                        dbg_out!("creating pipeline, engine is {:?}", params.engine());
                        crate::pipeline::create(params.engine())
                    });
                }
                state.params = params;
                if let Some(ref pipeline) = state.pipeline {
                    self.reload(pipeline.deref());
                }
            }
            GetBitmap(callback) => {
                if let Some(bitmap) = self.render(state) {
                    callback(bitmap);
                }
            }
        };

        WorkerStatus::Continue
    }
}

#[derive(Default)]
pub struct RendererState {
    pipeline: Option<Box<dyn Pipeline>>,
    params: Option<RenderParams>,
}
