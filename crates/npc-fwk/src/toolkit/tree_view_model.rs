/*
 * niepce - npc-fwk/toolkit/tree_view_model.rs
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

use std::cell::RefCell;
use std::rc::Rc;

use crate::gio;
use crate::glib;
use crate::gtk4;
use gtk4::prelude::*;

use super::ListViewRow;
use crate::base::{PathTree, PathTreeItem};

/// Downcast a ref `glib::Object` down to `gtk4::ListItem`
macro_rules! list_item {
    ( $x:expr ) => {
        $x.downcast_ref::<gtk4::ListItem>().unwrap()
    };
}

/// Get the child widget and downcast down to the type.
macro_rules! child_widget {
    ( $x:expr, $t:ty ) => {
        $x.child().and_downcast_ref::<$t>().unwrap().clone()
    };
}

/// Downcast to a `gtk4::TreeListRow`
macro_rules! tree_list_row {
    ( $x:expr ) => {
        $x.downcast_ref::<gtk4::TreeListRow>()
            .expect("to be a TreeListRow")
    };
}

/// Trait to implement for TreeViewItems
pub trait TreeViewItem {
    /// The children ``gio::ListModel``.
    fn children(&self) -> Option<gio::ListStore>;

    fn set_autohide_expander(&self, expander: &gtk4::TreeExpander) {
        if let Some(children) = self.children() {
            expander.set_hide_expander(children.n_items() == 0);
            children.connect_items_changed(glib::clone!(
                #[weak]
                expander,
                move |model, _, _, _| expander.set_hide_expander(model.n_items() == 0)
            ));
        }
    }
}

/// A tree view model
pub struct TreeViewModel<T>
where
    T: PathTreeItem,
{
    model: gtk4::TreeListModel,
    selection_model: gtk4::SelectionModel,
    items: Rc<RefCell<PathTree<T>>>,
}

impl<T> TreeViewModel<T>
where
    T: IsA<glib::Object> + PathTreeItem,
{
    pub fn new() -> Rc<TreeViewModel<T>>
    where
        <T as PathTreeItem>::Id: Ord + Copy,
        T: TreeViewItem,
    {
        let rootstore = gio::ListStore::new::<T>();
        let model = gtk4::TreeListModel::new(
            rootstore,
            false,
            true,
            glib::clone!(move |item| item.downcast_ref::<T>()?.children().and_upcast()),
        );
        // XXX deal with other selection models.
        // XXX This can probably be done by passing a enum to new()
        let selection_model = gtk4::SingleSelection::new(Some(model.clone()));
        selection_model.set_autoselect(false);
        selection_model.set_can_unselect(true);
        let items = Rc::new(RefCell::new(PathTree::<T>::new('/')));
        Rc::new(TreeViewModel {
            model,
            selection_model: selection_model.upcast(),
            items,
        })
    }

    pub fn model(&self) -> &gtk4::TreeListModel {
        &self.model
    }

    pub fn contains(&self, id: T::Id) -> bool
    where
        <T as PathTreeItem>::Id: Ord + Copy,
    {
        self.items.borrow().get_by_id(id).is_some()
    }

    /// Append an item. If it already exist (by id) then
    /// it is a no-op.
    /// If the item has a known parent, will return the id of the parent.
    pub fn append(&self, item: &T) -> Option<T::Id>
    where
        T: TreeViewItem,
        <T as PathTreeItem>::Id: Ord + Copy,
    {
        if self.contains(item.id()) {
            return None;
        }
        let parent = self.items.borrow_mut().push(item.clone());
        if let Some(parent) = parent {
            self.items
                .borrow()
                .get_by_id(parent)
                .and_then(|parent| parent.children())
                .inspect(|children| children.append(item));
            Some(parent)
        } else if let Ok(store) = self.model.model().downcast::<gio::ListStore>() {
            // if we don't know the parent, it gets added in the top-level.
            store.append(item);
            None
        } else {
            None
        }
    }

    /// Append a root item. If it already exist (by id) then
    /// it is a no-op
    pub fn append_root(&self, item: &T)
    where
        <T as PathTreeItem>::Id: Ord + Copy,
    {
        if self.contains(item.id()) {
            return;
        }
        self.items.borrow_mut().push(item.clone());
        if let Ok(store) = self.model.model().downcast::<gio::ListStore>() {
            store.append(item);
        }
    }

    pub fn select_item(&self, idx: u32) -> bool {
        self.selection_model.select_item(idx, true)
    }

    pub fn unselect_all(&self) -> bool {
        self.selection_model.unselect_all()
    }

    fn remove_in_liststore(&self, store: &gio::ListStore, id: &T::Id)
    where
        <T as PathTreeItem>::Id: Ord + Copy,
    {
        store
            .find_with_equal_func(|item| {
                item.downcast_ref::<T>()
                    .map(|item| item.id() == *id)
                    .unwrap_or(false)
            })
            .inspect(|idx| store.remove(*idx));
    }

    /// Remove item with `id`.
    pub fn remove(&self, id: &T::Id)
    where
        <T as PathTreeItem>::Id: Ord + Copy,
        T: TreeViewItem,
    {
        let path = self
            .items
            .borrow()
            .get_by_id(*id)
            .map(|item| item.path())
            .as_ref()
            .map(|path| {
                trace_out!("Looking to remove item at path {path}");
                self.items
                    .borrow()
                    .parent_for(path)
                    .and_then(|parent| parent.children())
                    .or_else(|| self.model.model().downcast::<gio::ListStore>().ok())
                    .inspect(|children| self.remove_in_liststore(children, id));
                path.clone()
            });
        if let Some(path) = path {
            self.items.borrow_mut().remove(&path);
        }
    }

    /// Get the item at index. This is the index in the selection model.
    pub fn item(&self, idx: u32) -> Option<T>
    where
        <T as PathTreeItem>::Id: Ord + Copy,
    {
        tree_list_row!(self.selection_model.item(idx)?)
            .item()
            .and_downcast::<T>()
    }

    pub fn item_for_path(&self, path: &str) -> Option<T>
    where
        <T as PathTreeItem>::Id: Ord + Copy,
    {
        let items = self.items.borrow();
        items.get(path).cloned()
    }

    // XXX This is very slow. Didn't find a way to get the index
    // of an object.
    pub fn item_index_for_path(&self, path: &str) -> Option<u32>
    where
        <T as PathTreeItem>::Id: Ord + Copy,
    {
        self.item_for_path(path).and_then(|item| {
            let n = self.selection_model.n_items();
            (0..n).find(|&idx| {
                self.selection_model
                    .item(idx)
                    .and_downcast_ref::<gtk4::TreeListRow>()
                    .and_then(|item| item.item())
                    .and_downcast_ref::<T>()
                    .map(|dest| dest.id() == item.id())
                    .unwrap_or(false)
            })
        })
    }

    /// Bind to the listview widget. Will setup the factory.
    pub fn bind<F>(self: &Rc<Self>, listview: &gtk4::ListView, factory: &Rc<F>)
    where
        F: TreeViewFactory<T>,
    {
        let factory = factory.build();
        listview.set_model(Some(&self.selection_model));
        listview.set_factory(Some(&factory));
    }
}

/// Trait to implement for the item factory in a the ListView.
pub trait TreeViewFactory<T>
where
    T: IsA<glib::Object>,
    Self: 'static + Sized,
{
    /// The widget type for the list item.
    type Widget: IsA<gtk4::Widget> + ListViewRow<T>;

    /// Setup the widget.
    fn setup(&self) -> Self::Widget;

    /// Create the factory for gtk4::ListView.
    fn build(self: &Rc<Self>) -> gtk4::SignalListItemFactory {
        let factory = gtk4::SignalListItemFactory::new();
        let f = self.clone();
        factory.connect_setup(glib::clone!(
            #[weak]
            f,
            move |_, item| {
                let item = list_item!(item);
                let item_row = f.setup();
                item.set_child(Some(&item_row));
            }
        ));
        factory.connect_bind(glib::clone!(move |_, list_item| {
            let list_item: &gtk4::ListItem = list_item!(list_item);
            let child = child_widget!(list_item, Self::Widget);
            if let Some(item) = list_item.item() {
                let tree_list_row = tree_list_row!(item);
                if let Some(item) = tree_list_row.item() {
                    let folder = item.downcast_ref::<T>().unwrap();
                    child.bind(folder, Some(tree_list_row));
                }
            }
        }));
        factory.connect_unbind(glib::clone!(move |_, list_item| {
            let list_item = list_item!(list_item);
            let child = child_widget!(list_item, Self::Widget);
            child.unbind();
        }));
        factory
    }
}
