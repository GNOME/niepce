/*
 * niepce - libraryclient/libraryclient.cpp
 *
 * Copyright (C) 2007-2018 Hubert Figuière
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


#include "fwk/base/moniker.hpp"

#include "libraryclient.hpp"
#include "uidataprovider.hpp"

#include "rust_bindings.hpp"

using eng::library_id_t;

namespace libraryclient {

const char * s_thumbcacheDirname = "thumbcache";

LibraryClient::LibraryClient(const fwk::Moniker & moniker,
                             uint64_t notif_id)
    : m_client(
        ffi::libraryclient_new(moniker.path().c_str(), notif_id),
        ffi::libraryclient_delete)
    , m_thumbnailCache(moniker.path() + "/" + s_thumbcacheDirname, notif_id)
    , m_uidataprovider(new UIDataProvider())
{
}

LibraryClient::~LibraryClient()
{
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
