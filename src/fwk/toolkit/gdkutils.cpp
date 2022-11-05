/*
 * niepce - fwk/toolkit/gdkutils.cpp
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

#include "fwk/base/debug.hpp"
#include "gdkutils.hpp"

namespace fwk {

	Glib::RefPtr<Gdk::Pixbuf> gdkpixbuf_scale_to_fit(const Glib::RefPtr<Gdk::Pixbuf> & pix,
													 int dim)
	{
		int height, width;
		int orig_h, orig_w;
		if(!pix) {
			ERR_OUT("NULL pixbuf");
			return pix;
		}
		orig_h = pix->get_height();
		orig_w = pix->get_width();
		int orig_dim = std::max(orig_h, orig_w);
		double ratio = (double)dim / (double)orig_dim;
		width = (int)(ratio * orig_w);
		height = (int)(ratio * orig_h);
		return pix->scale_simple(width, height,
								 Gdk::InterpType::BILINEAR);
	}

	Glib::RefPtr<Gdk::Pixbuf> gdkpixbuf_exif_rotate(const Glib::RefPtr<Gdk::Pixbuf> & tmp,
													int exif_orientation)
	{
		Glib::RefPtr<Gdk::Pixbuf> pixbuf;
		switch(exif_orientation) {
		case 0:
		case 1:
			pixbuf = tmp;
			break;
		case 2:
			pixbuf = tmp->flip(true);
			break;
		case 3:
			pixbuf = tmp->rotate_simple(Gdk::Pixbuf::Rotation::UPSIDEDOWN);
			break;
		case 4:
			pixbuf = tmp->rotate_simple(Gdk::Pixbuf::Rotation::UPSIDEDOWN)->flip(true);
			break;
		case 5:
			pixbuf = tmp->rotate_simple(Gdk::Pixbuf::Rotation::CLOCKWISE)->flip(false);
			break;
		case 6:
			pixbuf =  tmp->rotate_simple(Gdk::Pixbuf::Rotation::CLOCKWISE);
			break;
		case 7:
			pixbuf =  tmp->rotate_simple(Gdk::Pixbuf::Rotation::COUNTERCLOCKWISE)->flip(false);
			break;
		case 8:
			pixbuf =  tmp->rotate_simple(Gdk::Pixbuf::Rotation::COUNTERCLOCKWISE);
			break;
		default:
			break;
		}
		return pixbuf;
	}


  Gdk::RGBA rgbcolour_to_gdkcolor(const fwk::RgbColour& colour)
  {
    Gdk::RGBA gdkcolour;
    gdkcolour.set_rgba_u(colour.r, colour.g, colour.b);
    return gdkcolour;
  }

  fwk::RgbColourPtr gdkcolor_to_rgbcolour(const Gdk::RGBA & colour)
  {
    return fwk::RgbColour_new(colour.get_red_u(), colour.get_green_u(), colour.get_blue_u());
  }


}
