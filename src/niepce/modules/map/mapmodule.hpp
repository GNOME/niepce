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

#ifndef _IN_RUST_BINDINGS_

#pragma once

#include <gtkmm/widget.h>
#include <gtkmm/box.h>

#include "fwk/toolkit/controller.hpp"
#include "fwk/toolkit/mapcontroller.hpp"
#include "niepce/ui/ilibrarymodule.hpp"

#include "rust_bindings.hpp"

namespace mapm {

class MapModule
    : public ui::ILibraryModule
{
public:
    MapModule();

    /* ILibraryModule */
    virtual void dispatch_action(const std::string & action_name) override;
    virtual void set_active(bool active) const override;
    virtual Glib::RefPtr<Gio::MenuModel> getMenu() override
        { return Glib::RefPtr<Gio::MenuModel>(); }

    void on_lib_notification(const eng::LibNotification &ln) const;

protected:
    virtual Gtk::Widget * buildWidget() override;

private:
    void on_selected(eng::library_id_t id);

    Gtk::Box*                    m_box;
    fwk::MapController::Ptr           m_map;

    // state
    mutable bool m_active;
};

inline
std::unique_ptr<MapModule> map_module_new() {
    return std::make_unique<MapModule>();
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

#endif
