/*
 * niepce - niepce/ui/workspace_controller/ws_item_widget.rs
 *
 * Copyright (C) 2022-2024 Hubert Figuière
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
use gtk4::prelude::*;

use npc_fwk::toolkit::tree_view_model::css::NOCHILDREN_CSS;

use super::{Event, Item, TreeItemType};

glib::wrapper! {
    /// This is the row item for the workspace.
    /// It containes and expander, icon, label and count.
    pub struct WsItemRow(
        ObjectSubclass<imp::WsItemRow>)
    @extends gtk4::Box, gtk4::Widget;
}

impl WsItemRow {
    pub(super) fn new(tx: npc_fwk::toolkit::Sender<Event>) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().tx.replace(Some(tx));
        obj
    }

    pub fn bind(&self, item: &Item, tree_list_row: &gtk4::TreeListRow) {
        self.imp().update(item);
        self.imp().expander.set_list_row(Some(tree_list_row));
        match item.tree_item_type() {
            // The top levels always have the expander
            TreeItemType::Folders | TreeItemType::Keywords | TreeItemType::Albums => {
                self.remove_css_class(NOCHILDREN_CSS)
            }
            _ => {
                if item
                    .children()
                    .map(|children| children.n_items())
                    .unwrap_or(0)
                    == 0
                {
                    self.add_css_class(NOCHILDREN_CSS);
                } else {
                    self.remove_css_class(NOCHILDREN_CSS);
                }
            }
        }
    }

    pub fn unbind(&self) {
        self.remove_css_class(NOCHILDREN_CSS);
        self.imp().expander.set_list_row(None);
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;

    use super::super::{Event, Item, TreeItemType};
    use npc_engine::db;
    use npc_fwk::dbg_out;

    #[derive(Default)]
    pub struct WsItemRow {
        pub(super) tx: RefCell<Option<npc_fwk::toolkit::Sender<Event>>>,
        pub(super) expander: gtk4::TreeExpander,
        icon: gtk4::Image,
        label: gtk4::Label,
        count: gtk4::Label,
        type_: Cell<TreeItemType>,
        id: Cell<db::LibraryId>,
    }

    impl WsItemRow {
        pub(super) fn update(&self, item: &Item) {
            self.icon.set_from_gicon(&item.icon());
            self.label.set_label(&item.label());
            self.count.set_label(&format!("{}", item.count()));
            self.type_.set(item.tree_item_type());
            self.id.set(item.id());
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

            let box_ = &self.obj();
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

            let drop_target = gtk4::DropTarget::new(
                npc_engine::db::LibFile::static_type(),
                gdk4::DragAction::COPY,
            );
            drop_target.connect_accept(
                glib::clone!(@weak self as this => @default-return false, move |_, _| {
                    matches!(this.type_.get(), TreeItemType::Album)
                }),
            );
            drop_target.connect_drop(glib::clone!(@weak self as this => @default-return false, move |_, value, _, _| {
                if let Ok(libfile) = value.get::<npc_engine::db::LibFile>() {
                    dbg_out!("accepted value {}", libfile.id());
                    if let Some(ref tx) = *this.tx.borrow() {
                        let event = Event::DropLibFile(this.id.get(), this.type_.get(), vec![libfile.id()]);
                        npc_fwk::send_async_local!(event, tx);
                    }
                    true
                } else {
                    dbg_out!("no value to accept");
                    false
                }
            }));
            self.obj().add_controller(drop_target);
        }
    }

    impl WidgetImpl for WsItemRow {}
    impl BoxImpl for WsItemRow {}
}
