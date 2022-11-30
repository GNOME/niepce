/*
 * niepce - niepce/ui/workspace_controller/ws_item_widget.rs
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

use glib::subclass::prelude::*;

use super::Item;

glib::wrapper! {
    /// This is the row item for the workspace.
    /// It containes and expander, icon, label and count.
    pub struct WsItemRow(
        ObjectSubclass<imp::WsItemRow>)
    @extends gtk4::Box, gtk4::Widget;
}

impl Default for WsItemRow {
    fn default() -> Self {
        Self::new()
    }
}

impl WsItemRow {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn bind(&self, item: &Item, tree_list_row: &gtk4::TreeListRow) {
        self.imp().update(item);
        self.imp().expander.set_list_row(Some(tree_list_row));
    }

    pub fn unbind(&self) {
        self.imp().expander.set_list_row(None);
    }
}

mod imp {
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;

    use super::super::Item;

    #[derive(Default)]
    pub struct WsItemRow {
        pub(super) expander: gtk4::TreeExpander,
        icon: gtk4::Image,
        label: gtk4::Label,
        count: gtk4::Label,
    }

    impl WsItemRow {
        pub(super) fn update(&self, item: &Item) {
            self.icon.set_from_gicon(&item.icon());
            self.label.set_label(&item.label());
            self.count.set_label(&format!("{}", item.count()));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WsItemRow {
        const NAME: &'static str = "WsItemRow";
        type ParentType = gtk4::Box;
        type Type = super::WsItemRow;
    }

    impl ObjectImpl for WsItemRow {
        fn constructed(&self) {
            self.parent_constructed();

            let box_ = &self.instance();
            self.expander.set_hexpand(true);
            self.expander.set_indent_for_icon(true);
            box_.append(&self.expander);
            let inner = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
            self.expander.set_child(Some(&inner));
            inner.set_margin_top(3);
            inner.set_margin_bottom(3);
            inner.set_hexpand(true);
            inner.set_vexpand(true);
            inner.append(&self.icon);
            self.icon.set_margin_start(4);
            self.icon.set_margin_end(4);
            inner.append(&self.label);
            self.label.set_hexpand(true);
            self.label.set_xalign(0.0);
            inner.append(&self.count);
        }
    }

    impl WidgetImpl for WsItemRow {}
    impl BoxImpl for WsItemRow {}
}
