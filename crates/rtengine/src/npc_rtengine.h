/*
 * niepce - npc_rtengine.h
 *
 * Copyright (C) 2023-2024 Hubert Figuière
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
#include <string>
#include "rtengine/rtengine.h"
#include "rtengine/imageio.h"
#include "rtengine/profilestore.h"
#include "rtgui/options.h"

namespace rtengine {

  void init_();

  inline
  void Options_load() {
    // false mean load everything.
    Options::load(false);
  }

  inline
  std::unique_ptr<InitialImage> InitialImage_load (const std::string& fname, bool isRaw, int& errorCode) {
    return std::unique_ptr<InitialImage>(InitialImage::load(fname, isRaw, &errorCode, nullptr));
  }

  inline
  void decrease_ref(InitialImage* image) {
    image->decreaseRef();
  }

  inline
  std::unique_ptr<ImageIO> process_image (ProcessingJob* job, int& errorCode, bool flush = false) {
    return std::unique_ptr<ImageIO>(dynamic_cast<ImageIO*>(processImage (job, errorCode, nullptr, flush)));
  }

  inline
  int image_io_width(const ImageIO& image) {
    return image.getWidth();
  }

  inline
  int image_io_height(const ImageIO& image) {
    return image.getHeight();
  }


  inline
  std::unique_ptr<ProcessingJob> ProcessingJob_create (InitialImage& ii, const procparams::ProcParams& pparams, bool fast = false) {
    return std::unique_ptr<ProcessingJob>(ProcessingJob::create (&ii, pparams, fast));
  }

  inline
  void ProcessingJob_destroy(ProcessingJob* job) {
    ProcessingJob::destroy(job);
  }

  inline
  std::unique_ptr<procparams::PartialProfile> ProfileStore_load_dynamic_profile (const FramesMetaData* metadata, const std::string& input_file) {
    return std::unique_ptr<procparams::PartialProfile>(ProfileStore::getInstance()->loadDynamicProfile(metadata, input_file));
  }


  namespace procparams {
    using LcMode = LensProfParams::LcMode;

    inline
    void partial_profile_apply_to(const std::unique_ptr<PartialProfile>& profile, ProcParams& params, bool from_last_saved) {
      profile->applyTo(&params, from_last_saved);
    }

    inline
    void ProcParams_set_lcmode(ProcParams& params, LcMode mode) {
      params.lensProf.lcMode = mode;
    }

    inline
    std::unique_ptr<procparams::ProcParams> ProcParams_new() {
      return std::make_unique<procparams::ProcParams>();
    }
  }
}
