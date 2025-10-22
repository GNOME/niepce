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

use pyo3::prelude::*;

#[pyfunction]
/// Return the application version.
/// This is used to test basic python works.
fn version() -> String {
    crate::config::VERSION.into()
}

#[pyfunction(name = "println")]
/// Println a string
///
fn py_println(s: &str) {
    println!("{s}");
}

pub struct NiepcePython;

impl npc_python::PythonApp for NiepcePython {
    /// Create the `niepce` python module.
    fn module<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
        let niepce = PyModule::new(py, self.module_name())?;
        niepce.add_function(wrap_pyfunction!(version, &niepce)?)?;
        niepce.add_function(wrap_pyfunction!(py_println, &niepce)?)?;

        Ok(niepce)
    }

    fn module_name(&self) -> &'static str {
        "niepce"
    }
}

#[cfg(test)]
mod tests {
    use pyo3::ffi::c_str;
    use pyo3::prelude::*;
    use pyo3::types::IntoPyDict;

    use super::NiepcePython;
    use npc_python::PythonApp;

    #[test]
    fn test_python() {
        Python::attach(|py| {
            let app = NiepcePython {};
            let niepce = app.module(py)?;
            let locals = [(app.module_name(), niepce)].into_py_dict(py)?;
            let code = c_str!(
                r#"
niepce.version()
"#
            );
            let res = py.eval(code, None, Some(&locals)).unwrap();
            let v: &str = res.extract().unwrap();
            assert_eq!(v, crate::config::VERSION);

            Ok::<(), PyErr>(())
        })
        .expect("Fail to run python");
    }
}
