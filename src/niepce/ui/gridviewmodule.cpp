/*
 * niepce - ui/gridviewmodule.cpp
 *
 * Copyright (C) 2009-2022 Hubert Figuière
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

#include <gtkmm/liststore.h>
#include <gtkmm/treestore.h>
#include <gtkmm/treeselection.h>
#include <gtkmm/popovermenu.h>

#include <exempi/xmpconsts.h>

#include "rust_bindings.hpp"

#include "fwk/base/debug.hpp"
#include "fwk/toolkit/application.hpp"
#include "fwk/toolkit/configdatabinder.hpp"
#include "fwk/toolkit/widgets/dock.hpp"
#include "niepce/ui/imageliststore.hpp"
#include "gridviewmodule.hpp"

#include "rust_bindings.hpp"

namespace ui {

GridViewModule::GridViewModule(const ui::SelectionController& selection_controller,
                               Glib::RefPtr<Gio::Menu> menu, const npc::UIDataProvider& ui_data_provider,
                               const ImageListStorePtr& store)
  : m_selection_controller(selection_controller)
  , m_model(store->clone())
  , m_menu(menu)
  , m_ui_data_provider(ui_data_provider)
  , m_librarylistview(nullptr)
  , m_lib_splitview(Gtk::Orientation::HORIZONTAL)
  , m_dock(nullptr)
  , m_context_menu(nullptr)
{
}

GridViewModule::~GridViewModule()
{
    m_widget = nullptr;
}

void
GridViewModule::on_lib_notification(const eng::LibNotification &ln, const npc::LibraryClientWrapper& client)
{
    switch (ffi::engine_library_notification_type(&ln)) {
    case eng::NotificationType::METADATA_QUERIED:
    {
        auto lm = ffi::engine_library_notification_get_libmetadata(&ln);
        DBG_OUT("received metadata");
        if (lm) {
            m_metapanecontroller->display(lm->id(), lm);
        } else {
            ERR_OUT("Invalid LibMetadata (nullptr)");
        }
        break;
    }
    case eng::NotificationType::METADATA_CHANGED:
    {
        DBG_OUT("metadata changed");
        auto id = ffi::engine_library_notification_get_id(&ln);
        if(id && id == m_metapanecontroller->displayed_file()) {
            // FIXME: actually just update the metadata
            ffi::libraryclient_request_metadata(&client, id);
        }
        break;
    }
    default:
        break;
    }
}

void GridViewModule::display_none()
{
    m_metapanecontroller->display(0, nullptr);
}

bool GridViewModule::get_colour_callback_c(int32_t label, ffi::RgbColour* out,
                                           const void* user_data)
{
    if (user_data == nullptr) {
        return false;
    }

    std::optional<fwk::RgbColour> colour =
        static_cast<const GridViewModule*>(user_data)->get_colour_callback(label);

    if (colour.has_value() && out) {
        *out = colour.value();
        return true;
    }

    return false;
}

std::optional<fwk::RgbColour> GridViewModule::get_colour_callback(int32_t label) const
{
    return std::optional(m_ui_data_provider.colourForLabel(label));
}

Gtk::Widget * GridViewModule::buildWidget()
{
  if(m_widget) {
    return m_widget;
  }
  m_widget = &m_lib_splitview;
  m_context_menu = Gtk::manage(new Gtk::PopoverMenu(m_menu));

  m_image_grid_view = std::shared_ptr<ffi::ImageGridView>(
      ffi::npc_image_grid_view_new(
          GTK_TREE_MODEL(m_model->unwrap_ref().gobj()),
          GTK_POPOVER_MENU(m_context_menu->gobj())
      ),
      ffi::npc_image_grid_view_release);
  m_librarylistview = Gtk::manage(
      Glib::wrap(
          GTK_ICON_VIEW(ffi::npc_image_grid_view_get_icon_view(m_image_grid_view.get())))
      );
  m_librarylistview->set_selection_mode(Gtk::SelectionMode::SINGLE);
  m_librarylistview->property_row_spacing() = 0;
  m_librarylistview->property_column_spacing() = 0;
  m_librarylistview->property_spacing() = 0;
  m_librarylistview->property_margin() = 0;
  m_librarylistview->set_vexpand(true);

  m_context_menu->set_parent(*m_librarylistview);
  m_context_menu->set_has_arrow(false);
  m_librarylistview->signal_unrealize().connect(
      sigc::mem_fun(*m_context_menu, &Gtk::PopoverMenu::unparent));

  auto gesture = Gtk::GestureClick::create();
  m_librarylistview->add_controller(gesture);
  gesture->signal_pressed().connect([this, gesture] (int, double x, double y) {
      this->on_librarylistview_click(gesture, x, y);
  });

  // the main cell
  Gtk::CellRenderer* libcell = manage(
      Glib::wrap(
          ffi::npc_library_cell_renderer_new(&get_colour_callback_c, this)));
  g_signal_connect(
      libcell->gobj(), "rating-changed", G_CALLBACK(GridViewModule::on_rating_changed), this);

  Glib::RefPtr<Gtk::CellArea> cell_area = m_librarylistview->property_cell_area();
  cell_area->pack_start(*libcell, FALSE);
  cell_area->add_attribute(*libcell, "pixbuf",
                           static_cast<gint>(ffi::ColIndex::Thumb));
  cell_area->add_attribute(*libcell, "libfile",
                           static_cast<gint>(ffi::ColIndex::File));
  cell_area->add_attribute(*libcell, "status",
                           static_cast<gint>(ffi::ColIndex::FileStatus));

  m_scrollview.set_child(*m_librarylistview);
  m_scrollview.set_policy(Gtk::PolicyType::AUTOMATIC, Gtk::PolicyType::AUTOMATIC);
  m_lib_splitview.set_wide_handle(true);

  // build the toolbar
  auto box = Gtk::manage(new Gtk::Box(Gtk::Orientation::VERTICAL));
  box->append(m_scrollview);
  auto toolbar = ffi::image_toolbar_new();
  gtk_box_append(box->gobj(), GTK_WIDGET(toolbar));
  m_lib_splitview.set_start_child(*box);

  m_dock = new fwk::Dock();
  m_metapanecontroller = MetaDataPaneController::Ptr(new MetaDataPaneController);
  m_metapanecontroller->signal_metadata_changed.connect(
      sigc::mem_fun(*this, &GridViewModule::on_metadata_changed));
  add(m_metapanecontroller);
  m_lib_splitview.set_end_child(*m_dock);
  m_dock->vbox().append(*m_metapanecontroller->buildWidget());

  m_databinders.add_binder(new fwk::ConfigDataBinder<int>(
                             m_lib_splitview.property_position(),
                             fwk::Application::app()->config(),
                             "meta_pane_splitter"));
  return m_widget;
}

void GridViewModule::dispatch_action(const std::string & /*action_name*/)
{
}

