/*
 * niepce - fwk/toolkit/dock.hpp
 *
 * Copyright (C) 2011 Hubert Figuiere
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

#ifndef __FWK_TOOLKIT_DOCK_HPP__
#define __FWK_TOOLKIT_DOCK_HPP__

#include <gtkmm/box.h>
#include <gtkmm/scrolledwindow.h>


namespace fwk {


class Dock
  : public Gtk::ScrolledWindow
{
public:
    Dock();

    Gtk::Box & vbox()
        { return m_vbox; }
private:
    Gtk::Box m_vbox;
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
#endif
