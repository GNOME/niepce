/*
 * niepce - fwk/base/colour.hpp
 *
 * Copyright (C) 2009-2017 Hubert Figuiere
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

#include <memory>
#include <string>

#include "rust_bindings.hpp"

namespace fwk {

#if RUST_BINDGEN
class RgbColour;
#endif

typedef std::shared_ptr<RgbColour> RgbColourPtr;

RgbColourPtr rgbcolour_new(uint16_t r, uint16_t g, uint16_t b);
RgbColourPtr rgbcolour_clone(const RgbColour*);
RgbColourPtr rgbcolour_wrap(RgbColour*);

std::string rgbcolour_to_string(uint16_t r, uint16_t g, uint16_t b);
std::string rgbcolour_to_string(const RgbColour*);
}
