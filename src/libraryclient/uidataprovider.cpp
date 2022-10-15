/*
 * niepce - libraryclient/uidataprovider.cpp
 *
 * Copyright (C) 2011-2022 Hubert Figuière
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

#include <algorithm>

#include "fwk/base/debug.hpp"
#include "uidataprovider.hpp"

namespace libraryclient {

void UIDataProvider::updateLabel(const eng::LabelPtr& l)
{
    // TODO: will work as long as we have 5 labels or something.
    for (auto & label : m_labels) {
        if (label->id() == l->id()) {
            label = l->clone_boxed();
        }
    }
}


void UIDataProvider::addLabel(const eng::LabelPtr& l)
{
    m_labels.push_back(l->clone_boxed());
}


void UIDataProvider::deleteLabel(int id)
{
    // TODO: will work as long as we have 5 labels or something.
    for(auto iter = m_labels.begin();
        iter != m_labels.end(); ++iter) {

        if ((*iter)->id() == id) {
            DBG_OUT("remove label %d", id);
            iter = m_labels.erase(iter);
            break;
        }
    }
}

std::optional<fwk::RgbColourPtr> UIDataProvider::colourForLabel(int32_t id) const
{
    for(auto& label : m_labels) {
        if (label->id() == id) {
            return std::optional<fwk::RgbColourPtr>(label->colour());
        }
    }
    return std::nullopt;
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

