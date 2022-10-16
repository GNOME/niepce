/*
 * niepce - eng/db/properties.cpp
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

#include "properties.hpp"

#include "rust_bindings.hpp"

namespace eng {

#define DEFINE_PROPERTY(a,b,c,d,e)               \
    { NiepcePropertyIdx::a, { NiepcePropertyIdx::a, #a, typeid(e) } },

static const PropDescMap propmap = {

    #include "engine/db/properties-def.hpp"

};

#undef DEFINE_PROPERTY

const char * _propertyName(eng::NiepcePropertyIdx idx)
{
    PropDescMap::const_iterator iter = propmap.find(idx);
    if(iter != propmap.end()) {
        if(iter->second.name) {
            return iter->second.name;
        }
    }
    return "UNKNOWN";
}

bool check_property_type(eng::NiepcePropertyIdx idx, const std::type_info & ti)
{
    PropDescMap::const_iterator iter = propmap.find(idx);
    if(iter != propmap.end()) {
        return iter->second.type == ti;
    }
    // we don't know the type. Assume it is OK
    return true;
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
