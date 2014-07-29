/*
 * niepce - modules/map/mapmodule.hpp
 *
 * Copyright (C) 2014 Hubert Figuiere
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


#ifndef _MAP_MODULE_H__
#define _MAP_MODULE_H__

#include <gtkmm/widget.h>
#include <gtkmm/box.h>
#include <gtkmm/actiongroup.h>

#include "fwk/toolkit/controller.hpp"
#include "fwk/toolkit/mapcontroller.hpp"
#include "engine/db/libfile.hpp"
#include "libraryclient/libraryclient.hpp"
#include "niepce/ui/ilibrarymodule.hpp"
#include "niepce/ui/imoduleshell.hpp"

namespace mapm {

class MapModule
    : public ui::ILibraryModule
{
public:
    typedef std::shared_ptr<MapModule> Ptr;

    MapModule(const ui::IModuleShell & shell);

    /* ILibraryModule */
    virtual void dispatch_action(const std::string & action_name);
    virtual void set_active(bool active);
    virtual Glib::RefPtr<Gio::MenuModel> getMenu()
        { return Glib::RefPtr<Gio::MenuModel>(); }

    void on_lib_notification(const eng::LibNotification &ln);

protected:
    virtual Gtk::Widget * buildWidget();

private:
    void on_selected(eng::library_id_t id);

    const ui::IModuleShell &     m_shell;
    Gtk::Box*                    m_box;
    fwk::MapController::Ptr           m_map;

    // state
    bool                         m_active;
};


}

#endif
/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  fill-column:80
  End:
*/
