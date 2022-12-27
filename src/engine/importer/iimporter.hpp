/*
 * niepce - engine/importer/iimporter.hpp
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

#ifndef _IN_RUST_BINDINGS_

#pragma once

#include <string>
#include <vector>
#include <functional>

#include "fwk/utils/files.hpp"

#include "rust_bindings.hpp"

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
    typedef std::function<void (std::vector<::rust::Box<eng::WrappedImportedFile>>&&)> SourceContentReady;
    /** list the source content and store it. */
    virtual bool list_source_content(const std::string& source,
                                     const SourceContentReady& callback) = 0;

    typedef std::function<void (std::string&& path,
                                fwk::ThumbnailPtr&&)> PreviewReady;
    virtual bool get_previews_for(const std::string& source,
                                  const std::list<std::string>& paths,
                                  const PreviewReady& callback) = 0;

    /** file importer callback
     */
    typedef std::function<bool (const std::string& path, const fwk::FileList&, Managed)> FileImporter;
    /** perform import from source
     * @param source the source identified by a string.
     * @param dest_dir the suggested destination directory is the importer needs to copy
     * @param importer the callback to import. Copy shall occur first if needed.
     */
    virtual bool do_import(const std::string& source, const std::string& dest_dir,
                           const FileImporter& importer) = 0;

    // cxx glue
    void do_import_(rust::Str source, rust::Str dest_dir,
                    rust::Fn<bool(const eng::LibraryClientWrapper&, rust::Str, const fwk::FileList&, Managed)> cb,
                    const eng::LibraryClientWrapper& client) const {
        const_cast<IImporter*>(this)->do_import(std::string(source), std::string(dest_dir),
                                                [cb, &client] (const std::string& p, const fwk::FileList& fl, Managed m) -> bool {
                                                    return cb(client, p, fl, m);
                                                });
    }

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
#endif
