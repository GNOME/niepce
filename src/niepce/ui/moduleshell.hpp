/*
 * niepce - niepce/ui/moduleshell.hpp
 *
 * Copyright (C) 2007-2022 Hubert Figuière
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

#include <vector>

#include <giomm/simpleactiongroup.h>

#include "moduleshellwidget.hpp"
#include "fwk/toolkit/uicontroller.hpp"
#include "fwk/toolkit/notification.hpp"
#include "niepce/ui/gridviewmodule.hpp"
#include "modules/darkroom/darkroommodule.hpp"
#include "modules/map/mapmodule.hpp"
#include "imageliststore.hpp"
#include "imoduleshell.hpp"

#include "rust_bindings.hpp"

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

    const ImageListStoreWrap& get_list_store() const
        {
            return m_selection_controller->obj()->get_list_store();
        }
    virtual const SelectionControllerPtr & get_selection_controller() const override
        {
            return m_selection_controller->obj();
        }
    virtual libraryclient::LibraryClientPtr getLibraryClient() const override
        {
            return m_libraryclient;
        }
    virtual Glib::RefPtr<Gio::Menu> getMenu() const override
        { return m_menu; }

    /** called when the content will change
     * the content being what the grid view displays.
     */
    void on_content_will_change();

    /** called when something is selected by the shared selection */
    void on_image_selected(eng::library_id_t id);
    /** called when an image is activate in the shared selection */
    void on_image_activated(eng::library_id_t id);

    virtual Gtk::Widget * buildWidget() override;

    void action_edit_delete();

    void on_lib_notification(const eng::LibNotification &ln);
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
    ui::SelectionController_2::Ptr  m_selection_controller;
    std::map<std::string, ILibraryModule::Ptr> m_modules;

    // managed widgets...
    ModuleShellWidget             m_shell;
    Glib::RefPtr<Gio::Menu>       m_menu;
    Glib::RefPtr<Gio::Menu>       m_module_menu;

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
