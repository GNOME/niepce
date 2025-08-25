/*
 * niepce - npc_craw/pipeline/ncr.rs
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

/*! Niepce Camera Raw pipeline */

use std::cell::{Cell, RefCell};

use gegl::Node as GeglNode;
use npc_fwk::gdk_pixbuf;

use npc_fwk::MimeType;
use npc_fwk::toolkit::ImageBitmap;
use npc_fwk::toolkit::mimetype::{ImgFormat, MType};
use npc_fwk::{dbg_out, err_out};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
enum ImageStatus {
    Unset = 0,
    Loading,
    Loaded,
    Error,
    NotFound,
}

struct PipelineState {
    width: u32,
    height: u32,
    orientation: u32,
    vertical: bool,
    flip: bool,
    tilt: f64,
    graph: Option<GeglNode>,
    rotate_n: Option<GeglNode>,
    scale: Option<GeglNode>,

    pixbuf_cache: Option<gdk_pixbuf::Pixbuf>,
}

impl Default for PipelineState {
    fn default() -> PipelineState {
        PipelineState {
            width: 0,
            height: 0,
            orientation: 0,
            vertical: false,
            flip: false,
            tilt: 0.0,
            graph: None,
            rotate_n: None,
            scale: None,
            pixbuf_cache: None,
        }
    }
}

pub(crate) struct NcrPipeline {
    status: Cell<ImageStatus>,
    state: RefCell<PipelineState>,
}

impl Default for NcrPipeline {
    fn default() -> NcrPipeline {
        NcrPipeline {
            status: Cell::new(ImageStatus::Unset),
            state: RefCell::default(),
        }
    }
}

impl NcrPipeline {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn set_status(&self, status: ImageStatus) {
        self.status.set(status);
    }

    fn status(&self) -> ImageStatus {
        self.status.get()
    }

    fn rotate_node(&self, orientation: u32) -> Option<GeglNode> {
        dbg_out!("rotation is {orientation}");
        {
            let mut state = self.state.borrow_mut();
            match orientation {
                1 => {
                    state.orientation = 0;
                    state.vertical = false;
                    state.flip = false;
                }
                2 => {
                    state.orientation = 0;
                    state.vertical = false;
                    state.flip = true;
                }
                4 => {
                    state.orientation = 180;
                    state.vertical = false;
                    state.flip = true;
                }
                3 => {
                    state.orientation = 180;
                    state.vertical = false;
                    state.flip = false;
                }
                5 => {
                    state.orientation = 270;
                    state.vertical = true;
                    state.flip = true;
                }
                6 => {
                    state.orientation = 270;
                    state.vertical = true;
                    state.flip = false;
                }
                7 => {
                    state.orientation = 90;
                    state.vertical = true;
                    state.flip = true;
                }
                8 => {
                    state.orientation = 90;
                    state.vertical = true;
                    state.flip = false;
                }
                _ => {
                    state.orientation = 0;
                    state.vertical = false;
                    state.flip = false;
                }
            }
            let rotate = state.orientation as f64 + state.tilt;
            state.graph.as_ref().and_then(|graph| {
                graph.new_child(Some("gegl:rotate"), &[("degrees", rotate.into())])
            })
        }
    }

    fn load_dcraw(&self, p: &str) -> Option<GeglNode> {
        self.state
            .borrow()
            .graph
            .as_ref()
            .and_then(|graph| graph.new_child(Some("gegl:raw-load"), &[("path", p.into())]))
    }

    fn scale_node(&self) -> Option<GeglNode> {
        let scale = 1.0_f64;
        self.state.borrow().graph.as_ref().and_then(|graph| {
            graph.new_child(
                Some("gegl:scale-ratio"),
                &[("x", scale.into()), ("y", scale.into())],
            )
        })
    }

    fn prepare_reload(&self) {
        self.set_status(ImageStatus::Loading);
        {
            let mut state = self.state.borrow_mut();
            state.pixbuf_cache = None;
            state.graph = Some(GeglNode::new());
        }
    }

    fn reload_pixbuf(&self, pixbuf: gdk_pixbuf::Pixbuf) {
        self.prepare_reload();

        {
            let mut state = self.state.borrow_mut();
            state.pixbuf_cache = Some(pixbuf.clone());
        }

        let graph = &self.state.borrow().graph;
        let graph = graph.as_ref().unwrap();
        let load_file = graph.new_child(Some("gegl:pixbuf"), &[("pixbuf", pixbuf.into())]);

        self.reload_node(load_file, 0);
    }

