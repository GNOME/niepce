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

#ifndef _IN_RUST_BINDINGS_

#pragma once

#include <optional>

#include <gtkmm/widget.h>
#include <gtkmm/paned.h>
#include <gtkmm/box.h>
#include <gtkmm/scrolledwindow.h>

#include "fwk/toolkit/controller.hpp"
#include "ncr/image.hpp"
#include "niepce/ui/ilibrarymodule.hpp"
#include "niepce/modules/darkroom/imagecanvas.hpp"
#include "niepce/modules/darkroom/toolboxcontroller.hpp"

namespace fwk {
class Dock;
}

namespace dr {

class DarkroomModule
    : public ui::ILibraryModule
{
public:
    DarkroomModule();

    void set_image(eng::LibFile* file) const;

    virtual void dispatch_action(const std::string & action_name) override;

    virtual void set_active(bool active) const override;

    virtual Glib::RefPtr<Gio::MenuModel> getMenu() override
        { return Glib::RefPtr<Gio::MenuModel>(); }

protected:
    void reload_image() const;

    virtual Gtk::Widget * buildWidget() override;

private:
    // darkroom split view
    Gtk::Paned                   m_dr_splitview;
    Gtk::Box                     m_vbox;
    ImageCanvas*                 m_imagecanvas;
    Gtk::ScrolledWindow          m_canvas_scroll;
    ToolboxController::Ptr       m_toolbox_ctrl;
    Glib::RefPtr<Gio::ActionGroup> m_actionGroup;
    mutable std::optional<eng::LibFilePtr> m_imagefile;
    ncr::Image::Ptr              m_image;
    fwk::Dock                   *m_dock;

    // state
    mutable bool m_active;
    mutable bool m_need_reload;
};

inline
std::shared_ptr<DarkroomModule> darkroom_module_new() {
    return std::make_shared<DarkroomModule>();
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
