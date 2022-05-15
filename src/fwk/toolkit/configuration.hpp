/*
 * niepce - fwk/toolkit/configuration.hpp
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

#include <glibmm/ustring.h>
#include <glibmm/keyfile.h>

namespace fwk {

class Configuration
{
public:
    typedef std::shared_ptr<Configuration> Ptr;

    Configuration(const Glib::ustring & file);
    ~Configuration();

    static Glib::ustring make_config_path(const Glib::ustring & file);

    bool hasKey(const Glib::ustring & key) const;
    const Glib::ustring getValue(const Glib::ustring & key,
                                 const Glib::ustring & def) const;

    void setValue(const Glib::ustring & key, const Glib::ustring & value);
private:

    void save();

    Glib::ustring          m_filename;
    Glib::RefPtr<Glib::KeyFile> m_keyfile;
    Glib::ustring          m_root;
};

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
