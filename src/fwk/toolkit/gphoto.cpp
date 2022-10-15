/* -*- mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode:nil; -*- */
/*
 * niepce - fwk/toolkit/gphoto.cpp
 *
 * Copyright (C) 2009-2017 Hubert Figuiere
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

#include <string.h>
#include <string>
#include <vector>

#include <gphoto2-port-info-list.h>
#include <gphoto2-abilities-list.h>
#include <gphoto2-camera.h>
#include <gphoto2-context.h>
#include <gphoto2-port-result.h>

#include <glibmm/miscutils.h>
#include <giomm/asyncresult.h>
#include <giomm/file.h>
#include <giomm/volumemonitor.h>
#include <gdkmm/pixbufloader.h>

#include "fwk/base/debug.hpp"
#include "fwk/utils/exempi.hpp"
#include "fwk/utils/files.hpp"
#include "fwk/utils/pathutils.hpp"
#include "fwk/toolkit/gdkutils.hpp"
#include "gphoto.hpp"


namespace fwk {


#define GP_CHECK(x, op)   if (x < GP_OK) { \
    DBG_OUT("%s failed with %d", #op, x); }


namespace gp {

CameraFilePtr file_new()
{
    CameraFile* file;
    gp_file_new(&file);
    return CameraFilePtr(file, &gp_file_unref);
}

CameraListPtr list_new()
{
    CameraList* list;
    gp_list_new(&list);
    return CameraListPtr(list, &gp_list_unref);
}

}

GpDevice::GpDevice(const std::string& model, const std::string& path)
    : m_model(model)
    , m_path(path)
{
}



GpDeviceList::GpDeviceList()
    : m_abilities(nullptr)
    , m_ports(nullptr)
{
    reload();
}

GpDeviceList::~GpDeviceList()
{
    gp_cleanup();
}


void GpDeviceList::gp_cleanup()
{
    if (m_abilities) {
        gp_abilities_list_free(m_abilities);
        m_abilities = nullptr;
    }
    if (m_ports) {
        gp_port_info_list_free(m_ports);
        m_ports = nullptr;
    }
}

void GpDeviceList::reload()
{
    gp_cleanup();

    int ret;
    ret = gp_port_info_list_new(&m_ports);
    GP_CHECK(ret, gp_port_info_list_new);
    ret = gp_port_info_list_load(m_ports);
    GP_CHECK(ret, gp_port_list_load);
	ret = gp_abilities_list_new(&m_abilities);
    GP_CHECK(ret, gp_abilities_list_new);
    ret = gp_abilities_list_load(m_abilities, NULL);
    GP_CHECK(ret, gp_abilities_list_load);
}


void GpDeviceList::detect()
{
    if((!m_ports) || (!m_abilities)) {
        reload();
    }

    clear();

    gp::CameraListPtr camera_list = gp::list_new();
    int ret = gp_abilities_list_detect(m_abilities, m_ports, camera_list.get(), nullptr);
    GP_CHECK(ret, gp_abilities_list_detect);

    int count = gp_list_count(camera_list.get());
    for (int i = 0; i < count; i++) {
        const char * name = nullptr;
        const char * value = nullptr;

        ret = gp_list_get_name(camera_list.get(), i, &name);
        GP_CHECK(ret, gp_list_get_name);

        ret = gp_list_get_value(camera_list.get(), i, &value);
        GP_CHECK(ret, gp_list_get_value);

        if ((count > 1) && (strcmp(value, "usb:") == 0)) {
            continue;
        }
        DBG_OUT("found %s %s", name, value);
        push_back(GpDevicePtr(new GpDevice(name, value)));
    }
}

fwk::Option<GpDevicePtr> GpDeviceList::get_device(const std::string& device)
{
    for (auto d : fwk::GpDeviceList::obj()) {
        if (d->get_path() == device) {
            return fwk::Option<GpDevicePtr>(d);
        }
    }
    return fwk::Option<GpDevicePtr>();
}

class GpCamera::Priv {
public:
    Priv()
        : camera(nullptr)
        , context(gp_context_new())
        {
        }
    ~Priv()
        {
            if (camera) {
                gp_camera_unref(camera);
            }
            gp_context_unref(context);
        }
    ::Camera* camera;
    ::GPContext* context;
};

GpCamera::GpCamera(const GpDevicePtr& device)
    : m_device(device)
    , m_priv(new Priv)
{
}

GpCamera::~GpCamera()
{
    delete m_priv;
}

bool GpCamera::open()
{
    bool success = false;
    if (m_priv->camera) {
        close();
    }

    if (m_temp_dir_path.empty()) {
        m_temp_dir_path = fwk::make_tmp_dir("niepce-gphoto-XXXXXX");
    }

    gp_camera_new(&m_priv->camera);

    CameraAbilities abilities;
    auto al = fwk::GpDeviceList::obj().get_abilities_list();
    int model_index = gp_abilities_list_lookup_model(al, m_device->get_model().c_str());
	gp_abilities_list_get_abilities(al, model_index, &abilities);
    gp_camera_set_abilities(m_priv->camera, abilities);

	GPPortInfo info;
    auto info_list = ::fwk::GpDeviceList::obj().get_port_info_list();
    DBG_OUT("looking up port %s",  m_device->get_path().c_str());
    int port_index = gp_port_info_list_lookup_path(info_list, m_device->get_path().c_str());
    DBG_OUT("port index = %d", port_index);
    if (port_index >= 0) {
        gp_port_info_list_get_info(info_list, port_index, &info);
        gp_camera_set_port_info(m_priv->camera, info);

        int result = gp_camera_init(m_priv->camera, m_priv->context);
        DBG_OUT("camera init returned %d", result);
        switch (result) {
        case GP_OK:
            DBG_OUT("camera open");
            success = true;
            break;
        case GP_ERROR_IO_USB_CLAIM:
            DBG_OUT("will try to unmount GVFS");
            try_unmount_camera();
            break;
        case GP_ERROR_CANCEL:
            break;
        default:
            close();
        }
    }

    return success;
}

/** A hackish attempt to unmount the camera
 *  The code is specific to Linux as we assume the form of the
 *  device used for USB.
 *
 *  A better solution should be thought.
 */
