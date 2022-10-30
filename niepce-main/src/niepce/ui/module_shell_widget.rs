/*
 * niepce - niepce/ui/module_shell_widget.rs
 *
 * Copyright (C) 2022 Hubert Figuière
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

use std::ffi::c_char;

use glib::translate::*;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

glib::wrapper! {
    pub struct ModuleShellWidget(
    ObjectSubclass<imp::ModuleShellWidget>)
    @extends gtk4::Box, gtk4::Widget;
}

impl Default for ModuleShellWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[gtk4::template_callbacks]
impl ModuleShellWidget {
    pub fn new() -> ModuleShellWidget {
        glib::Object::new(&[]).expect("Coudln't build ModuleShellWiget")
    }

    // cxx
    pub fn gobj(&self) -> *mut c_char {
        let w: *mut gtk4_sys::GtkBox = self.upcast_ref::<gtk4::Box>().to_glib_none().0;

        w as *mut c_char
    }

    #[template_callback]
    fn stack_changed(&self, _stack: &glib::Value) {
        self.imp().stack_changed();
    }

    /// Append a `page` widget with `name` and `label`
    pub fn append_page(&self, page: &impl IsA<gtk4::Widget>, name: &str, label: &str) {
        self.imp().stack.add_titled(page, Some(name), label);
    }

    // cxx
    /// # Safety
    /// Dereference a raw pointer.
    pub unsafe fn append_page_(&self, widget: *mut c_char, name: &str, label: &str) {
        let w = gtk4::Widget::from_glib_none(widget as *mut gtk4_sys::GtkWidget);

        self.append_page(&w, name, label);
    }

    /// Activate page by `name`
    pub fn activate_page(&self, name: &str) {
        self.imp().stack.set_visible_child_name(name);
    }

    /// Get the [menu button](`gtk4::MenuButton`)
    pub fn menu_button(&self) -> &gtk4::MenuButton {
        &self.imp().menubutton
    }

    // cxx
    pub fn get_menu_button(&self) -> *mut c_char {
        let w: *mut gtk4_sys::GtkMenuButton = self.menu_button().to_glib_none().0;

        w as *mut c_char
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::subclass::Signal;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use gtk4::TemplateChild;

    #[derive(Default, gtk4::CompositeTemplate)]
    #[template(string = r#"
    <interface>
      <template class="ModuleShellWidget" parent="GtkBox">
        <property name="orientation">vertical</property>
        <child>
          <object class="GtkCenterBox" id="mainbox">
            <property name="margin-start">4</property>
            <property name="margin-end">4</property>
            <property name="margin-top">4</property>
            <property name="margin-bottom">4</property>
            <child type="end">
              <object class="GtkMenuButton" id="menubutton">
                <property name="direction">none</property>
                <property name="icon_name">view-more-symbolic</property>
              </object>
            </child>
            <child type="center">
              <object class="GtkStackSwitcher" id="switcher">
                <property name="stack">stack</property>
              </object>
            </child>
          </object>
        </child>
        <child>
          <object class="GtkStack" id="stack">
            <signal name="notify::visible-child-name" handler="stack_changed" swapped="true" />
            <child>
              <placeholder />
            </child>
          </object>
        </child>
      </template>
    </interface>
    "#)]
    pub struct ModuleShellWidget {
        #[template_child]
        mainbox: TemplateChild<gtk4::CenterBox>,
        #[template_child]
        pub(super) menubutton: TemplateChild<gtk4::MenuButton>,
        #[template_child]
        pub(super) stack: TemplateChild<gtk4::Stack>,
        #[template_child]
        switcher: TemplateChild<gtk4::StackSwitcher>,

        current_module: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ModuleShellWidget {
        const NAME: &'static str = "ModuleShellWidget";
        type Type = super::ModuleShellWidget;
        type ParentType = gtk4::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ModuleShellWidget {
        fn signals() -> &'static [Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder(
                        "activated",
                        &[<String>::static_type().into()],
                        <()>::static_type().into(),
                    )
                    .run_last()
                    .build(),
                    Signal::builder(
                        "deactivated",
                        &[<String>::static_type().into()],
                        <()>::static_type().into(),
                    )
                    .run_last()
                    .build(),
                ]
            });
            SIGNALS.as_ref()
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl ModuleShellWidget {
        pub(super) fn stack_changed(&self) {
            if let Some(ref current_module) = *self.current_module.borrow() {
                self.instance()
                    .emit_by_name::<()>("deactivated", &[current_module]);
            }
            let current_module = self
                .stack
                .visible_child_name()
                .map(|s| s.as_str().to_string());
            self.current_module.replace(current_module);
            if let Some(ref current_module) = *self.current_module.borrow() {
                self.instance()
                    .emit_by_name::<()>("activated", &[&current_module]);
            }
        }
    }

    impl BoxImpl for ModuleShellWidget {}
    impl WidgetImpl for ModuleShellWidget {}
}

pub fn module_shell_widget_new() -> Box<ModuleShellWidget> {
    Box::new(ModuleShellWidget::new())
}
