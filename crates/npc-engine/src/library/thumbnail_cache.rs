/*
 * niepce - library/thumbnail_cache.rs
 *
 * Copyright (C) 2020-2022 Hubert Figuière
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

use std::cmp;
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync;
use std::sync::atomic;
use std::thread;

use crate::db::libfile::{FileStatus, LibFile};
use crate::db::LibraryId;
use crate::library::notification;
use crate::library::notification::LibNotification::{FileStatusChanged, ThumbnailLoaded};
use crate::library::notification::{FileStatusChange, LcChannel, LibNotification};
use npc_fwk::toolkit;
use npc_fwk::toolkit::thumbnail::Thumbnail;
use npc_fwk::{dbg_out, err_out};

/// Thumbnail task
struct ThumbnailTask {
    /// Requested width.
    width: i32,
    /// Requested height.
    height: i32,
    /// File to generate thumbnail for.
    file: LibFile,
}

impl ThumbnailTask {
    /// Create a new ThumbnailTask
    pub fn new(file: LibFile, width: i32, height: i32) -> Self {
        ThumbnailTask {
            file,
            width,
            height,
        }
    }
}

fn get_thumbnail(f: &LibFile, w: i32, h: i32, cached: &Path) -> Thumbnail {
    let filename = f.path();
    if ThumbnailCache::is_thumbnail_cached(filename, cached) {
        dbg_out!("thumbnail for {:?} is cached!", filename);
        return Thumbnail::from(gdk_pixbuf::Pixbuf::from_file(cached).ok());
    }

    dbg_out!("creating thumbnail for {:?}", filename);
    if let Some(cached_dir) = cached.parent() {
        if let Err(err) = fs::create_dir_all(cached_dir) {
            err_out!("Coudln't create directories for {:?}: {}", cached, err);
        }
    }

    let thumbnail = Thumbnail::thumbnail_file(filename, w, h, f.orientation());
    if thumbnail.ok() {
        thumbnail.save(cached, "png");
    } else {
        dbg_out!("couldn't get the thumbnail for {:?}", filename);
    }
    thumbnail
}

type Tasks = sync::Arc<(sync::Mutex<VecDeque<ThumbnailTask>>, sync::Condvar)>;
type Running = sync::Arc<atomic::AtomicBool>;

pub struct ThumbnailCache {
    cache_dir: PathBuf,
    tasks: Tasks,
    running: Running,
    sender: async_channel::Sender<LibNotification>,
}

impl ThumbnailCache {
    fn new(dir: &Path, sender: async_channel::Sender<LibNotification>) -> Self {
        Self {
            cache_dir: PathBuf::from(dir),
            tasks: sync::Arc::new((sync::Mutex::new(VecDeque::new()), sync::Condvar::new())),
            running: sync::Arc::new(atomic::AtomicBool::new(false)),
            sender,
        }
    }

    fn execute(
        task: ThumbnailTask,
        cache_dir: &Path,
        sender: async_channel::Sender<LibNotification>,
    ) {
        let w = task.width;
        let h = task.height;
        let libfile = task.file;

        let path = libfile.path();
        let id = libfile.id();
        let dest = Self::path_for_thumbnail(path, id, cmp::max(w, h), cache_dir);

        let pix = get_thumbnail(&libfile, w, h, &dest);
        if !path.is_file() {
            dbg_out!("file doesn't exist");
            if let Err(err) = toolkit::thread_context().block_on(sender.send(FileStatusChanged(
                FileStatusChange {
                    id,
                    status: FileStatus::Missing,
                },
            ))) {
                err_out!("Sending file status change notification failed: {}", err);
            }
        }

        if !pix.ok() {
            return;
        }
        // notify the thumbnail
        if let Err(err) = toolkit::thread_context().block_on(sender.send(ThumbnailLoaded(
            notification::Thumbnail {
                id,
                width: pix.get_width(),
                height: pix.get_height(),
                pix,
            },
        ))) {
            err_out!("Sending thumbnail notification failed: {}", err);
        }
    }

    fn main(
        running: &Running,
        tasks: &Tasks,
        cache_dir: PathBuf,
        sender: async_channel::Sender<LibNotification>,
    ) {
        while running.load(atomic::Ordering::Relaxed) {
            let elem: Option<ThumbnailTask>;
            {
                let mut queue = tasks.0.lock().unwrap();
                if !queue.is_empty() {
                    elem = queue.pop_front();
                } else {
                    elem = None;
                }
            }

            if let Some(task) = elem {
                Self::execute(task, &cache_dir, sender.clone());
            }

            let queue = tasks.0.lock().unwrap();
            if queue.is_empty() {
                running.store(false, atomic::Ordering::Relaxed);
            }
        }
    }

    fn schedule(&self, task: ThumbnailTask) {
        if let Ok(ref mut q) = self.tasks.0.lock() {
            q.push_back(task);
            if !self.running.load(atomic::Ordering::Relaxed) {
                self.running.store(true, atomic::Ordering::Relaxed);
                let running = self.running.clone();
                let tasks = self.tasks.clone();
                let cache_dir = self.cache_dir.clone();
                let sender = self.sender.clone();
                thread::spawn(move || {
                    Self::main(&running, &tasks, cache_dir, sender);
                });
            }
        }
    }

    pub fn request(&self, fl: &[LibFile]) {
        for f in fl {
            self.schedule(ThumbnailTask::new(f.clone(), 160, 160));
        }
    }

    pub fn is_thumbnail_cached(_file: &Path, thumb: &Path) -> bool {
        thumb.is_file()
    }

    pub fn path_for_thumbnail(
        filename: &Path,
        id: LibraryId,
        size: i32,
        cache_dir: &Path,
    ) -> PathBuf {
        let base_name = filename.file_name().and_then(|f| f.to_str()).unwrap(); // XXX fatal if fails.
        let thumb_name = format!("{}-{}.png", id, base_name);
        let mut path = Self::dir_for_thumbnail(size, cache_dir);
        path.push(thumb_name);
        path
    }

    pub fn dir_for_thumbnail(size: i32, cache_dir: &Path) -> PathBuf {
        let subdir = if size == 0 {
            "full".to_string()
        } else {
            size.to_string()
        };
        let mut dir = PathBuf::from(cache_dir);
        dir.push(subdir);
        dir
    }
}

// cxx
/// # Safety
/// Dereference raw pointer.
pub unsafe fn thumbnail_cache_new(
    dir: &str,
    channel: *const crate::ffi::LcChannel,
) -> Box<ThumbnailCache> {
    let channel = channel as *const LcChannel;
    Box::new(ThumbnailCache::new(
        &PathBuf::from(&dir),
        (*channel).0.clone(),
    ))
}
