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

use npc_fwk::gtk4::prelude::*;

use npc_fwk::dbg_out;

pub use render_worker::{RenderImpl, RenderWorker};

fn ncr_init() {
    static START: Once = Once::new();

    START.call_once(|| {
        if std::env::var("DISABLE_OPENCL").is_ok() {
            let config = gegl::config();
            dbg_out!("Disabling opencl");
            config.set_property("use-opencl", false);
        }
        gegl::init();
    });
}
