/*
 * niepce - fwk/toolkit/dialog.cpp
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


#include <boost/format.hpp>

#include <gtkmm/box.h>
#include <gtkmm/label.h>

#include "dialog.hpp"
#include "fwk/base/debug.hpp"

namespace fwk {

void Dialog::add_header(const std::string & label)
{
    Gtk::Box * vbox = builder()->get_widget<Gtk::Box>("dialog-vbox1");
    auto header = Gtk::manage(new Gtk::Label());
    auto markup = str(boost::format("<span size=\"x-large\">%1%</span>") % label);
    header->set_markup(markup);
    vbox->insert_child_at_start(*header);
}

/** Run the dialog modal. on_ok is called if the dialog response is ok */
void Dialog::run_modal(const Frame::Ptr& parent, std::function<void(int)> on_ok)
{
    DBG_OUT("run_modal");
    if (!m_is_setup) {
        setup_widget();
    }
    gtkDialog().set_transient_for(parent->gtkWindow());
    gtkDialog().set_default_response(Gtk::ResponseType::CLOSE);
    gtkDialog().set_modal();
    gtkDialog().signal_response().connect(on_ok);
    gtkDialog().show();
    DBG_OUT("dialog shown");
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
