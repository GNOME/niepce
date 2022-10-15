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
#include "fwk/base/debug.hpp"
#include "fwk/toolkit/application.hpp"
#include "niepcewindow.hpp"

#include "rust_bindings.hpp"

namespace ui {

ImageListStorePtr ImageListStore::create()
{
    return std::make_shared<ImageListStore>(ffi::npc_image_list_store_new());
}

ImageListStore::ImageListStore(ffi::ImageListStore* store)
    : m_store(store)
    , m_store_wrap(Glib::wrap(GTK_LIST_STORE(ffi::npc_image_list_store_gobj(store)), true))
{
}

ImageListStore::~ImageListStore()
{
    m_store_wrap.reset();
    ffi::npc_image_list_store_delete(m_store);
}

Gtk::TreeModel::iterator ImageListStore::get_iter_from_id(eng::library_id_t id) const
{
    if (m_store == nullptr) {
        return Gtk::TreeModel::iterator();
    }
    auto iter = ffi::npc_image_list_store_get_iter_from_id(m_store, id);
    if (!iter) {
        return Gtk::TreeModel::iterator();
    }
    return Gtk::TreeModel::iterator(GTK_TREE_MODEL(m_store_wrap->gobj()), iter);
}

Gtk::TreeModel::Path ImageListStore::get_path_from_id(eng::library_id_t id) const
{
    if (m_store) {
        Gtk::TreeModel::iterator iter = get_iter_from_id(id);
        if (iter) {
            return m_store_wrap->get_path(iter);
        }
    }
    return Gtk::TreeModel::Path();
}

eng::library_id_t ImageListStore::get_libfile_id_at_path(const Gtk::TreeModel::Path& path) const
{
    return ffi::npc_image_list_store_get_file_id_at_path(m_store, path.gobj());
}

eng::LibFilePtr ImageListStore::get_file(eng::library_id_t id) const
{
    auto f = ffi::npc_image_list_store_get_file(m_store, id);
    if (f) {
        return eng::libfile_wrap(f);
    }
    return eng::LibFilePtr();
}

void ImageListStore::clear_content()
{
    ffi::npc_image_list_store_clear_content(m_store);
}

void ImageListStore::on_lib_notification(const eng::LibNotification &ln)
{
    if (ffi::npc_image_list_store_on_lib_notification(
            m_store, &ln, getLibraryClient()->thumbnailCache())) {
        return;
    }

    auto type = engine_library_notification_type(&ln);
    switch (type) {
    case eng::NotificationType::XMP_NEEDS_UPDATE:
    {
        auto& cfg = fwk::Application::app()->config()->cfg;
        int write_xmp = false;
        std::string xmp_pref;
        try {
            xmp_pref = std::string(cfg->getValue("write_xmp_automatically", "0"));
            write_xmp = std::stoi(xmp_pref);
        }
        catch(const std::exception & e)
        {
            ERR_OUT("couldn't cast %s: %s", xmp_pref.c_str(),
                    e.what());
        }
        ffi::libraryclient_process_xmp_update_queue(getLibraryClient()->client(), write_xmp);
        break;
    }
    default:
        break;
    }
}

libraryclient::LibraryClientPtr ImageListStore::getLibraryClient()
{
    ModuleShell::Ptr shell = std::dynamic_pointer_cast<ModuleShell>(m_controller.lock());
    DBG_ASSERT(static_cast<bool>(shell), "parent not a ModuleShell");
    return shell->getLibraryClient();
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
