/*
 * niepce - fwk/utils/pathutils.cpp
 *
 * Copyright (C) 2009-2022 Hubert Figuière
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


#include <giomm/file.h>

#include "fwk/base/debug.hpp"
#include "pathutils.hpp"

namespace fwk {

void _path_remove_recursive(const Glib::RefPtr<Gio::File> & dir);

/** return the basename of a path. Example:
    /foo/bar/baz.txt returns baz.txt
 */
std::string path_basename(const std::string & path)
{
    auto slash_idx = path.find_last_of("/");
    if(slash_idx == std::string::npos) {
        return path;
    }
    return std::string(path.cbegin() + slash_idx + 1, path.cend());
}

static
std::string::size_type _path_extension_pos(const std::string &path)
{
    auto idx = path.find_last_of(".");
    if(idx == std::string::npos) {
        return std::string::npos;
    }
    auto slash_idx = path.find_last_of("/");
    // if the '.' is not part of the last component
    if((slash_idx != std::string::npos) && (idx < slash_idx)) {
        return std::string::npos;
    }

    return idx;
}

std::string path_dirname(const std::string & path)
{
    auto slash_idx = path.find_last_of("/");
    if(slash_idx == std::string::npos) {
        return ".";
    }
    return std::string(path.cbegin(), path.cbegin() + slash_idx);
}

/** return the extension of a path
    /foo/bar/baz.txt returns .txt
    It always return the "." if any. No extension = "".
 */
std::string path_extension(const std::string & path)
{
    std::string extension;
    auto idx = _path_extension_pos(path);
    if(idx != std::string::npos) {
        extension = std::string(path.cbegin() + idx, path.cend());
    }

    return extension;
}


std::string path_replace_extension(const std::string & path, const char * ext)
{
    std::string result = path;
    auto idx = _path_extension_pos(result);
    if(idx != std::string::npos) {
        result.replace(result.begin() + idx, result.end(), ext);
    }
    return result;
}

bool path_exists(const std::string & path)
{
    Glib::RefPtr<Gio::File> file = Gio::File::create_for_path(path);
    return file->query_exists();
}

bool ensure_path_for_file(const std::string & path)
{
    try {
        Glib::RefPtr<Gio::File> dir = Gio::File::create_for_path(path_dirname(path));
        if(dir->query_exists()) {
            return true;
        }
        return dir->make_directory_with_parents();
    }
    catch(...) {
        return false;
    }
}

void _path_remove_recursive(const Glib::RefPtr<Gio::File> & dir)
{
    Glib::RefPtr<Gio::FileEnumerator> enumerator = dir->enumerate_children();
    Glib::RefPtr<Gio::FileInfo> file_info;
    while(file_info = enumerator->next_file()) {
        Glib::RefPtr<Gio::File> child;
        child = dir->get_child(file_info->get_name());
        if(file_info->get_file_type() == Gio::FileType::DIRECTORY) {
            _path_remove_recursive(child);
        }
        child->remove();
    }
}


void path_remove_recursive(const std::string & path)
{
    Glib::RefPtr<Gio::File> dir = Gio::File::create_for_path(path);
    _path_remove_recursive(dir);
    dir->remove();
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

