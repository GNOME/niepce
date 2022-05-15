/* -*- mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode:nil; -*- */
/*
 * niepce - ui/dialogs/importer/cameraimporterui.cpp
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

#include <boost/format.hpp>

#include <glibmm/i18n.h>
#include <gtkmm/label.h>
#include <gtkmm/grid.h>
#include <gtkmm/comboboxtext.h>

#include "fwk/base/debug.hpp"
#include "fwk/toolkit/frame.hpp"
#include "engine/importer/cameraimporter.hpp"
#include "cameraimporterui.hpp"

namespace ui {

CameraImporterUI::CameraImporterUI()
    : ImporterUI(std::make_shared<eng::CameraImporter>(), _("Camera"))
    , m_camera_list_combo(nullptr)
    , m_select_camera_btn(nullptr)
{
}

Gtk::Widget* CameraImporterUI::setup_widget(const fwk::Frame::Ptr&)
{
    m_builder = Gtk::Builder::create_from_resource("/org/gnome/Niepce/ui/cameraimporterui.ui",
                                                   "main_widget");
    Gtk::Grid* main_widget = m_builder->get_widget<Gtk::Grid>("main_widget");
    m_select_camera_btn = m_builder->get_widget<Gtk::Button>("select_camera_btn");
    m_select_camera_btn->signal_clicked()
        .connect(sigc::mem_fun(*this, &CameraImporterUI::select_camera));
    m_camera_list_combo = m_builder->get_widget<Gtk::ComboBoxText>("camera_list_combo");

    fwk::GpDeviceList::obj().detect();

    // XXX restore the selection from the preferences.
    int i = 0;
    for (auto device : fwk::GpDeviceList::obj()) {
        m_camera_list_combo->append(device->get_path(), device->get_model());
        if (i == 0) {
            m_camera_list_combo->set_active_id(device->get_path());
        }
        i++;
    }
    if (i == 0) {
        m_camera_list_combo->append("", _("No camera found"));
        m_camera_list_combo->set_active_id("");
        m_camera_list_combo->set_sensitive(false);
        m_select_camera_btn->set_sensitive(false);
    }

    return main_widget;
}

void CameraImporterUI::select_camera()
{
    std::string source = m_camera_list_combo->get_active_id();
    if (!source.empty() && m_source_selected_cb) {
        auto datetime = Glib::DateTime::create_now_local();
        std::string today = datetime.format("%F %H%M%S").raw();
        std::string dest_dir = str(boost::format(_("Camera import %1%")) % today);
        m_source_selected_cb(source, dest_dir);
    }
}

}
