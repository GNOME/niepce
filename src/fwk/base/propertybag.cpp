/*
 * niepce - fwk/base/propertybag.cpp
 *
 * Copyright (C) 2011-2021 Hubert Figuière
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


#include "string.hpp"
#include "debug.hpp"
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

PropertyValuePtr property_value_wrap(PropertyValue* v)
{
    return PropertyValuePtr(v, &ffi::fwk_property_value_delete);
}

PropertyValuePtr property_value_new(const std::string& v)
{
    return property_value_wrap(ffi::fwk_property_value_new_str(v.c_str()));
}

PropertyValuePtr property_value_new(int v)
{
    return property_value_wrap(ffi::fwk_property_value_new_int(v));
}

PropertyValuePtr property_value_new(const std::vector<std::string>& sa)
{
    PropertyValue* value = ffi::fwk_property_value_new_string_array();
    for (auto s : sa) {
        ffi::fwk_property_value_add_string(value, s.c_str());
    }
    return property_value_wrap(value);
}

std::string property_value_get_string(const PropertyValue &value)
{
    auto s = fwk::RustFfiString(ffi::fwk_property_value_get_string(&value));
    return s.str();
}

std::vector<std::string> property_value_get_string_array(const PropertyValue &value)
{
    std::vector<std::string> v;
    auto len = ffi::fwk_property_value_count_string_array(&value);
    for (size_t i = 0; i < len; i++) {
        auto s = fwk::RustFfiString(ffi::fwk_property_value_get_string_at(&value, i));
        v.push_back(s.str());
    }
    return v;
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
    auto value = ffi::eng_property_bag_value(bag.get(), idx);
    return property_value_wrap(value);
}

bool set_value_for_property(PropertyBag& bag, ffi::NiepcePropertyIdx idx,
                            const PropertyValue & value)
{
    return ffi::eng_property_bag_set_value(&bag, static_cast<uint32_t>(idx), &value);
}

fwk::Option<PropertyValuePtr> get_value_for_property(const PropertyBag& bag,
                                                     ffi::NiepcePropertyIdx idx)
{
    auto value = ffi::eng_property_bag_value(&bag, static_cast<uint32_t>(idx));
    if (!value) {
        return fwk::Option<PropertyValuePtr>();
    }
    return fwk::Option<PropertyValuePtr>(property_value_wrap(value));
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
