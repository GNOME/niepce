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
#include <gtkmm/gridview.h>
#include <gtkmm/paned.h>
#include <gtkmm/popovermenu.h>
#include <gtkmm/scrolledwindow.h>

#include "fwk/base/propertybag.hpp"
#include "niepce/ui/ilibrarymodule.hpp"
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
                 Glib::RefPtr<Gio::Menu> menu, const eng::LibraryClientHost& libclient_host);
  virtual ~GridViewModule();

  void on_lib_notification(const eng::LibNotification &, const eng::LibraryClientWrapper& client) const;
  void display_none() const;

  /* ILibraryModule */
  virtual void dispatch_action(const std::string & action_name) override;
  virtual void set_active(bool) const override {}
  virtual Glib::RefPtr<Gio::MenuModel> getMenu() override
    { return Glib::RefPtr<Gio::MenuModel>(); }

  virtual Gtk::GridView * image_list() const;

  const GtkGridView* cxx_image_list() const {
    return const_cast<GridViewModule*>(this)->image_list()->gobj();
  }
protected:
  virtual Gtk::Widget * buildWidget() override;

private:
  void on_metadata_changed(const fwk::WrappedPropertyBagPtr&, const fwk::WrappedPropertyBagPtr& old);
  void on_rating_changed(eng::library_id_t id, int rating) const;
  void on_librarylistview_click(const Glib::RefPtr<Gtk::GestureClick>& gesture, double, double);

  const ui::SelectionController& m_selection_controller;
  Glib::RefPtr<Gio::Menu> m_menu;
  const eng::LibraryClientHost& m_libclient_host;

  // library split view
  std::optional<rust::Box<npc::ImageGridView>> m_image_grid_view;
  Gtk::GridView* m_librarylistview;
  Gtk::ScrolledWindow          m_scrollview;
  MetaDataPaneController::Ptr  m_metapanecontroller;
  Gtk::Paned                   m_lib_splitview;
  fwk::Dock                   *m_dock;
  Gtk::PopoverMenu* m_context_menu;
};

std::shared_ptr<GridViewModule> grid_view_module_new(const ui::SelectionController& selection_controller,
                                                     const GMenu* menu_,
                                                     const eng::LibraryClientHost& libclient_host);

}

#endif
