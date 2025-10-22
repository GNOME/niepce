/*
 * niepce - npc-python/src/engine.rs
 *
 * Copyright (C) 2025 Hubert Figuière
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

//! Python engine.

use std::ffi::CString;

use pyo3::prelude::*;

use crate::PythonApp;

pub(crate) struct Engine {
    app: Box<dyn PythonApp>,
}

impl Engine {
    pub fn new(python_app: Box<dyn PythonApp>) -> Self {
        Self { app: python_app }
    }

    /// Execute `code` into the python interpreter
    pub fn exec(&self, code: &str) -> pyo3::PyResult<()> {
        Python::attach(|py| {
            let app_module = self.app.module(py)?;
            py.import("sys")?
                .getattr("modules")?
                .set_item(self.app.module_name(), app_module)?;
            let code = CString::new(code).unwrap();
            py.run(&code, None, None)?;

            Ok::<(), PyErr>(())
        })
    }
}
