/*
 * niepce - eng/db/libmetadata.hpp
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

#pragma once

#include "engine/db/metadata.hpp"
#include "fwk/base/propertybag.hpp"

#include "rust_bindings.hpp"

namespace eng {

fwk::PropertyBagPtr libmetadata_to_properties(const LibMetadata *meta,
                                              const fwk::PropertySet &propset);
fwk::WrappedPropertyBagPtr libmetadata_to_wrapped_properties(const LibMetadata* meta,
                                                             const fwk::PropertySet& propset);

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
