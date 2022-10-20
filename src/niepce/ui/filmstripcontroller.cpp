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

Gtk::Widget * FilmStripController::buildWidget()
{
    if(m_widget) {
        return m_widget;
    }
    DBG_ASSERT(static_cast<bool>(m_store), "m_store NULL");
    m_thumb_strip_view = std::shared_ptr<ffi::ThumbStripView>(
        ffi::npc_thumb_strip_view_new(
            GTK_TREE_MODEL(g_object_ref(m_store->gobjmm()->gobj()))),
        ffi::npc_thumb_strip_view_release);
    // XXX this should be maybe automatically computed
    ffi::npc_thumb_strip_view_set_item_height(m_thumb_strip_view.get(), 120);
    // We need to ref m_store since it's held by the RefPtr<>
    // and the ThumbStripView in Rust gets full ownership.
    m_thumbview = Gtk::manage(
        Glib::wrap(GTK_ICON_VIEW(ffi::npc_thumb_strip_view_get_icon_view(m_thumb_strip_view.get()))));
    GtkWidget *thn = ffi::npc_thumb_nav_new(m_thumbview->gobj(),
                                            ffi::ThumbNavMode::OneRow, true);
    m_thumbview->set_selection_mode(Gtk::SelectionMode::SINGLE);
    m_thumbview->set_hexpand(true);
    m_widget = Glib::wrap(thn);
    m_widget->set_size_request(-1, 134);
    m_widget->set_hexpand(true);
    return m_widget;
}

Gtk::IconView * FilmStripController::image_list()
{
    return m_thumbview;
}

eng::library_id_t FilmStripController::get_selected()
{
    DBG_OUT("get selected in filmstrip");
    std::vector<Gtk::TreePath> paths = m_thumbview->get_selected_items();

    if(paths.empty()) {
        return 0;
    }

    Gtk::TreePath path(*(paths.begin()));
    DBG_OUT("found path %s", path.to_string().c_str());
    return m_store->get_libfile_id_at_path(path);
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
