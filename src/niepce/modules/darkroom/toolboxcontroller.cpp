/*
 * niepce - darkroom/toolboxcontroller.cpp
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

#include <glibmm/i18n.h>

#include <gtkmm/box.h>
#include <gtkmm/scale.h>
#include <gtkmm/adjustment.h>
#include <gtkmm/stock.h>

#include "toolboxcontroller.hpp"
#include "fwk/base/debug.hpp"
#include "fwk/toolkit/widgets/editablehscale.hpp"
#include "dritemwidget.hpp"

#ifndef DATADIR
#error DATADIR is not defined
#endif

namespace darkroom {

ToolboxController::ToolboxController()
    : Dockable("tools", _("Develop"), Gtk::Stock::APPLY.id)
{

}

Gtk::Widget * 
ToolboxController::buildWidget(const Glib::RefPtr<Gtk::UIManager> & )
{
    if(m_widget) {
        return m_widget;
    }
    DrItemWidget *item = NULL;
    fwk::EditableHScale *s = NULL;

    Gtk::VBox *toolbox = Dockable::build_vbox();
    m_widget = toolbox;
    DBG_ASSERT(toolbox, "vbox not found.");

    item = manage(new DrItemWidget(_("Crop")));
    toolbox->pack_start(*item, Gtk::PACK_SHRINK);
    s = Gtk::manage(new fwk::EditableHScale(DATADIR"/niepce/pixmaps/niepce-transform-rotate.png",
                                            -45.0, 45.0, 0.5));    
    item->add_widget(_("Tilt"), *s);

    item = manage(new DrItemWidget(_("White balance")));
    toolbox->pack_start(*item, Gtk::PACK_SHRINK);
    s = Gtk::manage(new fwk::EditableHScale(0.0, 100.0, 1.0));
    item->add_widget(_("Color temperature"), *s);

    item = manage(new DrItemWidget(_("Tone and colour")));
    toolbox->pack_start(*item, Gtk::PACK_SHRINK);
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
