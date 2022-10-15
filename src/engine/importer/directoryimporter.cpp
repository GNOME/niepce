/* -*- mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode:nil; -*- */
/*
 * niepce - engine/importer/directoryimporter.cpp
 *
 * Copyright (C) 2014-2022 Hubert Figuière
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

#include <glibmm/i18n.h>
#include <glibmm/miscutils.h>

#include "fwk/base/debug.hpp"
#include "fwk/base/string.hpp"
#include "fwk/utils/pathutils.hpp"
#include "engine/importer/directoryimporter.hpp"
#include "engine/importer/importedfile.hpp"

namespace eng {

class DirectoryImportedFile
  : public ImportedFile
{
public:
    DirectoryImportedFile(const std::string & path)
        : m_path(path)
        {
            m_name = fwk::path_basename(path);
        }
    const std::string& name() const override
        { return m_name; }
    const std::string& path() const override
        { return m_path; }
private:
    std::string m_name;
    std::string m_path;
};


DirectoryImporter::DirectoryImporter()
{
}

DirectoryImporter::~DirectoryImporter()
{
}

const std::string& DirectoryImporter::id() const
{
    static std::string _id = "DirectoryImporter";
    return _id;
}

bool DirectoryImporter::list_source_content(const std::string & source,
                                            const SourceContentReady& callback)
{
    auto files = fwk::FileList_get_media_files_from_directory(source);
    DBG_OUT("files size: %lu", files->size());
    std::list<ImportedFilePtr> content;
    for (size_t i = 0; i < files->size(); i++) {
        auto entry = files->at(i);
        content.push_back(ImportedFilePtr(new DirectoryImportedFile(std::string(entry))));
    }
    callback(std::move(content));

    return true;
}

bool DirectoryImporter::get_previews_for(const std::string& /*source*/,
                                         const std::list<std::string>& paths,
                                         const PreviewReady& callback)
{
    for (auto path : paths) {
        DBG_OUT("path %s", path.c_str());
        auto thumbnail = fwk::Thumbnail_for_file(path, 160, 160, 0);
        callback(std::move(path), std::move(thumbnail));
    }
    return true;
}

bool DirectoryImporter::do_import(const std::string& source, const std::string& /*dest_dir*/,
                                  const FileImporter& callback)
{
    fwk::FileListPtr files = fwk::FileList_get_files_from_directory(source);

    return callback(source, files, Managed::NO);
}

}
