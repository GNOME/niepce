/*
 * niepce - niepce/notification_center.rs
 *
 * Copyright (C) 2022-2024 Hubert Figuière
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

use npc_fwk::glib;

use npc_engine::library::notification::{LcChannel, LibNotification};
use npc_fwk::base::Signal;

pub struct NotificationCenter {
    channel: LcChannel,
    pub signal_notify: Signal<LibNotification>,
}

impl Default for NotificationCenter {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationCenter {
    pub fn new() -> NotificationCenter {
        let (sender, receiver) = async_channel::unbounded();
        let nc = NotificationCenter {
            channel: sender,
            signal_notify: Signal::new(),
        };

        let signal_notify = nc.signal_notify.clone();
        let event_handler = async move {
            while let Ok(n) = receiver.recv().await {
                signal_notify.emit(n);
            }
        };
        glib::MainContext::default().spawn_local(event_handler);

        nc
    }

    /// Get the sender channel
    pub fn channel(&self) -> &LcChannel {
        &self.channel
    }
}
