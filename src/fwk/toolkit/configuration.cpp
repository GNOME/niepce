/*
 * niepce - fwk/toolkit/configuration.cpp
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


#include <memory>

#include <glibmm/miscutils.h>
#include <glibmm/fileutils.h>

#include "fwk/base/debug.hpp"
#include "fwk/utils/pathutils.hpp"
#include "configuration.hpp"

//
// NOTE: this does not use GSettings because GSettings is broken by design.
//       like abort() on many simple things that it does not like
//       missing keys, missing schema.
//       This is non negotiable
//

namespace fwk {


Glib::ustring Configuration::make_config_path(const Glib::ustring & file)
{
    Glib::ustring filename = Glib::build_filename(Glib::get_user_config_dir(),
                                                  file, "config");
    fwk::ensure_path_for_file(filename);
    return filename;
}

Configuration::Configuration(const Glib::ustring & file)
    : m_filename(file)
    , m_keyfile(Glib::KeyFile::create())
    , m_root("main")
{
    try {
        m_keyfile->load_from_file(m_filename);
    }
    catch(...) {
        DBG_OUT("conf file %s not found - will be created", m_filename.c_str());
    }
}


Configuration::~Configuration()
{
}

void Configuration::save()
{
    Glib::file_set_contents(m_filename, m_keyfile->to_data());
}

bool Configuration::hasKey(const Glib::ustring & key) const
{
    //
    return m_keyfile->has_group(m_root) && m_keyfile->has_key(m_root, key);
}


const Glib::ustring Configuration::getValue(const Glib::ustring & key,
                                            const Glib::ustring & def) const
{
    bool has_key = hasKey(key);
    if (!has_key) {
        return def;
    }

    return m_keyfile->get_string(m_root, key);
}

void Configuration::setValue(const Glib::ustring & key,
                             const Glib::ustring & value)
{
    m_keyfile->set_string(m_root, key, value);
    save();
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
