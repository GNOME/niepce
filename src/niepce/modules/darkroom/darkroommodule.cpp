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

DarkroomModule::DarkroomModule(const ui::SelectionController& selection_controller)
    : m_selection_controller(selection_controller)
    , m_dr_splitview(Gtk::Orientation::HORIZONTAL)
    , m_vbox(Gtk::Orientation::VERTICAL)
    , m_image(new ncr::Image)
    , m_active(false)
    , m_need_reload(true)
{
    m_selection_controller.add_selected_listener(
        std::make_unique<npc::SelectionListener>([this] (eng::library_id_t id) {
            this->on_selected(id);
        }));
}

void DarkroomModule::reload_image()
{
    if(!m_need_reload) {
        return;
    }
    if (m_imagefile.has_value()) {
        const eng::LibFilePtr& file = m_imagefile.value();
        // currently we treat RAW + JPEG as RAW.
        // TODO: have a way to actually choose the JPEG.
        auto file_type = file->file_type();
        bool isRaw = (file_type == eng::FileType::Raw)
            || (file_type == eng::FileType::RawJpeg);
        std::string path = std::string(file->path());
        m_image->reload(path, isRaw, file->orientation());
    }
    else {
        // reset
        Glib::RefPtr<Gdk::Pixbuf> p = Gdk::Pixbuf::create_from_resource(
            "/org/gnome/Niepce/pixmaps/niepce-image-generic.png", -1, -1);
        m_image->reload(p);
    }
    m_need_reload = false;
}

void DarkroomModule::set_image(std::optional<eng::LibFilePtr>&& file)
{
    m_imagefile = std::move(file);
    m_need_reload = true;

    if (m_need_reload && m_active) {
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
    m_imagecanvas->set_hexpand(true);
    m_imagecanvas->set_vexpand(true);
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
    auto file = m_selection_controller.get_file(id);
    DBG_OUT("selection is %ld", id);
    if (file) {
        set_image(std::optional(rust::Box<eng::LibFile>::from_raw(file)));
    } else {
        set_image(std::nullopt);
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
