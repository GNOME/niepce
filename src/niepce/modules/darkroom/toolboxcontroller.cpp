/*
 * niepce - darkroom/toolboxcontroller.cpp
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

#include <glibmm/i18n.h>

#include <gtkmm/box.h>
#include <gtkmm/scale.h>
#include <gtkmm/adjustment.h>

#include "toolboxcontroller.hpp"
#include "fwk/base/debug.hpp"
#include "fwk/toolkit/widgets/editablehscale.hpp"
#include "dritemwidget.hpp"

#ifndef DATADIR
#error DATADIR is not defined
#endif

namespace dr {

ToolboxController::ToolboxController()
    : Dockable("tools", _("Develop"), "apply") // FIXME there is no equivalent it seems.
{

}

Gtk::Widget * 
ToolboxController::buildWidget()
{
    if(m_widget) {
        return m_widget;
    }
    DrItemWidget *item = nullptr;
    fwk::EditableHScale *s = nullptr;

    Gtk::Box *toolbox = Dockable::build_vbox();
    m_widget = toolbox;
    DBG_ASSERT(toolbox, "vbox not found.");

    item = manage(new DrItemWidget(_("Crop")));
    toolbox->append(*item);
    s = Gtk::manage(new fwk::EditableHScale("/org/gnome/Niepce/pixmaps/niepce-transform-rotate.png",
                                            -45.0, 45.0, 0.5));
    item->add_widget(_("Tilt"), *s);

    item = manage(new DrItemWidget(_("White balance")));
    toolbox->append(*item);
    s = Gtk::manage(new fwk::EditableHScale(0.0, 100.0, 1.0));
    item->add_widget(_("Color temperature"), *s);

    item = manage(new DrItemWidget(_("Tone and colour")));
    toolbox->append(*item);
    s = Gtk::manage(new fwk::EditableHScale(-5.0, 5.0, 0.1));
//    s->signal_value_changed().connect();
    item->add_widget(_("Exposure"), *s);
    s = Gtk::manage(new fwk::EditableHScale(0, 100, 1));
    item->add_widget(_("Recovery"), *s);
    s = Gtk::manage(new fwk::EditableHScale(0, 100, 1));
    item->add_widget(_("Fill Light"), *s);
    s = Gtk::manage(new fwk::EditableHScale(0, 100, 1));
    item->add_widget(_("Blacks"), *s);
    s = Gtk::manage(new fwk::EditableHScale(-100, 100, 1));
    item->add_widget(_("Brightness"), *s);
    s = Gtk::manage(new fwk::EditableHScale(-100, 100, 1));
    item->add_widget(_("Contrast"), *s);
    s = Gtk::manage(new fwk::EditableHScale(-100, 100, 1));
    item->add_widget(_("Saturation"), *s);
    s = Gtk::manage(new fwk::EditableHScale(-100, 100, 1));
    item->add_widget(_("Vibrance"), *s);

    return m_widget;
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
