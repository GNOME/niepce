/*
 * niepce - crates/npc-fwk/src/toolkit/undo.rs
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

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;

use crate::base::Signal;
use crate::toolkit::AppController;

pub enum Storage {
    Int(i64),
    Void,
}

impl From<i64> for Storage {
    fn from(v: i64) -> Self {
        Storage::Int(v)
    }
}

impl From<()> for Storage {
    fn from(_: ()) -> Self {
        Storage::Void
    }
}

impl From<&Storage> for i64 {
    fn from(v: &Storage) -> i64 {
        if let Storage::Int(i) = *v {
            i
        } else {
            0
        }
    }
}

impl From<Storage> for () {
    fn from(v: Storage) {
        assert!(matches!(v, Storage::Void));
    }
}

/// The base command for an undo operation. Redo is executing, undo is
/// the reverse.
pub struct UndoCommand {
    /// Storage for the return value of redo_fn to pass to undo_fn
    storage: RefCell<Storage>,
    redo_fn: Box<dyn Fn() -> Storage>,
    undo_fn: Box<dyn Fn(&Storage)>,
}

impl UndoCommand {
    pub fn new(redo_fn: Box<dyn Fn() -> Storage>, undo_fn: Box<dyn Fn(&Storage)>) -> UndoCommand {
        UndoCommand {
            storage: RefCell::new(Storage::Void),
            redo_fn,
            undo_fn,
        }
    }

    /// Call undo_fn
    pub fn undo(&self) {
        (self.undo_fn)(&self.storage.borrow());
    }

    /// Call redo_fn
    pub fn redo(&self) {
        self.storage.replace((self.redo_fn)());
    }
}

/// And `UndoTransaction` is we is run for an undo or redo
/// Operations are executed in reverse order for undo.
pub struct UndoTransaction {
    name: String,
    operations: Vec<UndoCommand>,
}

impl UndoTransaction {
    /// Create a transaction with `name`
    ///
    /// Name is meant to be used displayed, so it should be
    /// properly phrased and localized.
    pub fn new(name: &str) -> UndoTransaction {
        UndoTransaction {
            name: name.to_string(),
            operations: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// add a command to the transaction
    pub fn add(&mut self, command: UndoCommand) {
        self.operations.push(command);
    }

    /// Perform the undo
    pub fn undo(&self) {
        self.operations.iter().rev().for_each(|op| op.undo());
    }

    /// Perform the redo
    pub fn redo(&self) {
        self.operations.iter().for_each(|op| op.redo());
    }

    pub fn execute(&self) {
        self.redo();
    }
}

/// The history of all the transactions.
/// This is the hear of the undo / redo.
#[derive(Default)]
pub struct UndoHistory {
    /// A LIFO queue of the undos
    undos: RefCell<VecDeque<UndoTransaction>>,
    /// A LIFO queue of the redo
    redos: RefCell<VecDeque<UndoTransaction>>,
    /// When the state changed.
    pub signal_changed: Signal<()>,
}

impl UndoHistory {
    /// Add the transaction. This clear the redos.
    pub fn add(&self, transaction: UndoTransaction) {
        {
            self.undos.borrow_mut().push_back(transaction);
            self.redos.borrow_mut().clear();
        }
        self.signal_changed.emit(());
    }

    pub fn has_undo(&self) -> bool {
        !self.undos.borrow().is_empty()
    }

    pub fn has_redo(&self) -> bool {
        !self.redos.borrow().is_empty()
    }

    /// The name of the next undo operation
    pub fn next_undo(&self) -> String {
        self.undos
            .borrow()
            .back()
            .map(|t| t.name.to_string())
            .unwrap_or_default()
    }

    /// The name of the next undo operation
    pub fn next_redo(&self) -> String {
        self.redos
            .borrow()
            .back()
            .map(|t| t.name.to_string())
            .unwrap_or_default()
    }

    /// Perform the undo operation
    pub fn undo(&self) {
        let changed = if let Some(transaction) = self.undos.borrow_mut().pop_back() {
            transaction.undo();
            self.redos.borrow_mut().push_back(transaction);
            true
        } else {
            false
        };
        if changed {
            self.signal_changed.emit(());
        }
    }

    /// Perform the redo operation
    pub fn redo(&self) {
        let changed = if let Some(transaction) = self.redos.borrow_mut().pop_back() {
            transaction.redo();
            self.undos.borrow_mut().push_back(transaction);
            true
        } else {
            false
        };
        if changed {
            self.signal_changed.emit(());
        }
    }
}

/// An all around wrapper to create and run and undoable command
pub fn do_command(
    app: &Arc<dyn AppController>,
    label: &str,
    redo_fn: Box<dyn Fn() -> Storage>,
    undo_fn: Box<dyn Fn(&Storage)>,
) {
    let mut transaction = UndoTransaction::new(label);
    let command = UndoCommand::new(redo_fn, undo_fn);
    transaction.add(command);
    transaction.execute();
    app.begin_undo(transaction);
}

#[cfg(test)]
mod test {
    use super::{UndoHistory, UndoTransaction};

    #[test]
    fn test_undo_history() {
        let history = UndoHistory::default();
        assert!(!history.has_undo());
        assert!(!history.has_redo());

        let transaction = UndoTransaction::new("Jump");
        history.add(transaction);
        assert!(history.has_undo());
        assert!(!history.has_redo());
        assert_eq!(&history.next_undo(), "Jump");
        assert_eq!(&history.next_redo(), "");

        history.undo();
        assert!(history.has_redo());
        assert!(!history.has_undo());
        assert_eq!(&history.next_undo(), "");
        assert_eq!(&history.next_redo(), "Jump");

        let transaction = UndoTransaction::new("Duck");
        history.add(transaction);
        assert!(history.has_undo());
        assert!(!history.has_redo());
        assert_eq!(&history.next_undo(), "Duck");

        let transaction = UndoTransaction::new("Run");
        history.add(transaction);
        assert!(history.has_undo());
        assert!(!history.has_redo());
        assert_eq!(&history.next_undo(), "Run");

        history.undo();
        assert!(history.has_undo());
        assert!(history.has_redo());
        assert_eq!(&history.next_undo(), "Duck");
        assert_eq!(&history.next_redo(), "Run");

        history.redo();
        assert!(history.has_undo());
        assert!(!history.has_redo());
        assert_eq!(&history.next_undo(), "Run");
    }
}
