/*
 * niepce - ui/niepceapplication.h
 *
 * Copyright (C) 2007-2022 Hubert Figuiere
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

#ifndef _IN_RUST_BINDINGS_

#pragma once

#include "fwk/toolkit/application.hpp"

namespace ui {

class NiepceApplication
    : public fwk::Application
{
public:
    static std::shared_ptr<NiepceApplication> create(int & argc, char** & argv);

    virtual fwk::Frame::Ptr makeMainFrame() override;
    NiepceApplication(int & argc, char** & argv);
protected:

    virtual void on_action_file_open() override;
    virtual void on_about() override;
    virtual void on_action_preferences() override;
};

inline
std::shared_ptr<NiepceApplication> niepce_application_create()
{
    int argc = 0;
    char** argv = nullptr;
    return NiepceApplication::create(argc, argv);
}

}
#endif
/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  fill-column:99
  End:
*/
