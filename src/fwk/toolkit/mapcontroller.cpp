/*
 * niepce - fwk/toolkit/mapcontroller.cpp
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

#include "mapcontroller.hpp"

#include <gtkmm/widget.h>
#include <gtkmm/label.h>

#include <shumate/shumate.h>

namespace fwk {

class MapController::Priv {
public:
    Priv()
        : m_map(nullptr)
        , m_registry(nullptr)
        {
        }
    ~Priv()
        {
            if (m_map) {
                g_object_unref(G_OBJECT(m_map));
            }
            if (m_registry) {
                g_object_unref(G_OBJECT(m_registry));
            }
        }
    void create_widget()
        {
            m_map = SHUMATE_SIMPLE_MAP(shumate_simple_map_new());
            m_registry = shumate_map_source_registry_new_with_defaults();
            shumate_simple_map_set_map_source(
                SHUMATE_SIMPLE_MAP(m_map),
                SHUMATE_MAP_SOURCE(g_list_model_get_item(G_LIST_MODEL(m_registry), 0)));
            g_object_ref(m_map);
        }
    ShumateSimpleMap* m_map;
    ShumateMapSourceRegistry* m_registry;
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

Gtk::Widget* MapController::buildWidget()
{
    if (m_widget) {
        return m_widget;
    }

    m_priv->create_widget();

    m_widget = Gtk::manage(Glib::wrap(GTK_WIDGET(m_priv->m_map)));
    m_widget->set_vexpand(true);

    // Default position. Somewhere over Montréal, QC
    setZoomLevel(10.0);
    centerOn(45.5030854, -73.5698944);

    return m_widget;
}

void MapController::centerOn(double lat, double longitude)
{
    ShumateViewport* viewport =
        shumate_simple_map_get_viewport(m_priv->m_map);
    shumate_location_set_location(SHUMATE_LOCATION(viewport), lat, longitude);
}

void MapController::zoomIn()
{
    ShumateMap* map = shumate_simple_map_get_map(m_priv->m_map);
    shumate_map_zoom_in(map);
}

void MapController::zoomOut()
{
    ShumateMap* map = shumate_simple_map_get_map(m_priv->m_map);
    shumate_map_zoom_in(map);
}

void MapController::setZoomLevel(double level)
{
    ShumateViewport* viewport =
        shumate_simple_map_get_viewport(m_priv->m_map);
    shumate_viewport_set_zoom_level(viewport, level);
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
