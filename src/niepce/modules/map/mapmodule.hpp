/*
 * niepce - modules/map/mapmodule.hpp
 *
 * Copyright (C) 2014-2022 Hubert Figuiere
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

#include <gtkmm/widget.h>
#include <gtkmm/box.h>

#include "fwk/toolkit/controller.hpp"
#include "fwk/toolkit/mapcontroller.hpp"
#include "engine/db/libfile.hpp"
#include "niepce/ui/ilibrarymodule.hpp"

namespace mapm {

class MapModule
    : public ui::ILibraryModule
{
public:
    typedef std::shared_ptr<MapModule> Ptr;

    MapModule();

    /* ILibraryModule */
    virtual void dispatch_action(const std::string & action_name) override;
    virtual void set_active(bool active) override;
    virtual Glib::RefPtr<Gio::MenuModel> getMenu() override
        { return Glib::RefPtr<Gio::MenuModel>(); }

    void on_lib_notification(const eng::LibNotification &ln);

protected:
    virtual Gtk::Widget * buildWidget() override;

private:
    void on_selected(eng::library_id_t id);

    Gtk::Box*                    m_box;
    fwk::MapController::Ptr           m_map;

    // state
    bool                         m_active;
};

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
