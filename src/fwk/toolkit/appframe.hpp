/*
 * niepce - fwk/toolkit/appframe.hpp
 *
 * Copyright (C) 2019 Hubert Figuière
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

#include <gtkmm/applicationwindow.h>

#include "fwk/toolkit/frame.hpp"

namespace fwk {

class AppFrame
  : public Frame
{
public:
  typedef std::shared_ptr<AppFrame> Ptr;
  typedef std::weak_ptr<AppFrame> WeakPtr;

  AppFrame(const std::string & layout_cfg_key = "");

  virtual void on_ready() override;

  Gtk::ApplicationWindow* gtkAppWindow()
    {
      return dynamic_cast<Gtk::ApplicationWindow*>(&gtkWindow());
    }
  virtual Glib::RefPtr<Gio::Menu> get_menu() const override
    { return m_menu; }

protected:
  Glib::RefPtr<Gio::Menu> m_menu;
};

}
