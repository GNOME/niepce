/*
 * niepce - niepce/ui/dialogs/editlabels.hpp
 *
 * Copyright (C) 2009-2022 Hubert Figuière
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

#include <array>

#include <gtkmm/colorbutton.h>
#include <gtkmm/entry.h>
#include <gtkmm/label.h>

#include "libraryclient/libraryclient.hpp"
#include "fwk/toolkit/dialog.hpp"

namespace ui {


class EditLabels
    : public fwk::Dialog
{
public:
    typedef std::shared_ptr<EditLabels> Ptr;
    EditLabels(const libraryclient::LibraryClientPtr &);

    virtual void setup_widget() override;

    constexpr static int NUM_LABELS = 5;
private:
    void label_name_changed(size_t idx);
    void label_colour_changed(size_t idx);
    void update_labels(int /*response*/);
    eng::LabelList m_labels;
    std::array<Gtk::ColorButton*, NUM_LABELS> m_colours;
    std::array<Gtk::Entry*, NUM_LABELS> m_entries;
    std::array<bool, NUM_LABELS> m_status;
    libraryclient::LibraryClientPtr m_lib_client;
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
