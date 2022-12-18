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

use std::cell::{Cell, RefCell};
use std::rc::Weak;

use gdk4::prelude::*;
use gdk4::Texture;
use glib::subclass::prelude::*;
use glib::subclass::Signal;
use graphene::Rect;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use npc_engine::db;
use npc_engine::db::libfile::{FileStatus, FileType, LibFile};
use npc_engine::libraryclient::UIDataProvider;
use npc_fwk::base::rgbcolour::RgbColour;
use npc_fwk::base::Size;
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
    /// The cell renderer is actually a gtk widget as a per the new `GtkGridView`.
    pub struct LibraryCellRenderer(
        ObjectSubclass<LibraryCellRendererPriv>)
        @extends gtk4::Widget;
}

impl LibraryCellRenderer {
    /// Create a library cell renderer.
    /// callback: an optional callback used to get a colour for labels.
    pub fn new(ui_provider: Option<Weak<UIDataProvider>>) -> Self {
        let obj: Self = glib::Object::new(&[]);

        obj.imp().ui_provider.replace(ui_provider);

        obj
    }

    /// Create a `LibraryCellRenderer` with some options for the fil strip.
    /// Mostly just draw the thumbnail.
    pub fn new_thumb_renderer() -> Self {
        let cell_renderer = Self::new(None);

        cell_renderer.set_pad(6);
        cell_renderer.set_size(100);
        cell_renderer.set_drawborder(false);
        cell_renderer.set_drawemblem(false);
        cell_renderer.set_drawrating(false);
        cell_renderer.set_drawlabel(false);
        cell_renderer.set_drawflag(false);

        cell_renderer
    }

    pub fn hit(&self, x: f64, y: f64) -> bool {
        self.imp().hit(x, y)
    }

    pub fn pixbuf(&self) -> Option<gdk4::Paintable> {
        self.imp().pixbuf.borrow().clone()
    }

    pub fn set_height(&self, h: i32) {
        err_out!("set_height {} isn't implemented", h);
    }
}

/// Option to set for the LibraryCellRenderer
pub trait LibraryCellRendererExt {
    /// Set padding
    fn set_pad(&self, pad: u32);
    /// Set size
    fn set_size(&self, size: u32);
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
    fn set_pad(&self, pad: u32) {
        self.imp().pad.set(pad);
    }
    fn set_size(&self, size: u32) {
        self.imp().size.set(size);
    }
    fn set_drawborder(&self, draw: bool) {
        self.imp().drawborder.set(draw);
    }
    fn set_drawemblem(&self, draw: bool) {
        self.imp().draw_emblem.set(draw);
    }
    fn set_drawrating(&self, draw: bool) {
        self.imp().draw_rating.set(draw);
    }
    fn set_drawlabel(&self, draw: bool) {
        self.imp().draw_label.set(draw);
    }
    fn set_drawflag(&self, draw: bool) {
        self.imp().draw_flag.set(draw);
    }
}

pub struct LibraryCellRendererPriv {
    pixbuf: RefCell<Option<gdk4::Paintable>>,
    libfile: RefCell<Option<LibFile>>,
    status: Cell<FileStatus>,
    size: Cell<u32>,
    pad: Cell<u32>,
    drawborder: Cell<bool>,
    draw_emblem: Cell<bool>,
    draw_rating: Cell<bool>,
    draw_label: Cell<bool>,
    draw_flag: Cell<bool>,
    draw_status: Cell<bool>,
    ui_provider: RefCell<Option<Weak<UIDataProvider>>>,
}

impl LibraryCellRendererPriv {
    fn set_status(&self, status: FileStatus) {
        self.status.set(status);
    }

    fn set_libfile(&self, libfile: Option<LibFile>) {
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
        self.ui_provider
            .borrow()
            .as_ref()
            .and_then(|weak| weak.upgrade())
            .map(|ui_provider| ui_provider.colour_for_label(label_id as db::LibraryId))
    }

    /// Test hit on rating and emit the signal if applicable.
    /// Returns `true` if it hits.
    fn hit(&self, x: f64, y: f64) -> bool {
        // if we don't draw the rating, then nothing.
        if !self.draw_rating.get() {
            return false;
        }

        // hit test with the rating region
        let x = x as f32;
        let y = y as f32;
        let allocation = self.instance().allocation();
        let r = Rect::new(
            allocation.x() as f32,
            allocation.y() as f32,
            allocation.width() as f32,
            allocation.height() as f32,
        );

        let (rw, rh) = RatingLabel::geometry();
        let rect = Rect::new(
            r.x() + CELL_PADDING,
            r.y() + r.height() - rh - CELL_PADDING,
            rw,
            rh,
        );

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
            if f.rating() != new_rating {
                // emit signal if changed
                self.instance()
                    .emit_by_name::<()>("rating-changed", &[&f.id(), &new_rating]);
            }
        }
        true
    }
}

#[glib::object_subclass]
impl ObjectSubclass for LibraryCellRendererPriv {
    const NAME: &'static str = "LibraryCellRenderer";
    type Type = LibraryCellRenderer;
    type ParentType = gtk4::Widget;

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
            ui_provider: RefCell::new(None),
        }
    }
}

