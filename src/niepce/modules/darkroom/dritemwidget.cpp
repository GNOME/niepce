/*
 * niepce - darkroom/dritem.cpp
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

#include <gtkmm/label.h>

#include "dritemwidget.hpp"

namespace dr {

DrItemWidget::DrItemWidget(const Glib::ustring & title)
    : fwk::ToolboxItemWidget(title)
    , m_box(Gtk::Orientation::VERTICAL)
{
    set_child(m_box);
//    m_box.set_border_width(6);
}

void DrItemWidget::add_widget(const Glib::ustring & label, Gtk::Widget & w)
{
    Gtk::Label *l = manage(new Gtk::Label(label, Gtk::Align::START, Gtk::Align::CENTER));
    m_box.append(*l);
    m_box.append(w);
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

