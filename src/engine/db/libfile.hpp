/*
 * niepce - eng/db/libfile.hpp
 *
 * Copyright (C) 2007-2022 Hubert Figuière
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

#include <list>
#include <memory>

#include "rust_bindings.hpp"

namespace eng {

typedef std::shared_ptr<LibFile> LibFilePtr;
typedef std::weak_ptr<LibFile> LibFileWeakPtr;
typedef std::list<LibFilePtr> LibFileList;
typedef std::shared_ptr<LibFileList> LibFileListPtr;

LibFilePtr libfile_wrap(LibFile *);
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
