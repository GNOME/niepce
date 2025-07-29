/*
 * niepce - niepce/ui/module_shell_widget.rs
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

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use npc_fwk::{glib, gtk4, gtk4 as gtk};

glib::wrapper! {
    pub struct ModuleShellWidget(
    ObjectSubclass<imp::ModuleShellWidget>)
    @extends gtk4::Box, gtk4::Widget,
    @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for ModuleShellWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[gtk4::template_callbacks]
impl ModuleShellWidget {
    pub fn new() -> ModuleShellWidget {
        glib::Object::new()
    }

    #[template_callback]
    fn stack_changed(&self, _stack: &glib::Value) {
        self.imp().stack_changed();
    }

    /// Append a `page` widget with `name` and `label`
    pub fn append_page(&self, page: &impl IsA<gtk4::Widget>, name: &str, label: &str) {
        self.imp().stack.add_titled(page, Some(name), label);
    }

    /// Activate page by `name`
    pub fn activate_page(&self, name: &str) {
        self.imp().stack.set_visible_child_name(name);
    }

    /// Get the [menu button](`gtk4::MenuButton`)
    pub fn menu_button(&self) -> &gtk4::MenuButton {
        &self.imp().menubutton
    }
}

mod imp {
    use std::cell::RefCell;

    use glib::subclass::Signal;
    use gtk4::TemplateChild;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use npc_fwk::{glib, gtk4, gtk4 as gtk};

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
            use std::sync::LazyLock;
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![
                    Signal::builder("activated")
                        .param_types([<String>::static_type()])
                        .run_last()
                        .build(),
                    Signal::builder("deactivated")
                        .param_types([<String>::static_type()])
                        .run_last()
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl ModuleShellWidget {
        pub(super) fn stack_changed(&self) {
            if let Some(ref current_module) = *self.current_module.borrow() {
                self.obj()
                    .emit_by_name::<()>("deactivated", &[current_module]);
            }
            let current_module = self
                .stack
                .visible_child_name()
                .map(|s| s.as_str().to_string());
            self.current_module.replace(current_module);
            if let Some(ref current_module) = *self.current_module.borrow() {
                self.obj()
                    .emit_by_name::<()>("activated", &[&current_module]);
            }
        }
    }

    impl BoxImpl for ModuleShellWidget {}
    impl WidgetImpl for ModuleShellWidget {}
}
