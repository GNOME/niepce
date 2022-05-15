/*
 * niepce - fwk/widgets/editablehscale.hpp
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

#pragma once

#include <gtkmm/box.h>
#include <gtkmm/image.h>
#include <gtkmm/scale.h>
#include <gtkmm/spinbutton.h>

namespace fwk {

/** A widget similar to a Gtk::Scale with a edit box */
class EditableHScale
    : public Gtk::Box
{
public:
    EditableHScale(double min, double max, double step);
    EditableHScale(const std::string & icon_name,
                   double min, double max, double step);

    const Glib::RefPtr<Gtk::Adjustment>  & get_adjustment() const
        { return m_adj; }

    sigc::signal<void(double)> & signal_value_changed();
    sigc::signal<void(double)> & signal_value_changing();

    void on_button_press_event();

private:

    void on_adj_value_changed();

    void _init();

    Gtk::Image      *m_icon;
    Glib::RefPtr<Gtk::Adjustment>  m_adj;
    Gtk::Scale       m_scale;
    Gtk::SpinButton  m_entry;
    bool             m_dirty;
    /** emitted once the value changed */
    sigc::signal<void(double)> m_sig_value_changed;
    /** emitted when the value is changing (think live update) */
    sigc::signal<void(double)> m_sig_value_changing;
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
