/*
 * niepce - libraryclient/uidataprovider.hpp
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

#pragma once

#include <stdint.h>
#include <memory>
#include <optional>

#include "engine/db/label.hpp"

#include "rust_bindings.hpp"

namespace libraryclient {

class UIDataProvider
{
public:
    // label management

    void updateLabel(const eng::LabelPtr&);
    void addLabel(const eng::LabelPtr& l);
    void deleteLabel(int id);
    std::optional<fwk::RgbColourPtr> colourForLabel(int32_t id) const;
    const eng::LabelList & getLabels() const
        { return m_labels; }
private:
    eng::LabelList m_labels;
};

typedef std::shared_ptr<UIDataProvider> UIDataProviderPtr;
typedef std::weak_ptr<UIDataProvider> UIDataProviderWeakPtr;

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

