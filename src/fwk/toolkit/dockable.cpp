/*
 * niepce - fwk/toolkit/dockable.cpp
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


#include "dockable.hpp"

namespace fwk {

Dockable::Dockable(const Glib::ustring& name,
                   const Glib::ustring& long_name,
                   const Glib::ustring& icon_name)
    : m_name(name)
    , m_long_name(long_name)
    , m_icon_name(icon_name)
{
}

Gtk::Box *
Dockable::build_vbox()
{
    m_box = Gtk::manage(new Gtk::Box(Gtk::Orientation::VERTICAL));
    // do the label, the name, etc.
    m_box->set_margin_start(8);
    m_box->set_margin_end(8);
    m_box->set_margin_top(8);
    m_box->set_margin_bottom(8);
    // TODO
    return m_box;
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