    fn reload_node(&self, node: Option<GeglNode>, orientation: u32) {
        if node.is_none() {
            return;
        }
        let node = node.unwrap();

        let rotate_n = self.rotate_node(orientation);
        let scale = self.scale_node();

        node.link_many(&[rotate_n.as_ref().unwrap(), scale.as_ref().unwrap()]);

        {
            let mut state = self.state.borrow_mut();
            state.rotate_n = rotate_n;
            state.scale = scale;
        }

        let rect = node.bounding_box();
        let width = rect.width() as u32;
        let height = rect.height() as u32;

        dbg_out!("width {width} height {height} = status {:?}", self.status());
        if self.status() < ImageStatus::Error && (width == 0 || height == 0) {
            self.set_status(ImageStatus::Error);
        }

        {
            let mut state = self.state.borrow_mut();
            if state.vertical {
                state.width = height;
                state.height = width;
            } else {
                state.width = width;
                state.height = height;
            }
        }
        self.signal_update();
    }

    fn to_buffer(&self, buffer: &mut [u8]) -> bool {
        if self.status() == ImageStatus::Error {
            dbg_out!("status error");
            return false;
        }
        {
            let state = self.state.borrow();
            if state.scale.is_none() {
                dbg_out!("nothing loaded");
                return false;
            }
            dbg_out!("processing");
            if let Some(scale) = &state.scale {
                scale.process();
                let roi = scale.bounding_box();

                let w = roi.width();
                let h = roi.height();
                dbg_out!("w = {w}, h = {h}");

                let format = gegl::babl::Format::from_encoding("R'G'B' u8");
                scale.blit(
                    1.0,
                    &roi,
                    &format,
                    Some(buffer),
                    0,
                    gegl::BlitFlags::CACHE | gegl::BlitFlags::DIRTY,
                );
            }
        }

        self.set_status(ImageStatus::Loaded);
        true
    }

    fn signal_update(&self) {}
}

impl super::Pipeline for NcrPipeline {
    fn output_width(&self) -> u32 {
        let scale = &self.state.borrow().scale;
        scale
            .as_ref()
            .map(|scale| scale.bounding_box())
            .map(|bbox| bbox.width())
            .unwrap_or(0_i32) as u32
    }

    fn output_height(&self) -> u32 {
        let scale = &self.state.borrow().scale;
        scale
            .as_ref()
            .map(|scale| scale.bounding_box())
            .map(|bbox| bbox.height())
            .unwrap_or(0_i32) as u32
    }

    fn rendered_image(&self) -> Option<ImageBitmap> {
        let w = self.output_width();
        let h = self.output_height();
        dbg_out!("rendered image");
        let mut buffer = vec![0; (w * h * 3) as usize];
        let success = self.to_buffer(buffer.as_mut_slice());
        dbg_out!("to buffer {success}");
        if success {
            Some(ImageBitmap::new(buffer, w, h))
        } else {
            err_out!("Failed to get buffer");
            None
        }
    }

    fn reload(&self, path: &str, is_raw: bool, orientation: u32) {
        self.prepare_reload();

        dbg_out!("loading file {path}");
        if !std::path::Path::new(path).exists() {
            self.set_status(ImageStatus::NotFound);
            self.signal_update();
            return;
        }
        let load_file = if !is_raw {
            // We should panic here. Graph is supposed to exist.
            let graph = &self.state.borrow().graph;
            let graph = graph.as_ref().expect("Graph");

            // XXX maybe suboptimal but GEGL doesn't support HEIF
            // XXX without bloat (image magick)
            let mime_type = MimeType::new(std::path::Path::new(path));
            match mime_type.mime_type() {
                MType::Image(ImgFormat::Heif) | MType::Image(ImgFormat::Avif) => {
                    let pixbuf = npc_fwk::toolkit::heif::gdkpixbuf_from_heif(path).ok();
                    graph.new_child(Some("gegl:pixbuf"), &[("pixbuf", pixbuf.into())])
                }
                _ => graph.new_child(Some("gegl:load"), &[("path", path.into())]),
            }
        } else {
            self.load_dcraw(path)
        };
        self.reload_node(load_file, orientation);
    }

    fn set_placeholder(&self, placeholder: gdk_pixbuf::Pixbuf) {
        self.reload_pixbuf(placeholder);
    }
}