bool GpCamera::try_unmount_camera()
{
    // Turn the gphoto device ID into a device path.
    // This code is Linux specific at the moment.
    std::string device_path = m_device->get_path();
    // The device path has to start with "usb:".
    // XXX figure out non USB.
    if (device_path.find("usb:") != 0) {
        ERR_OUT("Device %s is not USB", device_path.c_str());
        return false;
    }
    // Conveniently we can replace the ':' by a '/'
    device_path[3] = '/';
    auto comma = device_path.find(',');
    if (comma == std::string::npos) {
        ERR_OUT("Device %s is not USB", device_path.c_str());
        return false;
    }
    device_path[comma] = '/';
    // XXX this is specific to Linux
    std::string device_id = "/dev/bus/";
    device_id += device_path;

    auto mounts = Gio::VolumeMonitor::get()->get_mounts();
    Glib::RefPtr<Gio::Mount> to_unmount;
    for (auto mount : mounts) {
        auto volume = mount->get_volume();
        if (!volume) {
            DBG_OUT("no volume");
            continue;
        }
        auto vol_id = volume->get_identifier("unix-device");
        if (vol_id == device_id) {
            to_unmount = mount;
            break;
        }
    }
    if (!to_unmount) {
        return false;
    }
    try {
        auto mount_op = Gio::MountOperation::create();
        // We need to pass a callback or gvfs will crash
        to_unmount->unmount(mount_op, [] (Glib::RefPtr<Gio::AsyncResult>&) {}, Gio::Mount::UnmountFlags::NONE);
    } catch(const Gio::Error& e) {
        ERR_OUT("Gio::Error unmounting camera %d", e.code());
        return false;
    }
    return true;
}

bool GpCamera::close()
{
    gp_camera_unref(m_priv->camera);
    m_priv->camera = nullptr;
    return true;
}

void GpCamera::process_folders(const std::vector<std::string>& folders,
                               std::list<std::pair<std::string, std::string>>& files) const
{
    for (auto folder : folders) {
        gp::CameraListPtr flist = gp::list_new();
        int result = gp_camera_folder_list_files(m_priv->camera, folder.c_str(), flist.get(),
                                                m_priv->context);
        DBG_OUT("listed folder %d", result);
        if (result == GP_OK) {
            int count = gp_list_count(flist.get());
            DBG_OUT("processing folder %s, count %d", folder.c_str(), count);
            for (int i = 0; i < count; i++) {
                const char *name = nullptr;
                gp_list_get_name(flist.get(), i, &name);
                files.push_back(std::make_pair(folder, name));
            }
        }
    }
}

