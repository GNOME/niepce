/*
 * npc-fwk - toolkit/resources.rs
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

use crate::glib;

/// Initialize the resources from static bytes.
pub fn init_resources(res: &'static [u8]) -> Result<(), glib::Error> {
    // Create Resource it will live as long the value lives.
    let gbytes = glib::Bytes::from_static(res);
    let resource = gio::Resource::from_data(&gbytes)?;

    // Register the resource so it won't be dropped and will continue
    // to live in memory.
    gio::resources_register(&resource);
    Ok(())
}
