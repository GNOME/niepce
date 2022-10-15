/* -*- mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode:nil; -*- */
/*
 * niepce - engine/importer/cameraimporter.cpp
 *
 * Copyright (C) 2017-2022 Hubert Figuière
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

#include <glibmm/miscutils.h>
#include <giomm/file.h>

#include "cameraimporter.hpp"

#include "fwk/base/debug.hpp"

namespace eng {


class CameraImportedFile
    : public ImportedFile
{
public:
    CameraImportedFile(const std::pair<std::string, std::string>& desc)
        : CameraImportedFile(desc.first, desc.second)
        { }

    CameraImportedFile(const std::string& folder, const std::string& filename)
        : m_folder(folder)
        , m_filename(filename)
        , m_path(m_folder + "/" + m_filename)
        { }
    const std::string& name() const override
        { return m_filename; }
    const std::string& path() const override
        { return m_path; }
    const std::string& folder() const
        { return m_folder; }
private:
    std::string m_folder;
    std::string m_filename;
    std::string m_path;
};


CameraImporter::CameraImporter()
{
}

CameraImporter::~CameraImporter()
{
    if (m_camera) {
        m_camera->close();
    }
}

const std::string& CameraImporter::id() const
{
    static std::string _id = "CameraImporter";
    return _id;
}

bool CameraImporter::list_source_content(const std::string& source,
                                         const SourceContentReady& callback)
{
    if (ensure_camera_open(source)) {
        auto content = m_camera->list_content();
        std::list<ImportedFilePtr> file_list;
        for (auto item : content) {
            file_list.push_back(ImportedFilePtr(new CameraImportedFile(item)));
        }

        callback(std::move(file_list));
        return true;
    }
    return false;
}

bool CameraImporter::get_previews_for(const std::string& source,
                                      const std::list<std::string>& paths,
                                      const PreviewReady& callback)
{
    if (ensure_camera_open(source)) {
        for (auto path: paths) {
            DBG_OUT("want thumbnail %s", path.c_str());
            auto thumbnail = m_camera->get_preview(path);
            if (thumbnail.has_value()) {
                callback(std::move(path), std::move(thumbnail.value()));
            }
        }

        return true;
    }
    return false;
}

bool CameraImporter::do_import(const std::string& source, const std::string& dest_dir,
                               const FileImporter& importer)
{
    // XXX we shouldn't have to do that.
    list_source_content(
        source,
        [this, dest_dir, importer] (std::list<ImportedFilePtr>&& file_list) {
            auto tmp_dir_path = dest_dir.empty() ?
                fwk::make_tmp_dir("niepce-camera-import-XXXXXX") :
                dest_dir;
            if (!dest_dir.empty()) {
                auto dir = Gio::File::create_for_path(tmp_dir_path);
                dir->make_directory_with_parents();
                // XXX check for errors.
            }
            DBG_ASSERT(!tmp_dir_path.empty(), "Dest dir is empty");
            // XXX check we don't return an empty string.

            fwk::FileListPtr files = fwk::FileList_new();
            for (auto file: file_list) {
                auto imported_camera_file =
                    std::dynamic_pointer_cast<CameraImportedFile>(file);
                if (!imported_camera_file) {
                    continue;
                }

                std::string output_path =
                    Glib::build_filename(tmp_dir_path,
                                         imported_camera_file->name());
                if (this->m_camera->download_file(imported_camera_file->folder(),
                                                  imported_camera_file->name(),
                                                  output_path)) {
                    files->push_back(output_path);
                }
            }

            return importer(tmp_dir_path, files, Managed::NO);
        });
    return true;
}

bool CameraImporter::ensure_camera_open(const std::string& source)
{
    if (!m_camera || m_camera->path() != source) {
        auto result = fwk::GpDeviceList::obj().get_device(source);
        if (result.ok()) {
            m_camera.reset(new fwk::GpCamera(result.unwrap()));
        }
    }
    if (m_camera) {
        m_camera->open();
    }
    return !!m_camera;
}

}
