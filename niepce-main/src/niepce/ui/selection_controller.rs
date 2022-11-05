/*
 * niepce - niepce/ui/selection_controller.rs
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

use std::cell::{Ref, RefCell, RefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use gettextrs::gettext;
use gtk4::prelude::*;

use super::image_list_store::ImageListStoreWrap;
use super::{are_same_selectable, ImageSelectable};
use crate::ffi::SelectionListener;
use crate::libraryclient::clientinterface::ClientInterface;
use crate::libraryclient::LibraryClient;
use crate::LibraryClientHost;
use npc_engine::db;
use npc_engine::db::props::NiepceProperties as Np;
use npc_engine::db::{LibFile, NiepcePropertyIdx};
use npc_engine::library::notification::LibNotification;
use npc_engine::ThumbnailCache;
use npc_fwk::base::Signal;
use npc_fwk::toolkit::widgets::WrappedPropertyBag;
use npc_fwk::toolkit::{Controller, ControllerImpl, UndoCommand, UndoTransaction};
use npc_fwk::{dbg_out, err_out, PropertyValue};

#[derive(PartialEq)]
enum Direction {
    Forward,
    Backwards,
}

#[derive(Default)]
pub struct SelectionHandler {
    store: Box<ImageListStoreWrap>,
    selectables: RefCell<Vec<Weak<dyn ImageSelectable>>>,
    pub signal_selected: Signal<db::LibraryId>,
    pub signal_activated: Signal<db::LibraryId>,
}

impl SelectionHandler {
    fn activated(&self, path: &gtk4::TreePath) {
        let selection = self.store.get_file_id_at_path(path);
        if selection != 0 {
            self.signal_activated.emit(selection);
        }
    }

    fn selected(&self, selectable: Rc<dyn ImageSelectable>) {
        if let Some(selection) = selectable.selected() {
            for all_selectable in self.selectables.borrow().iter() {
                if let Some(all_selectable) = all_selectable.upgrade() {
                    if !are_same_selectable(&*all_selectable, &*selectable) {
                        all_selectable.select_image(selection);
                    }
                }
            }
            self.signal_selected.emit(selection);
        }
    }

    fn add_selectable(&self, selectable: Weak<dyn ImageSelectable>) {
        self.selectables.borrow_mut().push(selectable);
    }
}

pub struct SelectionController {
    imp_: RefCell<ControllerImpl>,
    client: Arc<LibraryClient>,
    pub handler: Rc<SelectionHandler>,
}

impl Controller for SelectionController {
    /// What to do when ready.
    fn on_ready(&self) {}

    /// Return the implementation
    fn imp(&self) -> Ref<'_, ControllerImpl> {
        self.imp_.borrow()
    }

    /// Return the mutable implementation
    fn imp_mut(&self) -> RefMut<'_, ControllerImpl> {
        self.imp_.borrow_mut()
    }
}

impl SelectionController {
    pub fn new(client_host: &LibraryClientHost) -> Rc<SelectionController> {
        Rc::new(SelectionController {
            imp_: RefCell::new(ControllerImpl::default()),
            client: client_host.client().client(),
            handler: Rc::new(SelectionHandler::default()),
        })
    }

    pub fn on_lib_notification(&self, ln: &LibNotification, thumbnail_cache: &ThumbnailCache) {
        self.handler
            .store
            .on_lib_notification(ln, &self.client, thumbnail_cache);
    }

    pub fn add_selected_listener(&self, listener: cxx::UniquePtr<SelectionListener>) {
        self.handler.signal_selected.connect(move |id| {
            listener.call(id);
        })
    }

    pub fn add_activated_listener(&self, listener: cxx::UniquePtr<SelectionListener>) {
        self.handler.signal_activated.connect(move |id| {
            listener.call(id);
        })
    }

    pub fn add_selectable<S: ImageSelectable + 'static>(&self, selectable: &Rc<S>) {
        let wselectable = Rc::downgrade(selectable);
        selectable.image_list().unwrap().connect_selection_changed(
            glib::clone!(@strong wselectable, @weak self.handler as handler => move |_| {
                if let Some(selectable) = wselectable.upgrade() {
                    handler.selected(selectable);
                }
            }),
        );
        selectable.image_list().unwrap().connect_item_activated(
            glib::clone!(@weak self.handler as handler => move |_, path| {
                handler.activated(path);
            }),
        );
        self.handler.add_selectable(wselectable);
    }

    pub fn list_store(&self) -> &ImageListStoreWrap {
        &self.handler.store
    }

    /// Get the file with `id`.
    pub fn file(&self, id: db::LibraryId) -> Option<LibFile> {
        self.handler.store.file(id)
    }

    pub fn selection(&self) -> Option<db::LibraryId> {
        if self.handler.selectables.borrow().is_empty() {
            err_out!("Selectables list is empty");
            return None;
        }

        self.handler.selectables.borrow()[0]
            .upgrade()
            .and_then(|selectable| selectable.selected())
    }

    pub fn select_previous(&self) {
        self.selection_move(Direction::Backwards)
    }

    pub fn select_next(&self) {
        self.selection_move(Direction::Forward)
    }

    fn selection_move(&self, direction: Direction) {
        let selection = self.selection();
        if selection.is_none() {
            return;
        }

        let iter = self.handler.store.iter_from_id(selection.unwrap());
        if iter.is_none() {
            return;
        }

        let mut path = self.handler.store.liststore().path(iter.as_ref().unwrap());

        let moved = if direction == Direction::Backwards {
            path.prev()
        } else {
            path.next();
            true
        };

        if moved {
            let file_id = self.handler.store.get_file_id_at_path(&path);

            self.handler
                .selectables
                .borrow()
                .iter()
                .for_each(move |selectable| {
                    if let Some(selectable) = selectable.upgrade() {
                        selectable.select_image(file_id);
                    }
                });

            self.handler.signal_selected.emit(file_id)
        }
    }

    /// Rotate the selection by angle (in degrees), clockwise.
    /// A negative value goes counter clockwise.
    pub fn rotate(&self, _angle: i32) {
        err_out!("rotate is not implemented");
    }

    fn set_one_metadata(
        &self,
        undo_label: &str,
        file_id: db::LibraryId,
        meta: NiepcePropertyIdx,
        old_value: i32,
        new_value: i32,
    ) -> bool {
        let mut undo = Box::new(UndoTransaction::new(undo_label));
        let client_undo = self.client.clone();
        let client_redo = self.client.clone();
        let command = UndoCommand::new(
            Box::new(move || {
                client_redo.set_metadata(file_id, Np::Index(meta), &PropertyValue::Int(new_value));
                npc_fwk::toolkit::Storage::Void
            }),
            Box::new(move |_| {
                client_undo.set_metadata(file_id, Np::Index(meta), &PropertyValue::Int(old_value));
            }),
        );
        undo.add(command);
        undo.execute();
        npc_fwk::ffi::Application_app().begin_undo(undo);
        true
    }

    fn set_metadata(
        &self,
        undo_label: &str,
        file_id: db::LibraryId,
        props: &WrappedPropertyBag,
        old: &WrappedPropertyBag,
    ) -> bool {
        let mut undo = Box::new(UndoTransaction::new(undo_label));
        for key in props.0.keys() {
            let old_value = old.0.value(key).cloned().unwrap_or(PropertyValue::Empty);
            let new_value = props.0.value(key).cloned().unwrap();
            let key = *key;
            let client_undo = self.client.clone();
            let client_redo = self.client.clone();
            let command = UndoCommand::new(
                Box::new(move || {
                    client_redo.set_metadata(file_id, Np::from(key), &new_value);
                    npc_fwk::toolkit::Storage::Void
                }),
                Box::new(move |_| {
                    client_undo.set_metadata(file_id, Np::from(key), &old_value);
                }),
            );
            undo.add(command);
        }
        undo.execute();
        npc_fwk::ffi::Application_app().begin_undo(undo);
        true
    }

    pub fn set_label(&self, label: i32) {
        self.set_property(db::NiepcePropertyIdx::NpXmpLabelProp, label);
    }

    pub fn set_rating(&self, rating: i32) {
        self.set_property(db::NiepcePropertyIdx::NpXmpRatingProp, rating);
    }

    pub fn set_flag(&self, flag: i32) {
        self.set_property(db::NiepcePropertyIdx::NpNiepceFlagProp, flag);
    }

    fn set_property(&self, idx: db::NiepcePropertyIdx, value: i32) {
        dbg_out!("property {} = {}", idx.repr, value);
        if let Some(selection) = self.selection() {
            if let Some(mut file) = self.handler.store.file(selection) {
                dbg_out!("old property is {}", file.property(Np::Index(idx)));
                let old_value = file.property(Np::Index(idx));
                let action = match idx {
                    NiepcePropertyIdx::NpNiepceFlagProp => gettext("Set Flag"),
                    NiepcePropertyIdx::NpXmpRatingProp => gettext("Set Rating"),
                    NiepcePropertyIdx::NpXmpLabelProp => gettext("Set Label"),
                    _ => gettext("Set Property"),
                };
                self.set_one_metadata(&action, selection, idx, old_value, value);
                // we need to set the property here so that undo/redo works
                // consistently.
                file.set_property(Np::Index(idx), value);
            } else {
                err_out!("requested file {} not found!", selection);
            }
        }
    }

    pub fn set_properties(&self, props: &WrappedPropertyBag, old: &WrappedPropertyBag) {
        if let Some(selection) = self.selection() {
            self.set_metadata(&gettext("Set Properties"), selection, props, old);
        }
    }

    pub fn content_will_change(&self) {
        self.handler.store.clear_content();
    }

    pub fn write_metadata(&self) {
        if let Some(selection) = self.selection() {
            self.client.write_metadata(selection);
        }
    }

    /// Move selection to trash
    pub fn move_to_trash(&self) {
        let trash_folder = self.client.get_trash_id();
        if let Some(selection) = self.selection() {
            if let Some(f) = self.handler.store.file(selection) {
                let from_folder = f.folder_id();
                let mut undo = Box::new(UndoTransaction::new(&gettext("Move to Trash")));
                let client_undo = self.client.clone();
                let client_redo = self.client.clone();
                let command = UndoCommand::new(
                    Box::new(move || {
                        client_undo.move_file_to_folder(selection, from_folder, trash_folder);
                        npc_fwk::toolkit::Storage::Void
                    }),
                    Box::new(move |_| {
                        client_redo.move_file_to_folder(selection, trash_folder, from_folder)
                    }),
                );
                undo.add(command);
                undo.execute();
                npc_fwk::ffi::Application_app().begin_undo(undo);
            }
        }
    }
}
