/*
 * niepce - niepce-main/python.rs
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

//! Python bindings.

use pyo3::ffi::c_str;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

#[pyfunction(name = "version")]
fn version() {
    println!("Hello this is Niepce");
}

pub struct NiepcePython;

impl npc_python::PythonApp for NiepcePython {
    /// Create the `niepce` python module.
    fn module<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
        let niepce = PyModule::new(py, Self::module_name())?;
        niepce.add_function(wrap_pyfunction!(version, &niepce)?)?;

        Ok(niepce)
    }

    fn module_name() -> &'static str {
        "niepce"
    }
}

pub fn test_python<App: npc_python::PythonApp>() {
    Python::attach(|py| {
        let sys = py.import("sys")?;
        let version: String = sys.getattr("version")?.extract()?;
        let niepce = App::module(py)?;

        let locals = [("os", py.import("os")?)].into_py_dict(py)?;
        let code = c"os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'";
        let user: String = py.eval(code, None, Some(&locals))?.extract()?;
        println!("Hello {user}, I'm Python {version}");

        let locals = [(App::module_name(), niepce)].into_py_dict(py)?;
        let code = c_str!(
            r#"
niepce.version()
"#
        );
        py.run(code, None, Some(&locals))?;

        Ok::<(), PyErr>(())
    })
    .expect("Python initialization failed");
}
