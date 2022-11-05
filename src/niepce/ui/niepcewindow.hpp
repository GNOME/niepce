/*
 * niepce - ui/niepcewindow.hpp
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

#include <memory>

#include <gtkmm/widget.h>

#include "fwk/base/debug.hpp"

#include "rust_bindings.hpp"

namespace ui {

class NiepceWindow_2
    : public fwk::Frame
{
public:
    NiepceWindow_2(::rust::Box<npc::NiepceWindowWrapper>&& wrapper)
        : Frame(Glib::wrap((GtkWindow*)wrapper->window(), "mainWindow-frame"))
        , m_wrapper(std::move(wrapper)) {}
protected:
    virtual Gtk::Widget* buildWidget() override
    {
        DBG_OUT("wrapper buildWidget");
        auto w = Gtk::manage(Glib::wrap((GtkWidget*)m_wrapper->widget()));
        m_wrapper->on_open_catalog();
        return w;
    }
    virtual void on_ready() override
    {
        gtkWindow().show();
        m_wrapper->on_ready();
    }
    virtual Glib::RefPtr<Gio::Menu> get_menu() const override
    {
        return Glib::RefPtr<Gio::Menu>(Glib::wrap((GMenu*)m_wrapper->menu()));
    }
private:
    ::rust::Box<npc::NiepceWindowWrapper> m_wrapper;
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
