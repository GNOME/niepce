/*
 * niepce - fwk/toolkit/mapcontroller.cpp
 *
 * Copyright (C) 2014-2019 Hubert Figuière
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

#include "mapcontroller.hpp"

#include <gtkmm/widget.h>
#include <osm-gps-map.h>

namespace fwk {

class MapController::Priv {
public:
    Priv()
        : m_map(nullptr)
        {
        }
    ~Priv()
        {
            if (m_map) {
                g_object_unref(G_OBJECT(m_map));
            }
        }
    void create_widget()
        {
            m_map = OSM_GPS_MAP(osm_gps_map_new());
            g_object_ref(m_map);

            OsmGpsMapLayer* osd = OSM_GPS_MAP_LAYER(
                g_object_new (OSM_TYPE_GPS_MAP_OSD,
                              "show-scale", TRUE,
                              "show-coordinates", TRUE,
                              "show-crosshair", TRUE,
                              "show-dpad", TRUE,
                              "show-zoom", TRUE,
                              "show-gps-in-dpad", TRUE,
                              "show-gps-in-zoom", FALSE,
                              "dpad-radius", 30,
                              nullptr));
            osm_gps_map_layer_add(OSM_GPS_MAP(m_map), osd);
            g_object_unref(G_OBJECT(osd));
        }
    OsmGpsMap* m_map;
};

MapController::MapController()
    : UiController()
    , m_priv(new Priv)
{
}

MapController::~MapController()
{
    delete m_priv;
}

Gtk::Widget *
MapController::buildWidget()
{
    if (m_widget) {
        return m_widget;
    }

    m_priv->create_widget();

    m_widget = Gtk::manage(Glib::wrap(GTK_WIDGET(m_priv->m_map)));

    // Default position. Somewhere over Montréal, QC
    setZoomLevel(10);
    centerOn(45.5030854, -73.5698944);

    return m_widget;
}

void MapController::centerOn(double lat, double longitude)
{
    osm_gps_map_set_center(m_priv->m_map, lat, longitude);
}

void MapController::zoomIn()
{
    osm_gps_map_zoom_in(m_priv->m_map);
}

void MapController::zoomOut()
{
    osm_gps_map_zoom_out(m_priv->m_map);
}

void MapController::setZoomLevel(uint8_t level)
{
    osm_gps_map_set_zoom(m_priv->m_map, level);
}

}
/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  c-basic-offset:4
  tab-width:4
  indent-tabs-mode:nil
  fill-column:99
  End:
*/
