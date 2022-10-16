/*
 * niepce - modules/darkroom/darkroommodule.hpp
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

#include <gtkmm/widget.h>
#include <gtkmm/paned.h>
#include <gtkmm/box.h>
#include <gtkmm/scrolledwindow.h>

#include "fwk/toolkit/controller.hpp"
#include "engine/db/libfile.hpp"
#include "libraryclient/libraryclient.hpp"
#include "ncr/image.hpp"
#include "niepce/ui/ilibrarymodule.hpp"
#include "niepce/ui/imoduleshell.hpp"
#include "modules/darkroom/imagecanvas.hpp"
#include "modules/darkroom/toolboxcontroller.hpp"

namespace fwk {
class Dock;
}

namespace dr {

class DarkroomModule
    : public ui::ILibraryModule
{
public:
    typedef std::shared_ptr<DarkroomModule> Ptr;

    DarkroomModule(const ui::IModuleShell & shell);

    void set_image(std::optional<eng::LibFilePtr>&& file);

    virtual void dispatch_action(const std::string & action_name) override;

    virtual void set_active(bool active) override;

    virtual Glib::RefPtr<Gio::MenuModel> getMenu() override
        { return Glib::RefPtr<Gio::MenuModel>(); }

protected:
    void reload_image();

    virtual Gtk::Widget * buildWidget() override;

private:
    void on_selected(eng::library_id_t id);

    const ui::IModuleShell &     m_shell;
    // darkroom split view
    Gtk::Paned                   m_dr_splitview;
    Gtk::Box                     m_vbox;
    ImageCanvas*                 m_imagecanvas;
    Gtk::ScrolledWindow          m_canvas_scroll;
    ToolboxController::Ptr       m_toolbox_ctrl;
    Glib::RefPtr<Gio::ActionGroup> m_actionGroup;
    std::optional<eng::LibFilePtr> m_imagefile;
    ncr::Image::Ptr              m_image;
    fwk::Dock                   *m_dock;

    // state
    bool                         m_active;
    bool                         m_need_reload;
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
