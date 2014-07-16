/*
 * niepce - ui/niepcewindow.hpp
 *
 * Copyright (C) 2007-2013 Hubert Figuiere
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


#ifndef _UI_NIEPCEWINDOW_H_
#define _UI_NIEPCEWINDOW_H_


#include <giomm/simpleactiongroup.h>
#include <gtkmm/treemodel.h>
#include <gtkmm/box.h>
#include <gtkmm/menubar.h>
#include <gtkmm/statusbar.h>
#include <gtkmm/paned.h>

#include "fwk/toolkit/appframe.hpp"
#include "fwk/toolkit/configdatabinder.hpp"
#include "engine/db/label.hpp"
#include "libraryclient/libraryclient.hpp"
#include "ui/moduleshell.hpp"
#include "ui/workspacecontroller.hpp"
#include "ui/selectioncontroller.hpp"
#include "ui/filmstripcontroller.hpp"
#include "niepce/notificationcenter.hpp"

namespace ui {

class NiepceWindow
    : public fwk::AppFrame
{
public:
    NiepceWindow();
    virtual ~NiepceWindow();


    virtual void set_title(const std::string & title);

    libraryclient::LibraryClient::Ptr getLibraryClient() const
        { return m_libClient; }
    fwk::Configuration::Ptr getLibraryConfig() const
        { return m_library_cfg; }

protected:
    virtual Gtk::Widget * buildWidget(const Glib::RefPtr<Gtk::UIManager> & manager);
private:
    void on_action_file_import();

    void on_open_library();
    void on_action_edit_labels();
    void on_action_edit_delete();

    void preference_dialog_setup(const Glib::RefPtr<Gtk::Builder> &,
                                 Gtk::Dialog *);
    void on_preferences();

    void create_initial_labels();
    void on_lib_notification(const eng::LibNotification & n);

    void init_ui();
    void init_actions();

    // UI to open library
    std::string prompt_open_library();
    // open the library
    // @return false if error.
    bool open_library(const std::string & libMoniker);

    void _createModuleShell();

    niepce::NotificationCenter::Ptr m_notifcenter;

    Gtk::Box                       m_vbox;
    Gtk::HPaned                    m_hbox;
    ModuleShell::Ptr               m_moduleshell; // the main views stacked.
    WorkspaceController::Ptr       m_workspacectrl;
    FilmStripController::Ptr       m_filmstrip;
    Gtk::Statusbar                 m_statusBar;
    Glib::RefPtr<Gio::SimpleActionGroup> m_action_group;
    libraryclient::LibraryClient::Ptr m_libClient;
    fwk::Configuration::Ptr        m_library_cfg;
};

}


#endif

/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  fill-column:99
  End:
*/
