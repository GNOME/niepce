/*
 * niepce - niepce/ui/metadatapanecontroller.h
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

#pragma once

#include <gtkmm/box.h>

#include "engine/db/libmetadata.hpp"
#include "fwk/base/propertybag.hpp"
#include "fwk/utils/exempi.hpp"
#include "fwk/toolkit/dockable.hpp"

namespace fwk {
struct MetaDataSectionFormat;
class MetaDataWidget;
class Dock;
}

namespace ui {

class MetaDataPaneController
    : public fwk::Dockable
{
public:
    typedef std::shared_ptr<MetaDataPaneController> Ptr;
    MetaDataPaneController();
    ~MetaDataPaneController();
    virtual Gtk::Widget * buildWidget() override;
    void display(eng::library_id_t file_id, const eng::LibMetadata* meta);
    eng::library_id_t displayed_file() const
        { return m_fileid; }

    sigc::signal<void(const fwk::PropertyBagPtr &, const fwk::PropertyBagPtr &)> signal_metadata_changed;
private:
    void on_metadata_changed(const fwk::PropertyBagPtr &,
                             const fwk::PropertyBagPtr & old);

    std::vector<fwk::MetaDataWidget *> m_widgets;

    static const std::vector<fwk::MetaDataSectionFormat>& get_format();
    static const fwk::PropertySet* get_property_set();

    eng::library_id_t m_fileid;
};

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