void GridViewModule::set_active(bool /*active*/)
{
}

Gtk::IconView * GridViewModule::image_list()
{
    return m_librarylistview;
}

eng::library_id_t GridViewModule::get_selected()
{
    eng::library_id_t id = 0;
    Glib::RefPtr<Gtk::TreeSelection> selection;

    std::vector<Gtk::TreePath> paths = m_librarylistview->get_selected_items();
    if(!paths.empty()) {
        Gtk::TreePath path(*(paths.begin()));
        DBG_OUT("found path %s", path.to_string().c_str());
        id = m_model->unwrap_ref().get_libfile_id_at_path((const char*)path.gobj());
    }
    DBG_OUT("get_selected %Ld", (long long)id);
    return id;
}

void GridViewModule::select_image(eng::library_id_t id)
{
    DBG_OUT("library select %Ld", (long long)id);
    auto path = ImageListStore_get_path_from_id(m_model->unwrap_ref(), id);
    if (path) {
        m_librarylistview->scroll_to_path(path, false, 0, 0);
        m_librarylistview->select_path(path);
    }
    else {
        m_librarylistview->unselect_all();
    }
}

void GridViewModule::on_metadata_changed(const fwk::WrappedPropertyBagPtr& props,
                                         const fwk::WrappedPropertyBagPtr& old)
{
    // TODO this MUST be more generic
    DBG_OUT("on_metadata_changed()");
    m_selection_controller.set_properties(*props, *old);
}

void GridViewModule::on_rating_changed(GtkCellRenderer*, eng::library_id_t /*id*/,
                                       int32_t rating, gpointer user_data)
{
    auto self = static_cast<GridViewModule*>(user_data);
    self->m_selection_controller.set_rating(rating);
}

void GridViewModule::on_librarylistview_click(const Glib::RefPtr<Gtk::GestureClick>& gesture, double x, double y)
{
    auto button = gesture->get_current_button();
    DBG_OUT("GridView click handler, button: %u", button);
    if (button == 3 && !m_librarylistview->get_selected_items().empty()) {
        m_context_menu->set_pointing_to(Gdk::Rectangle(x, y, 1, 1));
        m_context_menu->popup();

        return;
    }
    Gtk::TreeModel::Path path;
    Gtk::CellRenderer * renderer = nullptr;
    DBG_OUT("GridView click (%f, %f)", x, y);
    if (m_librarylistview->get_item_at_pos(x, y, path, renderer)){
        DBG_OUT("found an item");
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
