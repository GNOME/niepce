/*
 * niepce - fwk/base/propertybag.cpp
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
#include <string>
#include <vector>
#include <memory>

#include <optional>

#include "rust_bindings.hpp"

namespace fwk {

typedef uint32_t PropertyIndex;

PropertyValuePtr property_value_new(const std::vector<std::string>&);

typedef std::shared_ptr<PropertySet> PropertySetPtr;

PropertySetPtr property_set_new();

/** a property bag
 * It is important that the values for PropertyIndex be properly name spaced
 * by the caller.
 */
typedef std::shared_ptr<PropertyBag> PropertyBagPtr;

PropertyBagPtr property_bag_new();
PropertyBagPtr property_bag_wrap(PropertyBag*);

PropertyValuePtr property_bag_value(const PropertyBagPtr& bag, PropertyIndex key);

/** return true if a property was removed prior to insertion */
bool set_value_for_property(PropertyBag&, ffi::NiepcePropertyIdx idx, const PropertyValue& value);
/** return property or an empty option */
std::optional<PropertyValuePtr> get_value_for_property(const PropertyBag&, ffi::NiepcePropertyIdx idx);

}

/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  fill-column:99
  End:
*/
