/*
 * niepce - npc-engine/src/importer/imported_file.rs
 *
 * Copyright (C) 2022 Hubert Figuière
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

pub trait ImportedFile {
    fn name(&self) -> &str;
    fn path(&self) -> &str;
    /// remove when the cxx binding is gone
    fn folder(&self) -> &str;
    fn clone_(&self) -> Box<dyn ImportedFile>;
}

// cxx use only
/// Wrap the box for C++ interface
/// `0` is the box, `1` is whether it's a camera imported file.
pub struct WrappedImportedFile(pub Box<dyn ImportedFile>, pub bool);

impl WrappedImportedFile {
    pub fn name(&self) -> &str {
        self.0.name()
    }

    pub fn path(&self) -> &str {
        self.0.path()
    }

    pub fn is_camera(&self) -> bool {
        self.1
    }

    /// Must be called only if `is_camera()` is true
    ///
    /// # Panic
    /// If is_camera is false
    pub fn folder(&self) -> &str {
        self.0.folder()
    }

    pub fn clone_(&self) -> Box<Self> {
        Box::new(Self(self.0.clone_(), self.1))
    }
}
