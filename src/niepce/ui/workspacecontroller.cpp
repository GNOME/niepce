/*
 * niepce - niepce/ui/workspacecontroller.cpp
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

#include <glibmm/i18n.h>

#include <gtkmm/icontheme.h>
#include <gtkmm/box.h>
#include <gtkmm/gestureclick.h>
#include <gtkmm/iconview.h>
#include <gtkmm/image.h>
#include <gtkmm/messagedialog.h>

#include "fwk/base/debug.hpp"
#include "fwk/toolkit/application.hpp"
#include "fwk/toolkit/gtkutils.hpp"
#include "engine/importer/iimporter.hpp"
#include "dialogs/importdialog.hpp"
#include "niepcewindow.hpp"
#include "workspacecontroller.hpp"

#include "rust_bindings.hpp"

using fwk::Application;
using fwk::Configuration;
using eng::Managed;
using eng::IImporter;

namespace ui {

WorkspaceController::WorkspaceController(const Glib::RefPtr<Gio::SimpleActionGroup>& action_group)
    : fwk::UiController()
    , m_action_group(action_group)
    , m_vbox(Gtk::Orientation::VERTICAL)
    , m_context_menu(nullptr)
{
    static struct _Icons {
        int icon_id;
        const char *icon_name;
    } icons[] = {
        { ICON_FOLDER, "folder-symbolic" },
        { ICON_PROJECT, "file-cabinet-symbolic" },
        { ICON_ROLL, "emblem-photos" },
        { ICON_TRASH, "user-trash" },
        { ICON_KEYWORD, "tag-symbolic" },
        { 0, nullptr }
    };

    int i = 0;
    while (icons[i].icon_name) {
        try {
            m_icons[icons[i].icon_id] = Gio::Icon::create(icons[i].icon_name);
        }
        catch (const std::exception& e)
        {
            ERR_OUT("Exception %s.", e.what());
        }
        i++;
    }
}

WorkspaceController::~WorkspaceController()
{
    m_context_menu->unparent();
    delete m_context_menu;
}

libraryclient::LibraryClientPtr WorkspaceController::getLibraryClient() const
{
    return std::dynamic_pointer_cast<NiepceWindow>(m_parent.lock())->getLibraryClient();
}

const fwk::ConfigurationPtr& WorkspaceController::getLibraryConfig() const
{
    return std::dynamic_pointer_cast<NiepceWindow>(m_parent.lock())->getLibraryConfig();
}

void WorkspaceController::action_new_folder()
{
    auto& window = std::dynamic_pointer_cast<NiepceWindow>(m_parent.lock())->gtkWindow();
    ui::dialog_request_new_folder(&getLibraryClient()->client(), window.gobj());
}

void WorkspaceController::action_delete_folder()
{
    auto id = get_selected_folder_id();
    if (id) {
        auto& window = std::dynamic_pointer_cast<NiepceWindow>(m_parent.lock())->gtkWindow();
        auto dialog = Glib::wrap(ui::dialog_confirm(_("Delete selected folder?"), window.gobj()));
        dialog->signal_response().connect([this, id, dialog] (int response) {
            if (response == Gtk::ResponseType::YES) {
                ffi::libraryclient_delete_folder(&getLibraryClient()->client(), id);
            }
            delete dialog;
        });
        dialog->show();
    }
}

void WorkspaceController::perform_file_import(ImportDialog::Ptr dialog)
{
    auto& cfg = Application::app()->config()->cfg; // XXX change to getLibraryConfig()
    // as the last import should be part of the library not the application.

    // import
    // XXX change the API to provide more details.
    std::string source = dialog->get_source();
    if (source.empty()) {
        return;
    }
    // XXX this should be a different config key
    // specific to the importer.
    cfg->setValue("last_import_location", source);

    auto importer = dialog->get_importer();
    DBG_ASSERT(!!importer, "Import can't be null if we clicked import");
    if (importer) {
        auto dest_dir = dialog->get_dest_dir();
        importer->do_import(
            source, dest_dir,
            [this] (const std::string& path, const fwk::FileListPtr& files, Managed manage) -> bool {
                ffi::libraryclient_import_files(
                    &getLibraryClient()->client(), path.c_str(), &*files, manage);
                // XXX the libraryclient function returns void
                return true;
            });
    }
}

void WorkspaceController::action_file_import()
{
    ImportDialog::Ptr import_dialog(new ImportDialog());

    import_dialog->run_modal(std::dynamic_pointer_cast<NiepceWindow>(m_parent.lock()),
                             [this, import_dialog] (int response) {
                                 DBG_OUT("import dialog response: %d", response);
                                 import_dialog->close();
                                 if (response == 0) {
                                     this->perform_file_import(import_dialog);
                                 }
                             });
}

void WorkspaceController::on_count_notification(int)
{
    DBG_OUT("received NOTIFICATION_COUNT");
}

eng::library_id_t WorkspaceController::get_selected_folder_id()
{
    auto selection = m_librarytree.get_selection();
    auto selected = selection->get_selected();
    if (!selected) {
        return 0;
    }
    int type = (*selected)[m_librarycolumns.m_type];
    eng::library_id_t id = (*selected)[m_librarycolumns.m_id];
    if (type != FOLDER_ITEM) {
        return 0;
    }
    return id;
}

void WorkspaceController::on_libtree_selection()
{
    Glib::RefPtr<Gtk::TreeSelection> selection = m_librarytree.get_selection();
    auto selected = selection->get_selected();
    if (!selected) {
        DBG_OUT("Invalid iterator");
        return;
    }
    int type = (*selected)[m_librarycolumns.m_type];
    eng::library_id_t id = (*selected)[m_librarycolumns.m_id];

    switch(type) {

    case FOLDER_ITEM:
        ffi::libraryclient_query_folder_content(&getLibraryClient()->client(), id);
        break;

    case KEYWORD_ITEM:
        ffi::libraryclient_query_keyword_content(&getLibraryClient()->client(), id);
        break;

    default:
        DBG_OUT("selected something not a folder");
    }

    std::dynamic_pointer_cast<Gio::SimpleAction>(
        m_action_group->lookup_action("DeleteFolder"))->set_enabled(type == FOLDER_ITEM);
    libtree_selection_changed.emit();
}

void WorkspaceController::on_row_expanded(const Gtk::TreeModel::iterator& iter,
                                          const Gtk::TreeModel::Path& path)
{
    on_row_expanded_collapsed(iter, path, true);
}

void WorkspaceController::on_row_collapsed(const Gtk::TreeModel::iterator& iter,
                                           const Gtk::TreeModel::Path& path)
{
    on_row_expanded_collapsed(iter, path, false);
}

}
/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  c-basic-offset:4
  tab-width:4
  indent-tabs-mode:nil
  fill-column:80
  End:
*/
