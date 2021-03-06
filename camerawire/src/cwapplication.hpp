/*
 * niepce - cwwindow.hpp
 *
 * Copyright (C) 2009-2014 Hubert Figuiere
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

#ifndef __CW_APPLICATION_HPP_
#define __CW_APPLICATION_HPP_


#include "fwk/toolkit/application.hpp"



namespace cw {

class CwApplication
  : public fwk::Application
{
public:
  static fwk::Application::Ptr create(int & argc, char** & argv);

  virtual fwk::AppFrame::Ptr makeMainFrame() override;
protected:
  CwApplication(int & argc, char** & argv);

  virtual void on_action_preferences() override;

};

}


#endif
