/*
 * niepce - ui/imageliststore.cpp
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

#include "imageliststore.hpp"

#include "rust_bindings.hpp"

namespace ui {

Gtk::TreePath ImageListStore_get_path_from_id(const ImageListStore& self, eng::library_id_t id)
{
    auto path = self.get_iter_from_id_(id);
    if (path) {
        auto iter = (GtkTreeIter*)const_cast<char*>(path);
        auto tree_path = Glib::wrap(gtk_tree_model_get_path(GTK_TREE_MODEL(self.gobj()), iter));
        return tree_path;
    }

    return Gtk::TreePath();
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
