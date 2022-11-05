/* -*- mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode:nil; -*- */
/*
 * niepce - fwk/toolkit/gphoto.hpp
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

#pragma once

#include <list>
#include <string>
#include <memory>
#include <optional>

#include <gphoto2-camera.h>
#include <gphoto2-port-info-list.h>
#include <gphoto2-abilities-list.h>

#include "fwk/base/option.hpp"
#include "fwk/base/singleton.hpp"
#include "fwk/base/util.hpp"

#include "rust_bindings.hpp"

namespace fwk {

namespace gp {
typedef std::unique_ptr<CameraFile, decltype(&gp_file_unref)> CameraFilePtr;

CameraFilePtr file_new();

typedef std::unique_ptr<CameraList, decltype(&gp_list_unref)> CameraListPtr;

CameraListPtr list_new();
}
/** Describe a gphoto device: model + port (path)
 */
class GpDevice
{
public:
    NON_COPYABLE(GpDevice);

    GpDevice(const std::string& model, const std::string& path);

    const std::string& get_model() const
        {
            return m_model;
        }
    const std::string& get_path() const
        {
            return m_path;
        }
private:
    std::string m_model;
    std::string m_path;
};

typedef std::shared_ptr<GpDevice> GpDevicePtr;

/** The device list from gphoto.
 */
class GpDeviceList
    : public fwk::Singleton<GpDeviceList>
    , public std::list<GpDevicePtr>
{
public:
    ~GpDeviceList();

    void reload();
    void detect();
    ::CameraAbilitiesList* get_abilities_list() const
        { return m_abilities; }
    ::GPPortInfoList* get_port_info_list() const
        { return m_ports; }
    fwk::Option<GpDevicePtr> get_device(const std::string& device);
protected:
    friend class fwk::Singleton<GpDeviceList>;
    GpDeviceList();
private:

    void gp_cleanup();
    ::CameraAbilitiesList* m_abilities;
    ::GPPortInfoList* m_ports;
};

/** A gphoto camera.
 */
class GpCamera
{
public:
    NON_COPYABLE(GpCamera);

    GpCamera(const GpDevicePtr& device);
    virtual ~GpCamera();

    const std::string& path() const
        { return m_device->get_path(); }
    bool open();
    bool close();
    bool try_unmount_camera();
    std::list<std::pair<std::string, std::string>> list_content() const;
    std::optional<fwk::ThumbnailPtr> get_preview(const std::string& path) const;
    bool download_file(const std::string& folder, const std::string& file,
                       const std::string& dest);
private:
    void process_folders(const std::vector<std::string>& folders,
                         std::list<std::pair<std::string, std::string>>& files) const;
    class Priv;
    GpDevicePtr m_device;
    std::string m_temp_dir_path;
    Priv* m_priv;
};

typedef std::unique_ptr<GpCamera> GpCameraPtr;

}
