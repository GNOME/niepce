/*
 * exiv2-rs - src/npc_exiv2.h
 *
 * Copyright (C) 2026 Hubert Figuière
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

#include <memory>
#include <string>

#include <exiv2/xmp_exiv2.hpp>
#include <exiv2/iptc.hpp>
#include <exiv2/exif.hpp>
#include <exiv2/image.hpp>

#include "rust/cxx.h"

namespace Exiv2 {

  Image::UniquePtr ImageFactory_open(rust::Str path, bool useCurl = true) {
    return ImageFactory::open(std::string(path), useCurl);
  }

  Image::UniquePtr ImageFactory_create(ImageType type) {
    return ImageFactory::create(type);
  }

  TypeId XmpProperties_type(rust::Str tag) {
    return XmpProperties::propertyType(XmpKey(std::string(tag)));
  }

  TypeId IptcDataSets_type(rust::Str tag) {
    IptcKey key = IptcKey(std::string(tag));
    return Exiv2::IptcDataSets::dataSetType(key.tag(), key.record());
  }

  std::unique_ptr<ExifKey> ExifKey_new(rust::Str key) {
    return std::make_unique<ExifKey>(std::string(key));
  }

  std::unique_ptr<XmpKey> XmpKey_new(rust::Str key) {
    return std::make_unique<XmpKey>(std::string(key));
  }

  std::unique_ptr<IptcKey> IptcKey_new(rust::Str key) {
    return std::make_unique<IptcKey>(std::string(key));
  }

  ByteOrder ExifParser_decode(ExifData& exifData, const byte* pData, size_t size) {
    return ExifParser::decode(exifData, pData, size);
  }

  const Metadatum &ExifData_get(const ExifData& data, rust::Str key) {
    auto iter = data.findKey(ExifKey(std::string(key)));
    if (iter == data.end()) {
      throw std::exception();
    }
    return *iter;
  }

  rust::Vec<rust::String> ExifData_tags(const ExifData& data) {
    rust::Vec<rust::String> tags;
    for (auto datum : data) {
      tags.push_back(datum.key());
    }
    return tags;
  }

  const Metadatum &IptcData_get(const IptcData& data, rust::Str key) {
    auto iter = data.findKey(IptcKey(std::string(key)));
    if (iter == data.end()) {
      throw std::exception();
    }
    return *iter;
  }

  rust::Vec<rust::String> IptcData_tags(const IptcData& data) {
    rust::Vec<rust::String> tags;
    for (auto datum : data) {
      tags.push_back(datum.key());
    }
    return tags;
  }

  const Metadatum &XmpData_get(const XmpData& data, rust::Str key) {
    auto iter = data.findKey(XmpKey(std::string(key)));
    if (iter == data.end()) {
      throw std::exception();
    }
    return *iter;
  }

  rust::Vec<rust::String> XmpData_tags(const XmpData& data) {
    rust::Vec<rust::String> tags;
    for (auto datum : data) {
      tags.push_back(datum.key());
    }
    return tags;
  }

  int64_t Metadatum_toInt64(const Metadatum &datum, int64_t n) {
    return datum.toInt64(n);
  }

  void Metadatum_toRational(const Metadatum &datum, int32_t &num, int32_t &den) {
    auto rational = datum.toRational();
    num = rational.first;
    den = rational.second;
  }

  rust::String Metadatum_toString(const Metadatum &datum) {
    return rust::String(datum.toString());
  }

  rust::String Metadatum_convertToString(const Metadatum &datum) {
    std::ostringstream os;
    datum.write(os);
    return rust::String(os.str());
  }
}
