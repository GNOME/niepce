/* -*- mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode:nil; -*- */
/*
 * niepce - engine/importer/directoryimporter.hpp
 *
 * Copyright (C) 2014-2017 Hubert Figuière
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

#include <string>
#include <list>

#include "fwk/utils/files.hpp"
#include "engine/importer/iimporter.hpp"

namespace eng {

class DirectoryImporter
    : public IImporter {
public:
    DirectoryImporter();
    virtual ~DirectoryImporter();

    const std::string& id() const override;
    bool list_source_content(const std::string & source,
                             const SourceContentReady& callback) override;
    bool get_previews_for(const std::string& source,
                          const std::list<std::string>& paths,
                          const PreviewReady& callback) override;

    bool do_import(const std::string & source,
                   const FileImporter & importer) override;

};

}
