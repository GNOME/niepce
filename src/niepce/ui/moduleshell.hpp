/*
 * niepce - ui/moduleshell.hpp
 *
 * Copyright (C) 2007-2017 Hubert Figuière
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


#ifndef __UI_MODULESHELL_HPP__
#define __UI_MODULESHELL_HPP__

#include <vector>

#include <giomm/simpleactiongroup.h>

#include "moduleshellwidget.hpp"
#include "libraryclient/libraryclient.hpp"
#include "fwk/toolkit/uicontroller.hpp"
#include "fwk/toolkit/notification.hpp"
#include "niepce/ui/gridviewmodule.hpp"
#include "modules/darkroom/darkroommodule.hpp"
#include "modules/map/mapmodule.hpp"
#include "imageliststore.hpp"
#include "imoduleshell.hpp"

namespace Gtk {
class Widget;
}

namespace ui {

class ModuleShell
    : public fwk::UiController
    , public IModuleShell
{
public:
    typedef std::shared_ptr<ModuleShell> Ptr;
    typedef std::weak_ptr<ModuleShell> WeakPtr;

    ModuleShell(const libraryclient::LibraryClientPtr& libclient)
        : m_libraryclient(libclient)
        , m_actionGroup(Gio::SimpleActionGroup::create())
        {
        }
    virtual ~ModuleShell();

    const GridViewModule::Ptr & get_gridview() const
        {
            return m_gridview;
        }
    const mapm::MapModule::Ptr & get_map_module() const
        {
            return m_mapm;
        }
    const Glib::RefPtr<ImageListStore> & get_list_store() const
        {
            return m_selection_controller->get_list_store();
        }
    virtual const SelectionController::Ptr & get_selection_controller() const override
        {
            return m_selection_controller;
        }
    virtual libraryclient::LibraryClientPtr getLibraryClient() const override
        {
            return m_libraryclient;
        }
    virtual const std::unique_ptr<libraryclient::UIDataProvider>& get_ui_data_provider() const override
        {
            return m_libraryclient->getDataProvider();
        }
    Glib::RefPtr<Gio::Menu> getMenu() const
        { return m_menu; }

    /** called when something is selected by the shared selection */
    void on_selected(eng::library_id_t id);
    void on_image_activated(eng::library_id_t id);

    virtual Gtk::Widget * buildWidget() override;

    void action_edit_delete();
protected:
    virtual void add_library_module(const ILibraryModule::Ptr & module,
                                    const std::string & name,
                                    const std::string & label);
    virtual void on_ready() override;
    void on_module_deactivated(const std::string & name) const;
    void on_module_activated(const std::string & name) const;
private:
    libraryclient::LibraryClientPtr m_libraryclient;
    Glib::RefPtr<Gio::SimpleActionGroup> m_actionGroup;

    // managed widgets...
    ModuleShellWidget             m_shell;
    Glib::RefPtr<Gio::Menu>       m_menu;
    Glib::RefPtr<Gio::Menu>       m_module_menu;

    ui::SelectionController::Ptr  m_selection_controller;
    std::map<std::string, ILibraryModule::Ptr> m_modules;
    // these should be dynamic
    GridViewModule::Ptr           m_gridview;
    dr::DarkroomModule::Ptr       m_darkroom;
    mapm::MapModule::Ptr          m_mapm;
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

#endif
