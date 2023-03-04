/*
 * niepce - fwk/base/geometry.cpp
 *
 * Copyright (C) 2007-2023 Hubert Figuiere
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


#include <boost/format.hpp>
#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string/split.hpp>
#include <boost/algorithm/string/classification.hpp>

#include "fwk/base/debug.hpp"
#include "geometry.hpp"

namespace fwk {

Rect::Rect()
{
    _r[X] = 0;
    _r[Y] = 0;
    _r[W] = 0;
    _r[H] = 0;
}

Rect::Rect(int _x, int _y, int _w, int _h)
{
    _r[X] = _x;
    _r[Y] = _y;
    _r[W] = _w;
    _r[H] = _h;
}


Rect::Rect(const std::string & s) noexcept(false)
{
    std::vector< std::string > v;
    v.reserve(4);
    boost::split(v, s, boost::is_any_of(" "));
    if(v.size() < 4) {
        throw std::bad_cast();
    }
    int i = 0;
    for_each(v.cbegin(), v.cend(),
             [&i, this] (const std::string &_s) {
                 try {
                     _r[i] = std::stoi(_s);
                     i++;
                 }
                 catch(...) {
                     // we likely got an invalid_argument.
                     // Doesn't matter, at that point it is a bad_cast
                     throw std::bad_cast();
                 }
             }
        );
}

}

namespace std {

std::string to_string(const fwk::Rect & r)
{
    return str(boost::format("%1% %2% %3% %4%")
               % r._r[fwk::Rect::X] % r._r[fwk::Rect::Y] % r._r[fwk::Rect::W] % r._r[fwk::Rect::H]);
}

}

/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  fill-column:80
  End:
*/
