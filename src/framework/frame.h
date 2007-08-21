/*
 * niepce - framework/frame.h
 *
 * Copyright (C) 2007 Hubert Figuiere
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


#ifndef _FRAMEWORK_FRAME_H_
#define _FRAMEWORK_FRAME_H_

#include <string>

#include <libglademm/xml.h>

#include "framework/controller.h"

namespace Gtk {
	class Window;
}

namespace framework {

	class Frame 
		: public Controller
	{
	public:
		Frame(const std::string & gladeFile, const Glib::ustring & widgetName);
		Frame();
		~Frame();

		virtual Gtk::Widget * widget();

		Gtk::Window & gtkWindow()
			{ return *m_window; }
		Glib::RefPtr<Gnome::Glade::Xml> & glade()
			{ return m_glade; }
	private:
		Gtk::Window *m_window;
		Glib::RefPtr<Gnome::Glade::Xml> m_glade;
	};

}


#endif
