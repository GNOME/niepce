/*
 * niepce - niepce/ui/moduleshell.cpp
 *
 * Copyright (C) 2007-2018 Hubert Figuière
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
#include <glibmm/ustring.h>

#include <gtkmm/celllayout.h>
#include <gtkmm/cellrenderer.h>

#include "fwk/base/debug.hpp"
#include "engine/db/libfile.hpp"
#include "fwk/toolkit/application.hpp"
#include "fwk/toolkit/gtkutils.hpp"
#include "moduleshell.hpp"
#include "niepcewindow.hpp"
#include "metadatapanecontroller.hpp"

namespace ui {

ModuleShell::~ModuleShell()
{
    m_widget = nullptr;
}

Gtk::Widget * ModuleShell::buildWidget()
{
    if(m_widget) {
        return m_widget;
    }

    m_widget = &m_shell;
    m_shell.insert_action_group("shell", m_actionGroup);

    m_selection_controller = SelectionController::Ptr(new SelectionController);
    add(m_selection_controller);

    m_menu = Gio::Menu::create();

    // "go-previous"
    fwk::add_menu_action(m_actionGroup.get(), "PrevImage",
                         sigc::mem_fun(*m_selection_controller,
                                       &SelectionController::select_previous),
                         m_menu, _("Back"), "shell", "Left");

    // "go-next"
    fwk::add_menu_action(m_actionGroup.get(), "NextImage",
                         sigc::mem_fun(*m_selection_controller,
                                  &SelectionController::select_next),
                         m_menu, _("Forward"), "shell", "Right");

    auto section = Gio::Menu::create();
    m_menu->append_section(section);

    // "object-rotate-left"
    fwk::add_menu_action(m_actionGroup.get(), "RotateLeft",
                         sigc::bind(
                             sigc::mem_fun(*m_selection_controller,
                                           &SelectionController::rotate), -90),
                         section, _("Rotate Left"), "shell", "bracketleft");

    // "object-rotate-right"
    fwk::add_menu_action(m_actionGroup.get(), "RotateRight",
                         sigc::bind(
                             sigc::mem_fun(*m_selection_controller,
                                           &SelectionController::rotate), 90),
                         section, _("Rotate Right"), "shell", "bracketright");

    section = Gio::Menu::create();
    m_menu->append_section(section);

    auto submenu = Gio::Menu::create();
    section->append_submenu(_("Set Label"), submenu);

    fwk::add_menu_action(m_actionGroup.get(),
                         "SetLabel6",
                         sigc::bind(sigc::mem_fun(*m_selection_controller,
                                                  &SelectionController::set_label),
                                    1),
                         submenu, _("Label 6"), "shell", "<Primary>6");
    fwk::add_menu_action(m_actionGroup.get(),
                         "SetLabel7",
                         sigc::bind(sigc::mem_fun(*m_selection_controller,
                                                  &SelectionController::set_label),
                                    2),
                         submenu, _("Label 7"), "shell", "<Primary>7");
    fwk::add_menu_action(m_actionGroup.get(),
                         "SetLabel8",
                         sigc::bind(sigc::mem_fun(*m_selection_controller,
                                                  &SelectionController::set_label),
                                    3),
                         submenu, _("Label 8"), "shell", "<Primary>8");
    fwk::add_menu_action(m_actionGroup.get(),
                         "SetLabel9",
                         sigc::bind(sigc::mem_fun(*m_selection_controller,
                                                  &SelectionController::set_label),
                                    4),
                         submenu, _("Label 9"), "shell", "<Primary>9");

    submenu = Gio::Menu::create();
    section->append_submenu(_("Set Rating"), submenu);

    fwk::add_menu_action(m_actionGroup.get(),
                         "SetRating0",
                         sigc::bind(sigc::mem_fun(*m_selection_controller,
                                                  &SelectionController::set_rating),
                                    0),
                         submenu, _("Unrated"), "shell", "<Primary>0");
    fwk::add_menu_action(m_actionGroup.get(),
                         "SetRating1",
                         sigc::bind(sigc::mem_fun(*m_selection_controller,
                                             &SelectionController::set_rating),
                                    1),
                         submenu, _("Rating 1"), "shell", "<Primary>1");
    fwk::add_menu_action(m_actionGroup.get(),
                         "SetRating2",
                         sigc::bind(sigc::mem_fun(*m_selection_controller,
                                                  &SelectionController::set_rating),
                                    2),
                         submenu, _("Rating 2"), "shell", "<Primary>2");
    fwk::add_menu_action(m_actionGroup.get(),
                         "SetRating3",
                         sigc::bind(sigc::mem_fun(*m_selection_controller,
                                                  &SelectionController::set_rating),
                                    3),
                         submenu, _("Rating 3"), "shell", "<Primary>3");
    fwk::add_menu_action(m_actionGroup.get(),
                         "SetRating4",
                         sigc::bind(sigc::mem_fun(*m_selection_controller,
                                                  &SelectionController::set_rating),
                                    4),
                         submenu, _("Rating 4"), "shell", "<Primary>4");
    fwk::add_menu_action(m_actionGroup.get(),
                         "SetRating5",
                         sigc::bind(sigc::mem_fun(*m_selection_controller,
                                                  &SelectionController::set_rating),
                                    5),
                         submenu, _("Rating 5"), "shell", "<Primary>5");

    submenu = Gio::Menu::create();
    section->append_submenu(_("Set Flag"), submenu);

    fwk::add_menu_action(m_actionGroup.get(),
                         "SetFlagReject",
                         sigc::bind(
                             sigc::mem_fun(*m_selection_controller,
                                           &SelectionController::set_flag),
                             -1),
                         submenu, _("Flag as Rejected"), "shell", "<Primary><Shift>x");
    fwk::add_menu_action(m_actionGroup.get(),
                         "SetFlagNone",
                         sigc::bind(
                             sigc::mem_fun(*m_selection_controller,
                                           &SelectionController::set_flag),
                             0),
                         submenu, _("Unflagged"), "shell", "<Primary><Shift>u");
    fwk::add_menu_action(m_actionGroup.get(),
                          "SetFlagPick",
                         sigc::bind(
                             sigc::mem_fun(*m_selection_controller,
                                           &SelectionController::set_flag),
                             1),
                         submenu, _("Flag as Pick"), "shell", "<Primary><Shift>p");

    section = Gio::Menu::create();
    m_menu->append_section(section);

    fwk::add_menu_action(m_actionGroup.get(),
                         "WriteMetadata",
                         sigc::mem_fun(*m_selection_controller,
                                       &SelectionController::write_metadata),
                         section, _("Write metadata"), "shell", nullptr);

    // Module menu placeholder
    m_module_menu = Gio::Menu::create();
    m_menu->append_section(m_module_menu);

    m_shell.getMenuButton().set_menu_model(m_menu);

    m_gridview = GridViewModule::Ptr(
        new GridViewModule(*this, m_selection_controller->get_list_store()));
    add_library_module(m_gridview, "grid", _("Catalog"));

    m_selection_controller->add_selectable(m_gridview);
    m_selection_controller->signal_selected
        .connect(sigc::mem_fun(*this, &ModuleShell::on_image_selected));
    m_selection_controller->signal_activated
        .connect(sigc::mem_fun(*this, &ModuleShell::on_image_activated));

    m_darkroom = dr::DarkroomModule::Ptr(new dr::DarkroomModule(*this));
    add_library_module(m_darkroom, "darkroom", _("Darkroom"));

    m_mapm = mapm::MapModule::Ptr(new mapm::MapModule(*this));
    add_library_module(m_mapm, "map", _("Map"));

    m_shell.signal_activated.connect(sigc::mem_fun(*this, &ModuleShell::on_module_activated));
    m_shell.signal_deactivated.connect(sigc::mem_fun(*this, &ModuleShell::on_module_deactivated));

    // TODO PrintModuleController
    // add_library_module(, _("Print"));
    return m_widget;
}

void ModuleShell::action_edit_delete()
{
    DBG_OUT("shell - delete");
    m_selection_controller->move_to_trash();
}

void ModuleShell::add_library_module(const ILibraryModule::Ptr & module,
                                     const std::string & name,
                                     const std::string & label)
{
    auto w = module->buildWidget();
    if(w) {
        add(module);
        m_shell.appendPage(*w, name, label);
        m_modules.insert(std::make_pair(name, module));
    }
}

void ModuleShell::on_ready()
{
}

void ModuleShell::on_content_will_change()
{
    m_selection_controller->content_will_change();
}

void ModuleShell::on_image_selected(eng::library_id_t id)
{
    DBG_OUT("selected callback %Ld", (long long)id);
    if(id > 0) {
        ffi::libraryclient_request_metadata(m_libraryclient->client(), id);
    }
    else  {
        m_gridview->display_none();
    }
}

void ModuleShell::on_image_activated(eng::library_id_t id)
{
    DBG_OUT("on image activated %Ld", (long long)id);
    auto store = m_selection_controller->get_list_store();
    auto libfile = store->get_file(id);
    if (libfile) {
        m_darkroom->set_image(libfile);
        m_shell.activatePage("darkroom");
    }
}

void ModuleShell::on_module_deactivated(const std::string & name) const
{
    auto module = m_modules.find(name);
    if (module != m_modules.end()) {
        m_module_menu->remove_all();
        module->second->set_active(false);
    }
}

void ModuleShell::on_module_activated(const std::string & name) const
{
    auto module = m_modules.find(name);
    if (module != m_modules.end()) {
        auto menu = module->second->getMenu();
        if (menu) {
            m_module_menu->append_section(menu);
        }
        module->second->set_active(true);
    }
}


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
