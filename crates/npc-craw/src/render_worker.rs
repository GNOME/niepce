/*
 * niepce - ncr/render_worker.rs
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

use std::cell::RefCell;

use glib::translate::*;

use crate::ImageBitmap;
use npc_engine::db;
use npc_fwk::base::{Worker, WorkerImpl};
use npc_fwk::{dbg_out, err_out, on_err_out};

pub type RenderWorker = Worker<RenderImpl>;

pub enum RenderMsg {
    SetImage(Option<db::LibFile>),
    Reload,
    GetBitmap(glib::Sender<RenderMsg>),
    Bitmap(ImageBitmap),
}

#[derive(Default)]
pub struct RenderImpl {
    imagefile: RefCell<Option<db::LibFile>>,
}

impl RenderImpl {
    pub fn new() -> Self {
        // We are trying to ensure that gegl is inited on the right thread.
        crate::ncr_init();
        Self {
            imagefile: RefCell::new(None),
        }
    }

    fn reload(&self, pipeline: &cxx::SharedPtr<crate::ImagePipeline>) {
        if let Some(file) = self.imagefile.borrow().as_ref() {
            // currently we treat RAW + JPEG as RAW.
            // TODO: have a way to actually choose the JPEG.
            let file_type = file.file_type();
            let is_raw = (file_type == db::FileType::Raw) || (file_type == db::FileType::RawJpeg);
            let path = file.path().to_string_lossy();

            pipeline.reload(&path, is_raw, file.orientation());
        } else if let Ok(p) =
            gdk_pixbuf::Pixbuf::from_resource("/org/gnome/Niepce/pixmaps/niepce-image-generic.png")
        {
            let p: *mut gdk_pixbuf_sys::GdkPixbuf = p.to_glib_none().0;
            unsafe {
                pipeline.reload_pixbuf(p as *mut crate::ffi::GdkPixbuf);
            }
        }
    }
}

impl WorkerImpl for RenderImpl {
    type Message = RenderMsg;
    type State = RendererState;

    fn dispatch(&self, msg: Self::Message, state: &mut RendererState) -> bool {
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
                    return true;
                }
                self.imagefile.replace(file);
            }
            Reload => self.reload(&state.pipeline),
            GetBitmap(sender) => {
                let w = state.pipeline.output_width();
                let h = state.pipeline.output_height();
                let mut buffer = vec![0; (w * h * 4) as usize];
                let success = state.pipeline.to_buffer(buffer.as_mut_slice());
                if success {
                    let bitmap = ImageBitmap::new(buffer, w as u32, h as u32);
                    on_err_out!(sender.send(Bitmap(bitmap)));
                } else {
                    err_out!("Failed to get buffer");
                }
            }
            _ => {}
        };

        true
    }
}

pub struct RendererState {
    pipeline: cxx::SharedPtr<crate::ImagePipeline>,
}

impl Default for RendererState {
    fn default() -> Self {
        Self {
            pipeline: crate::ffi::image_pipeline_new(),
        }
    }
}
