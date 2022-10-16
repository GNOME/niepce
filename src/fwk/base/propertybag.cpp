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

#include "propertybag.hpp"

namespace fwk {

PropertySetPtr property_set_wrap(PropertySet* s)
{
    return PropertySetPtr(s, &ffi::eng_property_set_delete);
}

PropertySetPtr property_set_new()
{
    return property_set_wrap(ffi::eng_property_set_new());
}

PropertyValuePtr property_value_new(const std::vector<std::string>& sa)
{
    PropertyValuePtr value = fwk::property_value_new_string_array();
    for (auto s : sa) {
        value->add_string(s);
    }
    return value;
}

PropertyBagPtr property_bag_wrap(PropertyBag* bag)
{
    return PropertyBagPtr(bag, &ffi::eng_property_bag_delete);
}

PropertyBagPtr property_bag_new()
{
    return property_bag_wrap(ffi::eng_property_bag_new());
}

PropertyValuePtr property_bag_value(const PropertyBagPtr& bag, PropertyIndex idx)
{
    return PropertyValuePtr::from_raw(ffi::eng_property_bag_value(bag.get(), idx));
}

bool set_value_for_property(PropertyBag& bag, ffi::NiepcePropertyIdx idx,
                            const PropertyValue& value)
{
    return ffi::eng_property_bag_set_value(&bag, static_cast<uint32_t>(idx), &value);
}

std::optional<PropertyValuePtr> get_value_for_property(const PropertyBag& bag,
                                                     ffi::NiepcePropertyIdx idx)
{
    auto value = ffi::eng_property_bag_value(&bag, static_cast<uint32_t>(idx));
    if (!value) {
        return std::nullopt;
    }
    return std::optional<PropertyValuePtr>(PropertyValuePtr::from_raw(value));
}

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
