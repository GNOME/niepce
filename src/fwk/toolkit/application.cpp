/*
 * niepce - fwk/toolkit/application.cpp
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

#include <adwaita.h>

#include <glibmm/i18n.h>
#include <glibmm/miscutils.h>

#include <gtkmm/aboutdialog.h>
#include <gtkmm/settings.h>

#include "fwk/base/debug.hpp"
#include "fwk/toolkit/gtkutils.hpp"
#include "application.hpp"
#include "uicontroller.hpp"
#include "frame.hpp"

namespace fwk {

Application::Ptr Application::m_application;

Application::Application(int & argc, char** &argv, const char* app_id,
                         const char * name)
    : m_config(Configuration_new(Configuration_make_config_path(name)))
    , m_undo(std::move(UndoHistory_new()))
    , m_gtkapp(Gtk::Application::create(app_id))
{
    Glib::set_prgname(app_id);
    m_gtkapp->signal_startup().connect(
        sigc::mem_fun(*this, &Application::on_startup));
    getIconTheme()->add_resource_path("/org/gnome/Niepce/pixmaps");
}

Application::Ptr Application::app()
{
    return m_application;
}


Glib::RefPtr<Gtk::IconTheme> Application::getIconTheme() const
{
    return Gtk::IconTheme::get_for_display(Gdk::Display::get_default());
}

bool Application::get_use_dark_theme() const
{
    bool v;
    try {
        v = std::stoi(std::string(m_config->cfg->getValue("ui_dark_theme", "0")));
    }
    catch(...) {
        v = false;
    }
    return v;
}

void Application::set_use_dark_theme(bool value)
{
    m_config->cfg->setValue("ui_dark_theme",
                      std::to_string(value));
}

/** Main loop.
 * @param constructor the Application object constructor
 * @return main return code
 */
void Application::main() const
{
    bool use_dark = this->get_use_dark_theme();
    auto settings = adw_style_manager_get_default();
    if (use_dark) {
        adw_style_manager_set_color_scheme(settings, ADW_COLOR_SCHEME_FORCE_DARK);
    } else {
        adw_style_manager_set_color_scheme(settings, ADW_COLOR_SCHEME_PREFER_LIGHT);
    }

    this->m_gtkapp->run();

    DBG_OUT("end run");
 }

void Application::on_startup()
{
    init_actions();

    auto window = makeMainFrame();
    _add(window, true);

    set_menubar(window->get_menu());

    _ready();
}

void Application::init_actions()
{
    fwk::add_action(m_gtkapp.get(), "OpenCatalog",
                    sigc::mem_fun(*this,
                                  &Application::on_action_file_open),
                    "app", "<Primary>o");
    fwk::add_action(m_gtkapp.get(), "Preferences",
                    sigc::mem_fun(*this,
                                  &Application::on_action_preferences));
    fwk::add_action(m_gtkapp.get(), "Help",
                    sigc::mem_fun(*this,
                                  &Application::about));
    fwk::add_action(m_gtkapp.get(), "About",
                    sigc::mem_fun(*this,
                                  &Application::about));
    fwk::add_action(m_gtkapp.get(), "Quit",
                    sigc::mem_fun(*this,
                                  &Application::quit),
                    "app", "<Primary>q");
}

void Application::terminate()
{
    DBG_OUT("terminating");
    Controller::terminate();
    DBG_OUT("done terminating");
}


void Application::quit()
{
    // TODO check we can quit

    terminate();
}

void Application::about()
{
    on_about();
}

/** adding a controller to an application build said controller
 * widget
 */
void Application::add(const Controller::Ptr & sub)
{
    _add(sub, true);
}

void Application::_add(const Controller::Ptr & sub, bool attach)
{
    Controller::add(sub);
    UiController::Ptr uictrl = std::dynamic_pointer_cast<UiController>(sub);
    if(uictrl) {
        auto w = uictrl->buildWidget();
        Gtk::Window *win = nullptr;
        if(attach && m_gtkapp && (win = dynamic_cast<Gtk::Window*>(w))) {
            m_gtkapp->add_window(*win);
        }
    }
}

void Application::on_action_file_open()
{
}

void Application::on_about()
{
    Gtk::AboutDialog dlg;
    dlg.show();
}

void Application::begin_undo(rust::Box<UndoTransaction> transaction) const
{
    return const_cast<rust::Box<UndoHistory>&>(m_undo)->add(std::move(transaction));
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
