/*
 * niepce - ui/dialogs/import/importer_ui.rs
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

use std::rc::Rc;

use npc_fwk::gtk4;

use npc_engine::importer::ImportBackend;

pub type SourceSelectedCallback = Box<dyn Fn(&str, &str)>;

/// An importer UI.
pub(super) trait ImporterUI {
    /// ID of the importer.
    fn id(&self) -> String;
    /// Name of the importer, displayed
    fn name(&self) -> &str;

    /// The actual importer
    fn backend(&self) -> Rc<dyn ImportBackend>;

    /// Setup the widget
    fn setup_widget(&self, parent: &gtk4::Window) -> gtk4::Widget;

    /// Callback for when the source is selected.
    ///
    /// XXX shall we switch to a signal?
    fn set_source_selected_callback(&self, callback: SourceSelectedCallback);
}
