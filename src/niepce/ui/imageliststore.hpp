/*
 * niepce - ui/imageliststore.h
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

#pragma once

#include <optional>

#include <gtkmm/liststore.h>

#include "engine/db/libfile.hpp"

#include "rust_bindings.hpp"

namespace ui {

typedef ::rust::Box<ImageListStoreWrap> ImageListStorePtr;

Gtk::TreeModel::iterator ImageListStore_get_iter_from_id(const ImageListStore& self, eng::library_id_t id);
Gtk::TreePath ImageListStore_get_path_from_id(const ImageListStore& self, eng::library_id_t id);
std::optional<eng::LibFilePtr> ImageListStore_get_file(const ImageListStore& self, eng::library_id_t id);

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

