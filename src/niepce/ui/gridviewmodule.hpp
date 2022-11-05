/*
 * niepce - ui/gridviewmodule.hpp
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

#ifndef _IN_RUST_BINDINGS_

#pragma once

#include <optional>

#include <gtkmm/gestureclick.h>
#include <gtkmm/iconview.h>
#include <gtkmm/liststore.h>
#include <gtkmm/paned.h>
#include <gtkmm/popovermenu.h>
#include <gtkmm/scrolledwindow.h>
#include <gtkmm/treestore.h>

#include "fwk/base/propertybag.hpp"
#include "niepce/ui/ilibrarymodule.hpp"
#include "niepce/ui/imageliststore.hpp"
#include "niepce/ui/metadatapanecontroller.hpp"

namespace fwk {
class Dock;
}

namespace npc {
class UIDataProvider;
}

namespace ui {

class GridViewModule
    : public ILibraryModule
{
public:
  typedef std::shared_ptr<GridViewModule> Ptr;

  GridViewModule(const ui::SelectionController& selection_controller,
                 Glib::RefPtr<Gio::Menu> menu, const eng::UIDataProvider& ui_data_provider);
  virtual ~GridViewModule();

  void on_lib_notification(const eng::LibNotification &, const eng::LibraryClientWrapper& client) const;
  void display_none() const;

  /* ILibraryModule */
  virtual void dispatch_action(const std::string & action_name) override;
  virtual void set_active(bool) const override {}
  virtual Glib::RefPtr<Gio::MenuModel> getMenu() override
    { return Glib::RefPtr<Gio::MenuModel>(); }

  /* IImageSelectable */
  virtual Gtk::IconView * image_list() const;
  virtual eng::library_id_t get_selected() const;
  virtual void select_image(eng::library_id_t id) const;

  const GtkIconView* cxx_image_list() const {
    return const_cast<GridViewModule*>(this)->image_list()->gobj();
  }
protected:
  virtual Gtk::Widget * buildWidget() override;

private:
  static bool get_colour_callback_c(int32_t label, ffi::RgbColour* out, const void* user_data);
  std::optional<fwk::RgbColour> get_colour_callback(int32_t label) const;
  void on_metadata_changed(const fwk::WrappedPropertyBagPtr&, const fwk::WrappedPropertyBagPtr& old);
  static void on_rating_changed(GtkCellRenderer*, eng::library_id_t id, int rating,
                                gpointer user_data);
  void on_librarylistview_click(const Glib::RefPtr<Gtk::GestureClick>& gesture, double, double);

  const ui::SelectionController& m_selection_controller;
  Glib::RefPtr<Gio::Menu> m_menu;
  const eng::UIDataProvider& m_ui_data_provider;

  // library split view
  std::optional<rust::Box<npc::ImageGridView>> m_image_grid_view;
  Gtk::IconView* m_librarylistview;
  Gtk::ScrolledWindow          m_scrollview;
  MetaDataPaneController::Ptr  m_metapanecontroller;
  Gtk::Paned                   m_lib_splitview;
  fwk::Dock                   *m_dock;
  Gtk::PopoverMenu* m_context_menu;
};

std::shared_ptr<GridViewModule> grid_view_module_new(const ui::SelectionController& selection_controller,
                                                     const GMenu* menu_,
                                                     const eng::UIDataProvider& ui_data_provider);

}

#endif
