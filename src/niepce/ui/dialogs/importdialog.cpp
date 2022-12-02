/*
 * niepce - niepce/ui/dialogs/importdialog.cpp
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

#include <memory>

#include <glibmm/miscutils.h>
#include <gtkmm/button.h>
#include <gtkmm/checkbutton.h>
#include <gtkmm/combobox.h>
#include <gtkmm/comboboxtext.h>
#include <gtkmm/label.h>
#include <gtkmm/builder.h>
#include <gtkmm/signallistitemfactory.h>
#include <gtkmm/stack.h>

#include "fwk/base/debug.hpp"
#include "fwk/utils/pathutils.hpp"
#include "fwk/toolkit/application.hpp"
#include "engine/importer/directoryimporter.hpp"
#include "engine/importer/importedfile.hpp"
#include "importdialog.hpp"
#include "importers/directoryimporterui.hpp"
#include "importers/cameraimporterui.hpp"

namespace ui {

/** The Item row (Widget) for the listview */
class ItemRow
    : public Gtk::Box
{
public:
    ItemRow()
        : Gtk::Box(Gtk::Orientation::VERTICAL, 2)
        {
            append(m_image);
            append(m_filename);
            m_image.set_size_request(100, 100);
            // Adwaita class
            m_filename.add_css_class("caption");
        }
    Gtk::Image m_image;
    Gtk::Label m_filename;
};

Glib::RefPtr<ThumbItem> ThumbItem::create(const eng::ImportedFilePtr& imported_file) {
    return Glib::make_refptr_for_instance(new ThumbItem(imported_file));
}

Glib::RefPtr<ThumbListStore> ThumbListStore::create()
{
    return Glib::make_refptr_for_instance(new ThumbListStore());
}

void ThumbListStore::set_thumbnail(uint32_t index, const Glib::RefPtr<Gdk::Pixbuf>& pixbuf)
{
    auto item = get_item(index);
    if (!item) {
        ERR_OUT("item at index %u not found", index);
        return;
    }
    item->m_pixbuf = pixbuf;
    items_changed(index, 0, 0);
}

ImportDialog::ImportDialog()
  : fwk::Dialog("/org/gnome/Niepce/ui/importdialog.ui", "importDialog")
  , m_current_importer(nullptr)
  , m_importer_ui_stack(nullptr)
  , m_date_tz_combo(nullptr)
  , m_ufraw_import_check(nullptr)
  , m_rawstudio_import_check(nullptr)
  , m_destination_folder(nullptr)
  , m_import_source_combo(nullptr)
  , m_attributes_scrolled(nullptr)
  , m_images_list_scrolled(nullptr)
{
    auto& cfg = fwk::Application::app()->config()->cfg;
    m_base_dest_dir = std::string(cfg->getValue("base_import_dest_dir",
                                   Glib::get_user_special_dir(
                                       Glib::UserDirectory::PICTURES)));
    DBG_OUT("base_dest_dir set to %s", m_base_dest_dir.c_str());
}

ImportDialog::~ImportDialog()
{
}

void ImportDialog::add_importer_ui(IImporterUI& importer)
{
    m_import_source_combo->append(importer.id(), importer.name());
    Gtk::Widget* importer_widget = importer.setup_widget(
        std::static_pointer_cast<Frame>(shared_from_this()));
    m_importer_ui_stack->add(*importer_widget, importer.id());
    importer.set_source_selected_callback(
        [this] (const std::string& source, const std::string& dest_dir) {
            this->set_source(source, dest_dir);
        });
}

