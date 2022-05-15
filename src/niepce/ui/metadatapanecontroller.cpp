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
#include "fwk/toolkit/metadatawidget.hpp"
#include "engine/db/properties.hpp"
#include "engine/db/libmetadata.hpp"
#include "metadatapanecontroller.hpp"

namespace ui {

using ffi::NiepcePropertyIdx;

const std::vector<fwk::MetaDataSectionFormat>&
MetaDataPaneController::get_format()
{
    static const std::vector<fwk::MetaDataSectionFormat> s_format = {
        { _("File Information"),
          {
              { _("File Name:"), NiepcePropertyIdx::NpFileNameProp, fwk::MetaDT::STRING, true },
              { _("Folder:"), NiepcePropertyIdx::NpFolderProp, fwk::MetaDT::STRING, true },
              { _("File Type:"), NiepcePropertyIdx::NpFileTypeProp, fwk::MetaDT::STRING, true },
              { _("File Size:"), NiepcePropertyIdx::NpFileSizeProp, fwk::MetaDT::SIZE, true },
              { _("Sidecar Files:"), NiepcePropertyIdx::NpSidecarsProp, fwk::MetaDT::STRING_ARRAY, true },
          }
        },
        { _("Camera Information"),
          {
              { _("Make:"), NiepcePropertyIdx::NpTiffMakeProp, fwk::MetaDT::STRING, true },
              { _("Model:"), NiepcePropertyIdx::NpTiffModelProp, fwk::MetaDT::STRING, true },
              { _("Lens:"), NiepcePropertyIdx::NpExifAuxLensProp, fwk::MetaDT::STRING, true },
          }
        },
        { _("Shooting Information"),
          {
              { _("Exposure Program:"), NiepcePropertyIdx::NpExifExposureProgramProp, fwk::MetaDT::STRING, true },
              { _("Speed:"), NiepcePropertyIdx::NpExifExposureTimeProp, fwk::MetaDT::FRAC, true },
              { _("Aperture:"), NiepcePropertyIdx::NpExifFNumberPropProp, fwk::MetaDT::FRAC_DEC, true },
              { _("ISO:"), NiepcePropertyIdx::NpExifIsoSpeedRatingsProp, fwk::MetaDT::STRING, true },
              { _("Exposure Bias:"), NiepcePropertyIdx::NpExifExposureBiasProp, fwk::MetaDT::FRAC_DEC, true },
              { _("Flash:"), NiepcePropertyIdx::NpExifFlashFiredProp, fwk::MetaDT::STRING, true },
              { _("Flash compensation:"), NiepcePropertyIdx::NpExifAuxFlashCompensationProp, fwk::MetaDT::STRING, true },
              { _("Focal length:"), NiepcePropertyIdx::NpExifFocalLengthProp, fwk::MetaDT::FRAC_DEC, true },
              { _("White balance:"), NiepcePropertyIdx::NpExifWbProp, fwk::MetaDT::STRING, true },
              { _("Date:"), NiepcePropertyIdx::NpExifDateTimeOriginalProp, fwk::MetaDT::DATE, false },
          }
        },
        { _("IPTC"),
          {
              { _("Headline:"), NiepcePropertyIdx::NpIptcHeadlineProp, fwk::MetaDT::STRING, false },
              { _("Caption:"), NiepcePropertyIdx::NpIptcDescriptionProp, fwk::MetaDT::TEXT, false },
              { _("Rating:"), NiepcePropertyIdx::NpXmpRatingProp, fwk::MetaDT::STAR_RATING, false },
              // FIXME change this type to the right one when there is a widget
              { _("Label:"), NiepcePropertyIdx::NpXmpLabelProp, fwk::MetaDT::STRING, true },
              { _("Keywords:"), NiepcePropertyIdx::NpIptcKeywordsProp, fwk::MetaDT::STRING_ARRAY, false },
          }
        },
        { _("Rights"),
          std::vector<fwk::MetaDataFormat>()
        },
    };
    return s_format;
}

const fwk::PropertySet* MetaDataPaneController::get_property_set()
{
    static fwk::PropertySet* propset = nullptr;
    if(!propset) {
        propset = ffi::eng_property_set_new();
        const std::vector<fwk::MetaDataSectionFormat>& formats = get_format();

        auto current = formats.begin();
        while (current != formats.end()) {
            auto format = current->formats.begin();
            while (format != current->formats.end()) {
                ffi::eng_property_set_add(propset, format->id);
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

    const auto& formats = get_format();

    auto current = formats.begin();
    while (current != formats.end()) {
        auto w = Gtk::manage(new fwk::MetaDataWidget(current->section));
        box->append(*w);
        w->set_data_format(&*current);
        m_widgets.push_back(w);
        w->signal_metadata_changed.connect(
            sigc::mem_fun(*this,
                          &MetaDataPaneController::on_metadata_changed));
        current++;
    }

    return m_widget;
}

void MetaDataPaneController::on_metadata_changed(const fwk::PropertyBagPtr& props,
                                                 const fwk::PropertyBagPtr& old)
{
    signal_metadata_changed.emit(props, old);
}

void MetaDataPaneController::display(eng::library_id_t file_id, const eng::LibMetadata* meta)
{
    m_fileid = file_id;
    DBG_OUT("displaying metadata");
    fwk::PropertyBagPtr properties;
    if(meta) {
        const fwk::PropertySet* propset = get_property_set();
        properties = eng::libmetadata_to_properties(meta, *propset);
    }
    std::for_each(m_widgets.begin(), m_widgets.end(),
                  [properties] (decltype(m_widgets)::value_type w) {
                      w->set_data_source(properties);
                  });
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
