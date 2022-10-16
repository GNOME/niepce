/*
 * niepce - rust_bindings.hpp
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

#pragma once

#include <memory>

#include <gtk/gtk.h>

#include "fwk/cxx_fwk_bindings.hpp"
#include "fwk/cxx_eng_bindings.hpp"

namespace ffi {
class rust_str;
struct Utc;
template <class T>
struct DateTime;
typedef fwk::Date Date;
typedef rust_str str;
typedef fwk::FileList FileList;
typedef fwk::PropertyValue PropertyValue;
typedef fwk::RgbColour RgbColour;
typedef eng::Label Label;
typedef eng::LibFile LibFile;
typedef eng::LibMetadata LibMetadata;
typedef eng::Keyword Keyword;
struct NiepcePropertyBag;
struct NiepcePropertySet;
}

#include "target/fwk_bindings.h"
#include "target/eng_bindings.h"
#include "target/bindings.h"

namespace fwk {
typedef std::shared_ptr<SharedConfiguration> ConfigurationPtr;
typedef rust::Box<fwk::Date> DatePtr;
typedef rust::Box<Thumbnail> ThumbnailPtr;
typedef rust::Box<FileList> FileListPtr;
typedef rust::Box<RgbColour> RgbColourPtr;
typedef rust::Box<PropertyValue> PropertyValuePtr;

typedef ffi::NiepcePropertyBag PropertyBag;
typedef ffi::NiepcePropertySet PropertySet;
}

namespace eng {
typedef rust::Box<Keyword> KeywordPtr;
typedef rust::Box<Label> LabelPtr;
typedef rust::Box<LibFile> LibFilePtr;

typedef ffi::NiepcePropertyIdx Np;
using NiepcePropertyIdx = ffi::NiepcePropertyIdx;
typedef ffi::LibraryId library_id_t; // XXX change this to LibraryId
typedef ffi::FileStatus FileStatus;
typedef ffi::LibFolder LibFolder;
typedef ffi::Managed Managed;
typedef ffi::LibNotification LibNotification;
typedef ffi::NotificationType NotificationType;
typedef ffi::FolderVirtualType FolderVirtualType;
}

namespace ui {
  using ffi::dialog_request_new_folder;
  using ffi::dialog_confirm;
}
