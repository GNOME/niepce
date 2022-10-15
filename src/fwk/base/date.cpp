/*
 * niepce - fwk/base/date.cpp
 *
 * Copyright (C) 2012-2022 Hubert Figuière
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

#include <stdio.h>
#include <time.h>

#include "string.hpp"
#include "date.hpp"
#include "debug.hpp"

#include "rust_bindings.hpp"

// cbindgen seems to not like to generate declaration for this.
// XXX remove when it does
namespace ffi {
extern "C" char* fwk_date_to_string(const fwk::Date*);
}

namespace fwk {

std::string date_to_string(const Date* d)
{
    DBG_ASSERT(d, "d is nullptr");
    if (!d) {
        return "";
    }
    auto p = fwk::RustFfiString(ffi::fwk_date_to_string(d));
    return p.str();
}

}

/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  fill-column:99
  End:
*/
