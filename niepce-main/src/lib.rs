/*
 * niepce - lib.rs
 *
 * Copyright (C) 2017-2022 Hubert Figuière
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

pub mod libraryclient;
pub mod niepce;
mod notification_center;

use std::sync::Once;

fn niepce_init() {
    static START: Once = Once::new();

    START.call_once(|| {
        gtk4::init().unwrap();
        npc_fwk::init();
    });
}

pub use notification_center::NotificationCenter;

// cxx bindings
use crate::libraryclient::{
    library_client_host_delete, library_client_host_new, LibraryClientHost, LibraryClientWrapper,
    UIDataProvider,
};

use niepce::ui::image_list_store::ImageListStoreWrap;
use niepce::ui::metadata_pane_controller::get_format;
use niepce::ui::niepce_window::{niepce_window_new, NiepceWindowWrapper};
use niepce::ui::{ImageListStore, SelectionController};
use notification_center::notification_center_new;
use npc_fwk::toolkit;

#[cxx::bridge(namespace = "npc")]
mod ffi {
    #[namespace = ""]
    unsafe extern "C++" {
        type GMenu;
        type GtkIconView;
        type GtkWidget;
    }

    extern "Rust" {
        fn niepce_init();
    }

    #[namespace = "fwk"]
    extern "C++" {
        include!("fwk/cxx_colour_bindings.hpp");
        include!("fwk/cxx_widgets_bindings.hpp");

        type Moniker = npc_fwk::base::Moniker;
        type RgbColour = npc_fwk::base::rgbcolour::RgbColour;
        type MetadataSectionFormat = crate::toolkit::widgets::MetadataSectionFormat;
    }

    #[namespace = "eng"]
    extern "C++" {
        include!("fwk/cxx_prelude.hpp");
        type Label = npc_engine::db::Label;
        type LibFile = npc_engine::db::LibFile;
        type ThumbnailCache = npc_engine::ThumbnailCache;
        type LcChannel = npc_engine::library::notification::LcChannel;
        type LibNotification = npc_engine::library::notification::LibNotification;
    }

    extern "Rust" {
        fn get_format() -> &'static [MetadataSectionFormat];
    }

    extern "Rust" {
        type UIDataProvider;

        #[cxx_name = "addLabel"]
        fn add_label(&self, label: &Label);
        #[cxx_name = "updateLabel"]
        fn update_label(&self, label: &Label);
        #[cxx_name = "deleteLabel"]
        fn delete_label(&self, label: i64);
        fn label_count(&self) -> usize;
        fn label_at(&self, idx: usize) -> *mut Label;
        #[cxx_name = "colourForLabel"]
        fn colour_for_label(&self, idx: i64) -> RgbColour;
    }

    extern "Rust" {
        type LibraryClientWrapper;
    }

    extern "Rust" {
        type LibraryClientHost;

        #[cxx_name = "LibraryClientHost_new"]
        fn library_client_host_new(
            moniker: &Moniker,
            channel: &LcChannel,
        ) -> *mut LibraryClientHost;
        #[cxx_name = "LibraryClientHost_delete"]
        unsafe fn library_client_host_delete(host: *mut LibraryClientHost);

        #[cxx_name = "getDataProvider"]
        fn ui_provider(&self) -> &UIDataProvider;
        fn client(&self) -> &LibraryClientWrapper;
        #[cxx_name = "thumbnailCache"]
        fn thumbnail_cache(&self) -> &ThumbnailCache;
    }

    unsafe extern "C++" {
        include!("niepce/lnlistener.hpp");
        type LnListener;

        fn call(&self, ln: &LibNotification);
    }

    extern "Rust" {
        type NotificationCenter;

        #[cxx_name = "NotificationCenter_new"]
        fn notification_center_new() -> Box<NotificationCenter>;
        #[cxx_name = "get_channel"]
        fn channel(&self) -> &LcChannel;
        fn add_listener(&self, listener: UniquePtr<LnListener>);
    }

    extern "Rust" {
        type NiepceWindowWrapper;

        unsafe fn niepce_window_new(app: *mut c_char) -> Box<NiepceWindowWrapper>;
        fn on_ready(&self);
        fn on_open_catalog(&self);
        fn widget(&self) -> *mut c_char;
        fn window(&self) -> *mut c_char;
        fn menu(&self) -> *mut c_char;
    }

    #[namespace = "ui"]
    extern "Rust" {
        type ImageListStore;

        fn clear_content(&self);
        fn gobj(&self) -> *mut c_char;
        fn get_file_(&self, id: i64) -> *mut LibFile;
        #[cxx_name = "get_libfile_id_at_path"]
        unsafe fn get_file_id_at_path_(&self, path: *const c_char) -> i64;
        fn get_iter_from_id_(&self, id: i64) -> *const c_char;
    }

    #[namespace = "ui"]
    extern "Rust" {
        type ImageListStoreWrap;

        unsafe fn unwrap_ref(&self) -> &ImageListStore;
        #[cxx_name = "clone"]
        fn clone_(&self) -> Box<ImageListStoreWrap>;
    }

    #[namespace = "fwk"]
    extern "C++" {
        include!("fwk/cxx_widgets_bindings.hpp");

        type WrappedPropertyBag = crate::toolkit::widgets::WrappedPropertyBag;
    }

    unsafe extern "C++" {
        include!("niepce/ui/selection_listener.hpp");
        type SelectionListener;

        fn call(&self, id: i64);
    }

    #[namespace = "ui"]
    extern "Rust" {
        type SelectionController;

        fn add_selected_listener(&self, listener: UniquePtr<SelectionListener>);
        fn add_activated_listener(&self, listener: UniquePtr<SelectionListener>);
        fn select_previous(&self);
        fn select_next(&self);
        #[cxx_name = "get_list_store"]
        fn list_store(&self) -> &ImageListStoreWrap;

        fn get_file(&self, id: i64) -> *mut LibFile;
        fn rotate(&self, angle: i32);
        fn set_label(&self, label: i32);
        fn set_rating(&self, rating: i32);
        fn set_flag(&self, flag: i32);
        fn set_properties(&self, props: &WrappedPropertyBag, old: &WrappedPropertyBag);
        fn content_will_change(&self);
        fn write_metadata(&self);
        fn move_to_trash(&self);
    }

    #[namespace = "ui"]
    unsafe extern "C++" {
        include!("niepce/ui/gridviewmodule.hpp");
        type GridViewModule;

        /// # Safety
        /// Dereference a pointer
        unsafe fn grid_view_module_new(
            selection_controller: &SelectionController,
            menu: *const GMenu,
            ui_data_provider: &UIDataProvider,
        ) -> SharedPtr<GridViewModule>;
        // call buildWidget(). But it's mutable.
        fn build_widget(&self) -> *const GtkWidget;
        fn on_lib_notification(&self, ln: &LibNotification, client: &LibraryClientWrapper);
        fn display_none(&self);
        #[cxx_name = "cxx_image_list"]
        fn image_list(&self) -> *const GtkIconView;
        fn get_selected(&self) -> i64;
        fn select_image(&self, id: i64);
    }
}
