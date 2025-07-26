/*
 * niepce - niepce/modules/darkroom/image_canvas.rs
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

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use npc_fwk::{glib, gtk4};

use npc_fwk::toolkit::ImageBitmap;

#[derive(Clone, Copy, Default, glib::Enum)]
#[enum_type(name = "NcrZoomMode")]
enum ZoomMode {
    None = 0,
    #[default]
    Fit,
    Fill,
    /// 100%
    OneOne,
    Custom, // xxx this should carry a value
}

glib::wrapper! {
    pub struct ImageCanvas(
        ObjectSubclass<imp::ImageCanvas>)
        @extends gtk4::DrawingArea, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for ImageCanvas {
    fn default() -> ImageCanvas {
        ImageCanvas::new()
    }
}

impl ImageCanvas {
    pub fn new() -> ImageCanvas {
        glib::Object::new()
    }

    pub fn set_image(&self, image: ImageBitmap) {
        let imp = self.imp();
        imp.request_redisplay();
        imp.image.replace(Some(image));
        self.queue_draw();
    }

    pub fn set_image_none(&self) {
        self.imp().request_redisplay();
        self.imp().image.replace(None);
        self.queue_draw();
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use npc_fwk::{cairo, gdk4, glib, graphene, gtk4};

    use super::ZoomMode;
    use npc_fwk::base::Rect;
    use npc_fwk::toolkit::ImageBitmap;
    use npc_fwk::{dbg_out, on_err_out};

    const IMAGE_INSET: f64 = 6.0;
    const SHADOW_OFFSET: f64 = 3.0;

    lazy_static::lazy_static! {
        static ref ERROR_PLACEHOLDER: gdk4::Texture = gdk4::Texture::from_resource("/net/figuiere/Niepce/pixmaps/niepce-image-generic.png");
        static ref MISSING_PLACEHOLDER: gdk4::Texture = gdk4::Texture::from_resource("/net/figuiere/Niepce/pixmaps/niepce-image-missing.png");
    }

    #[derive(Default)]
    pub struct ImageCanvas {
        need_redisplay: Cell<bool>,
        resized: Cell<bool>,
        zoom_mode: ZoomMode,
        pub(super) image: RefCell<Option<ImageBitmap>>,
    }

    impl ImageCanvas {
        pub(super) fn request_redisplay(&self) {
            self.need_redisplay.set(true);
        }

        fn calc_image_scale(&self, img_w: u32, img_h: u32) -> f64 {
            let obj = self.obj();
            let b_w = obj.width() as f64 - (IMAGE_INSET * 2.0);
            let b_h = obj.height() as f64 - (IMAGE_INSET * 2.0);

            let scale_w = b_w / img_w as f64;
            let scale_h = b_h / img_h as f64;

            scale_w.min(scale_h)
        }

        fn on_draw(this: &gtk4::DrawingArea, ctx: &cairo::Context, w: i32, h: i32) {
            if let Some(this) = this.downcast_ref::<super::ImageCanvas>() {
                this.imp().on_draw_(ctx, w, h);
            }
        }

        fn on_draw_(&self, context: &cairo::Context, _: i32, _: i32) {
            if self.need_redisplay.get() || self.resized.get() {
                self.redisplay();

                let mut img_w = ERROR_PLACEHOLDER.width() as u32;
                let mut img_h = ERROR_PLACEHOLDER.height() as u32;
                let texture = {
                    self.image
                        .borrow()
                        .as_ref()
                        .map(|image| {
                            img_w = image.original_width();
                            img_h = image.original_height();
                            // query the image.
                            image.to_gdk_texture()
                        })
                        .or_else(|| Some(MISSING_PLACEHOLDER.clone()))
                };

                dbg_out!("texture? {}", texture.is_some());

                let obj = self.obj();
                let canvas_h = obj.height();
                let canvas_w = obj.width();
                dbg_out!("canvas w = {canvas_w} ; h = {canvas_h}");

                dbg_out!("image w = {img_w} ; h = {img_h}");
                let scale = self.calc_image_scale(img_w, img_h);
                dbg_out!("scale = {scale}");

                let out_w = img_w as f64 * scale;
                let out_h = img_h as f64 * scale;
                let x = (canvas_w as f64 - out_w) / 2.0;
                let y = (canvas_h as f64 - out_h) / 2.0;
                dbg_out!("x = {x} ; y = {y}");

                context.rectangle(x + SHADOW_OFFSET, y + SHADOW_OFFSET + 1.0, out_w, out_h);
                context.set_source_rgb(0.0, 0.0, 0.0);
                on_err_out!(context.fill());

                if let Some(ref texture) = texture {
                    let snapshot = gtk4::Snapshot::new();
                    snapshot.translate(&graphene::Point::new(x as f32, y as f32));
                    texture.snapshot(&snapshot, out_w, out_h);
                    if let Some(node) = snapshot.to_node() {
                        node.draw(context);
                    }
                }
            }

            self.need_redisplay.set(false);
            self.resized.set(false);
        }

        /// Recalculate the display frame.
        fn redisplay(&self) -> Option<Rect> {
            if let Some(image) = self.image.borrow().as_ref() {
                let img_w = image.original_width();
                let img_h = image.original_height();
                dbg_out!("set image w {img_w} h {img_h}");

                let obj = self.obj();
                let dest = Rect::new(0, 0, (obj.width() - 8) as u32, (obj.height() - 8) as u32);
                let source = Rect::new(0, 0, img_w, img_h);
                let frame = match self.zoom_mode {
                    ZoomMode::Fit => source.fit_into(&dest),
                    ZoomMode::Fill => source.fill_into(&dest),
                    _ => source,
                };

                Some(frame)
            } else {
                None
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ImageCanvas {
        const NAME: &'static str = "NcrImageCanvas";
        type Type = super::ImageCanvas;
        type ParentType = gtk4::DrawingArea;
    }

    impl ObjectImpl for ImageCanvas {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.set_draw_func(Self::on_draw);
            obj.connect_resize(|this, _, _| {
                this.imp().resized.set(true);
            });
        }
    }

    impl DrawingAreaImpl for ImageCanvas {}

    impl WidgetImpl for ImageCanvas {}
}
