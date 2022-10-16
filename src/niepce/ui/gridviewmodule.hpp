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
#include "niepce/ui/imoduleshell.hpp"
#include "niepce/ui/metadatapanecontroller.hpp"
#include "niepce/ui/selectioncontroller.hpp"

#include "rust_bindings.hpp"

namespace fwk {
class Dock;
}

namespace libraryclient {
class UIDataProvider;
}

namespace ui {

class ModuleShell;

class GridViewModule
    : public ILibraryModule
    , public IImageSelectable
{
public:
  typedef std::shared_ptr<GridViewModule> Ptr;

  GridViewModule(const IModuleShell & shell,
                 const ImageListStorePtr& store);
  virtual ~GridViewModule();

  void on_lib_notification(const eng::LibNotification &);
  void display_none();

  /* ILibraryModule */
  virtual void dispatch_action(const std::string & action_name) override;
  virtual void set_active(bool active) override;
  virtual Glib::RefPtr<Gio::MenuModel> getMenu() override
    { return Glib::RefPtr<Gio::MenuModel>(); }

  /* IImageSelectable */
  virtual Gtk::IconView * image_list() override;
  virtual eng::library_id_t get_selected() override;
  virtual void select_image(eng::library_id_t id) override;

protected:
  virtual Gtk::Widget * buildWidget() override;

private:
  static bool get_colour_callback_c(int32_t label, ffi::RgbColour* out, const void* user_data);
  std::optional<fwk::RgbColourPtr> get_colour_callback(int32_t label) const;
  void on_metadata_changed(const fwk::WrappedPropertyBagPtr&, const fwk::WrappedPropertyBagPtr& old);
  static void on_rating_changed(GtkCellRenderer*, eng::library_id_t id, int rating,
                                gpointer user_data);
  void on_librarylistview_click(const Glib::RefPtr<Gtk::GestureClick>& gesture, double, double);

  const IModuleShell &               m_shell;
  ImageListStorePtr m_model;

  // library split view
  std::shared_ptr<ffi::ImageGridView> m_image_grid_view;
  Gtk::IconView* m_librarylistview;
  Gtk::ScrolledWindow          m_scrollview;
  MetaDataPaneController::Ptr  m_metapanecontroller;
  Gtk::Paned                   m_lib_splitview;
  fwk::Dock                   *m_dock;
  Gtk::PopoverMenu* m_context_menu;
};

}
