/*
 * niepce - fwk/toolkit/application.cpp
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

#include <list>
#include <vector>
#include <functional>

#include <glibmm/i18n.h>
#include <giomm/simpleaction.h>
#include <gtkmm/dialog.h>

#include "fwk/base/debug.hpp"
#include "fwk/base/geometry.hpp"
#include "fwk/toolkit/gtkutils.hpp"
#include "frame.hpp"
#include "application.hpp"

namespace fwk {

Frame::Frame(Gtk::Window* win, const std::string & layout_cfg_key)
    : m_window(win),
      m_header(nullptr),
      m_builder(nullptr),
      m_layout_cfg_key(layout_cfg_key)
{
    connectSignals();
    frameRectFromConfig();
}

Frame::Frame(const std::string & layout_cfg_key)
    : Frame(new Gtk::Window(), layout_cfg_key)
{
}

Frame::Frame(const std::string & gladeFile,
             const Glib::ustring & widgetName,
             const std::string & layout_cfg_key)
    : m_window(nullptr),
      m_builder(Gtk::Builder::create_from_file(gladeFile)),
      m_layout_cfg_key(layout_cfg_key)
{
    if (m_builder) {
        m_builder->get_widget(widgetName, m_window);
        connectSignals();
        frameRectFromConfig();
    }
}


void Frame::connectSignals()
{
    m_window->signal_delete_event().connect(
        sigc::hide(sigc::mem_fun(this, &Frame::_close)));
    m_window->signal_hide().connect(
        sigc::retype_return<void>(sigc::mem_fun(this, &Frame::_close)));
}

Frame::~Frame()
{
    frameRectToConfig();
    DBG_OUT("destroy Frame");
    delete m_window;
}


void Frame::set_icon_from_theme(const Glib::ustring & name)
{
    using Glib::RefPtr;
    using Gtk::IconTheme;
    using std::vector;
    using std::list;

    RefPtr< IconTheme > icon_theme(Application::app()->getIconTheme());
    vector<int> icon_sizes(icon_theme->get_icon_sizes(name));

    vector< RefPtr <Gdk::Pixbuf> > icons;

    for_each(icon_sizes.begin(), icon_sizes.end(),
             [&icons, icon_theme, name](int s) {
                 icons.push_back(
                     icon_theme->load_icon(name, s,
                                           Gtk::ICON_LOOKUP_USE_BUILTIN));
             }
        );
    gtkWindow().set_icon_list(icons);
}

void Frame::set_title(const std::string & title)
{
    if (m_header) {
        m_header->set_title(Glib::ustring(title));
    }
    else {
        gtkWindow().set_title(Glib::ustring(title));
    }
}


void Frame::toggle_tools_visible()
{
#if 0
    if(m_hide_tools_action->get_state()) {
        signal_hide_tools.emit();
    }
    else {
        signal_show_tools.emit();
    }
#endif
}

void Frame::undo_state()
{
    DBG_ASSERT(static_cast<bool>(m_undo_action), "undo action is NULL");
    fwk::UndoHistory & history(Application::app()->undo_history());
    m_undo_action->set_enabled(history.has_undo());
    std::string s = history.next_undo();
//    m_undo_action->property_label() = Glib::ustring(_("Undo ")) + s;
}


void Frame::redo_state()
{
    DBG_ASSERT(static_cast<bool>(m_redo_action), "redo action is NULL");
    fwk::UndoHistory & history(Application::app()->undo_history());
    m_redo_action->set_enabled(history.has_redo());
    std::string s = history.next_redo();
//    m_redo_action->property_label() = Glib::ustring(_("Redo ")) + s;
}


Glib::RefPtr<Gio::Action>
Frame::create_undo_action(const Glib::RefPtr<Gio::ActionMap> & g,
                          const Glib::RefPtr<Gio::Menu> & menu)
{
    m_undo_action = fwk::add_action(g, "Undo",
                                        sigc::mem_fun(Application::app()->undo_history(),
                                                          &UndoHistory::undo),
                                        menu, _("Undo"), "win", "<control>Z");
    Application::app()->undo_history().signal_changed.connect(
        sigc::mem_fun(*this, &Frame::undo_state));
    undo_state();
    return m_undo_action;
}


Glib::RefPtr<Gio::Action>
Frame::create_redo_action(const Glib::RefPtr<Gio::ActionMap> & g,
                          const Glib::RefPtr<Gio::Menu> & menu)
{
    m_redo_action = fwk::add_action(g, "Redo",
                                    sigc::mem_fun(Application::app()->undo_history(),
                                                  &UndoHistory::redo),
                                    menu, _("Redo"), "win", "<control><shift>Z");
    Application::app()->undo_history().signal_changed.connect(
        sigc::mem_fun(*this, &Frame::redo_state));
    redo_state();
    return m_redo_action;
}


bool Frame::_close()
{
    if(Controller::Ptr parent = m_parent.lock()) {
        parent->remove(shared_from_this());
    }
    return false;
}

void Frame::frameRectFromConfig()
{
    DBG_OUT("loading frame rect (%s)", m_layout_cfg_key.c_str());
    if(!m_layout_cfg_key.empty()) {
        Configuration & cfg = Application::app()->config();
        std::string val;
        val = cfg.getValue(m_layout_cfg_key, "");
        if(!val.empty()) {
            try {
                fwk::Rect r(val);
                m_window->move(r.x(), r.y());
                m_window->resize(r.w(), r.h());
            }
            catch(std::bad_cast)
            {
                ERR_OUT("wrong value in configuration: %s", val.c_str());
            }
        }
    }
}


void Frame::frameRectToConfig()
{
    DBG_OUT("saving frame rect (%s)", m_layout_cfg_key.c_str());
    if(!m_layout_cfg_key.empty()) {
        Configuration & cfg = Application::app()->config();
        int x, y, w, h;
        x = y = w = h = 0;
        m_window->get_position(x, y);
        m_window->get_size(w, h);
        fwk::Rect r(x, y, w, h);
        cfg.setValue(m_layout_cfg_key, std::to_string(r));
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
}
