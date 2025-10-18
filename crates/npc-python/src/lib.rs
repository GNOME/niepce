/*
 * npc-python - lib.rs
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

#[macro_use]
extern crate gtk_macros;

mod editor;
mod python;

pub use editor::Editor;
pub use python::PythonApp;

use npc_fwk::glib;

// Initialize the resource as we can use the C trick,
// we inline and load them.
pub fn init_resources() -> Result<(), glib::Error> {
    // load the gresource binary at build time and include/link it into the final
    // binary.
    // The assumption here is that it's built within the build system.
    let res_bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_DIR"),
        "/../crates/npc-python/npc-python-resources.gresource"
    ));
    npc_fwk::toolkit::resources::init_resources(res_bytes)
}