void ImportDialog::setup_widget()
{
    if(m_is_setup) {
        return;
    }

    auto& cfg = fwk::Application::app()->config()->cfg;

    Glib::RefPtr<Gtk::Builder> a_builder = builder();
    m_date_tz_combo = a_builder->get_widget<Gtk::ComboBox>("date_tz_combo");
    m_ufraw_import_check = a_builder->get_widget<Gtk::CheckButton>("ufraw_import_check");
    m_rawstudio_import_check = a_builder->get_widget<Gtk::CheckButton>("rawstudio_import_check");
    m_destination_folder = a_builder->get_widget<Gtk::Entry>("destinationFolder");

    // Sources
    m_importer_ui_stack = a_builder->get_widget<Gtk::Stack>("importer_ui_stack");
    m_import_source_combo = a_builder->get_widget<Gtk::ComboBoxText>("import_source_combo");
    m_import_source_combo->signal_changed()
        .connect([this]() {
            this->import_source_changed();
        });

    std::shared_ptr<IImporterUI> importer = std::make_shared<DirectoryImporterUI>();
    m_importers[importer->id()] = importer;
    add_importer_ui(*importer);
    importer = std::make_shared<CameraImporterUI>();
    m_importers[importer->id()] = importer;
    add_importer_ui(*importer);

    auto last_importer = std::string(cfg->getValue("last_importer", "DirectoryImporter"));
    m_import_source_combo->set_active_id(last_importer);

    // Metadata pane.
    m_attributes_scrolled = a_builder->get_widget<Gtk::ScrolledWindow>("attributes_scrolled");
    m_metadata_pane = MetaDataPaneController::Ptr(new MetaDataPaneController);
    auto w = m_metadata_pane->buildWidget();
    add(m_metadata_pane);
    m_attributes_scrolled->set_child(*w);

    // Gridview of previews.
    m_images_list_scrolled = a_builder->get_widget<Gtk::ScrolledWindow>("images_list_scrolled");
    m_images_list_model = ThumbListStore::create();
    auto selection_model = Gtk::SingleSelection::create(m_images_list_model);
    auto image_gridview = npc::npc_image_grid_view_new2(selection_model->gobj());
    m_gridview = Gtk::manage(Glib::wrap(image_gridview->get_grid_view()));
    m_image_gridview = std::move(image_gridview);
    auto item_factory = Gtk::SignalListItemFactory::create();
    m_gridview->set_factory(item_factory);
    item_factory->signal_setup().connect([] (const Glib::RefPtr<Gtk::ListItem>& list_item) {
        auto child = Gtk::manage(new ItemRow());
        list_item->set_child(*child);
    });
    item_factory->signal_bind().connect([] (const Glib::RefPtr<Gtk::ListItem>& list_item) {
        auto row = static_cast<ItemRow*>(list_item->get_child());
        auto item = std::dynamic_pointer_cast<ThumbItem>(list_item->get_item());
        row->m_filename.set_label(item->m_imported_file->name());
        row->m_image.set(item->m_pixbuf);
    });

    m_images_list_scrolled->set_child(*m_gridview);
    m_images_list_scrolled->set_policy(Gtk::PolicyType::AUTOMATIC, Gtk::PolicyType::AUTOMATIC);

    m_previews_to_import.connect([this]() {
        this->preview_received();
    });
    m_files_to_import.connect([this]() {
        this->append_files_to_import();
    });

    m_is_setup = true;
}

void ImportDialog::clear_import_list()
{
    if (m_images_list_model) {
        m_images_list_model->remove_all();
    }
    m_images_list_map.clear();
    m_files_to_import.clear();
    if (m_destination_folder) {
        m_destination_folder->set_text("");
    }
}

void ImportDialog::import_source_changed()
{
    auto id = m_import_source_combo->get_active_id();
    m_current_importer = m_importers[id];
    m_importer_ui_stack->set_visible_child(id);
    m_source = "";

    clear_import_list();

    auto& cfg = fwk::Application::app()->config()->cfg;
    cfg->setValue("last_importer", id.c_str());
}

void ImportDialog::set_source(const std::string& source, const std::string& dest_dir)
{
    clear_import_list();

    auto importer = get_importer();
    m_files_to_import.run(
        [this, source, importer] () {
            return importer->list_source_content(
                source,
                [this] (std::list<eng::ImportedFilePtr>&& list_to_import) {
                    this->m_files_to_import.send_data(std::move(list_to_import));
                });
        });

    m_source = source;
    m_dest_dir = Glib::build_filename(m_base_dest_dir, dest_dir);
    m_destination_folder->set_text(dest_dir);
}

const std::string& ImportDialog::get_dest_dir() const
{
    return m_dest_dir;
}

void ImportDialog::append_files_to_import()
{
    auto files_to_import = m_files_to_import.recv_data();

    if (!m_images_list_model) {
        ERR_OUT("No image list model");
        return;
    }
    // request the previews to the importer.
    std::list<std::string> paths;
    for(const auto & f : files_to_import) {
        DBG_OUT("selected %s", f->name().c_str());
        paths.push_back(f->path());
        m_images_list_model->append(ThumbItem::create(f));
        m_images_list_map.insert(std::make_pair(f->path(), m_images_list_model->get_n_items() - 1));
    }

    auto importer = get_importer();
    auto source = m_source;
    m_previews_to_import.run(
        [this, importer, source, paths] () {
            return importer->get_previews_for(
                source, paths,
                [this] (std::string&& path, fwk::ThumbnailPtr&& thumbnail) {
                    this->m_previews_to_import.send_data(
                        std::shared_ptr<decltype(this->m_previews_to_import)::value_type>(new std::pair(std::move(path), std::move(thumbnail))));
                });
        });
}

void ImportDialog::preview_received()
{
    auto preview = m_previews_to_import.recv_data();
    if (preview) {
        auto iter = m_images_list_map.find(preview->first);
        if (iter != m_images_list_map.end()) {
            auto index = iter->second;
            m_images_list_model->set_thumbnail(index, Glib::wrap((GdkPixbuf*)fwk::Thumbnail_to_pixbuf(*preview->second)));
        }
    }
}

void ImportDialog::run_modal(GtkWindow* parent, rust::Fn<void(const npc::ImportDialogArgument&, int32_t)> on_ok, npc::ImportDialogArgument* args) const
{
    run_modal_(parent, [args, on_ok] (int32_t r) {
        on_ok(*args, r);
        if (r == GTK_RESPONSE_DELETE_EVENT) {
            rust::Box<npc::ImportDialogArgument>::from_raw(args);
        }
    });
}

rust::Box<ImportRequest> ImportDialog::import_request() const
{
    return ui::import_request_new(m_source, get_dest_dir(), m_current_importer->get_importer());
}

}

/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  c-basic-offset:4
  indent-tabs-mode:nil
  tab-width:4
  fill-column:99
  End:
*/
