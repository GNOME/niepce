/*
 * niepce - rust_bindings.hpp
 *
 * Copyright (C) 2017-2018 Hubert Figuiere
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

#if !RUST_BINDGEN

#include <gtk/gtk.h>

#include "engine/db/properties-enum.hpp"

namespace fwk {
class FileList;
}
namespace eng {
struct QueriedContent;
}


namespace ffi {
class rust_str;
struct Utc;
template <class T>
struct DateTime;
typedef rust_str str;
typedef eng::QueriedContent QueriedContent;
typedef eng::NiepceProperties Np;
typedef fwk::FileList FileList;
}

#include "target/fwk_bindings.h"
#include "target/bindings.h"

namespace fwk {

typedef ffi::ExempiManager ExempiManager;
typedef ffi::PropertyValue PropertyValue;
typedef ffi::PropertyBag PropertyBag;
typedef ffi::PropertySet PropertySet;
typedef ffi::Date Date;
typedef ffi::RgbColour RgbColour;
}

namespace eng {
typedef ffi::LibraryId library_id_t; // XXX change this to LibraryId
typedef ffi::FileType FileType;
typedef ffi::FileStatus FileStatus;
typedef ffi::Keyword Keyword;
typedef ffi::LibFile LibFile;
typedef ffi::LibFolder LibFolder;
typedef ffi::LibMetadata LibMetadata;
typedef ffi::Label Label;
typedef ffi::Managed Managed;
typedef ffi::LibNotification LibNotification;
typedef ffi::NotificationType NotificationType;
typedef ffi::FolderVirtualType FolderVirtualType;
}

namespace ui {
  using ffi::dialog_request_new_folder;
  using ffi::dialog_confirm;
}

#endif
