/*
 * niepce - fwk/toolkit/controller.cpp
 *
 * Copyright (C) 2007-2017 Hubert Figuière
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

#include <gtkmm/widget.h>

#include "fwk/base/debug.hpp"
#include "controller.hpp"


namespace fwk {

Controller::Controller()
{
}


Controller::~Controller()
{
    DBG_DTOR;
}


void
Controller::add(const Controller::Ptr & sub)
{
    m_subs.push_back(sub);
    sub->_added();
}

void Controller::remove(const Ptr & sub)
{
    auto iter = std::find(m_subs.cbegin(), m_subs.cend(), sub);
    if(iter != m_subs.cend()) {
        m_subs.erase(iter);
    }
}

bool Controller::canTerminate()
{
    return true;
}

void Controller::terminate()
{
    DBG_OUT("terminating controller");
    std::for_each(m_subs.cbegin(), m_subs.cend(),
                  [] (const Ptr& ctrl) {
                    ctrl->terminate();
                  });
    m_subs.clear();
}

void Controller::_added()
{
}

void Controller::_ready()
{
    std::for_each(m_subs.cbegin(), m_subs.cend(),
                  [] (const Ptr& ctrl) {
                    ctrl->_ready();
                  });
    on_ready();
}

void Controller::on_ready()
{
}

}

/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  indent-tabs-mode:nil
  fill-column:99
  End:
*/
