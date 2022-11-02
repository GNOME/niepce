/*
 * niepce - ui/niepcewindow.cpp
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

#include <string>

#include <glibmm/i18n.h>
#include <glibmm/miscutils.h>
#include <giomm/menu.h>
#include <gtkmm/window.h>
#include <gtkmm/accelkey.h>
#include <gtkmm/separator.h>
#include <gtkmm/filechooserdialog.h>

#include "fwk/base/debug.hpp"
#include "fwk/utils/boost.hpp"
#include "fwk/toolkit/application.hpp"
#include "fwk/toolkit/notificationcenter.hpp"
#include "fwk/toolkit/configdatabinder.hpp"
#include "fwk/toolkit/undo.hpp"
#include "fwk/toolkit/gtkutils.hpp"

#include "niepcewindow.hpp"
#include "dialogs/editlabels.hpp"
#include "selectioncontroller.hpp"

#include "rust_bindings.hpp"

using libraryclient::LibraryClientPtr;
using fwk::Application;
using fwk::Configuration;

namespace ui {

NiepceWindow::NiepceWindow()
    : fwk::AppFrame("mainWindow-frame")
    , m_notifcenter(npc::NotificationCenter_new())
    , m_vbox(Gtk::Orientation::VERTICAL)
    , m_hbox(Gtk::Orientation::HORIZONTAL)

{
    // headerbar.
    Gtk::HeaderBar *header = Gtk::manage(new Gtk::HeaderBar);
    header->set_show_title_buttons(true);
    // XXX header->set_has_subtitle(true);

    Gtk::MenuButton* menu_btn = Gtk::manage(new Gtk::MenuButton);
    menu_btn->set_direction(Gtk::ArrowType::NONE);
    m_main_menu = Gio::Menu::create();
    menu_btn->set_menu_model(m_main_menu);
    header->pack_end(*menu_btn);

    // Undo redo buttons
    Gtk::Box *button_box = Gtk::manage(new Gtk::Box);
    button_box->get_style_context()->add_class("linked");
    Gtk::Button *undo_button = Gtk::manage(new Gtk::Button);
    undo_button->set_image_from_icon_name("edit-undo-symbolic");
    undo_button->set_label(_("Undo"));
    // XXX undo_button->set_always_show_image(true);
    undo_button->set_action_name("win.Undo");
    Gtk::Button *redo_button = Gtk::manage(new Gtk::Button);
    redo_button->set_image_from_icon_name("edit-redo-symbolic");
    redo_button->set_action_name("win.Redo");
    button_box->append(*undo_button);
    button_box->append(*redo_button);
    header->pack_start(*button_box);

    Gtk::Button *import_button = Gtk::manage(new Gtk::Button);
    import_button->set_label(_("Import..."));
    import_button->set_action_name("workspace.Import");
    header->pack_start(*import_button);

    setHeaderBar(header);
}


NiepceWindow::~NiepceWindow()
{
}

void
NiepceWindow::_createModuleShell()
{
    DBG_ASSERT(static_cast<bool>(m_libClient), "libclient not initialized");
    DBG_ASSERT(m_widget, "widget not built");

    DBG_OUT("creating module shell");

    // main view
    m_moduleshell = ModuleShell::Ptr(
        new ModuleShell(getLibraryClient()));
    m_moduleshell->buildWidget();

    add(m_moduleshell);

    m_notifcenter->add_listener(std::make_unique<npc::LnListener>(sigc::mem_fun(
                     *m_moduleshell,
                     &ModuleShell::on_lib_notification)));

    m_notifcenter->add_listener(std::make_unique<npc::LnListener>([this] (const eng::LibNotification& notification) {
            m_moduleshell->get_list_store()->on_lib_notification(notification);
    }));

    // workspace treeview
    auto workspace_actions = Gio::SimpleActionGroup::create();
    gtkWindow().insert_action_group("workspace", workspace_actions);
    m_workspacectrl = WorkspaceController::Ptr(new WorkspaceController(workspace_actions));
    m_workspacectrl->libtree_selection_changed.connect([this] {
        m_moduleshell->on_content_will_change();
    });

    m_notifcenter->add_listener(std::make_unique<npc::LnListener>([this] (const eng::LibNotification& notification) {
        m_workspacectrl->on_lib_notification(notification);
    }));
    add(m_workspacectrl);

    // m_hbox.set_border_width(4);
    m_hbox.set_wide_handle(true);
    m_hbox.set_start_child(*(m_workspacectrl->buildWidget()));
    m_hbox.set_end_child(*(m_moduleshell->buildWidget()));
    m_databinders.add_binder(new fwk::ConfigDataBinder<int>(m_hbox.property_position(),
                                                                  Application::app()->config(),
                                                                  "workspace_splitter"));

    static_cast<Gtk::Window*>(m_widget)->set_child(m_vbox);

    static_cast<Gtk::ApplicationWindow&>(gtkWindow()).set_show_menubar(true);
    m_vbox.append(m_hbox);


    SelectionController::Ptr selection_controller = m_moduleshell->get_selection_controller();
    m_filmstrip = FilmStripController::Ptr(
        new FilmStripController(m_moduleshell->get_list_store()));
    add(m_filmstrip);

    m_vbox.append(*(m_filmstrip->buildWidget()));

    // status bar
    m_vbox.append(m_statusBar);
    m_statusBar.push(Glib::ustring(_("Ready")));

    selection_controller->add_selectable(m_filmstrip);
}


void NiepceWindow::on_action_edit_labels()
{
    DBG_OUT("edit labels");
    auto libclient = getLibraryClient();
    DBG_ASSERT(!!getLibraryClient(), "No library client");
    // This is an error. Just make it safe.
    if (libclient) {
        // get the labels.
        m_editlabel_dialog = EditLabels::Ptr(new EditLabels(libclient));
        // XXX I do have serious doubt on the safety of this
        m_editlabel_dialog->run_modal(shared_frame_ptr(), [this] (int) {
            m_editlabel_dialog.reset((EditLabels*)nullptr);
        });
    }
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
