/*
 * niepce - engine/library/op.rs
 *
 * Copyright (C) 2017-2025 Hubert Figuière
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

use crate::catalog::CatalogDb;

type Function = dyn FnOnce(&CatalogDb) -> bool + Send + Sync + 'static;

pub struct Op {
    op: Box<Function>,
}

impl Op {
    pub fn new<F>(f: F) -> Op
    where
        F: FnOnce(&CatalogDb) -> bool + Send + Sync + 'static,
    {
        Op { op: Box::new(f) }
    }

    pub fn execute(self, lib: &CatalogDb) -> bool {
        (self.op)(lib)
    }
}
