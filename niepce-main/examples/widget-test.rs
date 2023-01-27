/*
 * niepce - examples/widget-test.rs
 *
 * Copyright (C) 2020-2023 Hubert Figuière
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

use std::ops::Deref;

use gio::{resources_register, Resource};
use glib::{Bytes, Error};
use gtk4::prelude::*;

use niepce_rust::niepce::ui::image_grid_view::{ImageGridView, ImageListItem};
use niepce_rust::niepce::ui::thumb_nav::{ThumbNav, ThumbNavMode};
use niepce_rust::niepce::ui::thumb_strip_view::ThumbStripView;
use niepce_rust::niepce::ui::ModuleShellWidget;
use npc_engine::db::libfile::FileStatus;
use npc_fwk::toolkit::widgets::prelude::*;
use npc_fwk::toolkit::widgets::rating_label::RatingLabel;

fn init() -> Result<(), Error> {
    // load the gresource binary at build time and include/link it into the final
    // binary.
    // The assumption here is that it's built within the build system.
    let res_bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_DIR"),
        "/../src/niepce/npc-resources.gresource"
    ));

    // Create Resource it will live as long the value lives.
    let gbytes = Bytes::from_static(res_bytes.as_ref());
    let resource = Resource::from_data(&gbytes)?;

    // Register the resource so it won't be dropped and will continue to live in
    // memory.
    resources_register(&resource);
    Ok(())
}

const ICON_NAMES: [&str; 3] = [
    "/org/gnome/Niepce/pixmaps/niepce-transform-rotate.png",
    "/org/gnome/Niepce/pixmaps/niepce-missing.png",
    "/org/gnome/Niepce/pixmaps/niepce-image-generic.png",
];

fn add_icon(store: &gio::ListStore) {
    let num = store.n_items() as usize;
    let index = num % ICON_NAMES.len();
    let texture = gdk4::Texture::from_resource(ICON_NAMES[index]);
    let item = ImageListItem::new(
        Some(texture.clone().upcast::<gdk4::Paintable>()),
        None,
        Some(texture.upcast::<gdk4::Paintable>()),
        FileStatus::Ok,
    );
    store.append(&item);
}

pub fn main() {
    if let Err(err) = gtk4::init() {
        println!("main: gtk::init failed: {err}");
        panic!();
    }

    if let Err(err) = init() {
        println!("main: init failed: {err}");
        panic!();
    }

    let app = gtk4::Application::new(
        Some("org.gnome.Niepce.WidgetTest"),
        gio::ApplicationFlags::FLAGS_NONE,
    );

    app.connect_activate(|app| {
        let store = gio::ListStore::new(ImageListItem::static_type());
        let model = gtk4::SingleSelection::new(Some(&store));
        add_icon(&store);
        add_icon(&store);
        add_icon(&store);
        let thumbview = ThumbStripView::new(&model);
        thumbview.set_hexpand(true);
        let thn = ThumbNav::new(thumbview.deref(), ThumbNavMode::OneRow, true);
        thn.set_size_request(-1, 134);

        let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let rating = RatingLabel::new(3, true);

        let tb_item = npc_fwk::toolkit::widgets::ToolboxItem::new("Rating");
        tb_item.set_child(Some(&rating));
        box_.append(&tb_item);
        box_.append(&thn);

        let shell = ModuleShellWidget::new();
        shell.append_page(&box_, "main", "Main");

        let image_grid = ImageGridView::new(&model, None, None);
        image_grid.set_hexpand(true);
        image_grid.set_vexpand(true);
        shell.append_page(image_grid.deref(), "grid", "Grid View");

        let window = gtk4::Window::new();
        window.set_child(Some(&shell));
        window.connect_close_request(move |win| {
            if let Some(app) = win.application() {
                app.quit();
            }
            gtk4::Inhibit(false)
        });

        app.add_window(&window);

        window.present();
    });
    app.run();
}
