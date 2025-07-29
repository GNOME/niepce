/*
 * niepce - crates/npc-fwk/src/toolkit/widgets/editable_hscale.rs
 *
 * Copyright (C) 2024-2025 Hubert Figuière
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
use glib::subclass::prelude::*;
use gtk4::prelude::*;

glib::wrapper! {
    pub struct EditableHScale(
        ObjectSubclass<imp::EditableHScale>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl EditableHScale {
    pub fn new(min: f64, max: f64, step: f64) -> Self {
        let obj: Self = glib::Object::builder::<Self>()
            .property("orientation", gtk4::Orientation::Horizontal)
            .build();

        let imp = obj.imp();
        imp.adj.set_lower(min);
        imp.adj.set_upper(max);
        imp.adj.set_step_increment(step);
        obj
    }
}

mod imp {
    use crate::glib;
    use crate::gtk4;
    use glib::subclass::Signal;
    use glib::subclass::prelude::*;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;

    use std::cell::Cell;

    #[derive(glib::Properties)]
    #[properties(wrapper_type = super::EditableHScale)]
    pub struct EditableHScale {
        #[property(get, set, default_value = 0.0)]
        value: Cell<f64>,
        dirty: Cell<bool>,
        pub(super) adj: gtk4::Adjustment,
        scale: gtk4::Scale,
        entry: gtk4::SpinButton,
    }

    impl EditableHScale {
        fn on_button_press(&self) {
            if self.dirty.get() {
                self.dirty.set(false);
                let value = self.adj.value();
                dbg_out!("value_change.emit({})", value);
                let obj = self.obj();
                obj.emit_by_name::<()>("value-changed", &[&value]);
            }
        }

        fn on_adj_value_changed(&self) {
            self.dirty.set(false);
            let obj = self.obj();
            let value = self.adj.value();
            obj.emit_by_name::<()>("value-changing", &[&value]);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EditableHScale {
        const NAME: &'static str = "EditableHScale";
        type Type = super::EditableHScale;
        type ParentType = gtk4::Box;

        fn new() -> Self {
            let adj = gtk4::Adjustment::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
            let scale = gtk4::Scale::new(gtk4::Orientation::Horizontal, Some(&adj));
            scale.set_hexpand(true);
            let entry = gtk4::SpinButton::new(Some(&adj), 0.0, 2);
            entry.set_width_chars(5);
            entry.set_editable(true);

            Self {
                adj,
                scale,
                entry,
                dirty: Cell::default(),
                value: Cell::default(),
            }
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for EditableHScale {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.upcast_ref::<gtk4::Box>().append(&self.scale);
            obj.upcast_ref::<gtk4::Box>().append(&self.entry);

            let gesture = gtk4::GestureClick::new();
            gesture.set_button(1);
            gesture.connect_released(glib::clone!(
                #[weak]
                obj,
                move |_, _, _, _| {
                    obj.imp().on_button_press();
                }
            ));
            self.scale.add_controller(gesture);
            let gesture = gtk4::GestureClick::new();
            gesture.set_button(1);
            gesture.connect_released(glib::clone!(
                #[weak]
                obj,
                move |_, _, _, _| {
                    obj.imp().on_button_press();
                }
            ));
            self.entry.add_controller(gesture);
            self.adj.connect_value_changed(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    obj.imp().on_adj_value_changed();
                }
            ));
        }

        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("value-changed")
                        .param_types([<f64>::static_type()])
                        .run_last()
                        .build(),
                    Signal::builder("value-changing")
                        .param_types([<f64>::static_type()])
                        .run_last()
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

    impl BoxImpl for EditableHScale {}
    impl WidgetImpl for EditableHScale {}
}
