/* -*- mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode:nil; -*- */
/*
 * niepce - ui/dialogs/importer/directoryimporterui.cpp
 *
 * Copyright (C) 2017-2022 Hubert Figuière
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
#include <gtkmm/filechooserdialog.h>
#include <gtkmm/label.h>

#include "fwk/utils/pathutils.hpp"
#include "fwk/toolkit/application.hpp"
#include "engine/importer/directoryimporter.hpp"
#include "directoryimporterui.hpp"

namespace ui {

DirectoryImporterUI::DirectoryImporterUI()
    : ImporterUI(std::make_shared<eng::DirectoryImporter>(), _("Directory"))
    , m_directory_name(nullptr)
{
}

Gtk::Widget* DirectoryImporterUI::setup_widget(const fwk::Frame::Ptr& frame)
{
    m_frame = frame;
    m_builder = Gtk::Builder::create_from_resource("/org/gnome/Niepce/ui/directoryimporterui.ui",
                                                   "main_widget");
    Gtk::Box* main_widget = m_builder->get_widget<Gtk::Box>("main_widget");
    Gtk::Button* select_directories = m_builder->get_widget<Gtk::Button>("select_directories");
    select_directories->signal_clicked().connect(
        sigc::mem_fun(*this, &DirectoryImporterUI::do_select_directories));
    m_directory_name = m_builder->get_widget<Gtk::Label>("directory_name");
    return main_widget;
}

void DirectoryImporterUI::do_select_directories()
{
    auto& cfg = fwk::Application::app()->config()->cfg;

    auto frame = m_frame.lock();
    auto dialog = new Gtk::FileChooserDialog(frame->gtkWindow(), _("Import picture folder"),
                                         Gtk::FileChooser::Action::SELECT_FOLDER);

    dialog->add_button(_("Cancel"), Gtk::ResponseType::CANCEL);
    dialog->add_button(_("Select"), Gtk::ResponseType::OK);
    dialog->set_select_multiple(false);

    std::string last_import_location(cfg->getValue("last_import_location", ""));
    if (!last_import_location.empty()) {
        auto file = Gio::File::create_for_path(last_import_location);
        dialog->set_current_folder(file);
    }
    dialog->signal_response().connect([this, dialog] (int result) {
        std::string source;
        switch (result)
        {
        case Gtk::ResponseType::OK:
            source = dialog->get_file()->get_path();
            m_source_selected_cb(source, fwk::path_basename(source));
            break;
        default:
            break;
        }
        m_directory_name->set_text(source);
        delete dialog;
    });
    dialog->show();
}

}
