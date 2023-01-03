/*
 * niepce - fwk/utils/pathutils.cpp
 *
 * Copyright (C) 2009-2023 Hubert Figuière
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


#include <giomm/file.h>

#include "fwk/base/debug.hpp"
#include "pathutils.hpp"

namespace fwk {

/** return the basename of a path. Example:
    /foo/bar/baz.txt returns baz.txt
 */
std::string path_basename(const std::string & path)
{
    auto slash_idx = path.find_last_of("/");
    if(slash_idx == std::string::npos) {
        return path;
    }
    return std::string(path.cbegin() + slash_idx + 1, path.cend());
}

bool path_exists(const std::string & path)
{
    Glib::RefPtr<Gio::File> file = Gio::File::create_for_path(path);
    return file->query_exists();
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

