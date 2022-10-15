/*
 * niepce - fwk/base/string.hpp
 *
 * Copyright (C) 2020-2022 Hubert Figuière
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

#include <string.h>

#include <memory>
#include <string>

namespace fwk {

/** A string coming from rust CString. */
class RustFfiString {
public:
  RustFfiString(char* p);

  bool empty() const {
    return !ptr || strlen(c_str()) == 0;
  }

  const char* c_str() const {
    return ptr.get();
  }

  std::string str() const {
    return std::string(ptr.get());
  }

private:
  std::shared_ptr<char> ptr;
};

}
