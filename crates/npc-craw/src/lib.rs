/*
 * niepce - npc-craw/lib.rs
 *
 * Copyright (C) 2023-2024 Hubert Figuière
 *
 * This program is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 3 of the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this program; if not, see
 * <http://www.gnu.org/licenses/>.
 */

mod pipeline;
mod render_worker;

use std::sync::Once;

pub use render_worker::{RenderImpl, RenderWorker};

fn ncr_init() {
    static START: Once = Once::new();

    START.call_once(|| {
        crate::ffi::init();
    });
}

#[cxx::bridge(namespace = "ncr")]
mod ffi {
    #[namespace = ""]
    unsafe extern "C++" {
        include!(<gdk-pixbuf/gdk-pixbuf.h>);
        include!(<gdk/gdk.h>);
        include!(<gegl.h>);
        include!(<babl/babl.h>);

        type GdkPixbuf;
        type GdkTexture;
        type GObject;
        type GeglNode;
        type Babl;
    }

    #[rust_name = "ImageStatus"]
    #[derive(Debug, PartialOrd)]
    enum Status {
        UNSET = 0,
        LOADING,
        LOADED,
        ERROR,
        NOT_FOUND,
    }

    #[cxx_name = "GeglRectangle_"]
    struct GeglRectangle {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    }

    unsafe extern "C++" {
        include!("ncr/init.hpp");
        fn init();
    }

    #[namespace = "ncr"]
    unsafe extern "C++" {
        unsafe fn gegl_node_new_child(
            node: *mut GeglNode,
            prop1: *const c_char,
            value1: *const u8,
            prop2: *const u8,
            value2: *const u8,
        ) -> *mut GeglNode;
        unsafe fn gegl_node_new_child_so(
            node: *mut GeglNode,
            prop1: *const c_char,
            value1: *const u8,
            prop2: *const u8,
            value2: *mut GObject,
        ) -> *mut GeglNode;
        unsafe fn gegl_node_new_child_sf(
            node: *mut GeglNode,
            prop1: *const c_char,
            value1: *const u8,
            prop2: *const u8,
            value2: f64,
        ) -> *mut GeglNode;
        unsafe fn gegl_node_new_child_sff(
            node: *mut GeglNode,
            prop1: *const c_char,
            value1: *const u8,
            prop2: *const u8,
            value2: f64,
            prop3: *const u8,
            value3: f64,
        ) -> *mut GeglNode;
        unsafe fn gegl_node_create_child(node: *mut GeglNode, op: *const c_char) -> *mut GeglNode;
        fn gegl_node_new() -> *mut GeglNode;
        unsafe fn gegl_node_process(node: *mut GeglNode);
        unsafe fn gegl_node_link_many(
            node: *mut GeglNode,
            node1: *mut GeglNode,
            node2: *mut GeglNode,
            node3: *mut GeglNode,
        );
        unsafe fn gegl_node_get_bounding_box_w(node: *mut GeglNode) -> i32;
        unsafe fn gegl_node_get_bounding_box_h(node: *mut GeglNode) -> i32;
        //        unsafe fn gegl_node_set(node: *mut GeglNode, prop1: *const c_char, val1: f64,
        //                                prop2: *const u8, val2: f64);
        unsafe fn gegl_node_blit(
            node: *mut GeglNode,
            scale: f64,
            roi: &GeglRectangle,
            format: *const Babl,
            destination: *mut u8,
            stride: i32,
            flags: i32,
        );

        unsafe fn babl_format(format: *const c_char) -> *const Babl;
    }
}

pub use ffi::ImageStatus;
