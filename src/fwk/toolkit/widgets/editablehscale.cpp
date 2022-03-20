/*
 * niepce - fwk/widgets/editablehscale.cpp
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

#include <boost/lexical_cast.hpp>

#include <glibmm/property.h>
#include <gtkmm/adjustment.h>
#include <gtkmm/gestureclick.h>

#include "fwk/base/debug.hpp"
#include "editablehscale.hpp"


namespace fwk {

EditableHScale::EditableHScale(double min, double max, double step)
    : Gtk::Box(Gtk::Orientation::HORIZONTAL),
      m_icon(nullptr),
      m_adj(Gtk::Adjustment::create(0, min, max, step)),
      m_scale(m_adj, Gtk::Orientation::HORIZONTAL),
      m_entry(m_adj),
      m_dirty(false)
{
    _init();
}

EditableHScale::EditableHScale(const std::string & icon_path,
                               double min, double max, double step)
    : Gtk::Box(Gtk::Orientation::HORIZONTAL),
      m_icon(Gtk::manage(new Gtk::Image(Gdk::Pixbuf::create_from_resource(icon_path, -1, -1)))),
      m_adj(Gtk::Adjustment::create(0, min, max, step)),
      m_scale(m_adj), m_entry(m_adj),
      m_dirty(false)
{
    _init();
}



void EditableHScale::_init()
{
    if(m_icon) {
        append(*m_icon);
    }
    m_scale.property_draw_value() = false;

    auto gesture = Gtk::GestureClick::create();
    gesture->set_button(1);
    gesture->signal_released()
        .connect([this] (int, double, double) {
            this->on_button_press_event();
        });
    m_scale.add_controller(gesture);
    append(m_scale);
    m_entry.set_width_chars(4);
    m_entry.set_digits(2);
    m_entry.set_editable(true);

    auto gesture2 = Gtk::GestureClick::create();
    gesture->set_button(1);
    gesture->signal_released()
        .connect([this] (int, double, double) {
            this->on_button_press_event();
        });
    m_entry.add_controller(gesture2);
    append(m_entry);

    m_adj->signal_value_changed()
        .connect([this] {
            this->on_adj_value_changed();
        });
}

void EditableHScale::on_button_press_event()
{
    if(m_dirty) {
        m_dirty = false;
        DBG_OUT("value_change.emit(%f)", m_adj->get_value());
        m_sig_value_changed.emit(m_adj->get_value());
    }
}

void EditableHScale::on_adj_value_changed()
{
    m_dirty = true;
    m_sig_value_changing.emit(m_adj->get_value());
}

sigc::signal<void(double)>& EditableHScale::signal_value_changed()
{
    return m_sig_value_changed;
}

sigc::signal<void(double)>& EditableHScale::signal_value_changing()
{
    return m_sig_value_changing;
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

