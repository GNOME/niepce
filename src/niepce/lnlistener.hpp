/*
 * niepce - niepce/lnlistener.hpp
 *
 * Copyright (C) 2022 Hubert Figuière
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

#include <functional>

namespace eng {
  class LibNotification;
}

namespace npc {

class LnListener {
public:
  typedef std::function<void (const eng::LibNotification&)> function_t;

  LnListener(function_t&& f)
    : m_f(f)
  {}
  void call(const eng::LibNotification& ln) const
  {
    m_f(ln);
  }

private:
  function_t m_f;
};

}
