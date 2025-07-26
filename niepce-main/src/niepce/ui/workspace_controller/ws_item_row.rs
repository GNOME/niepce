/*
 * niepce - niepce/ui/workspace_controller/ws_item_row.rs
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

use std::cell::RefMut;

use glib::subclass::prelude::*;
use gtk4::prelude::*;
use npc_fwk::{glib, gtk4};

use npc_fwk::toolkit::ListViewRow;

use super::{Event, Item, TreeItemType};

glib::wrapper! {
    /// This is the row item (widget) for the workspace.
    /// It containes and expander, icon, label and count.
    pub struct WsItemRow(
        ObjectSubclass<imp::WsItemRow>)
    @extends gtk4::Box, gtk4::Widget,
    @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl WsItemRow {
    pub(super) fn new(tx: npc_fwk::toolkit::Sender<Event>) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().tx.replace(Some(tx));
        obj
    }
}

impl ListViewRow<Item> for WsItemRow {
    fn bind(&self, item: &Item, tree_list_row: Option<&gtk4::TreeListRow>) {
        // Type resolution doesn't allow downcast<>, but it's
        // an error if it is not the right type.
        self.imp().update(item);
        let binding = item
            .bind_property("count", &self.imp().count, "label")
            .transform_to(|_, count: i32| {
                let value = format!("{count}");
                Some(value.to_value())
            })
            .sync_create()
            .build();
        self.save_binding(binding);
        self.bind_to(&self.imp().label, "label", item, "label");
        let expander = &self.imp().expander;
        expander.set_list_row(tree_list_row);
        match item.tree_item_type() {
            // The top levels always have the expander
            TreeItemType::Folders | TreeItemType::Keywords | TreeItemType::Albums => {
                expander.set_hide_expander(false);
            }
            _ => {
                if let Some(children) = item.children() {
                    children.connect_items_changed(glib::clone!(
                        #[weak]
                        expander,
                        move |model, _, _, _| expander.set_hide_expander(model.n_items() == 0)
                    ));
                }

                expander.set_hide_expander(
                    item.children()
                        .map(|children| children.n_items())
                        .unwrap_or(0)
                        == 0,
                );
            }
        }
    }

    fn unbind(&self) {
        let expander = &self.imp().expander;
        expander.set_hide_expander(false);
        expander.set_list_row(None);
        self.clear_bindings();
    }

    fn bindings_mut(&self) -> RefMut<'_, Vec<glib::Binding>> {
        self.imp().bindings.borrow_mut()
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use npc_fwk::{gdk4, glib, gtk4};

    use super::super::{Event, Item, TreeItemType};
    use npc_engine::catalog;
    use npc_fwk::dbg_out;

    #[derive(Default)]
    pub struct WsItemRow {
        pub(super) tx: RefCell<Option<npc_fwk::toolkit::Sender<Event>>>,
        pub(super) expander: gtk4::TreeExpander,
        icon: gtk4::Image,
        pub(super) label: gtk4::Label,
        pub(super) count: gtk4::Label,
        type_: Cell<TreeItemType>,
        id: Cell<catalog::LibraryId>,
        pub(super) bindings: RefCell<Vec<glib::Binding>>,
    }

    impl WsItemRow {
        pub(super) fn update(&self, item: &Item) {
            self.icon.set_from_gicon(&item.icon());
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
            let inner = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
            self.expander.set_child(Some(&inner));
            inner.set_hexpand(true);
            inner.set_vexpand(true);
            inner.append(&self.icon);
            inner.append(&self.label);
            self.label.set_hexpand(true);
            self.label.set_xalign(0.0);
            inner.append(&self.count);

            let drop_target = gtk4::DropTarget::new(
                npc_engine::catalog::LibFile::static_type(),
                gdk4::DragAction::COPY,
            );
            drop_target.connect_accept(glib::clone!(
                #[weak(rename_to = this)]
                self,
                #[upgrade_or]
                false,
                move |_, _| matches!(this.type_.get(), TreeItemType::Album)
            ));
            drop_target.connect_drop(glib::clone!(
                #[weak(rename_to = this)]
                self,
                #[upgrade_or]
                false,
                move |_, value, _, _| {
                    if let Ok(libfile) = value.get::<npc_engine::catalog::LibFile>() {
                        dbg_out!("accepted value {}", libfile.id());
                        if let Some(ref tx) = *this.tx.borrow() {
                            let event = Event::DropLibFile(
                                this.id.get(),
                                this.type_.get(),
                                vec![libfile.id()],
                            );
                            npc_fwk::send_async_local!(event, tx);
                        }
                        true
                    } else {
                        dbg_out!("no value to accept");
                        false
                    }
                }
            ));
            self.obj().add_controller(drop_target);
        }
    }

    impl WidgetImpl for WsItemRow {}
    impl BoxImpl for WsItemRow {}
}
