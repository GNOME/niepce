/*
 * niepce - ui/importdialog.h
 *
 * Copyright (C) 2008 Hubert Figuiere
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




#ifndef __UI_IMPORTDIALOG_H__
#define __UI_IMPORTDIALOG_H__

#include <glibmm/refptr.h>
#include <libglademm/xml.h>

#include "framework/frame.h"

namespace Gtk {
	class Dialog;
	class ComboBox;
	class CheckButton;
}

namespace ui {

class ImportDialog 
	: public framework::Frame
{
public:
	typedef boost::shared_ptr<ImportDialog> Ptr;

	ImportDialog();

 	Gtk::Widget * buildWidget();

	const Glib::ustring & to_import() const
		{ return m_to_import; }

private:
	class ImportParam;

	void do_select_directories();
	
	Glib::ustring m_to_import;
	Gtk::ComboBox *m_date_tz_combo;
	Gtk::CheckButton *m_ufraw_import_check;
	Gtk::CheckButton *m_rawstudio_import_check;
	Gtk::Label *m_directory_name;
};

}

#endif
