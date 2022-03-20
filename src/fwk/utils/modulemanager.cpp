/*
 * niepce
 * copied from
 * gnote
 *
 * Copyright (C) 2009-2022 Hubert Figuière
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */



#include <dlfcn.h>

#include <gmodule.h>
#include <glibmm/module.h>

#include "fwk/utils/files.hpp"
#include "fwk/utils/pathutils.hpp"
#include "fwk/base/debug.hpp"
#include "fwk/base/map.hpp"
#include "fwk/base/string.hpp"
#include "dynamicmodule.hpp"
#include "modulemanager.hpp"

namespace fwk {


  ModuleManager::~ModuleManager()
  {
    for(ModuleList::const_iterator mod_iter = m_modules.cbegin();
        mod_iter != m_modules.cend(); ++mod_iter) {
      delete *mod_iter;
    }
  }


  void ModuleManager::add_path(const std::string & dir)
  {
    m_dirs.insert(dir);
    DBG_OUT("add path %s", dir.c_str());
  }


  static bool filter(GFileInfo* f)
  {
    return fwk::filter_ext(Glib::wrap(f, true), std::string(".") + G_MODULE_SUFFIX);
  }

  void ModuleManager::load_modules()
  {
    for(auto iter = m_dirs.cbegin();
        iter != m_dirs.cend(); ++iter) {

      fwk::FileListPtr l;
      l = wrapFileList(ffi::fwk_file_list_get_files_from_directory(
                         iter->c_str(), filter));

      for(size_t i = 0; i < ffi::fwk_file_list_size(l.get()); i++) {

        auto file_path = fwk::RustFfiString(ffi::fwk_file_list_at(l.get(), i));
        Glib::Module module(*iter + "/" + path_basename(file_path.c_str()),
                            Glib::Module::Flags::LOCAL);
        DBG_OUT("load module %s", path_basename(file_path.c_str()).c_str());

        if(!module) {
          DBG_OUT("error loading %s", Glib::Module::get_last_error().c_str());
          continue;
        }
        void * func = NULL;
        bool found = module.get_symbol("dynamic_module_instanciate", func);

        if(!found) {
          DBG_OUT("error getting symbol %s", Glib::Module::get_last_error().c_str());
          continue;
        }
        instanciate_func_t real_func = (instanciate_func_t)func;
        DynamicModule * dmod = (*real_func)();

        if(dmod) {
          m_modules.push_back(dmod);
          module.make_resident();
        }
      }

    }
  }

}
