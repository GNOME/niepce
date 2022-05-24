/*
 * niepce - niepce/ui/library_cell_renderer.rs
 *
 * Copyright (C) 2020-2022 Hubert Figuière
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

use libc::c_void;
use std::cell::{Cell, RefCell};
use std::ptr;

use gdk4::prelude::*;
use gdk4::Texture;
use glib::subclass::prelude::*;
use glib::subclass::Signal;
use glib::translate::*;
use graphene::Rect;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use crate::niepce::ui::image_list_store::StoreLibFile;
use npc_engine::db::libfile::{FileStatus, FileType};
use npc_fwk::base::rgbcolour::RgbColour;
use npc_fwk::toolkit::clickable_cell_renderer::ClickableCellRenderer;
use npc_fwk::toolkit::widgets::rating_label::RatingLabel;
use npc_fwk::{dbg_out, err_out, on_err_out};

const CELL_PADDING: f32 = 4.0;

struct Emblems {
    raw: Texture,
    raw_jpeg: Texture,
    img: Texture,
    video: Texture,
    unknown: Texture,
    status_missing: Texture,
    flag_reject: Texture,
    flag_pick: Texture,
}

lazy_static::lazy_static! {
    static ref EMBLEMS: Emblems = Emblems {
        raw: Texture::from_resource("/org/gnome/Niepce/pixmaps/niepce-raw-fmt.png"),
        raw_jpeg: Texture::from_resource("/org/gnome/Niepce/pixmaps/niepce-rawjpeg-fmt.png"),
        img: Texture::from_resource("/org/gnome/Niepce/pixmaps/niepce-img-fmt.png"),
        video: Texture::from_resource("/org/gnome/Niepce/pixmaps/niepce-video-fmt.png"),
        unknown: Texture::from_resource("/org/gnome/Niepce/pixmaps/niepce-unknown-fmt.png"),
        status_missing: Texture::from_resource("/org/gnome/Niepce/pixmaps/niepce-missing.png"),
        flag_reject: Texture::from_resource("/org/gnome/Niepce/pixmaps/niepce-flag-reject.png"),
        flag_pick: Texture::from_resource("/org/gnome/Niepce/pixmaps/niepce-flag-pick.png"),
    };
}

glib::wrapper! {
    pub struct LibraryCellRenderer(
        ObjectSubclass<LibraryCellRendererPriv>)
        @extends gtk4::CellRenderer;
}

impl LibraryCellRenderer {
    /// Create a library cell renderer.
    /// callback: an optional callback used to get a colour for labels.
    /// callback_data: raw pointer passed as is to the callback.
    pub fn new(callback: Option<GetColourCallback>, callback_data: *const c_void) -> Self {
        let obj: Self = glib::Object::new(&[("mode", &gtk4::CellRendererMode::Activatable)])
            .expect("Failed to create Library Cell Renderer");

        if callback.is_some() {
            let priv_ = LibraryCellRendererPriv::from_instance(&obj);
            priv_.get_colour_callback.replace(callback);
            priv_.callback_data.set(callback_data);
        }

        obj
    }

    /// Create a new thumb renderer, basicall a LibraryCellRender with some options.
    /// Mostly just draw the thumbnail.
    /// Doesn't need the get_colour_callback
    pub fn new_thumb_renderer() -> Self {
        let cell_renderer = Self::new(None, ptr::null());

        cell_renderer.set_pad(0);
        cell_renderer.set_size(100);
        cell_renderer.set_drawborder(false);
        cell_renderer.set_drawemblem(false);
        cell_renderer.set_drawrating(false);
        cell_renderer.set_drawlabel(false);
        cell_renderer.set_drawflag(false);

        cell_renderer
    }

    pub fn pixbuf(&self) -> Option<gdk4::Paintable> {
        let priv_ = LibraryCellRendererPriv::from_instance(self);
        priv_.pixbuf.borrow().clone()
    }
}

/// Option to set for the LibraryCellRenderer
pub trait LibraryCellRendererExt {
    /// Set padding
    fn set_pad(&self, pad: i32);
    /// Set size
    fn set_size(&self, size: i32);
    /// Whether to draw the border
    fn set_drawborder(&self, draw: bool);
    /// Whether to draw the emblem
    fn set_drawemblem(&self, draw: bool);
    /// Whether to draw the rating
    fn set_drawrating(&self, draw: bool);
    /// Whether to draw the label
    fn set_drawlabel(&self, draw: bool);
    /// Whether to draw the flag
    fn set_drawflag(&self, draw: bool);
}

impl LibraryCellRendererExt for LibraryCellRenderer {
    fn set_pad(&self, pad: i32) {
        let priv_ = LibraryCellRendererPriv::from_instance(self);
        priv_.pad.set(pad);
    }
    fn set_size(&self, size: i32) {
        let priv_ = LibraryCellRendererPriv::from_instance(self);
        priv_.size.set(size);
    }
    fn set_drawborder(&self, draw: bool) {
        let priv_ = LibraryCellRendererPriv::from_instance(self);
        priv_.drawborder.set(draw);
    }
    fn set_drawemblem(&self, draw: bool) {
        let priv_ = LibraryCellRendererPriv::from_instance(self);
        priv_.draw_emblem.set(draw);
    }
    fn set_drawrating(&self, draw: bool) {
        let priv_ = LibraryCellRendererPriv::from_instance(self);
        priv_.draw_rating.set(draw);
    }
    fn set_drawlabel(&self, draw: bool) {
        let priv_ = LibraryCellRendererPriv::from_instance(self);
        priv_.draw_label.set(draw);
    }
    fn set_drawflag(&self, draw: bool) {
        let priv_ = LibraryCellRendererPriv::from_instance(self);
        priv_.draw_flag.set(draw);
    }
}

#[derive(Default)]
struct ClickableCell {
    x: i32,
    y: i32,
    hit: bool,
}

impl ClickableCellRenderer for LibraryCellRenderer {
    fn hit(&mut self, x: i32, y: i32) {
        let priv_ = LibraryCellRendererPriv::from_instance(self);
        priv_
            .clickable_cell
            .replace(ClickableCell { x, y, hit: true });
    }

    fn x(&self) -> i32 {
        let priv_ = LibraryCellRendererPriv::from_instance(self);
        priv_.clickable_cell.borrow().x
    }

    fn y(&self) -> i32 {
        let priv_ = LibraryCellRendererPriv::from_instance(self);
        priv_.clickable_cell.borrow().y
    }

    fn is_hit(&self) -> bool {
        let priv_ = LibraryCellRendererPriv::from_instance(self);
        priv_.clickable_cell.borrow().hit
    }

    fn reset_hit(&mut self) {
        let priv_ = LibraryCellRendererPriv::from_instance(self);
        priv_.clickable_cell.borrow_mut().hit = false;
    }
}

/// Callback type to get the label colour.
/// Return false if none is returned.
type GetColourCallback = unsafe extern "C" fn(i32, *mut RgbColour, *const c_void) -> bool;

pub struct LibraryCellRendererPriv {
    pixbuf: RefCell<Option<gdk4::Paintable>>,
    libfile: RefCell<Option<StoreLibFile>>,
    status: Cell<FileStatus>,
    size: Cell<i32>,
    pad: Cell<i32>,
    drawborder: Cell<bool>,
    draw_emblem: Cell<bool>,
    draw_rating: Cell<bool>,
    draw_label: Cell<bool>,
    draw_flag: Cell<bool>,
    draw_status: Cell<bool>,
    clickable_cell: RefCell<ClickableCell>,
    get_colour_callback: RefCell<Option<GetColourCallback>>,
    callback_data: Cell<*const c_void>,
}

impl LibraryCellRendererPriv {
    fn set_status(&self, status: FileStatus) {
        self.status.set(status);
    }

    fn set_libfile(&self, libfile: Option<StoreLibFile>) {
        self.libfile.replace(libfile);
    }

    fn set_pixbuf(&self, pixbuf: Option<gdk4::Paintable>) {
        self.pixbuf.replace(pixbuf);
    }

    fn do_draw_thumbnail_frame(
        &self,
        cr: &cairo::Context,
        pos: &graphene::Point,
        size: &graphene::Size,
    ) {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.rectangle(
            pos.x().into(),
            pos.y().into(),
            size.width() as f64,
            size.height() as f64,
        );
        on_err_out!(cr.stroke());
    }

    fn do_draw_thumbnail(
        &self,
        snapshot: &gtk4::Snapshot,
        pos: &graphene::Point,
        size: &graphene::Size,
        pixbuf: &gdk4::Paintable,
    ) {
        snapshot.save();
        snapshot.translate(pos);
        pixbuf.snapshot(
            snapshot.upcast_ref::<gdk4::Snapshot>(),
            size.width() as f64,
            size.height() as f64,
        );
        snapshot.restore();
    }

    fn do_draw_flag(snapshot: &gtk4::Snapshot, flag: i32, r: &Rect) {
        if flag == 0 {
            return;
        }
        let texture = match flag {
            -1 => EMBLEMS.flag_reject.clone(),
            1 => EMBLEMS.flag_pick.clone(),
            _ => return,
        };

        let w = texture.width() as f32;
        let x = r.x() + r.width() - CELL_PADDING - w;
        let y = r.y() + CELL_PADDING;
        snapshot.save();
        snapshot.translate(&graphene::Point::new(x, y));
        texture.snapshot(
            snapshot.upcast_ref::<gdk4::Snapshot>(),
            texture.width() as f64,
            texture.height() as f64,
        );
        snapshot.restore();
    }

    fn do_draw_status(snapshot: &gtk4::Snapshot, status: FileStatus, r: &Rect) {
        if status == FileStatus::Ok {
            return;
        }
        let x = r.x() + CELL_PADDING;
        let y = r.y() + CELL_PADDING;
        snapshot.save();
        snapshot.translate(&graphene::Point::new(x, y));
        let texture = &EMBLEMS.status_missing;
        texture.snapshot(
            snapshot.upcast_ref::<gdk4::Snapshot>(),
            texture.width() as f64,
            texture.height() as f64,
        );
        snapshot.restore();
    }

    fn do_draw_format_emblem(snapshot: &gtk4::Snapshot, emblem: &Texture, r: &Rect) -> f32 {
        let w = emblem.width() as f32;
        let h = emblem.height() as f32;
        let left = CELL_PADDING + w;
        let x = r.x() + r.width() - left;
        let y = r.y() + r.height() - CELL_PADDING - h;
        snapshot.save();
        snapshot.translate(&graphene::Point::new(x, y));
        emblem.snapshot(snapshot.upcast_ref::<gdk4::Snapshot>(), w as f64, h as f64);
        snapshot.restore();

        left
    }

    fn do_draw_label(cr: &cairo::Context, right: f32, colour: RgbColour, r: &Rect) {
        const LABEL_SIZE: f32 = 15.0;
        let x = r.x() + r.width() - CELL_PADDING - right - CELL_PADDING - LABEL_SIZE;
        let y = r.y() + r.height() - CELL_PADDING - LABEL_SIZE;

        cr.rectangle(x.into(), y.into(), LABEL_SIZE.into(), LABEL_SIZE.into());
        cr.set_source_rgb(1.0, 1.0, 1.0);
        on_err_out!(cr.stroke());
        cr.rectangle(x.into(), y.into(), LABEL_SIZE.into(), LABEL_SIZE.into());
        let rgb: gdk4::RGBA = colour.into();
        cr.set_source_rgba(
            rgb.red().into(),
            rgb.green().into(),
            rgb.blue().into(),
            rgb.alpha().into(),
        );
        on_err_out!(cr.fill());
    }

    fn get_colour(&self, label_id: i32) -> Option<RgbColour> {
        if let Some(f) = *self.get_colour_callback.borrow() {
            unsafe {
                let mut c = RgbColour::default();
                if f(label_id, &mut c, self.callback_data.get()) {
                    return Some(c);
                }
            }
        }
        None
    }
}

#[glib::object_subclass]
impl ObjectSubclass for LibraryCellRendererPriv {
    const NAME: &'static str = "LibraryCellRenderer";
    type Type = LibraryCellRenderer;
    type ParentType = gtk4::CellRenderer;

    fn new() -> Self {
        Self {
            pixbuf: RefCell::new(None),
            libfile: RefCell::new(None),
            status: Cell::new(FileStatus::Ok),
            size: Cell::new(160),
            pad: Cell::new(16),
            drawborder: Cell::new(true),
            draw_emblem: Cell::new(true),
            draw_rating: Cell::new(true),
            draw_label: Cell::new(true),
            draw_flag: Cell::new(true),
            draw_status: Cell::new(true),
            clickable_cell: RefCell::new(ClickableCell::default()),
            get_colour_callback: RefCell::new(None),
            callback_data: Cell::new(ptr::null()),
        }
    }
}

impl ObjectImpl for LibraryCellRendererPriv {
    fn constructed(&self, obj: &LibraryCellRenderer) {
        self.parent_constructed(obj);
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecObject::new(
                    "pixbuf",
                    "Thumbnail",
                    "Thumbnail to Display",
                    gdk4::Paintable::static_type(),
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecBoxed::new(
                    "libfile",
                    "Library File",
                    "File from the library in the cell",
                    StoreLibFile::static_type(),
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecInt::new(
                    "status",
                    "File Status",
                    "Status of the file in the cell",
                    FileStatus::Ok as i32,
                    FileStatus::Missing as i32,
                    FileStatus::Ok as i32,
                    glib::ParamFlags::READWRITE,
                ),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn signals() -> &'static [Signal] {
        use once_cell::sync::Lazy;
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder(
                "rating-changed",
                &[<i64>::static_type().into(), <i32>::static_type().into()],
                <()>::static_type().into(),
            )
            .run_last()
            .build()]
        });

        SIGNALS.as_ref()
    }

    fn set_property(
        &self,
        _obj: &LibraryCellRenderer,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "pixbuf" => {
                let pixbuf = value.get::<gdk4::Paintable>().ok();
                self.set_pixbuf(pixbuf);
            }
            "libfile" => {
                let libfile = value.get::<&StoreLibFile>().map(|f| f.clone()).ok();
                self.set_libfile(libfile);
            }
            "status" => {
                let status: i32 = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.set_status(FileStatus::from(status));
            }
            _ => unimplemented!(),
        }
    }

    fn property(
        &self,
        _obj: &LibraryCellRenderer,
        _id: usize,
        pspec: &glib::ParamSpec,
    ) -> glib::Value {
        match pspec.name() {
            "pixbuf" => self.pixbuf.borrow().to_value(),
            "libfile" => self.libfile.borrow().to_value(),
            "status" => (self.status.get() as i32).to_value(),
            _ => unimplemented!(),
        }
    }
}

impl CellRendererImpl for LibraryCellRendererPriv {
    fn preferred_width<P: IsA<gtk4::Widget>>(
        &self,
        _renderer: &LibraryCellRenderer,
        _widget: &P,
    ) -> (i32, i32) {
        let maxdim: i32 = self.size.get() + self.pad.get() * 2;
        (maxdim, maxdim)
    }

    fn preferred_height<P: IsA<gtk4::Widget>>(
        &self,
        _renderer: &LibraryCellRenderer,
        _widget: &P,
    ) -> (i32, i32) {
        let maxdim: i32 = self.size.get() + self.pad.get() * 2;
        (maxdim, maxdim)
    }

    fn snapshot<P: IsA<gtk4::Widget>>(
        &self,
        _renderer: &Self::Type,
        snapshot: &gtk4::Snapshot,
        widget: &P,
        _background_area: &gdk4::Rectangle,
        cell_area: &gdk4::Rectangle,
        flags: gtk4::CellRendererState,
    ) {
        let self_ = self.instance();
        let xpad = self_.xpad() as f32;
        let ypad = self_.ypad() as f32;

        let mut r = Rect::new(
            cell_area.x() as f32,
            cell_area.y() as f32,
            cell_area.width() as f32,
            cell_area.height() as f32,
        );

        r.offset(xpad, ypad);

        let file = self.libfile.borrow();

        let style_context = widget.style_context();

        style_context.save();
        style_context.set_state(if flags.contains(gtk4::CellRendererState::SELECTED) {
            gtk4::StateFlags::SELECTED
        } else {
            gtk4::StateFlags::NORMAL
        });

        style_context.restore();

        let cr = snapshot.append_cairo(&r);
        if let Some(pixbuf) = self_.pixbuf() {
            let w = pixbuf.intrinsic_width() as f32;
            let h = pixbuf.intrinsic_height() as f32;
            let thumb_size = graphene::Size::new(w, h);
            let offset_x = (self.size.get() as f32 - w) / 2.0;
            let offset_y = (self.size.get() as f32 - h) / 2.0;

            let thumb_pos = graphene::Point::new(
                r.x() + self.pad.get() as f32 + offset_x,
                r.y() + self.pad.get() as f32 + offset_y,
            );
            self.do_draw_thumbnail(snapshot, &thumb_pos, &thumb_size, &pixbuf);

            self.do_draw_thumbnail_frame(&cr, &thumb_pos, &thumb_size);
        }

        if self.draw_rating.get() {
            let rating = match &*file {
                Some(f) => f.0.rating(),
                None => 0,
            };
            let x = r.x() + CELL_PADDING;
            let y = r.y() + r.height() - CELL_PADDING;
            RatingLabel::draw_rating(
                &cr,
                rating,
                &RatingLabel::star(),
                &RatingLabel::unstar(),
                x,
                y,
            );
        }
        if self.draw_flag.get() {
            match &*file {
                Some(f) => Self::do_draw_flag(snapshot, f.0.flag(), &r),
                None => {}
            }
        }

        let status = self.status.get();
        if self.draw_status.get() && status != FileStatus::Ok {
            Self::do_draw_status(snapshot, status, &r);
        }

        if self.draw_emblem.get() {
            let file_type = match &*file {
                Some(f) => f.0.file_type(),
                None => FileType::Unknown,
            };
            let emblem: Texture = match file_type {
                FileType::Raw => EMBLEMS.raw.clone(),
                FileType::RawJpeg => EMBLEMS.raw_jpeg.clone(),
                FileType::Image => EMBLEMS.img.clone(),
                FileType::Video => EMBLEMS.video.clone(),
                FileType::Unknown => EMBLEMS.unknown.clone(),
            };
            let left = Self::do_draw_format_emblem(snapshot, &emblem, &r);

            if self.draw_label.get() {
                let label_id = match &*file {
                    Some(f) => f.0.label(),
                    None => 0,
                };
                if label_id != 0 {
                    if let Some(colour) = self.get_colour(label_id) {
                        Self::do_draw_label(&cr, left, colour, &r);
                    }
                }
            }
        }
    }

    fn activate<P: IsA<gtk4::Widget>>(
        &self,
        _renderer: &LibraryCellRenderer,
        _event: Option<&gdk4::Event>,
        _widget: &P,
        _path: &str,
        _background_area: &gdk4::Rectangle,
        cell_area: &gdk4::Rectangle,
        _flags: gtk4::CellRendererState,
    ) -> bool {
        let mut instance = self.instance().downcast::<LibraryCellRenderer>().unwrap();

        if instance.is_hit() {
            instance.reset_hit();

            // hit test with the rating region
            let xpad = instance.xpad() as f32;
            let ypad = instance.ypad() as f32;
            let mut r = Rect::new(
                cell_area.x() as f32,
                cell_area.y() as f32,
                cell_area.width() as f32,
                cell_area.height() as f32,
            );
            r.offset(xpad, ypad);

            let (rw, rh) = RatingLabel::geometry();
            let rect = Rect::new(
                r.x() + CELL_PADDING,
                r.y() + r.height() - rh - CELL_PADDING,
                rw,
                rh,
            );
            let x = instance.x() as f32;
            let y = instance.y() as f32;
            dbg_out!(
                "r({}, {}, {}, {}) p({}, {})",
                rect.x(),
                rect.y(),
                rect.width(),
                rect.height(),
                x,
                y
            );
            let hit = (rect.x() <= x)
                && (rect.x() + rect.width() >= x)
                && (rect.y() <= y)
                && (rect.y() + rect.height() >= y);
            if !hit {
                dbg_out!("not a hit");
                return false;
            }

            // hit test for the rating value
            let new_rating = RatingLabel::rating_value_from_hit_x((x - rect.x()).into());
            dbg_out!("new_rating {}", new_rating);

            let file = self.libfile.borrow();
            if let Some(f) = &*file {
                if f.0.rating() != new_rating {
                    // emit signal if changed
                    instance.emit_by_name::<()>("rating-changed", &[&f.0.id(), &new_rating]);
                }
            }
            true
        } else {
            false
        }
    }
}

// allow subclassing this
pub trait LibraryCellRendererImpl: CellRendererImpl + 'static {}

/// # Safety
/// Use raw pointers
#[no_mangle]
pub unsafe extern "C" fn npc_library_cell_renderer_new(
    get_colour: Option<unsafe extern "C" fn(i32, *mut RgbColour, *const c_void) -> bool>,
    callback_data: *const c_void,
) -> *mut gtk4_sys::GtkCellRenderer {
    LibraryCellRenderer::new(get_colour, callback_data)
        .upcast::<gtk4::CellRenderer>()
        .to_glib_full()
}
