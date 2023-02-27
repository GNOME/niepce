/*
 * niepce - ui/niepceapplication.cpp
 *
 * Copyright (C) 2007-2023 Hubert Figuière
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

#include "config.h"

#include <glibmm/i18n.h>
#include <giomm/menu.h>
#include <gtkmm/aboutdialog.h>

#include "fwk/toolkit/frame.hpp"
#include "dialogs/preferencesdialog.hpp"
#include "niepceapplication.hpp"
#include "niepcewindow.hpp"

using fwk::Frame;
using fwk::Application;

namespace ui {

NiepceApplication::NiepceApplication(int & argc, char** & argv)
    : Application(argc, argv, APP_ID, PACKAGE)
{
}

std::shared_ptr<NiepceApplication> NiepceApplication::create(int & argc, char** & argv)
{
    if (!m_application) {
        m_application = std::make_shared<NiepceApplication>(argc, argv);
    }
    return std::dynamic_pointer_cast<NiepceApplication>(m_application);
}


Frame::Ptr NiepceApplication::makeMainFrame()
{
    auto ptr = Frame::Ptr(new NiepceWindow_2(npc::niepce_window_new(gtkApp()->gobj())));
    m_main_frame = ptr;
    return ptr;
}

void NiepceApplication::on_action_file_open()
{

}

void NiepceApplication::on_about()
{
    DBG_OUT("on_about");
    Gtk::AboutDialog* dlg = new Gtk::AboutDialog();
    dlg->set_program_name("Niepce Digital");
    dlg->set_version(VERSION);
    dlg->set_logo_icon_name(APP_ID);
    dlg->set_license_type(Gtk::License::GPL_3_0);
    dlg->set_comments(Glib::ustring(_("A digital photo application.\n\n"
                                     "Build options: ")) +
                     NIEPCE_BUILD_CONFIG);
    dlg->set_transient_for(m_main_frame.lock()->gtkWindow());
    dlg->set_modal(true);
    dlg->set_hide_on_close(true);
    dlg->show();
}

void NiepceApplication::on_action_preferences()
{
    DBG_OUT("on_preferences");

    auto dlg(new PreferencesDialog());
    dlg->run_modal(Frame::Ptr(m_main_frame),
                   [dlg] (int) {
                       delete dlg;
                       DBG_OUT("destroyed pref dialog");
                       return false;
                   });

    DBG_OUT("end on_preferences");
}

}

/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  fill-column:99
  End:
*/
