/*
 * niepce - fwk/base/propertybag.cpp
 *
 * Copyright (C) 2011 Hubert Figuiere
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



#ifndef __FWK_PROPERTYBAG_HPP_
#define __FWK_PROPERTYBAG_HPP_

#include <map>
#include <set>
#include <tr1/memory>
#include <boost/variant.hpp>

namespace fwk {

typedef uint32_t PropertyIndex;
typedef boost::variant<int, std::string> PropertyValue;

typedef std::set<PropertyIndex> PropertySet;

/** a property bag 
 * It is important that the values for PropertyIndex be properly name spaced
 * by the caller.
 */
class PropertyBag
{
public:
    typedef std::tr1::shared_ptr<PropertyBag> Ptr;

    bool empty() const
        {
            return m_bag.empty();
        }

    /** return true if a property was removed prior to insertion */
    bool set_value_for_property(PropertyIndex idx, const PropertyValue & value);
    /** return true if a property is found */
    bool get_value_for_property(PropertyIndex idx, PropertyValue & value) const;
    /** return true if property exist */
    bool has_value_for_property(PropertyIndex idx) const;
    /** return true if the property was removed */
    bool remove_value_for_property(PropertyIndex idx);
private:
    typedef std::map<PropertyIndex, PropertyValue> _Map;
    _Map    m_bag;
};


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
#endif
