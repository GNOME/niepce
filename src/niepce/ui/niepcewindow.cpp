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

#include "niepce/notificationcenter.hpp"
#include "fwk/base/debug.hpp"
#include "fwk/base/moniker.hpp"
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

using libraryclient::LibraryClient;
using libraryclient::LibraryClientPtr;
using fwk::Application;
using fwk::Configuration;
using fwk::UndoHistory;

namespace ui {

NiepceWindow::NiepceWindow()
    : fwk::AppFrame("mainWindow-frame")
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

    m_notifcenter->signal_lib_notification
        .connect(sigc::mem_fun(
                     *m_moduleshell->get_gridview(),
                     &GridViewModule::on_lib_notification));
    m_notifcenter->signal_lib_notification
        .connect([this] (const eng::LibNotification& notification) {
            m_moduleshell->get_list_store()->on_lib_notification(notification);
        });
    m_notifcenter->signal_lib_notification
        .connect([this] (const eng::LibNotification& notification) {
            m_moduleshell->get_map_module()->on_lib_notification(notification);
        });

    // workspace treeview
    auto workspace_actions = Gio::SimpleActionGroup::create();
    gtkWindow().insert_action_group("workspace", workspace_actions);
    m_workspacectrl = WorkspaceController::Ptr(new WorkspaceController(workspace_actions));
    m_workspacectrl->libtree_selection_changed.connect([this] {
        m_moduleshell->on_content_will_change();
    });

