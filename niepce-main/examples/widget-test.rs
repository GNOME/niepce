/*
 * niepce - examples/widget-test.rs
 *
 * Copyright (C) 2020-2022 Hubert Figuière
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

use niepce_rust::niepce::ui::image_grid_view::ImageGridView;
use niepce_rust::niepce::ui::thumb_nav::{ThumbNav, ThumbNavMode};
use niepce_rust::niepce::ui::thumb_strip_view::ThumbStripView;
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

pub fn main() {
    if let Err(err) = gtk4::init() {
        println!("main: gtk::init failed: {}", err);
        panic!();
    }

    if let Err(err) = init() {
        println!("main: init failed: {}", err);
        panic!();
    }

    let app = gtk4::Application::new(
        Some("org.gnome.Niepce.WidgetTest"),
        gio::ApplicationFlags::FLAGS_NONE,
    );

    app.connect_activate(|app| {
        let model = gtk4::ListStore::new(&[gdk_pixbuf::Pixbuf::static_type()]);
        let thumbview = ThumbStripView::new(model.upcast_ref::<gtk4::TreeModel>());
        (&thumbview).set_hexpand(true);
        let thn = ThumbNav::new(&thumbview.deref(), ThumbNavMode::OneRow, true);
        thn.set_size_request(-1, 134);

        let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let rating = RatingLabel::new(3, true);

        let image_grid = ImageGridView::new(model.upcast_ref::<gtk4::TreeModel>(), None);
        (&image_grid).set_hexpand(true);
        (&image_grid).set_vexpand(true);
        box_.append(&rating);
        let tb_item = npc_fwk::toolkit::widgets::ToolboxItem::new("Grid View");
        tb_item.set_child(Some(image_grid.deref()));
        box_.append(&tb_item);
        box_.append(&thn);

        let window = gtk4::Window::new();
        window.set_child(Some(&box_));
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
