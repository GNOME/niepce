/*
 * niepce - npc-fwk/toolkit/movieutils.rs
 *
 * Copyright (C) 2020-2025 Hubert Figuière
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

//! Various utilities for movies. Currently only a thumbnailer.

use std::path::Path;

use crate::{gdk_pixbuf, glib};
use glib::prelude::*;
use gstreamer as gst;
use gstreamer::prelude::*;

use crate::toolkit::gdk_utils::gdkpixbuf_scale_to_fit;

/// Video thumbnailer using Gstreamer
/// Largely inspired from totem thumbnailer.
struct Thumbnailer {
    player: Option<gstreamer::Bin>,
}

impl Drop for Thumbnailer {
    /// Ensure we set the state to NULL on drop
    fn drop(&mut self) {
        if let Some(player) = &self.player {
            let _ = player.set_state(gst::State::Null);
        }
    }
}

impl Thumbnailer {
    fn new<P: AsRef<Path>>(input: P) -> Self {
        let player = gstreamer::ElementFactory::find("playbin")
            .and_then(|factory| factory.create_with_name(Some("play")).ok());
        let video_sink = gstreamer::ElementFactory::find("fakesink")
            .and_then(|factory| factory.create_with_name(Some("video-fake-sink")).ok())
            .inspect(|video_sink| video_sink.set_property("sync", true));
        player.as_ref().inspect(|player| {
            player.set_property("video-sink", &video_sink);
            if let Ok(uri) = glib::filename_to_uri(input, None) {
                player.set_property("uri", uri);
            }
        });
        Thumbnailer {
            player: player.and_downcast::<gst::Bin>(),
        }
    }

    /// If the thumbnailer has been loaded
    fn is_ok(&self) -> bool {
        self.player.is_some()
    }

    fn seek(&self, t: gst::ClockTime) {
        self.player.as_ref().inspect(|player| {
            let _ = player.seek(
                1.0,
                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                gst::SeekType::Set,
                t,
                gst::SeekType::None,
                gst::ClockTime::NONE,
            );
            let _ = player.state(gst::ClockTime::NONE);
        });
    }

    /// Get a the frame at the current seek position.
    fn get_frame(&self) -> Option<gdk_pixbuf::Pixbuf> {
        self.player.as_ref().and_then(|player| {
            let is_gl = player.by_name("glcolorbalance0").is_some();
            let to_caps = gst::Caps::builder("video/x-raw")
                .field("format", if is_gl { "RGBA" } else { "RGB" })
                .field("pixel-aspect-ratio", gst::Fraction::new(1, 1))
                .build();
            let sample: Option<gst::Sample> = player.emit_by_name("convert-sample", &[&to_caps]);
            sample.and_then(|sample| {
                let caps = sample.caps()?;
                let s = caps.structure(0)?;
                let out_w = s.get::<i32>("width").ok()?;
                let out_h = s.get::<i32>("height").ok()?;
                let bpp = if is_gl { 4 } else { 3 };

                let bytes = glib::Bytes::from_owned(
                    sample.buffer_owned()?.into_mapped_buffer_readable().ok()?,
                );
                let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(
                    &bytes,
                    gdk_pixbuf::Colorspace::Rgb,
                    is_gl,
                    8,
                    out_w,
                    out_h,
                    out_w * bpp,
                );
                Some(pixbuf)
            })
        })
    }

    fn capture_frame_at(&self, milli: u64) -> Option<gdk_pixbuf::Pixbuf> {
        if milli != 0 {
            self.seek(gst::ClockTime::from_mseconds(milli));
        }
        self.get_frame()
    }

    fn capture_frame(&self, duration: Option<u64>) -> Option<gdk_pixbuf::Pixbuf> {
        if duration.is_none() {
            self.capture_frame_at(0)
        } else {
            // XXX we should pick a frame that has some value
            // XXX in case it start with a uniform colour frame.
            self.capture_frame_at(0)
        }
    }

    fn duration(&self) -> Option<u64> {
        self.player
            .as_ref()
            .and_then(|player| player.query_duration::<gst::ClockTime>())
            .map(|clocktime| clocktime.seconds())
    }

    /// Create the thumbnail.
    ///
    /// # Panic
    /// Will panic if `self.player` is `None`.
    fn thumbnail(&self, w: u32, h: u32) -> Option<gdk_pixbuf::Pixbuf> {
        let player = self.player.as_ref().expect("player not initialised.");

        let _ = player.set_state(gst::State::Paused);
        let mut terminate = false;
        let mut async_received = false;
        let bus = player.bus();
        if bus.is_none() {
            err_out!("Can't get bus");
            return None;
        }
        let bus = bus.unwrap();
        let events = [gst::MessageType::AsyncDone, gst::MessageType::Error];
        while !terminate {
            let message = bus.timed_pop_filtered(None, &events);
            if let Some(message) = message {
                let source = message.src();
                match message.type_() {
                    gst::MessageType::AsyncDone => {
                        if source == self.player.as_ref().map(|o| o.upcast_ref()) {
                            async_received = true;
                            terminate = true;
                        }
                    }
                    gst::MessageType::Error => {
                        err_out!("gst error: {message:?}");
                        terminate = true;
                    }
                    _ => {}
                }
            }
        }

        if !async_received {
            err_out!("no async");
            return None;
        }

        let duration = self.duration();
        gdkpixbuf_scale_to_fit(self.capture_frame(duration).as_ref(), std::cmp::max(w, h))
    }
}

/// Thumbnail a move file at path.
///
/// Returns an pixbuf on succes.
pub fn thumbnail_movie<S>(source: S, w: u32, h: u32) -> Option<gdk_pixbuf::Pixbuf>
where
    S: AsRef<Path> + std::fmt::Debug,
{
    let thumbnailer = Thumbnailer::new(source);

    if !thumbnailer.is_ok() {
        err_out!("Thumbnailer is not ok");
        return None;
    }
    thumbnailer.thumbnail(w, h)
}
