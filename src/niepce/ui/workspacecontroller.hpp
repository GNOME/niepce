/*
 * niepce - niepce/ui/workspacecontroller.hpp
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

#include <array>

#include <glibmm/ustring.h>

#include <gtkmm/treeview.h>
#include <gtkmm/label.h>
#include <gtkmm/treestore.h>

#include "fwk/toolkit/uicontroller.hpp"
#include "fwk/toolkit/notification.hpp"
#include "niepce/ui/niepcewindow.hpp"
#include "dialogs/importdialog.hpp"

namespace ui {

class WorkspaceController
    : public fwk::UiController
{
public:
    typedef std::shared_ptr<WorkspaceController> Ptr;

    enum {
        FOLDERS_ITEM,
        PROJECTS_ITEM,
        KEYWORDS_ITEM,
        FOLDER_ITEM,
        PROJECT_ITEM,
        KEYWORD_ITEM
    };

    WorkspaceController(const Glib::RefPtr<Gio::SimpleActionGroup>& action_group);
    class WorkspaceTreeColumns
        : public Gtk::TreeModelColumnRecord
    {
    public:

        WorkspaceTreeColumns()
            {
                add(m_icon);
                add(m_id);
                add(m_label);
                add(m_type);
                add(m_count);
                add(m_count_n);
            }
        Gtk::TreeModelColumn<Glib::RefPtr<Gio::Icon>> m_icon;
        Gtk::TreeModelColumn<eng::library_id_t> m_id;
        Gtk::TreeModelColumn<Glib::ustring> m_label;
        Gtk::TreeModelColumn<int> m_type;
        Gtk::TreeModelColumn<Glib::ustring> m_count;
        Gtk::TreeModelColumn<int> m_count_n;
    };

    virtual void on_ready() override;

    void on_lib_notification(const eng::LibNotification &);
    void on_count_notification(int);
    void on_libtree_selection();

    virtual Gtk::Widget * buildWidget() override;

    sigc::signal<void(void)> libtree_selection_changed;
private:
    /** Return the selected folder id. 0 if not a folder or no selection*/
    eng::library_id_t get_selected_folder_id();

    /** action to create a new folder */
    void action_new_folder();
    void action_delete_folder();
    /** action to import images: run the dialog */
    void action_file_import();
    /** */
    void perform_file_import(ImportDialog::Ptr dialog);

    void on_row_expanded_collapsed(const Gtk::TreeModel::iterator& iter,
                                   const Gtk::TreeModel::Path& path, bool expanded);
    void on_row_expanded(const Gtk::TreeModel::iterator& iter,
                         const Gtk::TreeModel::Path& path);
    void on_row_collapsed(const Gtk::TreeModel::iterator& iter,
                          const Gtk::TreeModel::Path& path);
    bool on_popup_menu();
    void on_button_press_event(double x, double y);

    libraryclient::LibraryClientPtr getLibraryClient() const;
    fwk::Configuration::Ptr getLibraryConfig() const;

    /** add a folder item to the treeview */
    void add_folder_item(const eng::LibFolder* f);
    /** Remove a folder from the treeview */
    void remove_folder_item(eng::library_id_t id);
    /** add a keyword item to the treeview */
    void add_keyword_item(const eng::Keyword* k);
    /** add a tree item in the treeview
     * @param treestore the treestore to add to
     * @param childrens the children subtree to add to
     * @param icon the icon for the item
     * @param label the item label
     * @param id the item id (in the database)
     * @paran type the type of node
     */
    Gtk::TreeModel::iterator add_item(const Glib::RefPtr<Gtk::TreeStore>& treestore,
                                      const Gtk::TreeNodeChildren& childrens,
                                      const Glib::RefPtr<Gio::Icon>& icon,
                                      const Glib::ustring& label,
                                      eng::library_id_t id, int type) const;

    void expand_from_cfg(const char* key, const Gtk::TreeModel::iterator& treenode);

    enum {
        ICON_FOLDER = 0,
        ICON_PROJECT,
        ICON_ROLL,
        ICON_TRASH,
        ICON_KEYWORD,
        _ICON_SIZE
    };

    Glib::RefPtr<Gio::SimpleActionGroup> m_action_group;

    std::array<Glib::RefPtr<Gio::Icon>, _ICON_SIZE> m_icons;
    WorkspaceTreeColumns           m_librarycolumns;
    Gtk::Box                       m_vbox;
    Gtk::Label                     m_label;
    Gtk::TreeView                  m_librarytree;
    Gtk::PopoverMenu* m_context_menu;
    Gtk::TreeModel::iterator       m_folderNode;  /**< the folder node */
    Gtk::TreeModel::iterator       m_projectNode; /**< the project node */
    Gtk::TreeModel::iterator       m_keywordsNode; /**< the keywords node */
    Glib::RefPtr<Gtk::TreeStore>   m_treestore;   /**< the treestore */
    std::map<eng::library_id_t, Gtk::TreeModel::iterator>   m_folderidmap;
    std::map<eng::library_id_t, Gtk::TreeModel::iterator>   m_projectidmap;
    std::map<eng::library_id_t, Gtk::TreeModel::iterator>   m_keywordsidmap;
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
