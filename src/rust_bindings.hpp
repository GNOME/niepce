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

#define _IN_RUST_BINDINGS_

#include <memory>
#include <vector>

#include <gtk/gtk.h>

#include "fwk/cxx_fwk_bindings.hpp"
#include "fwk/cxx_eng_bindings.hpp"
#include "fwk/cxx_npc_bindings.hpp"

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
typedef fwk::WrappedPropertyBag WrappedPropertyBag;
typedef eng::Label Label;
typedef eng::LibFile LibFile;
typedef eng::LibFolder LibFolder;
typedef eng::LibMetadata LibMetadata;
typedef eng::Keyword Keyword;
typedef fwk::PropertyBag NiepcePropertyBag;
typedef fwk::PropertySet NiepcePropertySet;
typedef eng::FolderVirtualType FolderVirtualType;
typedef eng::ThumbnailCache ThumbnailCache;
typedef eng::LcChannel LcChannel;
typedef eng::LibNotification LibNotification;
typedef npc::LibraryClientWrapper LibraryClientWrapper;
}

#include "target/eng_bindings.h"
#include "target/bindings.h"

namespace fwk {
typedef rust::Box<fwk::Date> DatePtr;
typedef rust::Box<Thumbnail> ThumbnailPtr;
typedef rust::Box<FileList> FileListPtr;
typedef rust::Box<RgbColour> RgbColourPtr;
typedef rust::Box<PropertyValue> PropertyValuePtr;

typedef rust::Box<PropertyBag> PropertyBagPtr;
typedef rust::Box<PropertySet> PropertySetPtr;
}

namespace eng {
typedef rust::Box<Keyword> KeywordPtr;
typedef rust::Box<Label> LabelPtr;
typedef std::vector<LabelPtr> LabelList;
typedef rust::Box<LibFile> LibFilePtr;

typedef ffi::NiepcePropertyIdx Np;
using NiepcePropertyIdx = ffi::NiepcePropertyIdx;
typedef ffi::LibraryId library_id_t; // XXX change this to LibraryId
typedef ffi::Managed Managed;
typedef ffi::NotificationType NotificationType;
}

namespace libraryclient {
typedef rust::Box<npc::LibraryClientHost> LibraryClientPtr;
}

namespace npc {
typedef rust::Box<UIDataProvider> UIDataProviderPtr;
typedef rust::Box<NotificationCenter> NotificationCenterPtr;
}

#undef _IN_RUST_BINDINGS_
