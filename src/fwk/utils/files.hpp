/* -*- mode: C++; tab-width: 4; c-basic-offset: 4; indent-tabs-mode:nil; -*- */
/*
 * niepce - fwk/utils/files.hpp
 *
 * Copyright (C) 2007-2021 Hubert Figuière
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

#include <algorithm>
#include <vector>
#include <string>
#include <memory>

#include <functional>

#include <giomm/fileinfo.h>

#include "rust_bindings.hpp"

namespace fwk {

/** wrapper around g_dir_make_tmp() */
std::string make_tmp_dir(const std::string& base);

bool filter_none(const Glib::RefPtr<Gio::FileInfo> & file);
bool filter_ext(const char* file, const std::string & ext);
bool filter_only_media(const char* file);

typedef std::shared_ptr<ffi::FileList> FileListPtr;

FileListPtr wrapFileList(ffi::FileList* list);

}
