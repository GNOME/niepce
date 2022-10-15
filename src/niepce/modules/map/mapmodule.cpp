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
#include "engine/db/libmetadata.hpp"
#include "mapmodule.hpp"

#include "rust_bindings.hpp"

namespace mapm {

MapModule::MapModule(const ui::IModuleShell & shell)
    : m_shell(shell)
    , m_box(nullptr)
    , m_active(false)
{
}

void MapModule::dispatch_action(const std::string & /*action_name*/)
{
}


void MapModule::set_active(bool active)
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
MapModule::on_lib_notification(const eng::LibNotification &ln)
{
    if (!m_active) {
        return;
    }
    switch (engine_library_notification_type(&ln)) {
    case eng::NotificationType::METADATA_QUERIED:
    {
        auto lm = engine_library_notification_get_libmetadata(&ln);
        DBG_OUT("received metadata in MapModule");

        if (lm) {
            fwk::PropertySetPtr propset = fwk::property_set_new();
            ffi::eng_property_set_add(propset.get(), ffi::NiepcePropertyIdx::NpExifGpsLongProp);
            ffi::eng_property_set_add(propset.get(), ffi::NiepcePropertyIdx::NpExifGpsLatProp);

            fwk::PropertyBagPtr properties = eng::libmetadata_to_properties(lm, *propset);
            double latitude, longitude;
            latitude = longitude = NAN;
            auto result = fwk::get_value_for_property(*properties, ffi::NiepcePropertyIdx::NpExifGpsLongProp);
            if (!result.empty()) {
                fwk::PropertyValuePtr val = result.unwrap();
                // it is a string
                if (fwk_property_value_is_string(val.get())) {
                    longitude = fwk::gps_coord_from_xmp(fwk::property_value_get_string(*val));
                }
            }
            result = fwk::get_value_for_property(*properties, ffi::NiepcePropertyIdx::NpExifGpsLatProp);
            if (!result.empty()) {
                fwk::PropertyValuePtr val = result.unwrap();
                // it is a string
                if (fwk_property_value_is_string(val.get())) {
                    latitude = fwk::gps_coord_from_xmp(fwk::property_value_get_string(*val));
                }
            }

            if (!std::isnan(latitude) && !std::isnan(longitude)) {
                m_map->centerOn(latitude, longitude);
            }
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
