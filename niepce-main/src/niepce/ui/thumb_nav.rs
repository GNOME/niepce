/*
 * niepce - niepce/ui/thumb_nav.rs
 *
 * Copyright (C) 2020-2025 Hubert Figuière
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

use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use glib::ControlFlow;
use glib::prelude::*;
use glib::subclass::prelude::*;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use npc_fwk::{glib, gtk4};

use npc_fwk::err_out;

const SCROLL_INC: f64 = 1.;
const SCROLL_MOVE: f64 = 20.;
const SCROLL_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(20);

#[repr(i32)]
#[derive(Clone, Copy, Default, Eq, PartialEq, glib::Enum)]
#[enum_type(name = "ThumbNavMode")]
pub enum ThumbNavMode {
    #[default]
    OneRow = 0,
    OneColumn = 1,
    MultipleRows = 2,
    MultipleColumns = 3,
    Invalid = 4,
}

glib::wrapper! {
    pub struct ThumbNav(
        ObjectSubclass<ThumbNavPriv>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl ThumbNav {
    pub fn new(thumbview: &gtk4::GridView, mode: ThumbNavMode, show_buttons: bool) -> Self {
        glib::Object::builder::<Self>()
            .property("mode", mode)
            .property("show-buttons", show_buttons)
            .property("thumbview", thumbview)
            .property("homogeneous", false)
            .property("spacing", 0)
            .build()
    }
}

struct ThumbNavWidgets {
    button_left: gtk4::Button,
    button_right: gtk4::Button,
    sw: gtk4::ScrolledWindow,
}

#[derive(glib::Properties)]
#[properties(wrapper_type = ThumbNav)]
pub struct ThumbNavPriv {
    #[property(
        get,
        set,
        nick = "Mode",
        blurb = "Thumb navigator mode",
        builder(ThumbNavMode::default())
    )]
    mode: Cell<ThumbNavMode>,
    #[property(
        get,
        set,
        name = "show-buttons",
        nick = "Show Buttons",
        blurb = "Whether to show navigation buttons or not",
        default_value = true
    )]
    show_buttons: Cell<bool>,

    left_i: Cell<f64>,
    right_i: Cell<f64>,
    widgets: OnceCell<ThumbNavWidgets>,
    #[property(
        get,
        set,
        nick = "Thubmnail View",
        blurb = "The internal thumbnail viewer widget",
        construct_only
    )]
    thumbview: RefCell<Option<gtk4::GridView>>,
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

    fn scroll_left(ref_i: &Cell<f64>, adj: &gtk4::Adjustment) -> glib::ControlFlow {
        let value = adj.value();
        let i = ref_i.get();

        if i == SCROLL_MOVE || value - SCROLL_INC < 0.0 {
            ref_i.set(0.0);
            adj.emit_by_name::<()>("value-changed", &[]);

            return ControlFlow::Break;
        }

        ref_i.set(i + 1.0);

        let move_ = f64::min(SCROLL_MOVE, value);
        adj.set_value(value - move_);

        ControlFlow::Continue
    }

    fn scroll_right(ref_i: &Cell<f64>, adj: &gtk4::Adjustment) -> glib::ControlFlow {
        let upper = adj.upper();
        let page_size = adj.page_size();
        let value = adj.value();
        let i = ref_i.get();

        if i == SCROLL_MOVE || value + SCROLL_INC > upper - page_size {
            ref_i.set(0.0);
            return ControlFlow::Break;
        }

        ref_i.set(i + 1.0);

        let move_ = f64::min(SCROLL_MOVE, upper - page_size - value);
        adj.set_value(value + move_);

        ControlFlow::Continue
    }

    fn set_show_buttons(&self, show_buttons: bool) {
        self.show_buttons.set(show_buttons);

        let widgets = &self.widgets.get().unwrap();
        let visible = show_buttons && self.mode.get() == ThumbNavMode::OneRow;
        widgets.button_left.set_visible(visible);
        widgets.button_right.set_visible(visible);
    }

    fn set_mode(&self, mode: ThumbNavMode) {
        self.mode.set(mode);

        match mode {
            ThumbNavMode::OneRow => {
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
                    thumbview.set_max_columns(1);
                }
                if let Some(widgets) = self.widgets.get() {
                    widgets
                        .sw
                        .set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Always);

                    widgets.button_left.set_visible(false);
                    widgets.button_right.set_visible(false);
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
            mode: Cell::default(),
            show_buttons: Cell::new(true),
            left_i: Cell::new(0.0),
            right_i: Cell::new(0.0),
            widgets: OnceCell::new(),
            thumbview: RefCell::new(None),
        }
    }
}

#[glib::derived_properties]
impl ObjectImpl for ThumbNavPriv {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();
        let button_left = gtk4::Button::from_icon_name("pan-start-symbolic");
        // XXX
        // button_left.set_relief(gtk4::ReliefStyle::None);
        button_left.set_size_request(20, 0);
        obj.append(&button_left);
        button_left.connect_clicked(glib::clone!(
            #[weak]
            obj,
            move |_| {
                obj.imp().left_button_clicked();
            }
        ));

        let sw = gtk4::ScrolledWindow::new();
        // XXX
        // sw.set_shadow_type(gtk4::ShadowType::In);
        sw.set_policy(gtk4::PolicyType::Always, gtk4::PolicyType::Never);
        let adj = sw.hadjustment();
        adj.connect_changed(glib::clone!(
            #[weak]
            obj,
            move |adj| {
                obj.imp().adj_changed(adj);
            }
        ));
        adj.connect_value_changed(glib::clone!(
            #[weak]
            obj,
            move |adj| {
                obj.imp().adj_value_changed(adj);
            }
        ));
        obj.append(&sw);

        let button_right = gtk4::Button::from_icon_name("pan-end-symbolic");
        // XXX
        // button_right.set_relief(gtk4::ReliefStyle::None);
        button_right.set_size_request(20, 0);
        obj.append(&button_right);
        button_right.connect_clicked(glib::clone!(
            #[weak]
            obj,
            move |_| {
                obj.imp().right_button_clicked();
            }
        ));
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
}

impl WidgetImpl for ThumbNavPriv {}

impl BoxImpl for ThumbNavPriv {}
