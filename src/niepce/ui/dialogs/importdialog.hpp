/*
 * niepce - niepce/ui/dialogs/importdialog.h
 *
 * Copyright (C) 2008-2022 Hubert Figuière
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

#include <list>
#include <memory>
#include <optional>
#include <string>

#include <glibmm/refptr.h>
#include <glibmm/dispatcher.h>
#include <giomm/liststore.h>
#include <gtkmm/gridview.h>
#include <gtkmm/singleselection.h>

#include "engine/importer/importedfile.hpp"
#include "fwk/toolkit/gtkutils.hpp"
#include "fwk/toolkit/dialog.hpp"
#include "fwk/toolkit/uiresult.hpp"
#include "niepce/ui/metadatapanecontroller.hpp"
#include "importers/iimporterui.hpp"

namespace Gtk {
class Dialog;
class ComboBox;
class ComboBoxText;
class CheckButton;
class TreeView;
class Stack;
}

namespace fwk {
class ImageGridView;
}

namespace eng {
class IImporter;
}

namespace ui {

class ThumbItem
    : public Glib::Object
{
public:
    static Glib::RefPtr<ThumbItem> create(const eng::ImportedFilePtr& imported_file);

    eng::ImportedFilePtr m_imported_file;
    Glib::RefPtr<Gdk::Pixbuf> m_pixbuf;
protected:
    ThumbItem(const eng::ImportedFilePtr& imported_file)
        : Glib::ObjectBase(typeid(ThumbItem))
        , Glib::Object()
        , m_imported_file(imported_file) {}
};

class ThumbListStore
    : public Gio::ListStore<ThumbItem>
{
public:
    static Glib::RefPtr<ThumbListStore> create();
    void set_thumbnail(uint32_t index, const Glib::RefPtr<Gdk::Pixbuf>& pixbuf);
protected:
    ThumbListStore()
        : Glib::ObjectBase(typeid(ThumbListStore)) {}
};

class ImportDialog
	: public fwk::Dialog
{
public:
    typedef std::shared_ptr<ImportDialog> Ptr;

    ImportDialog();
    virtual ~ImportDialog();

    virtual void setup_widget() override;

//  const std::list<std::string> & to_import() const
//      { return m_list_to_import; }
    const std::string& get_source() const
        { return m_source; }
    void import_source_changed();
    void set_source(const std::string&, const std::string&);
    const std::shared_ptr<IImporterUI>& importer_ui() const
        { return m_current_importer; }
    std::shared_ptr<eng::IImporter> get_importer() const
        { return m_current_importer->get_importer(); }
    const std::string& get_dest_dir() const;

    // cxx
    void close() const {
        const_cast<ImportDialog*>(this)->fwk::Dialog::close();
    }
    void run_modal(GtkWindow* parent, rust::Fn<void(const npc::ImportDialogArgument&, int32_t)> on_ok, npc::ImportDialogArgument*) const;
    rust::Box<ImportRequest> import_request() const;
private:
    void clear_import_list();
    void do_select_directories();
    void append_files_to_import();
    void preview_received();
    void add_importer_ui(IImporterUI& importer);

    std::map<std::string, std::shared_ptr<ui::IImporterUI>> m_importers;
    std::shared_ptr<ui::IImporterUI> m_current_importer; // as shared_ptr<> for lambda capture
    std::string m_source; /// Abstract source. The importer knows what to do.
    std::string m_base_dest_dir;
    std::string m_dest_dir;

    Gtk::Stack *m_importer_ui_stack;
    Gtk::ComboBox *m_date_tz_combo;
    Gtk::CheckButton *m_ufraw_import_check;
    Gtk::CheckButton *m_rawstudio_import_check;
    Gtk::Entry *m_destination_folder;
    Gtk::ComboBoxText *m_import_source_combo;
    Gtk::ScrolledWindow *m_attributes_scrolled;
    Gtk::ScrolledWindow *m_images_list_scrolled;
    Glib::RefPtr<ThumbListStore> m_images_list_model;
    std::map<std::string, guint> m_images_list_map;

    std::optional<rust::Box<npc::ImageGridView>> m_image_gridview;
    Gtk::GridView *m_gridview;

    MetaDataPaneController::Ptr m_metadata_pane;

    fwk::UIResultSingle<std::list<eng::ImportedFilePtr>> m_files_to_import;
    fwk::UIResults<std::pair<std::string, fwk::ThumbnailPtr>> m_previews_to_import;
};

inline
std::shared_ptr<ImportDialog> import_dialog_new() {
    return std::make_shared<ImportDialog>();
}

}

/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  tab-width:4
  fill-column:80
  End:
*/

#endif