std::list<std::pair<std::string, std::string>> GpCamera::list_content() const
{
    std::list<std::pair<std::string, std::string>> files;

    DBG_OUT("list content");

    // XXX fixme this should not be hardcoded.
    // This is the path for PTP.
    std::string root_folder_ptp = "/store_00010001/DCIM";
    // This is the path for a regular DCIF.
    std::string root_folder = "/DCIM";
    gp::CameraListPtr list = gp::list_new();
    int result = gp_camera_folder_list_folders(m_priv->camera, root_folder.c_str(), list.get(),
                                               m_priv->context);
    if (result != GP_OK) {
        DBG_OUT("Folder %s not found, trying %s", root_folder.c_str(),
                root_folder_ptp.c_str());
        root_folder = root_folder_ptp;
        result = gp_camera_folder_list_folders(m_priv->camera, root_folder.c_str(),
                                               list.get(), m_priv->context);
    }
    DBG_OUT("initial folder list %d", result);
    if (result == GP_OK) {
        std::vector<std::string> folders;
        int count = gp_list_count(list.get());
        DBG_OUT("list count %d", count);

        for (int i = 0; i < count; i++) {
            const char* name = nullptr;
            gp_list_get_name(list.get(), i, &name);
            DBG_OUT("found folder %s", name);
            folders.push_back(root_folder + "/" + name);
        }

        process_folders(folders, files);
    }

    return files;
}

std::optional<fwk::ThumbnailPtr> GpCamera::get_preview(const std::string& path) const
{
    std::string folder = fwk::path_dirname(path);
    std::string name = fwk::path_basename(path);

    gp::CameraFilePtr file = gp::file_new();
    int result = gp_camera_file_get(m_priv->camera, folder.c_str(), name.c_str(),
                                     GP_FILE_TYPE_PREVIEW, file.get(),
                                     m_priv->context);
    if (result >= 0) {
        const char *fd;
        unsigned long fs;
        gp_file_get_data_and_size(file.get(), &fd, &fs);

        Glib::RefPtr<Gdk::PixbufLoader> loader = Gdk::PixbufLoader::create();
        loader->write(reinterpret_cast<const guint8*>(fd), fs);
        loader->close();

        auto pix = loader->get_pixbuf();
#if 0
        result = gp_camera_file_get(m_priv->camera, folder.c_str(), name.c_str(),
                                    GP_FILE_TYPE_EXIF, file.get(),
                                    m_priv->context);
        if (result >= 0) {
            const char *exifdata;
            unsigned long exifsize;
            int32_t orientation = 0;

            gp_file_get_data_and_size(file.get(), &exifdata, &exifsize);
            std::string exif_path = Glib::build_filename(m_temp_dir_path, name);
            FILE* exif_file = fopen(exif_path.c_str(), "w");
            fwrite(exifdata + 10, 1, exifsize - 10, exif_file);
            fclose(exif_file);

            DBG_OUT("exif block size %lu", exifsize);
            fwk::XmpMeta xmp(exif_path, false);
            DBG_OUT("xmp open exif %s", exif_path.c_str());
            if (xmp.isOk()) {
                orientation = xmp.orientation();
            } else {
                ERR_OUT("xmp is not ok");
            }
            pix = fwk::gdkpixbuf_exif_rotate(pix, orientation);
            unlink(exif_path.c_str());
        }
#endif
        return std::optional<fwk::ThumbnailPtr>(fwk::Thumbnail_from_pixbuf((char*)pix->gobj()));
    }

    return std::nullopt;
}

bool GpCamera::download_file(const std::string& folder, const std::string& file,
                             const std::string& dest)
{
    DBG_OUT("importing into %s", dest.c_str());
    gp::CameraFilePtr camerafile = gp::file_new();
    int result = gp_camera_file_get(m_priv->camera, folder.c_str(), file.c_str(),
                                    GP_FILE_TYPE_NORMAL, camerafile.get(),
                                    m_priv->context);
    if (result == GP_OK) {
        result = gp_file_save(camerafile.get(), dest.c_str());
    }

    return (result == GP_OK);
}

}
