/*
 * niepce - fwk/toolkit/application.hpp
 *
 * Copyright (C) 2007-2024 Hubert Figuière
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

#ifndef _IN_RUST_BINDINGS_

#pragma once

#include <glibmm/refptr.h>
#include <giomm/application.h>
#include <giomm/menu.h>
#include <gtkmm/application.h>
#include <gtkmm/icontheme.h>

#include "fwk/toolkit/appframe.hpp"

#include "rust_bindings.hpp"

namespace fwk {

class Application
    : public Controller
{
public:
    typedef std::shared_ptr<Application> Ptr;
    typedef std::pair<std::string, std::string> ThemeDesc;

    virtual bool get_use_dark_theme() const;
    virtual void set_use_dark_theme(bool value);

    // MUST set m_main_frame
    virtual Frame::Ptr makeMainFrame() = 0;
    const Glib::RefPtr<Gtk::Application> & gtkApp() const
        { return m_gtkapp; }

    const ConfigurationPtr& config() const
        { return m_config; }

    virtual void quit();
    void about();
    virtual void add(const Controller::Ptr & sub);
    virtual void terminate() override;

    Glib::RefPtr<Gtk::IconTheme> getIconTheme() const;
    void set_menubar(const Glib::RefPtr<Gio::Menu> & menu)
        { m_gtkapp->set_menubar(menu); }

    static Application::Ptr app();
    void main() const;

    UndoHistory& undo_history()
        { return *m_undo; }
    const UndoHistory& undo_history() const
        { return *m_undo; }
    void begin_undo(rust::Box<UndoTransaction>) const;

protected:
    Application(int & argc, char** &argv, const char* app_id, const char *name);
    static Application::Ptr m_application;
    void _add(const Controller::Ptr & sub, bool attach = true);
    virtual void on_action_file_open();
    virtual void on_about();
    virtual void on_action_preferences() = 0;

    void init_actions();

    const Frame::Ptr get_main_frame() const
        { return Frame::Ptr(m_main_frame); }
    /** bound the the GtkApplication startup signal */
    void on_startup();

    Frame::WeakPtr            m_main_frame;
private:
    ConfigurationPtr m_config;
    rust::Box<UndoHistory> m_undo;
    Glib::RefPtr<Gtk::Application> m_gtkapp;
};

inline
Application::Ptr Application_app() {
    return Application::app();
}

}

#endif
/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  fill-column:80
  End:
*/
