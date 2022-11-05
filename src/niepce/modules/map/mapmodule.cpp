/*
 * niepce - modules/map/mapmodule.cpp
 *
 * Copyright (C) 2014-2022 Hubert Figuière
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

#include <gtkmm/box.h>

#include "fwk/base/debug.hpp"
#include "fwk/utils/exempi.hpp"
#include "fwk/toolkit/application.hpp"
#include "engine/db/properties.hpp"
#include "mapmodule.hpp"

#include "rust_bindings.hpp"

namespace mapm {

MapModule::MapModule()
    : m_box(nullptr)
    , m_active(false)
{
}

void MapModule::dispatch_action(const std::string & /*action_name*/)
{
}


void MapModule::set_active(bool active) const
{
    m_active = active;
}


Gtk::Widget * MapModule::buildWidget()
{
    if(m_widget) {
        return m_widget;
    }

    m_box = Gtk::manage(new Gtk::Box(Gtk::Orientation::VERTICAL));
    m_widget = m_box;

    m_map = fwk::MapController::Ptr(new fwk::MapController());
    add(m_map);
    auto w = m_map->buildWidget();
    if (w) {
        m_box->append(*w);
    }

    return m_widget;
}

void
MapModule::on_lib_notification(const eng::LibNotification &ln) const
{
    if (!m_active) {
        return;
    }
    switch (ln.type_()) {
    case eng::NotificationType::METADATA_QUERIED:
    {
        auto& lm = ln.get_libmetadata();
        DBG_OUT("received metadata in MapModule");

        fwk::PropertySetPtr propset = fwk::PropertySet_new();
        propset->add((uint32_t)eng::NiepcePropertyIdx::NpExifGpsLongProp);
        propset->add((uint32_t)eng::NiepcePropertyIdx::NpExifGpsLatProp);

        fwk::PropertyBagPtr properties = lm.to_properties(*propset);
        double latitude, longitude;
        latitude = longitude = NAN;
        if (properties->contains_key((uint32_t)eng::NiepcePropertyIdx::NpExifGpsLongProp)) {
            const fwk::PropertyValue& val = properties->value((uint32_t)eng::NiepcePropertyIdx::NpExifGpsLongProp);
            // it is a string
            if (val.is_string()) {
                longitude = fwk::gps_coord_from_xmp(val.get_string());
            }
        }
        if (properties->contains_key((uint32_t)eng::NiepcePropertyIdx::NpExifGpsLatProp)) {
            const fwk::PropertyValue& val = properties->value((uint32_t)eng::NiepcePropertyIdx::NpExifGpsLatProp);
            // it is a string
            if (val.is_string()) {
                latitude = fwk::gps_coord_from_xmp(val.get_string());
            }
        }

        if (!std::isnan(latitude) && !std::isnan(longitude)) {
            m_map->centerOn(latitude, longitude);
        }
        break;
    }
    default:
        break;
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
