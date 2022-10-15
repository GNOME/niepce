/*
 * niepce - fwk/base/date.hpp
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

#pragma once

#include <memory>
#include <string>

#include <exempi/xmp.h>

#include "rust_bindings.hpp"

namespace fwk {

class Timezone;

/**
 * Fill the XmpDateTime %xmp_dt from a %t
 * @return false if gmtime_r failed.
 */
bool make_xmp_date_time(time_t t, XmpDateTime& xmp_dt);

typedef std::shared_ptr<Date> DatePtr;

std::string date_to_string(const Date*);
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
