/*
 * niepce - niepce/ui/dialogs/importlibrary.rs
 *
 * Copyright (C) 2021 Hubert Figuière
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

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use glib::clone;
use glib::translate::*;
use gtk;
use gtk::prelude::*;
use gtk::{Assistant, Builder};
use gtk_sys;

use crate::libraryclient::LibraryClientWrapper;

#[no_mangle]
pub extern "C" fn dialog_import_library(
    _client: &mut LibraryClientWrapper,
    parent: *mut gtk_sys::GtkWindow,
) {
    let parent_window = unsafe { gtk::Window::from_glib_none(parent) };
    ImportLibraryDialog::run(&parent_window);
}

#[derive(Default)]
struct ImportState {
    library_path: Option<PathBuf>,
}

type ImportStateRef = Rc<RefCell<ImportState>>;

struct ImportLibraryDialog {}

impl ImportLibraryDialog {
    fn run(parent: &gtk::Window) {
        let assistant = Assistant::new();

        let state: ImportStateRef = Rc::new(RefCell::new(ImportState::default()));

        assistant.connect_cancel(Self::cancel);
        assistant.set_forward_page_func(Some(Box::new(Self::forward_page)));

        let builder = Builder::new();
        if let Err(result) = builder.add_from_resource("/org/gnome/Niepce/ui/importlibrary.ui") {
            err_out!("couldn't find ui file: {}", result);
            return;
        }
        if let Some(page) = builder.object::<gtk::Widget>("page0") {
            assistant.insert_page(&page, 0);
            assistant.set_current_page(0);
        }
        if let Some(file_chooser) = builder.object::<gtk::FileChooserButton>("file_chooser") {
            file_chooser.connect_file_set(clone!(@weak state => move |w| {
                Self::library_file_set(w, state)
            }));
        }

        assistant.set_transient_for(Some(parent));
        assistant.set_modal(true);
        assistant.present();
    }

    fn forward_page(current: i32) -> i32 {
        match current {
            0 => 1,
            _ => 0,
        }
    }

    fn library_file_set(file_chooser: &gtk::FileChooserButton, state_ref: ImportStateRef) {
        let path = file_chooser.filename();
        state_ref.borrow_mut().library_path = path;
    }

    fn cancel(assistant: &Assistant) {
        dbg_out!("Assistant cancel");
        unsafe { assistant.destroy(); }
    }
}
