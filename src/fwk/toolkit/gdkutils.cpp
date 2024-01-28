/*
 * niepce - fwk/toolkit/gdkutils.cpp
 *
 * Copyright (C) 2008-2024 Hubert Figuière
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

#include "gdkutils.hpp"

namespace fwk {
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
