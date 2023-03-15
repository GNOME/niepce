/*
 * niepce - modules/darkroom/toolboxcontroller.h
 *
 * Copyright (C) 2008-2023 Hubert Figuiere
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

#ifndef _IN_RUST_BINDINGS

#pragma once

#include "fwk/toolkit/dockable.hpp"

namespace dr {

class ToolboxController
    : public fwk::Dockable
{
public:
    typedef std::shared_ptr<ToolboxController> Ptr;
    ToolboxController();
    virtual Gtk::Widget * buildWidget() override;

    GtkWidget* build_widget() {
        return buildWidget()->gobj();
    }
};

inline
std::unique_ptr<ToolboxController> ToolboxController_new() {
    return std::make_unique<ToolboxController>();
}


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
