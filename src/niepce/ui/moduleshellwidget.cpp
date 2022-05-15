/*
 * niepce - ui/moduleshellwidget.cpp
 *
 * Copyright (C) 2007-2022 Hubert Figuière
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

#include <gtkmm/image.h>
#include <gtkmm/togglebutton.h>
#include <gtkmm/stackswitcher.h>

#include "fwk/base/debug.hpp"
#include "ui/moduleshellwidget.hpp"

namespace ui {

ModuleShellWidget::ModuleShellWidget()
    : Gtk::Box(Gtk::Orientation::VERTICAL)
    , m_mainbox()
{
    m_menubutton.set_direction(Gtk::ArrowType::NONE);
    m_menubutton.set_icon_name("view-more-symbolic");
    m_mainbox.set_end_widget(m_menubutton);
    m_mainbox.set_margin(4);
    append(m_mainbox);

    m_mainbox.set_center_widget(m_switcher);
    m_stack.property_visible_child().signal_changed().connect(
        sigc::mem_fun(*this, &ModuleShellWidget::stack_changed));
    append(m_stack);

    m_switcher.set_stack(m_stack);
    m_current_module = m_stack.get_visible_child_name();
}

void
ModuleShellWidget::appendPage(Gtk::Widget & w, const Glib::ustring & name,
                              const Glib::ustring & label)
{
    m_stack.add(w, name, label);
}

/// Callback when the module stack has changed.
/// This allow activation / deactivation as need
void ModuleShellWidget::stack_changed()
{
    signal_deactivated(m_current_module);
    m_current_module = m_stack.get_visible_child_name();
    signal_activated(m_current_module);
}

void ModuleShellWidget::activatePage(const std::string& name)
{
    if (m_current_module != name) {
        m_stack.set_visible_child(name);
    }
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
