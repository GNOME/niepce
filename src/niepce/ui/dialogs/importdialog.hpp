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

#pragma once

#include <list>
#include <memory>
#include <string>

#include <glibmm/refptr.h>
#include <glibmm/dispatcher.h>

#include "engine/importer/importedfile.hpp"
#include "fwk/toolkit/gtkutils.hpp"
#include "fwk/toolkit/dialog.hpp"
#include "fwk/toolkit/uiresult.hpp"
#include "metadatapanecontroller.hpp"
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

class ImportDialog
	: public fwk::Dialog
{
public:
    typedef std::shared_ptr<ImportDialog> Ptr;

    class PreviewGridModel
        : public Gtk::TreeModel::ColumnRecord
    {
    public:
        Gtk::TreeModelColumn<Glib::ustring> filename;
        Gtk::TreeModelColumn<Glib::RefPtr<Gdk::Pixbuf>> pixbuf;
        Gtk::TreeModelColumn<eng::ImportedFilePtr> file;
        PreviewGridModel()
            { add(filename); add(pixbuf); add(file); }
    };

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
private:
    class ImportParam;

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
    PreviewGridModel m_grid_columns;
    Glib::RefPtr<Gtk::ListStore> m_images_list_model;
    std::map<std::string, Gtk::TreeModel::iterator> m_images_list_map;

    std::shared_ptr<ffi::ImageGridView> m_image_gridview;
    Gtk::IconView *m_gridview;

    MetaDataPaneController::Ptr m_metadata_pane;

    fwk::UIResultSingle<std::list<eng::ImportedFilePtr>> m_files_to_import;
    fwk::UIResults<std::pair<std::string, fwk::ThumbnailPtr>> m_previews_to_import;
};

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

