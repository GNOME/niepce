/*
 * niepce - utils/exempi.h
 *
 * Copyright (C) 2007-2022 Hubert Figuière
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
#include <vector>
#include <string>

#include <exempi/xmp.h>

#include "fwk/base/util.hpp"

#include "rust_bindings.hpp"

namespace xmp {

inline
void release(XmpIteratorPtr ptr)
{
    xmp_iterator_free(ptr);
}

inline
void release(XmpStringPtr ptr)
{
    xmp_string_free(ptr);
}

inline
void release(XmpFilePtr ptr)
{
    xmp_files_free(ptr);
}

inline
void release(XmpPtr ptr)
{
    xmp_free(ptr);
}

/**
 * @brief a scoped pointer for Xmp opaque types
 * @todo move to Exempi.
 */
template <class T>
class ScopedPtr
{
public:
    NON_COPYABLE(ScopedPtr);

    ScopedPtr(T p)
        : _p(p)
        {}
    ~ScopedPtr()
        { if (_p) release(_p); }
    operator T() const
        { return _p; }
private:
    T _p;
};

extern const char * NIEPCE_XMP_NAMESPACE;
extern const char * NIEPCE_XMP_NS_PREFIX;
extern const char * UFRAW_INTEROP_NAMESPACE;
extern const char * UFRAW_INTEROP_NS_PREFIX;

}


namespace fwk {

class XmpMeta;

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
