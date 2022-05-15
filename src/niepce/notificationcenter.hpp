/*
 * niepce - niepce/notificationcenter.hpp
 *
 * Copyright (C) 2009-2022 Hubert Figuière
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
#include <sigc++/signal.h>

#include "fwk/toolkit/notificationcenter.hpp"

#include "rust_bindings.hpp"

namespace niepce {

class NotificationCenter
  : public fwk::NotificationCenter
{
public:
  ~NotificationCenter();

  typedef std::shared_ptr<NotificationCenter> Ptr;
  static Ptr make()
    {
      Ptr nc = Ptr(new NotificationCenter());
      return nc;
    }

  sigc::signal<void(const eng::LibNotification &)> signal_lib_notification;

  const std::shared_ptr<ffi::LcChannel>& get_channel() const
    { return m_channel; };
protected:
  NotificationCenter();

private:
  static int32_t channel_callback(const eng::LibNotification *notification, void *data);
  void dispatch_lib_notification(const eng::LibNotification *n) const;
  void dispatch_notification(const fwk::Notification::Ptr &n) const;
  std::shared_ptr<ffi::LcChannel> m_channel;
};

}
/*
  Local Variables:
  mode:c++
  c-file-style:"stroustrup"
  c-file-offsets:((innamespace . 0))
  c-basic-offset: 2
  tab-width: 2
  indent-tabs-mode:nil
  fill-column:99
  End:
*/
