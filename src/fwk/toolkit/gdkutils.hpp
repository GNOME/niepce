/*
 * niepce - fwk/toolkit/gdkutils.hpp
 *
 * Copyright (C) 2008-2022 Hubert Figuière
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

#pragma once

#include <gdkmm/pixbuf.h>
#include <gdkmm/rgba.h>

#include "rust_bindings.hpp"

namespace fwk {

	/** scale the pixbuf to fit in the square
	 * @param dim the dimension of the square
	 */
	Glib::RefPtr<Gdk::Pixbuf> gdkpixbuf_scale_to_fit(const Glib::RefPtr<Gdk::Pixbuf> & pix,
													 int dim);
	/** Rotate a pixbuf following the Exif rotation (may mirror too) */
	Glib::RefPtr<Gdk::Pixbuf> gdkpixbuf_exif_rotate(const Glib::RefPtr<Gdk::Pixbuf> & pixbuf,
													int exif_orientation);

  Gdk::RGBA rgbcolour_to_gdkcolor(const fwk::RgbColour& colour);
  fwk::RgbColourPtr gdkcolor_to_rgbcolour(const Gdk::RGBA & colour);
}
