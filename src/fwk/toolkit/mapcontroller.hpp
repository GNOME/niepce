/*
 * niepce - fwk/toolkit/mapcontroller.hpp
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

#pragma once

#include "fwk/toolkit/uicontroller.hpp"

namespace fwk {

class MapController
  : public UiController
{
public:
    typedef std::shared_ptr<MapController> Ptr;

    MapController();
    virtual ~MapController();

    virtual Gtk::Widget * buildWidget() override;

    void centerOn(double lat, double longitude);

    void zoomIn();
    void zoomOut();
    void setZoomLevel(double level); // 1 to 20

private:
    class Priv;
    Priv* m_priv;
};


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
