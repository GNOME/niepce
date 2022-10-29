/*
 * niepce - niepce/ui/thumb_nav.rs
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
use std::rc::Rc;

use once_cell::unsync::OnceCell;

use glib::subclass::prelude::*;
use glib::translate::*;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use npc_fwk::err_out;

const SCROLL_INC: f64 = 1.;
const SCROLL_MOVE: f64 = 20.;
const SCROLL_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(20);

#[repr(i32)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ThumbNavMode {
    OneRow,
    OneColumn,
    MultipleRows,
    MultipleColumns,
    Invalid,
}

impl From<ThumbNavMode> for i32 {
    fn from(v: ThumbNavMode) -> i32 {
        match v {
            ThumbNavMode::OneRow => 0,
            ThumbNavMode::OneColumn => 1,
            ThumbNavMode::MultipleRows => 2,
            ThumbNavMode::MultipleColumns => 3,
            ThumbNavMode::Invalid => 4,
        }
    }
}

impl From<i32> for ThumbNavMode {
    fn from(value: i32) -> Self {
        match value {
            0 => ThumbNavMode::OneRow,
            1 => ThumbNavMode::OneColumn,
            2 => ThumbNavMode::MultipleRows,
            3 => ThumbNavMode::MultipleColumns,
            _ => ThumbNavMode::Invalid,
        }
    }
}

glib::wrapper! {
    pub struct ThumbNav(
        ObjectSubclass<ThumbNavPriv>)
        @extends gtk4::Box, gtk4::Widget;
}

impl ThumbNav {
    pub fn new(thumbview: &gtk4::IconView, mode: ThumbNavMode, show_buttons: bool) -> Self {
        let mode_n: i32 = mode.into();
        glib::Object::new(&[
            ("mode", &mode_n),
            ("show-buttons", &show_buttons),
            ("thumbview", thumbview),
            ("homogeneous", &false),
            ("spacing", &0),
        ])
    }
}

struct ThumbNavWidgets {
    button_left: gtk4::Button,
    button_right: gtk4::Button,
    sw: gtk4::ScrolledWindow,
}

pub struct ThumbNavPriv {
    mode: Cell<ThumbNavMode>,
    show_buttons: Cell<bool>,

    left_i: Cell<f64>,
    right_i: Cell<f64>,
    widgets: OnceCell<ThumbNavWidgets>,
    thumbview: RefCell<Option<gtk4::IconView>>,
}

pub trait ThumbNavExt {
    /// Get whether we show the left and right scroll buttons.
    fn show_buttons(&self) -> bool;
    /// Set whether we show the left and right scroll buttons.
    fn set_show_buttons(&self, show_buttons: bool);
    /// Get the navigation mode.
    fn mode(&self) -> ThumbNavMode;
    /// Set the navigation mode.
    fn set_mode(&self, mode: ThumbNavMode);
}

impl ThumbNavExt for ThumbNav {
    fn show_buttons(&self) -> bool {
        self.imp().show_buttons.get()
    }

    fn set_show_buttons(&self, show_buttons: bool) {
        self.imp().set_show_buttons(show_buttons);
    }

    fn mode(&self) -> ThumbNavMode {
        self.imp().mode.get()
    }

    fn set_mode(&self, mode: ThumbNavMode) {
        self.imp().set_mode(mode);
    }
}

impl ThumbNavPriv {
    fn left_button_clicked(&self) {
        if let Some(adj) = self.widgets.get().map(|w| w.sw.hadjustment()) {
            let adj = Rc::new(adj);
            let i = self.left_i.clone();
            glib::timeout_add_local(SCROLL_TIMEOUT, move || ThumbNavPriv::scroll_left(&i, &adj));
        }
    }

    fn right_button_clicked(&self) {
        if let Some(adj) = self.widgets.get().map(|w| w.sw.hadjustment()) {
            let adj = Rc::new(adj);
            let i = self.right_i.clone();
            glib::timeout_add_local(SCROLL_TIMEOUT, move || ThumbNavPriv::scroll_right(&i, &adj));
        }
    }

    fn adj_changed(&self, adj: &gtk4::Adjustment) {
        if let Some(widgets) = self.widgets.get() {
            let upper = adj.upper();
            let page_size = adj.page_size();
            widgets.button_right.set_sensitive(upper > page_size)
        }
    }

    fn adj_value_changed(&self, adj: &gtk4::Adjustment) {
        let upper = adj.upper();
        let page_size = adj.page_size();
        let value = adj.value();

        if let Some(w) = self.widgets.get() {
            w.button_left.set_sensitive(value > 0.0);
            w.button_right.set_sensitive(value < upper - page_size);
        }
    }

    fn scroll_left(ref_i: &Cell<f64>, adj: &gtk4::Adjustment) -> glib::Continue {
        let value = adj.value();
        let i = ref_i.get();

        if i == SCROLL_MOVE || value - SCROLL_INC < 0.0 {
            ref_i.set(0.0);
            adj.emit_by_name::<()>("value-changed", &[]);

            return Continue(false);
        }

        ref_i.set(i + 1.0);

        let move_ = f64::min(SCROLL_MOVE, value);
        adj.set_value(value - move_);

        Continue(true)
    }

    fn scroll_right(ref_i: &Cell<f64>, adj: &gtk4::Adjustment) -> glib::Continue {
        let upper = adj.upper();
        let page_size = adj.page_size();
        let value = adj.value();
        let i = ref_i.get();

        if i == SCROLL_MOVE || value + SCROLL_INC > upper - page_size {
            ref_i.set(0.0);
            return Continue(false);
        }

        ref_i.set(i + 1.0);

        let move_ = f64::min(SCROLL_MOVE, upper - page_size - value);
        adj.set_value(value + move_);

        Continue(true)
    }

    fn set_show_buttons(&self, show_buttons: bool) {
        self.show_buttons.set(show_buttons);

        let widgets = &self.widgets.get().unwrap();
        if show_buttons && self.mode.get() == ThumbNavMode::OneRow {
            widgets.button_left.show();
            widgets.button_right.show();
        } else {
            widgets.button_left.hide();
            widgets.button_right.hide();
        }
    }

    fn set_mode(&self, mode: ThumbNavMode) {
        self.mode.set(mode);

        match mode {
            ThumbNavMode::OneRow => {
                if let Some(thumbview) = &*self.thumbview.borrow() {
                    thumbview.set_size_request(-1, -1);
                    // XXX property is gone, need an API
                    // thumbview.set_property("item-height", &100);
                }
                self.widgets
                    .get()
                    .unwrap()
                    .sw
                    .set_policy(gtk4::PolicyType::Always, gtk4::PolicyType::Never);

                self.set_show_buttons(self.show_buttons.get());
            }
            ThumbNavMode::OneColumn
            | ThumbNavMode::MultipleRows
            | ThumbNavMode::MultipleColumns => {
                if let Some(thumbview) = &*self.thumbview.borrow() {
                    thumbview.set_columns(1);

                    thumbview.set_size_request(-1, -1);
                    // XXX property is gone, need an API
                    // thumbview.set_property("item-height", &-1);
                }
                if let Some(widgets) = self.widgets.get() {
                    widgets
                        .sw
                        .set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Always);

                    widgets.button_left.hide();
                    widgets.button_right.hide();
                }
            }
            _ => {}
        }
    }

    fn add_thumbview(&self) {
        if let Some(widgets) = self.widgets.get() {
            widgets.sw.set_child((*self.thumbview.borrow()).as_ref());
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for ThumbNavPriv {
    const NAME: &'static str = "NpcThumbNav";
    type Type = ThumbNav;
    type ParentType = gtk4::Box;

    fn new() -> Self {
        Self {
            mode: Cell::new(ThumbNavMode::OneRow),
            show_buttons: Cell::new(true),
            left_i: Cell::new(0.0),
            right_i: Cell::new(0.0),
            widgets: OnceCell::new(),
            thumbview: RefCell::new(None),
        }
    }
}

impl ObjectImpl for ThumbNavPriv {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.instance();
        let button_left = gtk4::Button::from_icon_name("pan-start-symbolic");
        // XXX
        // button_left.set_relief(gtk4::ReliefStyle::None);
        button_left.set_size_request(20, 0);
        obj.append(&button_left);
        button_left.connect_clicked(glib::clone!(@weak obj => move |_| {
            obj.imp().left_button_clicked();
        }));

        let sw = gtk4::ScrolledWindow::new();
        // XXX
        // sw.set_shadow_type(gtk4::ShadowType::In);
        sw.set_policy(gtk4::PolicyType::Always, gtk4::PolicyType::Never);
        let adj = sw.hadjustment();
        adj.connect_changed(glib::clone!(@weak obj => move |adj| {
            obj.imp().adj_changed(adj);
        }));
        adj.connect_value_changed(glib::clone!(@weak obj => move |adj| {
            obj.imp().adj_value_changed(adj);
        }));
        obj.append(&sw);

        let button_right = gtk4::Button::from_icon_name("pan-end-symbolic");
        // XXX
        // button_right.set_relief(gtk4::ReliefStyle::None);
        button_right.set_size_request(20, 0);
        obj.append(&button_right);
        button_right.connect_clicked(glib::clone!(@weak obj => move |_| {
            obj.imp().right_button_clicked();
        }));
        let adj = sw.hadjustment();

        if self
            .widgets
            .set(ThumbNavWidgets {
                button_left,
                button_right,
                sw,
            })
            .is_err()
        {
            err_out!("Widgets already set.");
        }

        // The value-changed signal might not be emitted because the value is already 0.
        // Ensure the state first.
        self.adj_value_changed(&adj);
        adj.emit_by_name::<()>("value-changed", &[]);

        self.add_thumbview();
    }

    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecBoolean::new(
                    "show-buttons",
                    "Show Buttons",
                    "Whether to show navigation buttons or not",
                    true, // Default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecObject::new(
                    "thumbview",
                    "Thumbnail View",
                    "The internal thumbnail viewer widget",
                    gtk4::IconView::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                ),
                glib::ParamSpecInt::new(
                    "mode",
                    "Mode",
                    "Thumb navigator mode",
                    ThumbNavMode::OneRow.into(),
                    ThumbNavMode::MultipleRows.into(),
                    ThumbNavMode::OneRow.into(),
                    glib::ParamFlags::READWRITE,
                ),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "show-buttons" => {
                let show_buttons = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.set_show_buttons(show_buttons);
            }
            "thumbview" => {
                let thumbview: Option<gtk4::IconView> = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.thumbview.replace(thumbview);
                self.add_thumbview();
            }
            "mode" => {
                let mode: i32 = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.set_mode(ThumbNavMode::from(mode));
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "show-buttons" => self.show_buttons.get().to_value(),
            "thumbview" => self.thumbview.borrow().to_value(),
            "mode" => {
                let n: i32 = self.mode.get().into();
                n.to_value()
            }
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for ThumbNavPriv {}

impl BoxImpl for ThumbNavPriv {}

/// # Safety
/// Use raw pointers
#[no_mangle]
pub unsafe extern "C" fn npc_thumb_nav_new(
    thumbview: *mut gtk4_sys::GtkIconView,
    mode: ThumbNavMode,
    show_buttons: bool,
) -> *mut gtk4_sys::GtkWidget {
    ThumbNav::new(
        &gtk4::IconView::from_glib_full(thumbview),
        mode,
        show_buttons,
    )
    .upcast::<gtk4::Widget>()
    .to_glib_full()
}
