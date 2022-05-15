/*
 * niepce - fwk/toolkit/notification.hpp
 *
 * Copyright (C) 2007-2022 Hubert Figuière
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
#include <sigc++/trackable.h>
#include <sigc++/signal.h>

#include "fwk/toolkit/notification.hpp"

namespace fwk {

class NotificationCenter
    : public sigc::trackable
    , public std::enable_shared_from_this<NotificationCenter>
{
public:
    typedef std::shared_ptr< NotificationCenter > Ptr;
    typedef sigc::slot<void(Notification::Ptr)> subscriber_t;

    ~NotificationCenter();

    // called from out of thread
    void post(Notification::Ptr && n);

    void subscribe(int type, const subscriber_t & );
    void unsubscribe(int type, const subscriber_t & );

protected:
    NotificationCenter();

private:
    typedef sigc::signal<void(Notification::Ptr)> subscription_t;

    void _dispatch(void);

    class Priv;
    Priv *p;
};

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
