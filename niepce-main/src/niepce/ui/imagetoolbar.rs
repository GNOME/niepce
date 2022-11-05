/*
 * niepce - niepce/ui/imagetoolbar.rs
 *
 * Copyright (C) 2018-2022 Hubert Figuière
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

use glib::translate::*;
use gtk4::prelude::*;

/// Create a box for linked button.
fn linked_box() -> gtk4::Box {
    let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    box_.add_css_class("linked");

    box_
}

pub fn image_toolbar_new() -> *mut crate::ffi::GtkBox {
    let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    // XXX set style class "toolbar"

    toolbar.set_margin_top(4);
    toolbar.set_margin_bottom(4);
    toolbar.set_margin_start(4);
    toolbar.set_margin_end(4);

    let box_ = linked_box();
    let tool_item = gtk4::Button::from_icon_name("go-previous-symbolic");
    tool_item.set_action_name(Some("shell.PrevImage"));
    box_.append(&tool_item);

    let tool_item = gtk4::Button::from_icon_name("go-next-symbolic");
    tool_item.set_action_name(Some("shell.NextImage"));
    box_.append(&tool_item);
    toolbar.append(&box_);

    // let separator = gtk4::SeparatorToolItem::new();
    // toolbar.add(&separator);

    let box_ = linked_box();
    let tool_item = gtk4::Button::from_icon_name("object-rotate-left-symbolic");
    tool_item.set_action_name(Some("shell.RotateLeft"));
    box_.append(&tool_item);

    let tool_item = gtk4::Button::from_icon_name("object-rotate-right-symbolic");
    tool_item.set_action_name(Some("shell.RotateRight"));
    box_.append(&tool_item);
    toolbar.append(&box_);

    let gtkbox: *mut gtk4_sys::GtkBox = toolbar.to_glib_full();
    gtkbox as *mut crate::ffi::GtkBox
}
