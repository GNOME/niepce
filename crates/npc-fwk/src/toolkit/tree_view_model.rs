/*
 * niepce - npc-fwk/toolkit/tree_view_model.rs
 *
 * Copyright (C) 2024 Hubert Figuière
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

use gtk4::prelude::*;

// XXX make non public
pub mod css {
    //! Handle CSS hack to hide the tree exander.
    //!
    //! There is an API in gtk 4.10 to do that.

    use std::sync::Once;

    /// The class to mark a row having no children
    // XXX make non public
    pub const NOCHILDREN_CSS: &str = "nochildren";

    static LOAD_CSS: Once = Once::new();

    /// Load the CSS for the item row. Will do it once.
    // XXX make non public
    pub fn load() {
        LOAD_CSS.call_once(|| {
            if let Some(display) = gdk4::Display::default() {
                let provider = gtk4::CssProvider::new();
                provider.load_from_data(include_str!("tree_view.css"));
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }
        });
    }
}

/// A tree view model
pub struct TreeViewModel<T> {
    model: gtk4::TreeListModel,
    items: RefCell<Vec<T>>,
}

impl<T: IsA<glib::Object>> TreeViewModel<T> {
    pub fn new() -> Rc<TreeViewModel<T>> {
        let rootstore = gio::ListStore::new::<T>();
        let model = gtk4::TreeListModel::new(rootstore, false, true, |_item| {
            Some(gio::ListStore::new::<T>().upcast())
        });
        let items = RefCell::default();
        Rc::new(TreeViewModel { model, items })
    }

    pub fn model(&self) -> &gtk4::TreeListModel {
        &self.model
    }

    /// Append an item.
    pub fn append(&self, item: &T) {
        if let Ok(store) = self.model.model().downcast::<gio::ListStore>() {
            store.append(item)
        }
        self.items.borrow_mut().push(item.clone());
    }

    /// Bind to the listview widget. Will setup the factory.
    pub fn bind<F>(self: &Rc<Self>, listview: &gtk4::ListView, factory: &Rc<F>)
    where
        F: TreeViewFactory<T>,
    {
        self::css::load();
        let factory = factory.build();
        let selection_model = gtk4::SingleSelection::new(Some(self.model.clone()));
        listview.set_model(Some(&selection_model));
        listview.set_factory(Some(&factory));
        listview.add_css_class("npc");
    }
}

/// Trait to implement for the item factory in a the ListView.
pub trait TreeViewFactory<T>
where
    T: IsA<glib::Object>,
    Self: 'static,
{
    /// The widget type for the list item.
    type Widget: IsA<gtk4::Widget>;

    /// Setup the widget.
    fn setup(&self) -> Self::Widget;
    /// Bind the widget to the item.
    fn bind(&self, item: &T, child: &Self::Widget);
    /// Unbind the widget to the item.
    fn unbind(&self, _item: &T, _child: &Self::Widget) {}

    /// Create the factory for gtk4::ListView.
    fn build(self: &Rc<Self>) -> gtk4::SignalListItemFactory {
        let factory = gtk4::SignalListItemFactory::new();
        let f = self.clone();
        factory.connect_setup(move |_, item| {
            let item = item.downcast_ref::<gtk4::ListItem>().unwrap();
            let item_row = f.setup();
            item.set_child(Some(&item_row));
        });
        let f = self.clone();
        factory.connect_bind(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
            let child = list_item
                .child()
                .and_downcast_ref::<Self::Widget>()
                .unwrap()
                .clone();
            if let Some(item) = list_item.item() {
                let tree_list_row = item
                    .downcast_ref::<gtk4::TreeListRow>()
                    .expect("to be a TreeListRow");
                if let Some(item) = tree_list_row.item() {
                    let folder = item.downcast_ref::<T>().unwrap();
                    f.bind(folder, &child);
                }
            }
        });
        factory
    }
}
