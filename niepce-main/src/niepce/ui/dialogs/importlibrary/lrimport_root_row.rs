/*
 * niepce - niepce/ui/dialogs/importlibrary/lrimport_root_row.rs
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

use glib::subclass::prelude::*;
use gtk4::CompositeTemplate;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use npc_fwk::{glib, gtk4, gtk4 as gtk};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/net/figuiere/Niepce/ui/lrimport_root_row.ui")]
pub struct LrImportRootRowPriv {
    #[template_child]
    pub label: TemplateChild<gtk4::Label>,
    #[template_child]
    pub remapped: TemplateChild<gtk4::EditableLabel>,
    #[template_child]
    pub check: TemplateChild<gtk4::CheckButton>,
}

#[glib::object_subclass]
impl ObjectSubclass for LrImportRootRowPriv {
    const NAME: &'static str = "LrImportRootRow";
    type Type = LrImportRootRow;
    type ParentType = gtk4::ListBoxRow;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for LrImportRootRowPriv {}

impl WidgetImpl for LrImportRootRowPriv {}

impl ListBoxRowImpl for LrImportRootRowPriv {}

glib::wrapper! {
    pub struct LrImportRootRow(ObjectSubclass<LrImportRootRowPriv>)
        @extends gtk4::ListBoxRow, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for LrImportRootRow {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl LrImportRootRow {
    pub fn new(root: String) -> Self {
        let obj = Self::default();
        obj.imp().label.set_label(&root);
        obj.imp().remapped.set_text(&root);
        obj.imp().check.set_active(true);
        obj
    }

    pub fn connect_changed<F: Fn(&gtk4::EditableLabel) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.imp().remapped.connect_changed(f)
    }

    pub fn connect_toggled<F: Fn(&gtk4::CheckButton) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.imp().check.connect_toggled(f)
    }
}
