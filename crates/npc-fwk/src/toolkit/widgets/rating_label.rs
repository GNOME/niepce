/*
 * niepce - crates/npc-fwk/src/toolkit/widgets/rating_label.rs
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

use libc::c_int;
use std::cell::Cell;

use gdk4::prelude::*;
use gdk_pixbuf::Pixbuf;
use glib::subclass::prelude::*;
use glib::subclass::Signal;
use glib::translate::*;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use once_cell::unsync::Lazy;

struct Pixbufs {
    star: Pixbuf,
    unstar: Pixbuf,
}

const PIXBUFS: Lazy<Pixbufs> = Lazy::new(|| Pixbufs {
    star: Pixbuf::from_resource("/org/gnome/Niepce/pixmaps/niepce-set-star.png").unwrap(),
    unstar: Pixbuf::from_resource("/org/gnome/Niepce/pixmaps/niepce-unset-star.png").unwrap(),
});

glib::wrapper! {
    pub struct RatingLabel(
        ObjectSubclass<RatingLabelPriv>)
        @extends gtk4::DrawingArea, gtk4::Widget;
}

pub struct RatingLabelPriv {
    editable: Cell<bool>,
    rating: Cell<i32>,
}

impl RatingLabelPriv {
    fn set_editable(&self, editable: bool) {
        self.editable.set(editable);
    }

    fn set_rating(&self, rating: i32) {
        self.rating.set(rating);
        let w = self.instance();
        w.queue_draw();
    }

    fn press_event(&self, _gesture: &gtk4::GestureClick, _: i32, x: f64, _: f64) {
        let new_rating = RatingLabel::rating_value_from_hit_x(x);
        if new_rating != self.rating.get() {
            self.set_rating(new_rating);
            self.instance()
                .emit_by_name::<()>("rating-changed", &[&new_rating]);
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for RatingLabelPriv {
    const NAME: &'static str = "RatingLabel";
    type Type = RatingLabel;
    type ParentType = gtk4::DrawingArea;

    fn new() -> Self {
        Self {
            editable: Cell::new(true),
            rating: Cell::new(0),
        }
    }
}

impl ObjectImpl for RatingLabelPriv {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        let click = gtk4::GestureClick::new();
        click.connect_pressed(glib::clone!(@weak obj => move |gesture, n, x, y| {
            let this = Self::from_instance(&obj);
            this.press_event(&gesture, n, x, y);
        }));
        obj.add_controller(&click);

        obj.set_draw_func(&RatingLabel::draw_func);
    }

    fn signals() -> &'static [Signal] {
        use once_cell::sync::Lazy;
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder(
                "rating-changed",
                &[<i32>::static_type().into()],
                <()>::static_type().into(),
            )
            .run_last()
            .build()]
        });
        SIGNALS.as_ref()
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![glib::ParamSpecInt::new(
                "rating",
                "Rating",
                "The rating value",
                0,
                5,
                0,
                glib::ParamFlags::READWRITE,
            )]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "rating" => {
                let rating = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.set_rating(rating);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "rating" => self.rating.get().to_value(),
            _ => unimplemented!(),
        }
    }
}

pub trait RatingLabelExt {
    fn set_rating(&self, rating: i32);
}

impl RatingLabelExt for RatingLabel {
    fn set_rating(&self, rating: i32) {
        let priv_ = RatingLabelPriv::from_instance(self);
        priv_.set_rating(rating);
    }
}

impl RatingLabel {
    pub fn star() -> Pixbuf {
        PIXBUFS.star.clone()
    }

    pub fn unstar() -> Pixbuf {
        PIXBUFS.unstar.clone()
    }

    /// Return the geometry as (width, height)
    pub fn geometry() -> (f32, f32) {
        let star = Self::star();
        (star.width() as f32 * 5.0, star.height() as f32)
    }

    pub fn draw_rating(
        cr: &cairo::Context,
        rating: i32,
        star: &Pixbuf,
        unstar: &Pixbuf,
        x: f32,
        y: f32,
    ) {
        let rating = if rating == -1 { 0 } else { rating };

        let w = star.width() as f32;
        let h = star.height() as f32;
        let mut y = y;
        y -= h;
        let mut x = x;
        for i in 1..=5 {
            if i <= rating {
                cr.set_source_pixbuf(star, x.into(), y.into());
            } else {
                cr.set_source_pixbuf(unstar, x.into(), y.into());
            }
            on_err_out!(cr.paint());
            x += w;
        }
    }

    fn draw_func(widget: &gtk4::DrawingArea, cr: &cairo::Context, _: i32, _: i32) {
        let star = RatingLabel::star();
        let x = 0_f32;
        let y = star.height() as f32;
        let rating = RatingLabelPriv::from_instance(widget.downcast_ref::<RatingLabel>().unwrap())
            .rating
            .get(); // this shouldn't fail.
        RatingLabel::draw_rating(cr, rating, &star, &RatingLabel::unstar(), x, y);
    }

    pub fn rating_value_from_hit_x(x: f64) -> i32 {
        let width: f64 = Self::star().width().into();
        (x / width).round() as i32
    }

    pub fn new(rating: i32, editable: bool) -> Self {
        let obj: Self = glib::Object::new(&[]).expect("Failed to create RatingLabel");

        let priv_ = RatingLabelPriv::from_instance(&obj);
        priv_.set_editable(editable);
        priv_.set_rating(rating);
        obj
    }
}

impl DrawingAreaImpl for RatingLabelPriv {}

impl WidgetImpl for RatingLabelPriv {
    fn measure(
        &self,
        _widget: &Self::Type,
        orientation: gtk4::Orientation,
        _for_size: i32,
    ) -> (i32, i32, i32, i32) {
        let m = match orientation {
            gtk4::Orientation::Horizontal => RatingLabel::star().width() * 5,
            gtk4::Orientation::Vertical => RatingLabel::star().height(),
            _ => -1,
        };

        (m, m, -1, -1)
    }
}

#[no_mangle]
pub extern "C" fn fwk_rating_label_new(rating: c_int, editable: bool) -> *mut gtk4_sys::GtkWidget {
    RatingLabel::new(rating, editable)
        .upcast::<gtk4::Widget>()
        .to_glib_full()
}

/// Set the rating for the %RatingLabel widget
///
/// # Safety
/// Dereference the widget pointer.
#[no_mangle]
pub unsafe extern "C" fn fwk_rating_label_set_rating(
    widget: *mut gtk4_sys::GtkDrawingArea,
    rating: i32,
) {
    let rating_label = gtk4::DrawingArea::from_glib_none(widget)
        .downcast::<RatingLabel>()
        .expect("Not a RatingLabel widget");
    rating_label.set_rating(rating);
}
