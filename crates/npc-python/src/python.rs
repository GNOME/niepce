/*
 * niepce - npc-python/src/python.rs
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

//! Python bindings infrastructure.

use pyo3::prelude::*;

/// Trait to implement a PythonApp.
pub trait PythonApp {
    /// Return the module for the Python application.
    fn module<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyModule>>;
    /// Return the name of the module.
    fn module_name(&self) -> &'static str;
}
