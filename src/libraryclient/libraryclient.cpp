/*
 * niepce - libraryclient/libraryclient.cpp
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

#include "libraryclient.hpp"

#include "rust_bindings.hpp"

using eng::library_id_t;

namespace libraryclient {

const char * s_thumbcacheDirname = "thumbcache";

LibraryClient::LibraryClient(const fwk::Moniker & moniker,
                             const std::shared_ptr<ffi::LcChannel>& channel)
    : m_client(
        ffi::libraryclient_new(std::string(moniker.path()).c_str(), channel.get()),
        ffi::libraryclient_delete)
    , m_thumbnailCache(eng::ThumbnailCache_new(
                           std::string(moniker.path()) + "/" + s_thumbcacheDirname,
                           (const eng::LcChannel*)channel.get()))
    , m_uidataprovider(npc::UIDataProvider_new())
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
