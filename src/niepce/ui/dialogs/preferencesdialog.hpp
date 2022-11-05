/*
 * niepce - ui/dialogs/preferencesdialog.hpp
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

#include <memory>

#include "fwk/toolkit/gtkutils.hpp"
#include "fwk/toolkit/dialog.hpp"

namespace ui {

class PreferencesDialog
	: public fwk::Dialog
{
public:
    PreferencesDialog()
        : fwk::Dialog("/org/gnome/Niepce/ui/preferences.ui", "preferences", "")
        {
        }
    virtual void setup_widget() override;

private:
    std::unique_ptr<fwk::DataBinderPool> m_binder_pool;
    fwk::TextPairModelRecord m_theme_combo_model;
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

