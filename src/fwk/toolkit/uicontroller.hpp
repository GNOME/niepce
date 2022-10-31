/*
 * niepce - fwk/toolkit/uicontroller.hpp
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

#include <gtkmm/widget.h>

#include "fwk/toolkit/controller.hpp"

namespace Gtk {
class Widget;
}

namespace fwk {

class UiController
  : public Controller
{
public:
    typedef std::shared_ptr<UiController> Ptr;

    UiController();
    virtual ~UiController();

    /** return the widget controlled (construct it if needed) */
    virtual Gtk::Widget * buildWidget() = 0;
    Gtk::Widget * widget() const;

    /* for `cxx` binding */
    const GtkWidget* build_widget() const {
        return GTK_WIDGET(const_cast<UiController*>(this)->buildWidget()->gobj());
    }
protected:
    Gtk::Widget*                 m_widget;
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
