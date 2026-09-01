/*
 * niepce - npc-engine/library/actions.rs
 *
 * Copyright (C) 2026 Hubert Figuière
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

//! Library actions. Actions have undo/redo and wrap commands.

use std::rc::Weak;
use std::sync::Arc;

use gettextrs::gettext as i18n;

use crate::catalog::{self, NiepcePropertyIdx};
use crate::libraryclient::{ClientInterface, LibraryClient};
use npc_fwk::PropertyValue;
use npc_fwk::base::RgbColour;
use npc_fwk::toolkit::widgets::MetadataPropertyBag;
use npc_fwk::toolkit::{AppController, Storage, UndoCommand, UndoTransaction};

/// Add `files` to `album`.
pub fn add_files_to_album<T>(
    client: &Arc<LibraryClient>,
    app: &Weak<T>,
    files: &[catalog::LibraryId],
    album: catalog::LibraryId,
) where
    T: AppController,
{
    let client_undo = client.clone();
    let client_redo = client.clone();
    let files1 = files.to_vec();
    let files2 = files1.clone();
    let app = Weak::upgrade(app).unwrap();
    npc_fwk::toolkit::undo_do_command(
        &app,
        &i18n("Add to Album"),
        Box::new(move || {
            client_redo.add_to_album(&files1, album);
            Storage::Void
        }),
        Box::new(move |_| client_undo.remove_from_album(&files2, album)),
    );
}

/// Remove `files` from `album`
pub fn remove_files_from_album<T>(
    client: &Arc<LibraryClient>,
    app: &Weak<T>,
    files: &[catalog::LibraryId],
    album: catalog::LibraryId,
) where
    T: AppController,
{
    let client_undo = client.clone();
    let client_redo = client.clone();
    let files1 = files.to_vec();
    let files2 = files1.clone();
    let app = Weak::upgrade(app).unwrap();
    npc_fwk::toolkit::undo_do_command(
        &app,
        &i18n("Remove from album"),
        Box::new(move || {
            client_redo.remove_from_album(&files1, album);
            Storage::Void
        }),
        Box::new(move |_| client_undo.add_to_album(&files2, album)),
    );
}

/// Move the `files` `from` folder to the trash.
///
/// XXX this shouldn't require to have `from` and instead should
/// be able to just move to trash.
pub fn move_files_to_trash<T>(
    client: &Arc<LibraryClient>,
    app: &Weak<T>,
    from: catalog::LibraryId,
    files: &[catalog::LibraryId],
) where
    T: AppController,
{
    let trash_folder = client.get_trash_id();
    let client_undo = client.clone();
    let client_redo = client.clone();
    let files1 = files.to_vec();
    let files2 = files1.clone();
    let app = Weak::upgrade(app).unwrap();
    npc_fwk::toolkit::undo_do_command(
        &app,
        &i18n("Move to Trash"),
        Box::new(move || {
            client_redo.move_files_to_folder(&files1, from, trash_folder);
            Storage::Void
        }),
        Box::new(move |_| client_undo.move_files_to_folder(&files2, trash_folder, from)),
    );
}

pub fn set_metadata_for_files<T>(
    client: &Arc<LibraryClient>,
    app: &Weak<T>,
    undo_label: &str,
    props: &MetadataPropertyBag,
    old: &MetadataPropertyBag,
    files: &[catalog::LibraryId],
) where
    T: AppController,
{
    let mut undo = UndoTransaction::new(undo_label);
    for key in props.keys() {
        let old_value = old.get(key).cloned().unwrap_or(PropertyValue::Empty);
        let new_value = props.get(key).cloned().unwrap();
        if let Ok(key) = NiepcePropertyIdx::try_from(*key) {
            let client_undo = client.clone();
            let client_redo = client.clone();
            let files1 = files.to_vec();
            let files2 = files1.clone();
            let command = UndoCommand::new(
                Box::new(move || {
                    client_redo.set_metadata(&files1, key, &new_value);
                    Storage::Void
                }),
                Box::new(move |_| {
                    client_undo.set_metadata(&files2, key, &old_value);
                }),
            );
            undo.add(command);
        }
    }
    undo.execute();
    let app = Weak::upgrade(app).unwrap();
    app.begin_undo(undo);
}

pub fn set_one_metadata_for_file<T>(
    client: &Arc<LibraryClient>,
    app: &Weak<T>,
    undo_label: &str,
    meta: NiepcePropertyIdx,
    old_value: i32,
    new_value: i32,
    file: catalog::LibraryId,
) where
    T: AppController,
{
    let client_undo = client.clone();
    let client_redo = client.clone();
    let app = Weak::upgrade(app).unwrap();
    npc_fwk::toolkit::undo_do_command(
        &app,
        undo_label,
        Box::new(move || {
            client_redo.set_metadata(&[file], meta, &PropertyValue::Int(new_value));
            Storage::Void
        }),
        Box::new(move |_| {
            client_undo.set_metadata(&[file], meta, &PropertyValue::Int(old_value));
        }),
    );
}

pub fn update_labels<T>(
    client: &Arc<LibraryClient>,
    app: &Weak<T>,
    labels: &[catalog::LibraryId],
    old: Vec<(String, RgbColour)>,
    new: Vec<(String, RgbColour)>,
) where
    T: AppController,
{
    let mut undo = UndoTransaction::new(&i18n("Update Label"));
    for label in std::iter::zip(labels, std::iter::zip(old, new)) {
        let client_undo = client.clone();
        let client_redo = client.clone();

        let label_id = *label.0;
        undo.add(UndoCommand::new(
            Box::new(move || {
                client_redo.update_label(label_id, label.1.1.0.clone(), label.1.1.1.clone());
                Storage::Void
            }),
            Box::new(move |_| {
                client_undo.update_label(label_id, label.1.0.0.clone(), label.1.0.1.clone());
            }),
        ));
    }
    undo.execute();
    if let Some(app) = Weak::upgrade(app) {
        app.begin_undo(undo);
    }
}
