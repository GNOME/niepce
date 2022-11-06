/*
 * niepce - main/main.cpp
 *
 * Copyright (C) 2007-2022 Hubert Figuière
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  
 * 02110-1301, USA
 */


#include "config.h"

#include <glibmm/i18n.h>

#include "fwk/base/debug.hpp"
#include "fwk/utils/init.hpp"
#include "fwk/utils/exempi.hpp"
#include "ui/niepceapplication.hpp"

int main(int argc, char ** argv)
{
  bindtextdomain(GETTEXT_PACKAGE, NIEPCE_LOCALEDIR);
  bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8");
  textdomain(GETTEXT_PACKAGE);

  fwk::utils::init();

  auto manager = fwk::ExempiManager_new();

  DBG_OUT("Starting up. DEBUG is on");
  fwk::Application::Ptr app = ui::NiepceApplication::create(argc, argv);
  return fwk::Application::main(app, argc, argv);
}


