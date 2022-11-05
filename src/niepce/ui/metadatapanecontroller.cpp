/*
 * niepce - niepce/ui/metadatapanecontroller.cpp
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

#include <vector>

#include <glibmm/i18n.h>
#include <gtkmm/label.h>
#include <gtkmm/entry.h>

#include "fwk/base/debug.hpp"
#include "engine/db/properties.hpp"
#include "metadatapanecontroller.hpp"

#include "rust_bindings.hpp"

namespace ui {

using eng::NiepcePropertyIdx;

const fwk::PropertySet* MetaDataPaneController::get_property_set()
{
    static fwk::PropertySet* propset = nullptr;
    if(!propset) {
        propset = fwk::PropertySet_new().into_raw();
        rust::Slice<const fwk::MetadataSectionFormat> formats = npc::get_format();

        auto current = formats.begin();
        while (current != formats.end()) {
            auto format = current->formats.begin();
            while (format != current->formats.end()) {
                propset->add(format->id);
                format++;
            }
            current++;
        }
    }
    return propset;
}

MetaDataPaneController::MetaDataPaneController()
    : Dockable("Metadata", _("Image Properties"),
	       "document-properties" /*, DockItem::DOCKED_STATE*/),
      m_fileid(0)
{
}

MetaDataPaneController::~MetaDataPaneController()
{
    for (const auto& w : m_widgets) {
        auto w_ptr = reinterpret_cast<GtkWidget*>(w.first->gobj());
        g_signal_handler_disconnect(w_ptr, w.second);
    }
}

void
MetaDataPaneController::metadata_changed_cb(GtkWidget*, const fwk::WrappedPropertyBag* props,
                                            const fwk::WrappedPropertyBag* old_props,
                                            MetaDataPaneController* self)
{
    self->on_metadata_changed(
        fwk::wrapped_property_bag_wrap(
            fwk::wrapped_property_bag_clone(*props)),
        fwk::wrapped_property_bag_wrap(
            fwk::wrapped_property_bag_clone(*old_props)));
}

Gtk::Widget *
MetaDataPaneController::buildWidget()
{
    if(m_widget) {
        return m_widget;
    }
    auto box = build_vbox();
    m_widget = box;
    DBG_ASSERT(box, "dockable vbox not found");

    const auto& formats = npc::get_format();

    auto current = formats.begin();
    while (current != formats.end()) {
        auto w = fwk::MetadataWidget_new(current->section);
        auto w_ptr = reinterpret_cast<GtkWidget*>(w->gobj());
        DBG_ASSERT(w_ptr, "MetadataWidget is null");
        gtk_box_append(box->gobj(), w_ptr);
        w->set_data_format(*current);
        auto handler = g_signal_connect(w_ptr, "metadata-changed",
                         G_CALLBACK(MetaDataPaneController::metadata_changed_cb),
                         this);
        m_widgets.push_back(std::make_pair(std::move(w), handler));

        current++;
    }

    return m_widget;
}

void MetaDataPaneController::on_metadata_changed(const fwk::WrappedPropertyBagPtr& props,
                                                 const fwk::WrappedPropertyBagPtr& old)
{
    signal_metadata_changed.emit(props, old);
}

void MetaDataPaneController::display(eng::library_id_t file_id, const eng::LibMetadata* meta)
{
    m_fileid = file_id;
    DBG_OUT("displaying metadata");
    if (meta) {
        const fwk::PropertySet* propset = get_property_set();
        auto properties = rust::Box<fwk::WrappedPropertyBag>::from_raw(meta->to_wrapped_properties(*propset));
        for (const auto& w : m_widgets) {
            w.first->set_data_source(*properties);
        }
    } else {
        for (const auto& w : m_widgets) {
            w.first->set_data_source_none();
        }
    }
}

}

// Required because this is implemented as an extern C++ type
// So we have to specialise.
namespace rust{
namespace cxxbridge1 {
template <>
void Box<::fwk::WrappedPropertyBag>::drop() noexcept {
    fwk::wrapped_property_bag_drop(this->ptr);
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
