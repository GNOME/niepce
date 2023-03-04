/*
 * niepce - niepce/modules/darkroom/image_canvas.rs
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

use glib::translate::*;
use glib::Cast;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use npc_craw::Image;

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
        @extends gtk4::DrawingArea, gtk4::Widget;
}

impl Default for ImageCanvas {
    fn default() -> ImageCanvas {
        ImageCanvas::new()
    }
}

/// # Safety
/// Pointer dereference. Albeit null is checked.
// cxx
fn on_image_reloaded(data: *const u8) {
    if data.is_null() {
        return;
    }

    let this = data as *const ImageCanvas;
    unsafe {
        (*this).on_image_reloaded();
    }
}

impl ImageCanvas {
    pub fn new() -> ImageCanvas {
        glib::Object::new()
    }

    pub fn set_image(&self, image: cxx::SharedPtr<Image>) {
        self.imp().request_redisplay();
        unsafe {
            image.connect_signal_update(on_image_reloaded, self as *const Self as *const u8);
        }
        self.imp().image.replace(Some(image));
    }

    pub fn set_image_none(&self) {
        self.imp().request_redisplay();
        self.imp().image.replace(None);
    }

    fn on_image_reloaded(&self) {
        self.imp().request_redisplay();
        self.queue_draw();
    }

    // cxx
    pub fn gobj(&self) -> *mut crate::ffi::GtkDrawingArea {
        let gobj: *mut gtk4_sys::GtkDrawingArea =
            self.upcast_ref::<gtk4::DrawingArea>().to_glib_none().0;
        gobj as *mut crate::ffi::GtkDrawingArea
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;

    use super::ZoomMode;
    use npc_craw::Image;
    use npc_fwk::base::Rect;
    use npc_fwk::{dbg_out, err_out, on_err_out};

    const IMAGE_INSET: f64 = 6.0;
    const SHADOW_OFFSET: f64 = 3.0;

    lazy_static::lazy_static! {
        static ref ERROR_PLACEHOLDER: gdk4::Texture = gdk4::Texture::from_resource("/org/gnome/Niepce/pixmaps/niepce-image-generic.png");
        static ref MISSING_PLACEHOLDER: gdk4::Texture = gdk4::Texture::from_resource("/org/gnome/Niepce/pixmaps/niepce-image-missing.png");
    }

    #[derive(Default)]
    pub struct ImageCanvas {
        need_redisplay: Cell<bool>,
        resized: Cell<bool>,
        zoom_mode: ZoomMode,
        pub(super) image: RefCell<Option<cxx::SharedPtr<Image>>>,
        backing_store: RefCell<Option<cairo::Surface>>,
    }

    impl ImageCanvas {
        pub(super) fn request_redisplay(&self) {
            self.need_redisplay.set(true);
        }

        fn calc_image_scale(&self, img_w: i32, img_h: i32) -> f64 {
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

        fn on_draw_(&self, ctx: &cairo::Context, _: i32, _: i32) {
            if self.image.borrow().is_none() {
                dbg_out!("No image");
                return;
            }

            if self.need_redisplay.get() || self.resized.get() {
                self.redisplay();

                let mut img_w = 32_i32;
                let mut img_h = 32_i32;
                let mut scale = 1.0_f64;
                let img_s = {
                    self.image.borrow().as_ref().and_then(|image| {
                        if image.status() < npc_craw::ImageStatus::ERROR {
                            img_w = image.original_width();
                            img_h = image.original_height();
                            dbg_out!("image w = {img_w} ; h = {img_h}");
                            scale = self.calc_image_scale(img_w, img_h);
                            dbg_out!("scale = {scale}");
                            image.set_output_scale(scale);

                            // query the image.
                            unsafe {
                                cairo::ImageSurface::from_raw_full(
                                    image.cairo_surface_for_display()
                                        as *mut cairo_sys::cairo_surface_t,
                                )
                                .ok()
                            }
                        } else {
                            None
                        }
                    })
                };

                let obj = self.obj();
                let canvas_h = obj.height();
                let canvas_w = obj.width();
                dbg_out!("canvas w = {canvas_w} ; h = {canvas_h}");

                let backing_store = img_s
                    .as_ref()
                    .and_then(|s| {
                        s.create_similar(cairo::Content::Color, canvas_w, canvas_h)
                            .ok()
                    })
                    .or_else(|| {
                        cairo::ImageSurface::create(cairo::Format::ARgb32, canvas_w, canvas_h)
                            .ok()
                            .as_deref()
                            .cloned()
                    });
                if let Some(ref backing_store) = backing_store {
                    let context = cairo::Context::new(backing_store)
                        .expect("Failed to create context from surface");

                    let st_ctx = self.obj().style_context();
                    st_ctx.save();
                    st_ctx.set_state(gtk4::StateFlags::NORMAL);
                    gtk4::render_background(
                        &st_ctx,
                        &context,
                        0.0,
                        0.0,
                        canvas_w as f64,
                        canvas_h as f64,
                    );
                    st_ctx.restore();

                    let out_w = img_w as f64 * scale;
                    let out_h = img_h as f64 * scale;
                    let x = (canvas_w as f64 - out_w) / 2.0;
                    let y = (canvas_h as f64 - out_h) / 2.0;
                    dbg_out!("x = {x} ; y = {y}");

                    context.rectangle(x + SHADOW_OFFSET, y + SHADOW_OFFSET + 1.0, out_w, out_h);
                    context.set_source_rgb(0.0, 0.0, 0.0);
                    on_err_out!(context.fill());

                    if let Some(ref img_s) = img_s {
                        on_err_out!(context.set_source_surface(img_s, x, y));
                        on_err_out!(context.paint());
                    } else {
                        dbg_out!("no image loaded");
                        let icon: gdk4::Paintable =
                            if self.image.borrow().as_ref().unwrap().status()
                                == npc_craw::ImageStatus::NOT_FOUND
                            {
                                MISSING_PLACEHOLDER.clone().into()
                            } else {
                                ERROR_PLACEHOLDER.clone().into()
                            };
                        let img_w = icon.intrinsic_width() as f64;
                        let img_h = icon.intrinsic_height() as f64;
                        let snapshot = gtk4::Snapshot::new();
                        icon.snapshot(&snapshot, img_w, img_h);
                        if let Some(node) = snapshot.to_node() {
                            node.draw(&context);
                        }
                    }
                }

                self.backing_store.replace(backing_store);
                self.need_redisplay.set(false);
                self.resized.set(false);
            }

            if let Some(ref surface) = *self.backing_store.borrow() {
                on_err_out!(ctx.set_source_surface(surface, 0., 0.));
                on_err_out!(ctx.paint());
            } else {
                err_out!("Failed to lock onto the surface");
            }
        }

        /// Recalculate the display frame.
        fn redisplay(&self) -> Option<Rect> {
            if let Some(image) = self.image.borrow().as_ref() {
                if image.status() != npc_craw::ImageStatus::LOADED {
                    err_out!("Image is in not loaded - status {:?}", image.status());
                    return None;
                }

                let img_w = image.original_width() as u32;
                let img_h = image.original_height() as u32;
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
