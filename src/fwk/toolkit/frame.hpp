/*
 * niepce - fwk/toolkit/frame.hpp
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

#pragma once

#include <memory>
#include <string>

#include <sigc++/signal.h>
#include <giomm/action.h>
#include <giomm/actiongroup.h>
#include <giomm/menu.h>
#include <gtkmm/builder.h>
#include <gtkmm/headerbar.h>
#include <gtkmm/window.h>

#include "fwk/toolkit/uicontroller.hpp"

namespace Gio {
class ActionMap;
class SimpleAction;
}

namespace Gtk {
class Dialog;
}

namespace fwk {

class Frame
    : public UiController
{
protected:
    Frame(Gtk::Window* win, const std::string & layout_cfg_key = "");

public:
    typedef std::shared_ptr<Frame> Ptr;
    typedef std::weak_ptr<Frame> WeakPtr;

    Frame(const std::string & gladeFile, const Glib::ustring & widgetName,
          const std::string & layout_cfg_key = "");
    Frame(const std::string & layout_cfg_key = "");
    virtual ~Frame();

    /** convenience to return the Frame::Ptr from this */
    Ptr shared_frame_ptr()
        {
            return std::static_pointer_cast<Frame>(shared_from_this());
        }

    Gtk::Window & gtkWindow() const
        {
            return *m_window;
        }
    Glib::RefPtr<Gtk::Builder> & builder()
        { return m_builder; }

    void close()
        {
            if (m_window) {
                m_window->close();
            }
        }

    virtual Glib::RefPtr<Gio::Menu> get_menu() const {
        return Glib::RefPtr<Gio::Menu>();
    }

    /** @param header MUST be managed */
    void setHeaderBar(Gtk::HeaderBar* header)
        {
            m_header = header;
            if (m_header) {
                m_window->set_titlebar(*m_header);
            }
        }
    Gtk::HeaderBar* getHeaderBar() const
        { return m_header; }

    /** set the title of the window.
     * @param title the title of the window.
     *
     * override to provide you own hooks - behaviour.
     */
    virtual void set_title(const std::string & title);
    /** set the window icon from the theme
     * @param name the icon name in the theme
     */
    void set_icon_from_theme(const Glib::ustring & name);

    void toggle_tools_visible();

    sigc::signal<void(void)> signal_hide_tools;
    sigc::signal<void(void)> signal_show_tools;
protected:

    void undo_state();
    void redo_state();
    Glib::RefPtr<Gio::Action> create_undo_action(Gio::ActionMap* g);
    Glib::RefPtr<Gio::Action> create_redo_action(Gio::ActionMap* g);

    /** close signal handler */
    virtual bool _close();
    Glib::RefPtr<Gio::SimpleAction> m_hide_tools_action;
    Glib::RefPtr<Gio::SimpleAction>      m_undo_action;
    Glib::RefPtr<Gio::SimpleAction>      m_redo_action;

private:
    /** frame have the widget set at construction time
     * from a ui file or directly.
     */
    void connectSignals();
    void frameRectFromConfig();
    void frameRectToConfig();

    Gtk::Window *m_window;
    Gtk::HeaderBar *m_header;
    Glib::RefPtr<Gtk::Builder> m_builder;
    std::string m_layout_cfg_key;
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
