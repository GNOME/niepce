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

#include <gtkmm/popovermenu.h>

#include <exempi/xmpconsts.h>

#include "rust_bindings.hpp"

#include "fwk/base/debug.hpp"
#include "fwk/toolkit/application.hpp"
#include "fwk/toolkit/configdatabinder.hpp"
#include "fwk/toolkit/widgets/dock.hpp"
#include "gridviewmodule.hpp"

#include "rust_bindings.hpp"

namespace ui {

std::shared_ptr<GridViewModule> grid_view_module_new(const ui::SelectionController& selection_controller,
                                                     const GMenu* menu_, const eng::LibraryClientHost& client_host)
{
    Glib::RefPtr<Gio::Menu> menu;
    if (menu_) {
        menu = Glib::wrap(const_cast<GMenu*>(menu_));
    }
    return std::make_shared<GridViewModule>(selection_controller, menu, client_host);
}

GridViewModule::GridViewModule(const ui::SelectionController& selection_controller,
                               Glib::RefPtr<Gio::Menu> menu, const eng::LibraryClientHost& client_host)
  : m_selection_controller(selection_controller)
  , m_menu(menu)
  , m_libclient_host(client_host)
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
GridViewModule::on_lib_notification(const eng::LibNotification &ln, const eng::LibraryClientWrapper& client) const
{
    switch (ln.type_()) {
    case eng::NotificationType::METADATA_QUERIED:
    {
        auto& lm = ln.get_libmetadata();
        DBG_OUT("received metadata");
        m_metapanecontroller->display(lm.id(), &lm);
        break;
    }
    case eng::NotificationType::METADATA_CHANGED:
    {
        DBG_OUT("metadata changed");
        auto id = ln.id();
        if(id && id == m_metapanecontroller->displayed_file()) {
            // FIXME: actually just update the metadata
            client.request_metadata(id);
        }
        break;
    }
    default:
        break;
    }
}

void GridViewModule::display_none() const
{
    m_metapanecontroller->display(0, nullptr);
}

Gtk::Widget * GridViewModule::buildWidget()
{
  if(m_widget) {
    return m_widget;
  }
  m_widget = &m_lib_splitview;
  m_context_menu = Gtk::manage(new Gtk::PopoverMenu(m_menu));
  auto& model = m_selection_controller.get_list_store();

  auto image_grid_view = npc::npc_image_grid_view_new(
      GTK_SINGLE_SELECTION(model.unwrap_ref().gobj()),
      GTK_POPOVER_MENU(m_context_menu->gobj()),
      m_libclient_host
  );
  m_librarylistview = Gtk::manage(Glib::wrap(image_grid_view->get_grid_view()));
  m_image_grid_view = std::move(image_grid_view);
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

  m_image_grid_view.value()->add_rating_listener(
      std::make_unique<npc::RatingClickListener>([this] (int64_t id, int32_t rating) {
          DBG_OUT("rating changed %ld %d", id, rating);
          this->on_rating_changed(id, rating);
      }));

  m_scrollview.set_child(*m_librarylistview);
  m_scrollview.set_policy(Gtk::PolicyType::AUTOMATIC, Gtk::PolicyType::AUTOMATIC);
  m_lib_splitview.set_wide_handle(true);

  // build the toolbar
  auto box = Gtk::manage(new Gtk::Box(Gtk::Orientation::VERTICAL));
  box->append(m_scrollview);
  auto toolbar = ui::image_toolbar_new();
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

Gtk::GridView * GridViewModule::image_list() const
{
    return m_librarylistview;
}

void GridViewModule::on_metadata_changed(const fwk::WrappedPropertyBagPtr& props,
                                         const fwk::WrappedPropertyBagPtr& old)
{
    // TODO this MUST be more generic
    DBG_OUT("on_metadata_changed()");
    m_selection_controller.set_properties(*props, *old);
}

void GridViewModule::on_rating_changed(eng::library_id_t id, int32_t rating) const
{
    m_selection_controller.set_rating_of(id, rating);
}

void GridViewModule::on_librarylistview_click(const Glib::RefPtr<Gtk::GestureClick>& gesture, double x, double y)
{
    auto button = gesture->get_current_button();
    DBG_OUT("GridView click handler, button: %u", button);
    if (button == 3 && !m_librarylistview->get_model()->get_selection()->is_empty()) {
        m_context_menu->set_pointing_to(Gdk::Rectangle(x, y, 1, 1));
        m_context_menu->popup();

        return;
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
