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
use pyo3::types::{PyCFunction, PyDict, PyTuple};

use crate::PythonApp;

/// Callback type to print to stdout.
type CoutCallback = std::sync::Arc<dyn Fn(&str) + Send + Sync>;

pub(crate) struct Engine {
    app: Box<dyn PythonApp>,
    cout: Option<CoutCallback>,
}

impl Engine {
    pub fn new(app: Box<dyn PythonApp>, cout: Option<CoutCallback>) -> Self {
        Self { app, cout }
    }

    /// Execute `code` into the python interpreter
    pub fn exec(&self, code: &str) -> pyo3::PyResult<()> {
        Python::attach(|py| {
            let app_module = self.app.module(py)?;
            let cout = self.cout.clone();
            let py_println = move |args: &Bound<'_, PyTuple>,
                                   _kwargs: Option<&Bound<'_, PyDict>>|
                  -> PyResult<_> {
                if let Some(cout) = &cout {
                    let mut s = args.extract::<(String,)>()?.0;
                    s += "\n";
                    cout(&s);
                }
                Ok(())
            };
            let py_println =
                PyCFunction::new_closure(py, Some(c"println"), None, py_println).unwrap();
            app_module.add_function(py_println)?;

            py.import("sys")?
                .getattr("modules")?
                .set_item(self.app.module_name(), app_module)?;
            let code = CString::new(code).unwrap();
            let result = py.run(&code, None, None);
            if let Some(cout) = &self.cout {
                if let Err(error) = result {
                    cout(&(error.value(py).to_string() + "\n"));
                    if let Some(traceback) = error.traceback(py) {
                        cout(&(traceback.to_string() + "\n"));
                    }
                }
            } else {
                result?;
            }

            Ok::<(), PyErr>(())
        })
    }
}
