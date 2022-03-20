/*
 * niepce - modules/darkroom/darkroommodule.cpp
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

#include <gdkmm/pixbuf.h>

#include "rust_bindings.hpp"

#include "fwk/base/debug.hpp"
#include "fwk/toolkit/application.hpp"
#include "fwk/toolkit/configdatabinder.hpp"
#include "fwk/toolkit/widgets/dock.hpp"
#include "ncr/init.hpp"
#include "darkroommodule.hpp"

namespace dr {

DarkroomModule::DarkroomModule(const ui::IModuleShell & shell)
    : m_shell(shell)
    , m_dr_splitview(Gtk::Orientation::HORIZONTAL)
    , m_vbox(Gtk::Orientation::VERTICAL)
    , m_image(new ncr::Image)
    , m_active(false)
    , m_need_reload(true)
{
    m_shell.get_selection_controller()->signal_selected.connect([this] (eng::library_id_t id) {
        this->on_selected(id);
    });
}

void DarkroomModule::reload_image()
{
    if(!m_need_reload) {
        return;
    }
    eng::LibFilePtr file = m_imagefile.lock();
    if(file) {
        // currently we treat RAW + JPEG as RAW.
        // TODO: have a way to actually choose the JPEG.
        auto file_type = engine_db_libfile_file_type(file.get());
        bool isRaw = (file_type == eng::FileType::Raw)
            || (file_type == eng::FileType::RawJpeg);
        std::string path = engine_db_libfile_path(file.get());
        m_image->reload(path, isRaw, engine_db_libfile_orientation(file.get()));
    }
    else {
        // reset
        Glib::RefPtr<Gdk::Pixbuf> p = Gdk::Pixbuf::create_from_resource(
            "/org/gnome/Niepce/pixmaps/niepce-image-generic.png", -1, -1);
        m_image->reload(p);
    }
    m_need_reload = false;
}

void DarkroomModule::set_image(const eng::LibFilePtr & file)
{
    if(m_imagefile.expired() || (file != m_imagefile.lock())) {
        m_imagefile = eng::LibFileWeakPtr(file);
        m_need_reload = true;
    }
    else if(!static_cast<bool>(file)) {
        m_imagefile.reset();
        m_need_reload = true;
    }

    if(m_need_reload && m_active) {
        reload_image();
    }
}

void DarkroomModule::dispatch_action(const std::string & /*action_name*/)
{
}


void DarkroomModule::set_active(bool active)
{
    m_active = active;
    if(active) {
        // if activated, force the refresh of the image.
        reload_image();
    }
}


Gtk::Widget * DarkroomModule::buildWidget()
{
    if(m_widget) {
        return m_widget;
    }
    ncr::init();
    m_widget = &m_dr_splitview;
    m_imagecanvas = Gtk::manage(new ImageCanvas());
// TODO set a proper canvas size
//    m_canvas_scroll.add(*m_imagecanvas);
    m_vbox.append(*m_imagecanvas);

    m_imagecanvas->set_image(m_image);

    // build the toolbar.
    auto toolbar = ffi::image_toolbar_new();
    gtk_box_append(m_vbox.gobj(), GTK_WIDGET(toolbar));

    m_dr_splitview.set_start_child(m_vbox);
    m_dock = Gtk::manage(new fwk::Dock());
    m_dr_splitview.set_end_child(*m_dock);

    m_databinders.add_binder(new fwk::ConfigDataBinder<int>(
                                 m_dr_splitview.property_position(),
                                 fwk::Application::app()->config(),
                                 "dr_toolbox_pane_splitter"));

    m_toolbox_ctrl = ToolboxController::Ptr(new ToolboxController);
    add(m_toolbox_ctrl);
    m_dock->vbox().append(*m_toolbox_ctrl->buildWidget());

    return m_widget;
}

void DarkroomModule::on_selected(eng::library_id_t id)
{
    auto file = m_shell.get_selection_controller()->get_file(id);
    DBG_OUT("selection is %ld", id);
    set_image(file);
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
