/*
 * niepce - niepce/ui/niepcelibrarymodule.hpp
 *
 * Copyright (C) 2009-2022 Hubert Figuière
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

#include <string>
#include <memory>

#include <giomm/menumodel.h>

#include "fwk/toolkit/uicontroller.hpp"

namespace ui {

/** interface to implement a library module */
class ILibraryModule
  : public fwk::UiController
{
public:
  typedef std::shared_ptr<ILibraryModule> Ptr;

  /** dispatch action by name to the controller */
  virtual void dispatch_action(const std::string & action_name) = 0;

  /** called when the module is set activate or inactive
      @param active true if brought to the front, false if defocused.
   */
  virtual void set_active(bool active) const = 0;
  /** Get the module menu */
  virtual Glib::RefPtr<Gio::MenuModel> getMenu() = 0;
};

}
