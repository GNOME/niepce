/*
 * niepce - crates/npc-fwk/src/toolkit/channels.rs
 *
 * Copyright (C) 2021-2024 Hubert Figuière
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

use std::future::Future;

use crate::glib;

pub type Sender<T> = async_channel::Sender<T>;
pub type Receiver<T> = async_channel::Receiver<T>;

/// Create a channel for the UI.
/// This abstract the API because a minor release of glib-rs did deprecate
/// their API.
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    async_channel::unbounded::<T>()
}

pub fn receiver_attach<T, F>(rx: Receiver<T>, rcv: F)
where
    T: 'static,
    F: Fn(T) + 'static,
{
    glib::spawn_future_local(glib::clone!(
        #[strong]
        rx,
        async move {
            dbg_out!("attaching for {}", std::any::type_name::<T>());
            while let Ok(message) = rx.recv().await {
                rcv(message)
            }
            dbg_out!("terminating {}", std::any::type_name::<T>());
        }
    ));
}

/// Send to an async channel from any thread.
/// It's a macro because of the async block.
#[macro_export]
macro_rules! send_async_any {
    ($message:expr, $sender:expr) => {{
        let sender = $sender.clone();
        $crate::toolkit::channels::spawn_any(async move {
            $crate::on_err_out!(sender.send($message).await);
        })
    }};
}

/// Send to an async channel from the "local" thread.
/// It's a macro because of the async block.
#[macro_export]
macro_rules! send_async_local {
    ($message:expr, $sender:expr) => {{
        let sender = $sender.clone();
        $crate::toolkit::channels::spawn_local(async move {
            $crate::on_err_out!(sender.send($message).await);
        })
    }};
}

pub use send_async_any;
pub use send_async_local;

// This comes from https://mmstick.github.io/gtkrs-tutorials/2x01-gtk-application.html

pub fn spawn_local<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    glib::MainContext::default().spawn_local(future);
}

pub fn spawn_any<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    glib::MainContext::default().spawn(future);
}
