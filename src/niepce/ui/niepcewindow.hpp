/*
 * niepce - ui/niepcewindow.hpp
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

#include <giomm/simpleactiongroup.h>
#include <gtkmm/treemodel.h>
#include <gtkmm/box.h>
#include <gtkmm/statusbar.h>
#include <gtkmm/paned.h>

#include "fwk/toolkit/appframe.hpp"
#include "fwk/toolkit/configdatabinder.hpp"
#include "ui/moduleshell.hpp"
#include "ui/workspacecontroller.hpp"
#include "ui/selectioncontroller.hpp"
#include "ui/filmstripcontroller.hpp"
#include "dialogs/editlabels.hpp"

#include "rust_bindings.hpp"

namespace ui {

class NiepceWindow_2
    : public fwk::Frame
{
public:
    NiepceWindow_2(::rust::Box<npc::NiepceWindowWrapper>&& wrapper)
        : Frame(Glib::wrap((GtkWindow*)wrapper->window(), "mainWindow-frame"))
        , m_wrapper(std::move(wrapper)) {}
protected:
    virtual Gtk::Widget* buildWidget() override
    {
        DBG_OUT("wrapper buildWidget");
        return Gtk::manage(Glib::wrap((GtkWidget*)m_wrapper->widget()));
    }
    virtual void on_ready() override
    {
        gtkWindow().show();
        m_wrapper->on_ready();
    }
    virtual Glib::RefPtr<Gio::Menu> get_menu() const override
    {
        return Glib::RefPtr<Gio::Menu>(Glib::wrap((GMenu*)m_wrapper->menu()));
    }
private:
    ::rust::Box<npc::NiepceWindowWrapper> m_wrapper;
};

class NiepceWindow
    : public fwk::AppFrame
{
public:
    NiepceWindow();
    virtual ~NiepceWindow();


    virtual void set_title(const std::string & title) override;

    libraryclient::LibraryClientPtr getLibraryClient() const
        { return m_libClient; }
    const fwk::ConfigurationPtr& getLibraryConfig() const
        { return m_library_cfg; }

protected:
    virtual Gtk::Widget * buildWidget() override;
private:
    void on_open_library();
    void on_action_edit_labels();
    void on_action_edit_delete();

    void create_initial_labels();
    void on_lib_notification(const eng::LibNotification & n);

    void init_actions();

    // UI to open library
    void prompt_open_library();
    // open the library
    // @return false if error.
    bool open_library(const std::string & libMoniker);

    void _createModuleShell();

    npc::NotificationCenterPtr m_notifcenter;
    ModuleShell::Ptr               m_moduleshell; // the main views stacked.

    Gtk::Box                       m_vbox;
    Gtk::Paned                     m_hbox;
    Glib::RefPtr<Gio::Menu>        m_main_menu;
    WorkspaceController::Ptr       m_workspacectrl;
    FilmStripController::Ptr       m_filmstrip;
    Gtk::Statusbar                 m_statusBar;
    EditLabels::Ptr m_editlabel_dialog;
    libraryclient::LibraryClientPtr m_libClient;
    fwk::ConfigurationPtr m_library_cfg;
};

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
