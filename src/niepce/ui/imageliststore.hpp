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

#include "fwk/base/propertybag.hpp"
#include "fwk/toolkit/controller.hpp"
#include "engine/db/libfile.hpp"

namespace ui {

class ImageListStore;

typedef std::shared_ptr<ImageListStore> ImageListStorePtr;

/** @brief the general list store. Wraps the list store from Rust. */
class ImageListStore
{
public:
    ImageListStore(ffi::ImageListStore*);
    ~ImageListStore();
    Glib::RefPtr<Gtk::ListStore> gobjmm() const
        { return m_store_wrap; }
    Gtk::TreeModel::Path get_path_from_id(eng::library_id_t id) const;
    Gtk::TreeModel::iterator get_iter_from_id(eng::library_id_t id) const;
    eng::library_id_t get_libfile_id_at_path(const Gtk::TreeModel::Path& path) const;
    std::optional<eng::LibFilePtr> get_file(eng::library_id_t id) const;
    size_t get_count() const
        { return m_store_wrap->children().size(); }

    static ImageListStorePtr create();

    void set_parent_controller(const fwk::Controller::WeakPtr & ctrl)
        { m_controller = ctrl; }

    // Should be called when the content will change
    void clear_content();
    void on_lib_notification(const eng::LibNotification &n);

private:
    libraryclient::LibraryClientPtr getLibraryClient();

    ffi::ImageListStore* m_store;
    Glib::RefPtr<Gtk::ListStore> m_store_wrap;
    fwk::Controller::WeakPtr m_controller;
};

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

