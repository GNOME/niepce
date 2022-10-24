/*
 * niepce - niepce/ui/filmstripcontroller.cpp
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


#include <gtkmm/iconview.h>

#include "fwk/base/debug.hpp"

#include "filmstripcontroller.hpp"

namespace ui {

FilmStripController::FilmStripController(const ImageListStorePtr& store)
    : m_store(store)
{
}

Gtk::IconView * FilmStripController::image_list()
{
    return m_thumbview;
}

void FilmStripController::select_image(eng::library_id_t id)
{
    DBG_OUT("filmstrip select %Ld", (long long)id);
    Gtk::TreePath path = m_store->get_path_from_id(id);
    if(path) {
        m_thumbview->scroll_to_path(path, false, 0, 0);
        m_thumbview->select_path(path);
    }
    else {
        m_thumbview->unselect_all();
    }
}


}

/*
  Local Variables:
  mode:c++
  c-basic-offset:4
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  fill-column:99
  End:
*/
