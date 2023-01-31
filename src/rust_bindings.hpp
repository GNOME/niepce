/*
 * niepce - rust_bindings.hpp
 *
 * Copyright (C) 2017-2023 Hubert Figuière
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
typedef fwk::PropertyValue PropertyValue;
typedef fwk::RgbColour RgbColour;
typedef fwk::WrappedPropertyBag WrappedPropertyBag;
typedef eng::Label Label;
typedef eng::LibFile LibFile;
typedef eng::LibMetadata LibMetadata;
typedef fwk::PropertyBag NiepcePropertyBag;
typedef fwk::PropertySet NiepcePropertySet;
typedef eng::LibNotification LibNotification;
typedef eng::LibraryClientWrapper LibraryClientWrapper;
}

namespace fwk {
typedef rust::Box<fwk::Date> DatePtr;
typedef rust::Box<Thumbnail> ThumbnailPtr;
typedef rust::Box<RgbColour> RgbColourPtr;
typedef rust::Box<PropertyValue> PropertyValuePtr;

typedef rust::Box<PropertyBag> PropertyBagPtr;
typedef rust::Box<PropertySet> PropertySetPtr;
}

namespace eng {
typedef rust::Box<Label> LabelPtr;
typedef std::vector<LabelPtr> LabelList;
typedef rust::Box<LibFile> LibFilePtr;
typedef int64_t library_id_t;
typedef rust::Box<LibraryClientHost> LibraryClientPtr;
typedef rust::Box<UIDataProvider> UIDataProviderPtr;
}

#undef _IN_RUST_BINDINGS_
