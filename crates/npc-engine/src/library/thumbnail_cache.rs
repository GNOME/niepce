/*
 * niepce - library/thumbnail_cache.rs
 *
 * Copyright (C) 2020-2023 Hubert Figuière
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
use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::db::libfile::{FileStatus, LibFile};
use crate::db::LibraryId;
use crate::library::notification;
use crate::library::notification::LibNotification::{FileStatusChanged, ThumbnailLoaded};
use crate::library::notification::{FileStatusChange, LcChannel};
use npc_fwk::toolkit;
use npc_fwk::toolkit::thumbnail::Thumbnail;
use npc_fwk::{dbg_out, err_out, on_err_out};

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

pub struct ThumbnailCache {
    /// Queue to send task
    queue_sender: std::sync::mpsc::Sender<Vec<ThumbnailTask>>,
}

impl ThumbnailCache {
    pub fn new(dir: &Path, sender: LcChannel) -> Self {
        let (queue_sender, queue) = std::sync::mpsc::channel();
        let cache_dir = PathBuf::from(dir);
        on_err_out!(std::thread::Builder::new()
            .name("thumbnail cache".to_string())
            .spawn(move || {
                Self::main(cache_dir, queue, sender);
            }));

        Self { queue_sender }
    }

    fn execute(task: &ThumbnailTask, cache_dir: &Path, sender: &LcChannel) {
        let w = task.width;
        let h = task.height;
        let libfile = &task.file;

        let path = libfile.path();
        let id = libfile.id();
        if let Some(dest) = Self::path_for_thumbnail(path, id, cmp::max(w, h), cache_dir) {
            let pix = get_thumbnail(libfile, w, h, &dest);
            if !path.is_file() {
                dbg_out!("file doesn't exist");
                if let Err(err) = toolkit::thread_context().block_on(sender.send(
                    FileStatusChanged(FileStatusChange {
                        id,
                        status: FileStatus::Missing,
                    }),
                )) {
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
        } else {
            err_out!("Failed to get thumbnail for {:?}", path);
        }
    }

    fn main(
        cache_dir: PathBuf,
        queue: std::sync::mpsc::Receiver<Vec<ThumbnailTask>>,
        sender: LcChannel,
    ) {
        while let Ok(tasks) = queue.recv() {
            tasks.iter().for_each(|task| {
                Self::execute(task, &cache_dir, &sender);
            })
        }
        dbg_out!("thumbnail cache thread terminating");
    }

    /// Request thumbnails.
    pub fn request(&self, fl: &[LibFile]) {
        on_err_out!(self.queue_sender.send(
            fl.par_iter()
                .map(|f| ThumbnailTask::new(f.clone(), 160, 160))
                .collect()
        ));
    }

    fn is_thumbnail_cached(_file: &Path, thumb: &Path) -> bool {
        thumb.is_file()
    }

    fn path_for_thumbnail(
        filename: &Path,
        id: LibraryId,
        size: i32,
        cache_dir: &Path,
    ) -> Option<PathBuf> {
        // XXX properly report the error
        let base_name = filename.file_name().and_then(|f| f.to_str())?;
        let thumb_name = format!("{id}-{base_name}.png");
        let mut path = Self::dir_for_thumbnail(size, cache_dir);
        path.push(thumb_name);
        Some(path)
    }

    fn dir_for_thumbnail(size: i32, cache_dir: &Path) -> PathBuf {
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
