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
#include "fwk/base/string.hpp"
#include "fwk/toolkit/application.hpp"
#include "fwk/toolkit/configuration.hpp"
#include "fwk/toolkit/gtkutils.hpp"
#include "engine/importer/iimporter.hpp"
#include "libraryclient/libraryclient.hpp"
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
        { ICON_FOLDER, "folder" },
        { ICON_PROJECT, "applications-accessories" },
        { ICON_ROLL, "emblem-photos" },
        { ICON_TRASH, "user-trash" },
        { ICON_KEYWORD, "application-certificate" },
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

libraryclient::LibraryClientPtr WorkspaceController::getLibraryClient() const
{
    return std::dynamic_pointer_cast<NiepceWindow>(m_parent.lock())->getLibraryClient();
}

fwk::Configuration::Ptr WorkspaceController::getLibraryConfig() const
{
    return std::dynamic_pointer_cast<NiepceWindow>(m_parent.lock())->getLibraryConfig();
}

void WorkspaceController::action_new_folder()
{
    auto& window = std::dynamic_pointer_cast<NiepceWindow>(m_parent.lock())->gtkWindow();
    ui::dialog_request_new_folder(getLibraryClient()->client(), window.gobj());
}

void WorkspaceController::action_delete_folder()
{
    auto id = get_selected_folder_id();
    if (id) {
        auto& window = std::dynamic_pointer_cast<NiepceWindow>(m_parent.lock())->gtkWindow();
        auto dialog = Glib::wrap(ui::dialog_confirm(_("Delete selected folder?"), window.gobj()));
        dialog->signal_response().connect([this, id, dialog] (int response) {
            if (response == Gtk::ResponseType::YES) {
                ffi::libraryclient_delete_folder(getLibraryClient()->client(), id);
            }
            delete dialog;
        });
        dialog->show();
    }
}

