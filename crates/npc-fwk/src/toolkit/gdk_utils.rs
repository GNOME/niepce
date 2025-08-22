/*
 * niepce - npc-fwk/toolkit/gdk_utils.rs
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

use std::cmp;

use crate::gdk_pixbuf;

/// Scale the pixbuf to fit in a square of %dim pixels
pub fn gdkpixbuf_scale_to_fit(
    pix: Option<&gdk_pixbuf::Pixbuf>,
    dim: u32,
) -> Option<gdk_pixbuf::Pixbuf> {
    pix.and_then(|pix| {
        let orig_h = pix.height();
        let orig_w = pix.width();
        let orig_dim = cmp::max(orig_h, orig_w);
        let ratio: f64 = dim as f64 / orig_dim as f64;
        let width = ratio * orig_w as f64;
        let height = ratio * orig_h as f64;
        pix.scale_simple(
            width as i32,
            height as i32,
            gdk_pixbuf::InterpType::Bilinear,
        )
    })
}