    m_notifcenter->signal_lib_notification
        .connect([this] (const eng::LibNotification& notification) {
            m_workspacectrl->on_lib_notification(notification);
        });
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


Gtk::Widget *
NiepceWindow::buildWidget()
{
    if(m_widget) {
        return m_widget;
    }
    Gtk::Window & win(gtkWindow());

    m_widget = &win;

    init_actions();

    m_notifcenter = niepce::NotificationCenter::make();

    Glib::ustring name("org.gnome.Niepce");
    set_icon_from_theme(name);
    win.set_icon_name(name);

    m_notifcenter->signal_lib_notification.connect(
        sigc::mem_fun(*this, &NiepceWindow::on_lib_notification));

    win.set_size_request(600, 400);
    on_open_library();
    return &win;
}


void NiepceWindow::init_actions()
{
    m_menu = Gio::Menu::create();
    Glib::RefPtr<Gio::Menu> submenu;
    Glib::RefPtr<Gio::Menu> section;

    // Get the action group.
    // Gtkmm doesn't allow GActionMap from GtkApplicationWindow
    auto action_map = dynamic_cast<Gio::ActionMap*>(&gtkWindow());
    DBG_ASSERT(action_map, "Not an action map");
    fwk::add_action(action_map, "Close",
                    sigc::mem_fun(
                        gtkWindow(), &Gtk::Window::hide), "win", "<Primary>w");

    // XXX Move to shell?
    create_undo_action(action_map);
    create_redo_action(action_map);

    fwk::add_action(action_map, "Cut",
                    Gio::ActionMap::ActivateSlot(),
                    "win", "<control>x");
    fwk::add_action(action_map, "Copy",
                    Gio::ActionMap::ActivateSlot(),
                    "win", "<control>c");
    fwk::add_action(action_map, "Paste",
                    Gio::ActionMap::ActivateSlot(),
                    "win" "<control>v");
    fwk::add_action(action_map, "Delete",
                    sigc::mem_fun(*this, &NiepceWindow::on_action_edit_delete),
                    "win", "Delete");

    // Main "hamburger" menu
    section = Gio::Menu::create();
    m_main_menu->append_section(section);
    section->append(_("New Catalog..."), "app.NewCatalog");
    section->append(_("Open Catalog..."), "app.OpenCatalog");

    section = Gio::Menu::create();
    m_main_menu->append_section(section);
    m_hide_tools_action
        = fwk::add_menu_action(action_map, "ToggleToolsVisible",
                               sigc::mem_fun(*this, &Frame::toggle_tools_visible),
                               section, _("Hide tools"), "win",
                               nullptr);
    fwk::add_menu_action(action_map, "EditLabels",
                         sigc::mem_fun(*this, &NiepceWindow::on_action_edit_labels),
                         section, _("Edit Labels..."), "win", nullptr);
    section->append(_("Preferences..."), "app.Preferences");

    section = Gio::Menu::create();
    m_main_menu->append_section(section);
    section->append(_("Help"), "app.Help");
    section->append(_("About"), "app.About");
}

void NiepceWindow::on_open_library()
{
    auto& cfg = Application::app()->config()->cfg;
    std::string libMoniker;
    int reopen = 0;
    try {
        reopen = std::stoi(std::string(cfg->getValue("reopen_last_catalog", "0")));
    }
    catch(...)
    {
    }
    if(reopen) {
        libMoniker = std::string(cfg->getValue("last_open_catalog", ""));
    }
    if (libMoniker.empty()) {
        prompt_open_library();
    } else {
        DBG_OUT("last library is %s", libMoniker.c_str());
        if (!open_library(libMoniker)) {
            ERR_OUT("library %s cannot be open. Prompting.",
                    libMoniker.c_str());
            prompt_open_library();
        }
    }
}

void NiepceWindow::create_initial_labels()
{
    // TODO make this parametric from resources
    ffi::libraryclient_create_label(m_libClient->client(), _("Label 1"), fwk::rgbcolour_to_string(55769, 9509, 4369).c_str()); /* 217, 37, 17 */
    ffi::libraryclient_create_label(m_libClient->client(), _("Label 2"), fwk::rgbcolour_to_string(24929, 55769, 4369).c_str()); /* 97, 217, 17 */
    ffi::libraryclient_create_label(m_libClient->client(), _("Label 3"), fwk::rgbcolour_to_string(4369, 50629, 55769).c_str()); /* 17, 197, 217 */
    ffi::libraryclient_create_label(m_libClient->client(), _("Label 4"), fwk::rgbcolour_to_string(35209, 4369, 55769).c_str()); /* 137, 17, 217 */
    ffi::libraryclient_create_label(m_libClient->client(), _("Label 5"), fwk::rgbcolour_to_string(55769, 35209, 4369).c_str()); /* 217, 137, 17 */
}


void NiepceWindow::on_lib_notification(const eng::LibNotification& ln)
{
    switch (engine_library_notification_type(&ln)) {
    case eng::NotificationType::NEW_LIBRARY_CREATED:
        create_initial_labels();
        break;
    case eng::NotificationType::ADDED_LABEL:
    {
        auto l = engine_library_notification_get_label(&ln);
        if (l) {
            m_libClient->getDataProvider()->addLabel(*eng::LabelPtr::from_raw(l));
        } else {
            ERR_OUT("Invalid label (nullptr)");
        }
        break;
    }
    case eng::NotificationType::LABEL_CHANGED:
    {
        auto l = engine_library_notification_get_label(&ln);
        if (l) {
            m_libClient->getDataProvider()->updateLabel(*eng::LabelPtr::from_raw(l));
        } else {
            ERR_OUT("Invalid label (nullptr)");
        }
        break;
    }
    case eng::NotificationType::LABEL_DELETED:
    {
        auto id = engine_library_notification_get_id(&ln);
        if (id) {
            m_libClient->getDataProvider()->deleteLabel(id);
        } else {
            ERR_OUT("Invalid ID");
        }
        break;
    }
    default:
        break;
    }
}

/** Prompt to open a library and open it on success */
void NiepceWindow::prompt_open_library()
{
    auto dialog = new Gtk::FileChooserDialog(
        gtkWindow(), _("Open catalog"),
        Gtk::FileChooser::Action::SELECT_FOLDER);
    dialog->add_button(_("Cancel"), Gtk::ResponseType::CANCEL);
    dialog->add_button(_("Open"), Gtk::ResponseType::OK);
    dialog->set_create_folders(true);

    dialog->signal_response().connect(
        [this, dialog] (int response) {
            DBG_OUT("response %d", response);
            if (response == Gtk::ResponseType::OK) {
                DBG_OUT("Accepted");
                auto& cfg = Application::app()->config()->cfg;
                auto file = dialog->get_file();
                Glib::ustring libraryToCreate = file->get_path();
                // pass it to the library
                std::string libMoniker = "local:";
                libMoniker += libraryToCreate.c_str();
                cfg->setValue("last_open_catalog", libMoniker);
                DBG_OUT("created catalog %s", libMoniker.c_str());
                this->open_library(libMoniker);
            }
            delete dialog;
        });
    dialog->show();
}

bool NiepceWindow::open_library(const std::string & libMoniker)
{
    fwk::Moniker mon = fwk::Moniker(libMoniker);
    m_libClient
        = LibraryClientPtr(new LibraryClient(
                               mon, m_notifcenter->get_channel()));
    // XXX ensure the library is open.
    set_title(libMoniker);
    m_library_cfg = fwk::Configuration_new(Glib::build_filename(mon.path(), "config.ini"));
    ffi::libraryclient_get_all_labels(m_libClient->client());
    if(!m_moduleshell) {
        _createModuleShell();
    }
    return true;
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

void NiepceWindow::on_action_edit_delete()
{
    // find the responder. And pass it.
    m_moduleshell->action_edit_delete();
}

void NiepceWindow::set_title(const std::string & title)
{
    Frame::set_title(_("Niepce Digital - ") + title);
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
