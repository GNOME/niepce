/*
 * niepce - examples/widget-test.rs
 *
 * Copyright (C) 2020-2025 Hubert Figuière
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

use gtk4::prelude::*;
use npc_fwk::{gdk4, gio, glib, gtk4};

use niepce_core::init_resources;
use niepce_core::modules::ImageCanvas;
use niepce_core::niepce::ui::ModuleShellWidget;
use niepce_core::niepce::ui::image_grid_view::{ImageGridView, ImageListItem};
use niepce_core::niepce::ui::thumb_nav::{ThumbNav, ThumbNavMode};
use niepce_core::niepce::ui::thumb_strip_view::ThumbStripView;
use npc_engine::catalog::libfile::FileStatus;
use npc_fwk::toolkit::widgets::prelude::*;
use npc_fwk::toolkit::widgets::rating_label::RatingLabel;

const ICON_NAMES: [&str; 3] = [
    "/net/figuiere/Niepce/pixmaps/niepce-transform-rotate.png",
    "/net/figuiere/Niepce/pixmaps/niepce-missing.png",
    "/net/figuiere/Niepce/pixmaps/niepce-image-generic.png",
];

fn add_icon(store: &gio::ListStore) {
    let num = store.n_items() as usize;
    let index = num % ICON_NAMES.len();
    let texture = gdk4::Texture::from_resource(ICON_NAMES[index]);
    let item = ImageListItem::new(
        Some(texture.upcast::<gdk4::Paintable>()),
        None,
        FileStatus::Ok,
    );
    store.append(&item);
}

pub fn main() {
    if let Err(err) = gtk4::init() {
        println!("main: gtk::init failed: {err}");
        panic!();
    }

    init_resources().expect("main: init failed: {err}");

    let app = gtk4::Application::new(
        Some("net.figuiere.Niepce.WidgetTest"),
        gio::ApplicationFlags::default(),
    );

    app.connect_activate(|app| {
        let store = gio::ListStore::new::<ImageListItem>();
        add_icon(&store);
        add_icon(&store);
        add_icon(&store);
        let model = gtk4::SingleSelection::new(Some(store));
        let thumbview = ThumbStripView::new(model.clone());
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

        let image_grid = ImageGridView::new(model, None, None);
        image_grid.set_hexpand(true);
        image_grid.set_vexpand(true);
        shell.append_page(image_grid.deref(), "grid", "Grid View");

        let image_canvas = ImageCanvas::new();
        image_canvas.set_hexpand(true);
        image_canvas.set_vexpand(true);
        shell.append_page(&image_canvas, "dr", "Darkroom");

        let window = gtk4::Window::new();
        window.set_child(Some(&shell));
        window.connect_close_request(move |win| {
            if let Some(app) = win.application() {
                app.quit();
            }
            glib::Propagation::Proceed
        });

        app.add_window(&window);

        window.present();
    });
    app.run();
}
