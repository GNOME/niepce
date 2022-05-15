/*
 * niepce - fwk/toolkit/mimetype.cpp
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

#include <string>
#include <exception>

#include <giomm/contenttype.h>

#include "config.h"

#include "fwk/base/debug.hpp"
#include "fwk/utils/pathutils.hpp"
#include "mimetype.hpp"

namespace fwk {

MimeType::MimeType(const char* filename)
    : MimeType(std::string(filename))
{
}

MimeType::MimeType(const std::string & filename)
    : m_name(filename)
{
    try {
        Glib::RefPtr<Gio::File> file = Gio::File::create_for_path(filename);
        auto fileinfo = file->query_info();
        m_type = fileinfo->get_content_type();
    }
    catch(const std::exception &e) {
        gboolean uncertainty = false;
        gchar *content_type = g_content_type_guess(filename.c_str(),
                                                   nullptr, 0, &uncertainty);
        m_type = content_type;

        g_free(content_type);
    }
}

MimeType::MimeType(const Glib::RefPtr<Gio::File> & file)
{
    DBG_ASSERT(static_cast<bool>(file), "file can't be NULL");
    auto fileinfo = file->query_info();
    m_name = fileinfo->get_name();
    m_type = fileinfo->get_content_type();
}

bool MimeType::isDigicamRaw() const
{
    return Gio::content_type_is_a(m_type, "image/x-dcraw");
}


bool MimeType::isImage() const
{
    return Gio::content_type_is_a(m_type, "image/*");
}

bool MimeType::isMovie() const
{
    return Gio::content_type_is_a(m_type, "video/*");
}

bool MimeType::isUnknown() const
{
    return Gio::content_type_is_unknown(m_type);
}


bool MimeType::isXmp() const
{
    return fwk::path_extension(m_name) == ".xmp";
}

const std::string & MimeType::string() const
{
    return m_type;
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