impl ObjectImpl for LibraryCellRendererPriv {
    fn constructed(&self) {
        self.parent_constructed();

        let gesture = gtk4::GestureClick::new();
        gesture.connect_pressed(move |gesture, n_press, x, y| {
            dbg_out!("list item clicked {}={},{}", n_press, x, y);
            let renderer = gesture
                .widget()
                .downcast::<LibraryCellRenderer>()
                .expect("couldn't get renderer");
            renderer.hit(x, y);
        });
        self.obj().add_controller(&gesture);

        // Drag and drop
        let drag_source = gtk4::DragSource::new();
        drag_source.connect_prepare(
            glib::clone!(@weak self as this => @default-return None, move |source, _, _| {
                source.set_icon(this.pixbuf.borrow().as_ref(), 0, 0);
                let libfile = this.libfile.borrow().clone();
                libfile.map(|libfile| gdk4::ContentProvider::for_value(&libfile.to_value()))
            }),
        );
        self.obj().add_controller(&drag_source);
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
                    LibFile::static_type(),
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecInt::new(
                    "status",
                    "File Status",
                    "Status of the file in the cell",
                    FileStatus::Invalid as i32,
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
            vec![Signal::builder("rating-changed")
                .param_types([<i64>::static_type(), <i32>::static_type()])
                .run_last()
                .build()]
        });

        SIGNALS.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "pixbuf" => {
                let pixbuf = value.get::<gdk4::Paintable>().ok();
                self.set_pixbuf(pixbuf);
            }
            "libfile" => {
                let libfile = value.get::<&LibFile>().map(|f| f.clone()).ok();
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
        self.instance().queue_draw();
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "pixbuf" => self.pixbuf.borrow().to_value(),
            "libfile" => self.libfile.borrow().to_value(),
            "status" => (self.status.get() as i32).to_value(),
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for LibraryCellRendererPriv {
    fn request_mode(&self) -> gtk4::SizeRequestMode {
        gtk4::SizeRequestMode::ConstantSize
    }

    fn measure(&self, _orientation: gtk4::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
        let size = self.size.get() + self.pad.get() * 2;
        (size as i32, size as i32, -1, -1)
    }

    fn snapshot(&self, snapshot: &gtk4::Snapshot) {
        let self_ = self.instance();
        let xpad = 0.0; // self_.xpad() as f32;
        let ypad = 0.0; // self_.ypad() as f32;
        let cell_area = self.instance().allocation();

        let mut r = Rect::new(
            cell_area.x() as f32,
            cell_area.y() as f32,
            cell_area.width() as f32,
            cell_area.height() as f32,
        );

        r.offset(xpad, ypad);

        let file = self.libfile.borrow();

        let cr = snapshot.append_cairo(&r);
        if let Some(pixbuf) = self_.pixbuf() {
            let size = Size {
                w: pixbuf.intrinsic_width() as u32,
                h: pixbuf.intrinsic_height() as u32,
            };
            let size = size.fit_into_square(self.size.get());
            let thumb_size = graphene::Size::new(size.w as f32, size.h as f32);
            let offset_x = (self.size.get() - size.w) as f32 / 2.0;
            let offset_y = (self.size.get() - size.h) as f32 / 2.0;

            let thumb_pos = graphene::Point::new(
                r.x() + self.pad.get() as f32 + offset_x,
                r.y() + self.pad.get() as f32 + offset_y,
            );
            self.do_draw_thumbnail(snapshot, &thumb_pos, &thumb_size, &pixbuf);

            self.do_draw_thumbnail_frame(&cr, &thumb_pos, &thumb_size);
        }

        if self.draw_rating.get() {
            let rating = match &*file {
                Some(f) => f.rating(),
                None => 0,
            };
            let x = r.x() + CELL_PADDING;
            let y = r.y() + r.height() - CELL_PADDING;
            RatingLabel::draw_rating(
                snapshot,
                rating,
                &RatingLabel::star(),
                &RatingLabel::unstar(),
                x,
                y,
            );
        }
        if self.draw_flag.get() {
            match &*file {
                Some(f) => Self::do_draw_flag(snapshot, f.flag(), &r),
                None => {}
            }
        }

        let status = self.status.get();
        if self.draw_status.get() && status != FileStatus::Ok {
            Self::do_draw_status(snapshot, status, &r);
        }

        if self.draw_emblem.get() {
            let file_type = match &*file {
                Some(f) => f.file_type(),
                None => FileType::Unknown,
            };
            let emblem: Texture = match file_type {
                FileType::Raw => EMBLEMS.raw.clone(),
                FileType::RawJpeg => EMBLEMS.raw_jpeg.clone(),
                FileType::Image => EMBLEMS.img.clone(),
                FileType::Video => EMBLEMS.video.clone(),
                FileType::Unknown => EMBLEMS.unknown.clone(),
                _ => unreachable!(),
            };
            let left = Self::do_draw_format_emblem(snapshot, &emblem, &r);

            if self.draw_label.get() {
                let label_id = match &*file {
                    Some(f) => f.label(),
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
}
