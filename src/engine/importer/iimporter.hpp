/*
 * niepce - engine/importer/iimporter.hpp
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
#include <functional>

#include "fwk/toolkit/thumbnail.hpp"
#include "engine/db/library.hpp"
#include "engine/importer/importedfile.hpp"

namespace eng {

/**
 * Interface for importers.
 */
class IImporter {
public:
    virtual ~IImporter() {}

    /** Return the id string */
    virtual const std::string& id() const = 0;

    /** Source content is ready */
    typedef std::function<void (std::list<ImportedFilePtr>&&)> SourceContentReady;
    /** list the source content and store it. */
    virtual bool list_source_content(const std::string& source,
                                     const SourceContentReady& callback) = 0;

    typedef std::function<void (const std::string& path,
                                const fwk::Thumbnail&)> PreviewReady;
    virtual bool get_previews_for(const std::string& source,
                                  const std::list<std::string>& paths,
                                  const PreviewReady& callback) = 0;

    enum class Import {
        SINGLE,
        DIRECTORY
    };
    /** file importer callback */
    typedef std::function<void (const std::string&, Import, Library::Managed)> FileImporter;
    /** perform import from source */
    virtual bool do_import(const std::string& source,
                           const FileImporter& importer) = 0;

};

}
/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  c-basic-offset:4
  indent-tabs-mode:nil
  tab-width:4
  fill-column:99
  End:
*/
