/*
 * niepce - fwk/toolkit/widgets/metadata_widget.rs
 *
 * Copyright (C) 2022-2025 Hubert Figuière
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

use crate::glib;
use crate::gtk4;
use glib::prelude::*;
use gtk4::subclass::prelude::*;

use super::ToolboxItem;
use crate::PropertyBag;

pub type MetadataPropertyBag = PropertyBag<u32>;

/// A wrapped `MetadataPropertyBag` for use with `glib::Value` in signals.
#[derive(Clone, Default, glib::Boxed)]
#[boxed_type(name = "PropertyBag")]
pub struct WrappedPropertyBag(pub MetadataPropertyBag);

// This bridge content should be moved when the bridge is removed.
#[repr(u32)]
#[derive(Clone, PartialEq)]
pub enum MetaDT {
    #[allow(dead_code)]
    NONE = 0,
    #[allow(dead_code)]
    STRING,
    StringArray,
    TEXT,
    DATE,
    FRAC,
    FracDec, // Fraction as decimal
    StarRating,
    #[allow(dead_code)]
    SIZE, // Size in bytes
}

#[derive(Clone)]
pub struct MetadataFormat {
    pub label: String,
    pub id: u32, // NiepcePropertyIdx
    pub type_: MetaDT,
    pub readonly: bool,
}

#[derive(Clone)]
pub struct MetadataSectionFormat {
    pub section: String,
    pub formats: Vec<MetadataFormat>,
}

glib::wrapper! {
    pub struct MetadataWidget(
    ObjectSubclass<imp::MetadataWidget>)
    @extends ToolboxItem, gtk4::Box, gtk4::Widget,
    @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl MetadataWidget {
    pub fn new(title: &str) -> MetadataWidget {
        let obj: MetadataWidget = glib::Object::new();
        obj.upcast_ref::<ToolboxItem>().set_title(title);

        obj
    }

    /// Connect to the signal `metadata-changed`
    pub fn connect_metadata_changed<F>(&self, f: F) -> glib::SignalHandlerId
    where
        F: Fn(&Self, WrappedPropertyBag, WrappedPropertyBag) + 'static,
    {
        self.connect_closure(
            "metadata-changed",
            true,
            glib::closure_local!(move |w, new, old| {
                f(&w, new, old);
            }),
        )
    }

    /// Set the data source of the metadata.
    pub fn set_data_source(&self, properties: Option<MetadataPropertyBag>) {
        self.imp().set_data_source(properties);
    }

    pub fn set_data_format(&self, fmt: Option<MetadataSectionFormat>) {
        self.imp().set_data_format(fmt);
    }
}

mod imp {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::glib;
    use crate::gtk4;
    use glib::subclass::*;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;

    use crate::PropertyValue;

    use super::super::prelude::*;
    use super::super::{RatingLabel, TokenTextView};
    use super::{
        MetaDT, MetadataFormat, MetadataPropertyBag, MetadataSectionFormat, WrappedPropertyBag,
    };

    fn clear_widget(widget: &gtk4::Widget) {
        if let Some(label) = widget.downcast_ref::<gtk4::Label>() {
            label.set_text("");
        } else if let Some(entry) = widget.downcast_ref::<gtk4::Entry>() {
            entry.set_text("");
        } else if let Some(ttv) = widget.downcast_ref::<TokenTextView>() {
            ttv.set_tokens(&[]);
        } else if let Some(tv) = widget.downcast_ref::<gtk4::TextView>() {
            tv.buffer().set_text("");
        } else if let Some(rating) = widget.downcast_ref::<RatingLabel>() {
            rating.set_rating(0);
        } else {
            err_out!("Unknow widget type {}", widget.type_().name());
        }
    }

    pub struct MetadataWidget {
        widget: gtk4::Grid,
        data_map: RefCell<HashMap<u32, gtk4::Widget>>,
        current_data: RefCell<Option<MetadataPropertyBag>>,
        fmt: RefCell<Option<MetadataSectionFormat>>,
    }

    impl MetadataWidget {
        fn create_star_rating_widget(&self, readonly: bool, id: u32) -> gtk4::Widget {
            let rating = RatingLabel::new(0, !readonly);
            if !readonly {
                let obj = self.obj();
                rating.connect_rating_changed(glib::clone!(
                    #[weak]
                    obj,
                    move |_, rating| {
                        obj.imp()
                            .emit_metadata_changed(id, &PropertyValue::Int(rating));
                    }
                ));
            }

            rating.upcast()
        }

        fn create_text_widget(&self, readonly: bool, id: u32) -> gtk4::Widget {
            if readonly {
                self.create_string_widget(readonly, id)
            } else {
                let entry = gtk4::TextView::new();
                entry.set_accepts_tab(false);
                entry.set_editable(true);
                entry.set_wrap_mode(gtk4::WrapMode::Word);
                let ctrl = gtk4::EventControllerFocus::new();
                let obj = self.obj();
                ctrl.connect_leave(glib::clone!(
                    #[weak]
                    entry,
                    #[weak]
                    obj,
                    move |_| {
                        let buffer = entry.buffer();
                        let start = buffer.start_iter();
                        let end = buffer.end_iter();
                        obj.imp().emit_metadata_changed(
                            id,
                            &PropertyValue::String(buffer.text(&start, &end, true).to_string()),
                        );
                    }
                ));

                entry.add_controller(ctrl);
                entry.upcast()
            }
        }
        fn create_string_widget(&self, readonly: bool, id: u32) -> gtk4::Widget {
            if readonly {
                let label = gtk4::Label::new(None);
                label.set_xalign(0.0);
                label.set_yalign(0.5);
                label.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);

                label.upcast()
            } else {
                let entry = gtk4::Entry::new();
                entry.set_has_frame(false);
                let ctrl = gtk4::EventControllerFocus::new();

                let obj = self.obj();
                ctrl.connect_leave(glib::clone!(
                    #[weak]
                    entry,
                    #[weak]
                    obj,
                    move |_| {
                        obj.imp().emit_metadata_changed(
                            id,
                            &PropertyValue::String(entry.text().to_string()),
                        );
                    }
                ));

                entry.add_controller(ctrl);
                entry.upcast()
            }
        }

        fn create_string_array_widget(&self, readonly: bool, id: u32) -> gtk4::Widget {
            let ttv = TokenTextView::new();
            if !readonly {
                let ctrl = gtk4::EventControllerFocus::new();

                let obj = self.obj();
                ctrl.connect_leave(glib::clone!(
                    #[weak]
                    ttv,
                    #[weak]
                    obj,
                    move |_| {
                        obj.imp()
                            .emit_metadata_changed(id, &PropertyValue::StringArray(ttv.tokens()));
                    }
                ));
                ttv.add_controller(ctrl);
            }

            ttv.upcast()
        }

        fn create_date_widget(&self, readonly: bool, id: u32) -> gtk4::Widget {
            self.create_string_widget(readonly, id)
        }

        fn create_widgets_for_format(&self, fmt: &MetadataSectionFormat) {
            for (i, f) in fmt.formats.iter().enumerate() {
                let label = gtk4::Label::new(Some(&format!("<b>{}</b>", &f.label)));
                label.set_use_markup(true);
                label.set_xalign(0.0);
                if f.type_ != MetaDT::StringArray {
                    label.set_yalign(0.5);
                } else {
                    label.set_yalign(0.0);
                }
                let w = match f.type_ {
                    MetaDT::StarRating => self.create_star_rating_widget(f.readonly, f.id),
                    MetaDT::StringArray => self.create_string_array_widget(f.readonly, f.id),
                    MetaDT::TEXT => self.create_text_widget(f.readonly, f.id),
                    MetaDT::DATE => self.create_date_widget(f.readonly, f.id),
                    _ => self.create_string_widget(f.readonly, f.id),
                };
                let row = i as i32;
                self.widget.insert_row(row + 1);
                self.widget.attach(&label, 0, row, 1, 1);
                self.widget
                    .attach_next_to(&w, Some(&label), gtk4::PositionType::Right, 1, 1);
                self.data_map.borrow_mut().insert(f.id, w);
            }
        }

        fn add_data(&self, fmt: &MetadataFormat, value: &PropertyValue) {
            let data_map = self.data_map.borrow();
            let w = data_map.get(&fmt.id);

            if w.is_none() {
                err_out!("No widget for property {}", fmt.id);
                return;
            }

            let w = w.as_ref().unwrap();
            match fmt.type_ {
                MetaDT::FracDec => self.set_fraction_dec_data(w, value),
                MetaDT::FRAC => self.set_fraction_data(w, value),
                MetaDT::StarRating => self.set_star_rating_data(w, value),
                MetaDT::StringArray => self.set_string_array_data(w, fmt.readonly, value),
                MetaDT::TEXT => self.set_text_data(w, fmt.readonly, value),
                MetaDT::DATE => self.set_date_data(w, value),
                _ => {
                    if !self.set_text_data(w, fmt.readonly, value) {
                        err_out!("failed to set value for {}", fmt.id);
                        false
                    } else {
                        true
                    }
                }
            };
        }

        pub(super) fn set_data_format(&self, fmt: Option<MetadataSectionFormat>) {
            self.fmt.replace(fmt);
            if let Some(fmt) = self.fmt.borrow().as_ref() {
                self.create_widgets_for_format(fmt);
            }
            // XXX what if None? Should we delete the widgets?
        }

        pub(super) fn set_data_source(&self, properties: Option<MetadataPropertyBag>) {
            self.current_data.replace(properties);
            self.data_map.borrow().values().for_each(clear_widget);

            let is_empty = self
                .current_data
                .borrow()
                .as_ref()
                .map(|v| v.is_empty())
                .unwrap_or(true);
            self.obj().set_sensitive(!is_empty);
            if is_empty {
                return;
            }
            let properties = self.current_data.borrow();
            if let Some(fmt) = self.fmt.borrow().as_ref() {
                fmt.formats.iter().for_each(|f| {
                    if let Some(value) = properties.as_ref().and_then(|v| v.get(&f.id)) {
                        self.add_data(f, value)
                    } else {
                        // XXX value is empty
                    }
                });
            }
        }

        fn set_fraction_dec_data(&self, w: &gtk4::Widget, value: &PropertyValue) -> bool {
            if let Some(s) = value.string() {
                dbg_out!("set faction dec {}", s);
                return if let Some(w) = w.downcast_ref::<gtk4::Label>() {
                    let dec_str = crate::fraction_to_decimal(s)
                        .map(|dec| dec.to_string())
                        .unwrap_or_else(|| "NaN".to_string());

                    w.set_text(&dec_str);
                    true
                } else {
                    err_out!(
                        "Incorrect widget type for fraction_dec: {}",
                        w.type_().name()
                    );
                    false
                };
            }

            err_out!("Data not a string");
            false
        }

        fn set_fraction_data(&self, w: &gtk4::Widget, value: &PropertyValue) -> bool {
            if let Some(s) = value.string() {
                dbg_out!("set fraction {}", s);
                return if let Some(w) = w.downcast_ref::<gtk4::Label>() {
                    if let Some((n, d)) = crate::parse_fraction(s) {
                        let frac_str = format!("{n}/{d}");
                        w.set_text(&frac_str);
                        true
                    } else {
                        err_out!("Invalid fraction {}", s);
                        false
                    }
                } else {
                    err_out!("Incorrect widget type for fraction: {}", w.type_().name());
                    false
                };
            }

            err_out!("Data not a string");
            false
        }

        fn set_star_rating_data(&self, w: &gtk4::Widget, value: &PropertyValue) -> bool {
            if let Some(i) = value.integer() {
                return if let Some(w) = w.downcast_ref::<RatingLabel>() {
                    w.set_rating(i);
                    true
                } else {
                    err_out!("Incorrect widget type for rating: {}", w.type_().name());
                    false
                };
            }

            err_out!("Data not integer");
            false
        }

        fn set_string_array_data(
            &self,
            w: &gtk4::Widget,
            readonly: bool,
            value: &PropertyValue,
        ) -> bool {
            if let Some(tokens) = value.string_array() {
                return if let Some(w) = w.downcast_ref::<TokenTextView>() {
                    w.set_tokens(tokens);
                    w.set_editable(!readonly);
                    true
                } else {
                    err_out!(
                        "Incorrect widget type for string array: {}",
                        w.type_().name()
                    );
                    false
                };
            }
            err_out!("Data not string array");
            false
        }

        fn set_text_data(&self, w: &gtk4::Widget, readonly: bool, value: &PropertyValue) -> bool {
            if let Some(s) = value.string() {
                if readonly {
                    return if let Some(w) = w.downcast_ref::<gtk4::Label>() {
                        w.set_text(s);
                        true
                    } else {
                        err_out!("Incorrect widget type {}", w.type_().name());
                        false
                    };
                } else {
                    return if let Some(w) = w.downcast_ref::<gtk4::Entry>() {
                        w.set_text(s);
                        true
                    } else if let Some(w) = w.downcast_ref::<gtk4::TextView>() {
                        w.buffer().set_text(s);
                        true
                    } else {
                        err_out!("Incorrect widget type for text: {}", w.type_().name());
                        false
                    };
                }
            }

            err_out!("Data not a string");
            false
        }

        fn set_date_data(&self, w: &gtk4::Widget, value: &PropertyValue) -> bool {
            if let Some(d) = value.date() {
                return if let Some(w) = w.downcast_ref::<gtk4::Label>() {
                    w.set_text(&d.to_string());
                    true
                } else if let Some(w) = w.downcast_ref::<gtk4::Entry>() {
                    w.set_text(&d.to_string());
                    true
                } else {
                    err_out!("Incorrect widget type for date: {}", w.type_().name());
                    false
                };
            }

            err_out!("Data not a date");
            false
        }

        fn emit_metadata_changed(&self, prop: u32, value: &PropertyValue) {
            let mut props = MetadataPropertyBag::default();
            let mut old_props = MetadataPropertyBag::default();
            props.set_value(prop, value.clone());
            if let Some(old_val) = self
                .current_data
                .borrow()
                .as_ref()
                .and_then(|props| props.get(&prop))
            {
                old_props.set_value(prop, old_val.clone());
            }
            self.obj().emit_by_name::<()>(
                "metadata-changed",
                &[&WrappedPropertyBag(props), &WrappedPropertyBag(old_props)],
            );
        }
    }

    impl ObjectImpl for MetadataWidget {
        fn constructed(&self) {
            self.parent_constructed();

            self.widget.set_column_homogeneous(true);
            self.widget.set_row_homogeneous(false);
            self.widget.insert_column(0);
            self.widget.insert_column(0);
            self.widget.set_margin_start(8);
            self.widget.set_hexpand(true);
            self.obj()
                .upcast_ref::<super::ToolboxItem>()
                .set_child(Some(&self.widget));
        }

        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("metadata-changed")
                        .param_types([
                            <WrappedPropertyBag>::static_type(),
                            <WrappedPropertyBag>::static_type(),
                        ])
                        .run_last()
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MetadataWidget {
        const NAME: &'static str = "NpcMetadataWidget";
        type Type = super::MetadataWidget;
        type ParentType = super::ToolboxItem;

        fn new() -> Self {
            Self {
                widget: gtk4::Grid::new(),
                data_map: RefCell::new(HashMap::default()),
                current_data: RefCell::new(None),
                fmt: RefCell::new(None),
            }
        }
    }

    impl ToolboxItemImpl for MetadataWidget {}
    impl BoxImpl for MetadataWidget {}
    impl WidgetImpl for MetadataWidget {}
}
