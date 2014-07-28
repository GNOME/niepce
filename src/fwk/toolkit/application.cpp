/*
 * niepce - framework/application.cpp
 *
 * Copyright (C) 2007-2014 Hubert Figuiere
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

#include <functional>

#include <glibmm/i18n.h>
#include <gtkmm/main.h>
#include <gtkmm/aboutdialog.h>
#include <gtkmm/settings.h>
#include <cluttermm/init.h>

#include "fwk/base/debug.hpp"
#include "fwk/utils/modulemanager.hpp"
#include "fwk/toolkit/gtkutils.hpp"
#include "application.hpp"
#include "uicontroller.hpp"
#include "frame.hpp"


namespace fwk {

Application::Ptr Application::m_application;

Application::Application(int & argc, char** &argv, const char* app_id,
                         const char * name)
    : m_config(Configuration::make_config_path(name))
    , m_module_manager(new ModuleManager())
    , m_gtkapp(Gtk::Application::create(argc, argv, app_id))
{
    Clutter::init(argc, argv);
    m_gtkapp->signal_startup().connect(
        sigc::mem_fun(*this, &Application::on_startup));
}


Application::~Application()
{
    delete m_module_manager;
}


Application::Ptr Application::app()
{
    return m_application;
}


Glib::RefPtr<Gtk::IconTheme> Application::getIconTheme() const
{
    return Gtk::IconTheme::get_default();
}

bool Application::get_use_dark_theme() const
{
    bool v;
    try {
        v = std::stoi(m_config.getValue("ui_dark_theme", "0"));
    }
    catch(...) {
        v = false;
    }
    return v;
}

void Application::set_use_dark_theme(bool value)
{
    m_config.setValue("ui_dark_theme",
                      std::to_string(value));
}

/** Main loop.
 * @param constructor the Application object constructor
 * @param argc
 * @param argv
 * @return main return code
 */
int Application::main(const Application::Ptr & app,
                      int argc, char ** argv)
{
    bool use_dark = app->get_use_dark_theme();
    auto settings = Gtk::Settings::get_default();
    settings->set_property("gtk-application-prefer-dark-theme", use_dark);

    app->m_gtkapp->run(argc, argv);

    DBG_OUT("end run");
    return 0;
}

void Application::on_startup()
{
    init_actions();

    auto window = makeMainFrame();
    _add(window, true);
    _ready();

    set_menubar(window->get_menu());
}

void Application::init_actions()
{
    auto menu = Gio::Menu::create();
    auto section = Gio::Menu::create();

    menu->append_section(section);
    section->append(_("New"), "action");
    fwk::add_action(m_gtkapp, "OpenLibrary",
                    sigc::mem_fun(*this,
                                  &Application::on_action_file_open),
                    section, _("Open"), "app", "<Primary>o");


    // separator
    section = Gio::Menu::create();
    menu->append_section(section);
    fwk::add_action(m_gtkapp, "Preferences",
                    sigc::mem_fun(*this,
                                  &Application::on_action_preferences),
                    section, _("Preferences..."), "app", nullptr);

    section = Gio::Menu::create();
    menu->append_section(section);
    fwk::add_action(m_gtkapp, "Help",
                    sigc::mem_fun(*this,
                                  &Application::about),
                    section, _("Help"), "app", nullptr);

    fwk::add_action(m_gtkapp, "About",
                    sigc::mem_fun(*this,
                                  &Application::about),
                    section, _("About"), "app", nullptr);

    section = Gio::Menu::create();
    menu->append_section(section);

    fwk::add_action(m_gtkapp, "Quit",
                    sigc::mem_fun(*this,
                                  &Application::quit),
                    section, _("Quit"), "app", "<Primary>q");

    m_gtkapp->set_app_menu(menu);
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
    dlg.run();
}

UndoTransaction * Application::begin_undo(const std::string & label)
{
    auto undo = new fwk::UndoTransaction(label);
    undo_history().add(undo);
    return undo;
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