void WorkspaceController::perform_file_import(ImportDialog::Ptr dialog)
{
    auto& cfg = Application::app()->config(); // XXX change to getLibraryConfig()
    // as the last import should be part of the library not the application.

    // import
    // XXX change the API to provide more details.
    std::string source = dialog->get_source();
    if (source.empty()) {
        return;
    }
    // XXX this should be a different config key
    // specific to the importer.
    cfg.setValue("last_import_location", source);

    auto importer = dialog->get_importer();
    DBG_ASSERT(!!importer, "Import can't be null if we clicked import");
    if (importer) {
        auto dest_dir = dialog->get_dest_dir();
        importer->do_import(
            source, dest_dir,
            [this] (const std::string& path, const fwk::FileListPtr& files, Managed manage) -> bool {
                ffi::libraryclient_import_files(
                    getLibraryClient()->client(), path.c_str(), files.get(), manage);
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


void WorkspaceController::on_lib_notification(const eng::LibNotification &ln)
{
    DBG_OUT("notification for workspace");
    auto type = engine_library_notification_type(&ln);
    switch (type) {
    case eng::NotificationType::ADDED_FOLDER:
    {
        auto f = engine_library_notification_get_libfolder(&ln);
        this->add_folder_item(f);
        break;
    }
    case eng::NotificationType::FOLDER_DELETED:
    {
        auto id = engine_library_notification_get_id(&ln);
        remove_folder_item(id);
        break;
    }
    case eng::NotificationType::ADDED_KEYWORD:
    {
        auto k = engine_library_notification_get_keyword(&ln);
        DBG_ASSERT(k, "keyword must not be NULL");
        add_keyword_item(k);
        break;
    }
    case eng::NotificationType::FOLDER_COUNTED:
    case eng::NotificationType::KEYWORD_COUNTED:
    {
        auto count = engine_library_notification_get_count(&ln);
        DBG_OUT("count for container %Ld is %Ld", (long long)count->id, (long long)count->count);
        std::map<eng::library_id_t, Gtk::TreeModel::iterator>::const_iterator iter;
        switch (type) {
        case eng::NotificationType::FOLDER_COUNTED:
            iter = m_folderidmap.find(count->id);
            break;
        case eng::NotificationType::KEYWORD_COUNTED:
            iter = m_keywordsidmap.find(count->id);
            break;
        default:
            DBG_ASSERT(false, "should never happen");
            break;
        }
        if(iter != m_folderidmap.cend()) {
            Gtk::TreeRow row = *(iter->second);
            row[m_librarycolumns.m_count_n] = count->count;
            row[m_librarycolumns.m_count] = std::to_string(count->count);
        }

        break;
    }
    case eng::NotificationType::FOLDER_COUNT_CHANGE:
    case eng::NotificationType::KEYWORD_COUNT_CHANGE:
    {
        auto count = engine_library_notification_get_count(&ln);
        DBG_OUT("count change for container %Ld is %Ld", (long long)count->id,
                (long long)count->count);
        std::map<eng::library_id_t, Gtk::TreeModel::iterator>::const_iterator iter;
        switch (type) {
        case eng::NotificationType::FOLDER_COUNT_CHANGE:
            iter = m_folderidmap.find(count->id);
            break;
        case eng::NotificationType::KEYWORD_COUNT_CHANGE:
            iter = m_keywordsidmap.find(count->id);
            break;
        default:
            DBG_ASSERT(false, "should never happen");
            break;
        }
        if(iter != m_folderidmap.cend()) {
            Gtk::TreeRow row = *(iter->second);
            int new_count = row[m_librarycolumns.m_count_n] + count->count;
            row[m_librarycolumns.m_count_n] = new_count;
            row[m_librarycolumns.m_count] = std::to_string(new_count);
        }

        break;
    }
    default:
        break;
    }
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
        ffi::libraryclient_query_folder_content(getLibraryClient()->client(), id);
        break;

    case KEYWORD_ITEM:
        ffi::libraryclient_query_keyword_content(getLibraryClient()->client(), id);
        break;

    default:
        DBG_OUT("selected something not a folder");
    }

    std::dynamic_pointer_cast<Gio::SimpleAction>(
        m_action_group->lookup_action("DeleteFolder"))->set_enabled(type == FOLDER_ITEM);
    libtree_selection_changed.emit();
}

void WorkspaceController::on_row_expanded_collapsed(const Gtk::TreeModel::iterator& iter,
                                                    const Gtk::TreeModel::Path& /*path*/,
                                                    bool expanded)
{
    int type = (*iter)[m_librarycolumns.m_type];
    fwk::Configuration::Ptr cfg = getLibraryConfig();
    const char* key = nullptr;
    switch(type) {
    case FOLDERS_ITEM:
        key = "workspace_folders_expanded";
        break;
    case PROJECTS_ITEM:
        key = "workspace_projects_expanded";
        break;
    case KEYWORDS_ITEM:
        key = "workspace_keywords_expanded";
        break;
    }
    if(cfg && key) {
        cfg->setValue(key, std::to_string(expanded));
    }
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

void WorkspaceController::add_keyword_item(const eng::Keyword* k)
{
    auto children = m_keywordsNode->children();
    bool was_empty = children.empty();
    auto keyword = fwk::RustFfiString(engine_db_keyword_keyword(k));
    auto iter = add_item(m_treestore, children,
                         m_icons[ICON_KEYWORD], keyword.c_str(),
                         engine_db_keyword_id(k), KEYWORD_ITEM);
    ffi::libraryclient_count_keyword(getLibraryClient()->client(), engine_db_keyword_id(k));
    m_keywordsidmap[engine_db_keyword_id(k)] = iter;
    if(was_empty) {
        expand_from_cfg("workspace_keywords_expanded", m_keywordsNode);
    }
}

void WorkspaceController::remove_folder_item(eng::library_id_t id)
{
    auto iter = m_folderidmap.find(id);
    if (iter == m_folderidmap.end()) {
        return;
    }
    m_treestore->erase(iter->second);
    m_folderidmap.erase(iter);
}

void WorkspaceController::add_folder_item(const eng::LibFolder* f)
{
    int icon_idx = ICON_ROLL;
    if(engine_db_libfolder_virtual_type(f) == eng::FolderVirtualType::TRASH) {
        icon_idx = ICON_TRASH;
        ffi::libraryclient_set_trash_id(getLibraryClient()->client(), engine_db_libfolder_id(f));
    }
    auto children = m_folderNode->children();
    bool was_empty = children.empty();
    auto iter = add_item(m_treestore, children,
                         m_icons[icon_idx],
                         engine_db_libfolder_name(const_cast<eng::LibFolder*>(f)),
                         engine_db_libfolder_id(f), FOLDER_ITEM);
    if(engine_db_libfolder_expanded(f)) {
        m_librarytree.expand_row(m_treestore->get_path(iter), false);
    }
    ffi::libraryclient_count_folder(getLibraryClient()->client(), engine_db_libfolder_id(f));
    m_folderidmap[engine_db_libfolder_id(f)] = iter;
    // expand if needed. Because Gtk doesn't expand empty
    if (was_empty) {
        expand_from_cfg("workspace_folders_expanded", m_folderNode);
    }
}

Gtk::TreeModel::iterator
WorkspaceController::add_item(const Glib::RefPtr<Gtk::TreeStore> &treestore,
                              const Gtk::TreeNodeChildren & childrens,
                              const Glib::RefPtr<Gio::Icon> & icon,
                              const Glib::ustring & label,
                              eng::library_id_t id, int type) const
{
    Gtk::TreeModel::iterator iter;
    Gtk::TreeModel::Row row;
    iter = treestore->append(childrens);
    row = *iter;
    row[m_librarycolumns.m_icon] = icon;
    row[m_librarycolumns.m_label] = label;
    row[m_librarycolumns.m_id] = id;
    row[m_librarycolumns.m_type] = type;
    row[m_librarycolumns.m_count] = "--";
    return iter;
}

Gtk::Widget * WorkspaceController::buildWidget()
{
    if(m_widget) {
        return m_widget;
    }
    m_widget = &m_vbox;
    m_treestore = Gtk::TreeStore::create(m_librarycolumns);
    m_librarytree.set_model(m_treestore);
    m_librarytree.set_activate_on_single_click(true);
    DBG_ASSERT(m_treestore->get_flags() == Gtk::TreeModel::Flags::ITERS_PERSIST,
        "Model isn't persistent");

    m_folderNode = add_item(m_treestore, m_treestore->children(),
                            m_icons[ICON_FOLDER],
                            Glib::ustring(_("Pictures")), 0,
                            FOLDERS_ITEM);
    m_projectNode = add_item(m_treestore, m_treestore->children(),
                             m_icons[ICON_PROJECT],
                             Glib::ustring(_("Projects")), 0,
                             PROJECTS_ITEM);
    m_keywordsNode = add_item(m_treestore, m_treestore->children(),
                              m_icons[ICON_KEYWORD],
                              Glib::ustring(_("Keywords")), 0,
                              KEYWORDS_ITEM);
    m_librarytree.set_headers_visible(false);

    auto renderer = Gtk::manage(new Gtk::CellRendererPixbuf());
    auto col = Gtk::manage(new Gtk::TreeViewColumn("", *renderer));
    m_librarytree.append_column(*col);
    col->add_attribute(*renderer, "gicon", m_librarycolumns.m_icon);

    int num = m_librarytree.append_column("", m_librarycolumns.m_label);
    col = m_librarytree.get_column(num - 1);
    col->set_expand(true);
    num = m_librarytree.append_column("", m_librarycolumns.m_count);
    col = m_librarytree.get_column(num - 1);
    col->set_alignment(1.0f);

    Gtk::Box* header = Gtk::manage(new Gtk::Box(Gtk::Orientation::HORIZONTAL));
    header->set_spacing(4);
    header->set_margin(4);
    // TODO make it a mnemonic
    m_label.set_text_with_mnemonic(Glib::ustring(_("_Workspace")));
    m_label.set_mnemonic_widget(m_librarytree);
    m_label.set_hexpand(true);
    header->append(m_label);
    Gtk::MenuButton* add_btn = Gtk::manage(new Gtk::MenuButton);
    add_btn->set_direction(Gtk::ArrowType::NONE);
    add_btn->set_icon_name("view-more-symbolic");

    m_menu = Gio::Menu::create();

    auto section = Gio::Menu::create();
    m_menu->append_section(section);
    fwk::add_menu_action(m_action_group.get(), "NewFolder",
                         sigc::mem_fun(*this,
                                       &WorkspaceController::action_new_folder),
                         section, _("New Folder..."), "workspace");

    section->append(_("New Project..."), "NewProject");

    auto action = fwk::add_menu_action(m_action_group.get(), "DeleteFolder",
                                       sigc::mem_fun(
                                           *this, &WorkspaceController::action_delete_folder),
                                       section, _("Delete Folder"), "workspace");
    action->set_enabled(false);

    section = Gio::Menu::create();
    m_menu->append_section(section);

    fwk::add_menu_action(m_action_group.get(), "Import",
                         sigc::mem_fun(*this,
                                       &WorkspaceController::action_file_import),
                         section, _("_Import..."), "workspace");

    add_btn->set_menu_model(m_menu);

    m_context_menu = Gtk::manage(new Gtk::PopoverMenu(m_menu));
    m_context_menu->set_parent(m_librarytree);
    m_librarytree.signal_unrealize().connect([this] {
        m_context_menu->unparent();
    });

    header->append(*add_btn);
    m_vbox.append(*header);
    Gtk::ScrolledWindow* scrolled = Gtk::manage(new Gtk::ScrolledWindow);
    m_vbox.append(*scrolled);
    m_librarytree.set_vexpand(true);
    scrolled->set_child(m_librarytree);

    m_librarytree.get_selection()->signal_changed().connect([this] {
        this->on_libtree_selection();
    });
    m_librarytree.signal_row_expanded().connect([this] (const Gtk::TreeModel::iterator& iter, const Gtk::TreeModel::Path& path) {
        this->on_row_expanded(iter, path);
    });
    m_librarytree.signal_row_collapsed().connect([this] (const Gtk::TreeModel::iterator& iter, const Gtk::TreeModel::Path& path) {
        this->on_row_collapsed(iter, path);
    });

    auto gesture = Gtk::GestureClick::create();
    gesture->set_button(3);
    m_librarytree.add_controller(gesture);
    gesture->signal_pressed()
        .connect([this] (int, double x, double y) {
            this->on_button_press_event(x, y);
        });
    return m_widget;
}

/** Expand treenode from a cfg key */
void WorkspaceController::expand_from_cfg(const char* key,
                                          const Gtk::TreeModel::iterator& treenode)
{
    fwk::Configuration::Ptr cfg = getLibraryConfig();

    bool expanded = std::stoi(cfg->getValue(key, "1"));
    if(expanded) {
        m_librarytree.expand_row(m_treestore->get_path(treenode),
                                 false);
    }
}

void WorkspaceController::on_ready()
{
    libraryclient::LibraryClientPtr libraryClient = getLibraryClient();
    if (libraryClient) {
        ffi::libraryclient_get_all_folders(libraryClient->client());
        ffi::libraryclient_get_all_keywords(libraryClient->client());
    }
}

void WorkspaceController::on_button_press_event(double x, double y)
{
    if (m_librarytree.get_selection()->count_selected_rows() != 0) {
        m_context_menu->set_pointing_to(Gdk::Rectangle(x, y, 1, 1));
        m_context_menu->popup();
    }
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
