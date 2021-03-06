/*
 * niepce - fwk/toolkit/dialog.cpp
 *
 * Copyright (C) 2009-2014 Hubert Figuiere
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


#include <boost/format.hpp>
#include "libview/header.hh"

#include "dialog.hpp"


namespace fwk {



void Dialog::add_header(const std::string & label)
{
    Gtk::Box * vbox;

    builder()->get_widget("dialog-vbox1", vbox);
    auto markup = str(boost::format("<span size=\"x-large\">%1%</span>") % label);
    auto header = manage(new view::Header(markup));
    header->show();
    vbox->pack_start(*header, false, true);
}

int Dialog::run_modal(const Frame::Ptr & parent)
{
    int result;
    if(!m_is_setup) {
        setup_widget();
    }
    gtkDialog().set_transient_for(parent->gtkWindow());
    gtkDialog().set_default_response(Gtk::RESPONSE_CLOSE);
    result = gtkDialog().run();
    gtkDialog().hide();
    return result;
}

Gtk::Widget *Dialog::buildWidget()
{
    return &gtkWindow();
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
