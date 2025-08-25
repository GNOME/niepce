/*
 * niepce - library/thumbnail_cache.rs
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

use std::cmp;
use std::fs;
use std::path::{Path, PathBuf};

use crate::catalog;
use crate::catalog::libfile::{FileStatus, LibFile};
use crate::library::notification;
use crate::library::notification::LibNotification::{
    FileStatusChanged, ImageRendered, ThumbnailLoaded,
};
use crate::library::notification::{FileStatusChange, LcChannel};
use crate::library::previewer::{Cache, RenderMsg, RenderParams, RenderSender, RenderType};
use npc_fwk::base::Size;
use npc_fwk::toolkit;
use npc_fwk::toolkit::ImageBitmap;
use npc_fwk::toolkit::thumbnail::Thumbnail;
use npc_fwk::{dbg_out, err_out, on_err_out};

/// Suffix to add to the stem catalog file name.
const THUMBCACHE_SUFFIX: &str = "-thumbcache";

/// Previewing task
struct Task {
    /// Params for the rendering task
    params: RenderParams,
    /// File to generate thumbnail for.
    file: LibFile,
    processor: Option<RenderSender>,
}

impl Task {
    /// Create a new thumbnailing Task
    pub fn new_thumbnail(file: LibFile, params: RenderParams) -> Self {
        Task {
            params,
            file,
            processor: None,
        }
    }

    pub fn new_rendering(
        file: LibFile,
        params: RenderParams,
        processor: Option<RenderSender>,
    ) -> Self {
        Task {
            params,
            file,
            processor,
        }
    }
}

/// Check the file status (ie is it still present?) and report.
fn check_file_status(id: catalog::LibraryId, path: &Path, sender: &LcChannel) {
    if !path.is_file() {
        dbg_out!("file doesn't exist");
        if let Err(err) =
            toolkit::thread_context().block_on(sender.send(FileStatusChanged(FileStatusChange {
                id,
                status: FileStatus::Missing,
            })))
        {
            err_out!("Sending file status change notification failed: {}", err);
        }
    }
}

/// Get a rendered preview
fn get_preview(cache: &Cache, task: &Task, sender: &LcChannel) -> Option<ImageBitmap> {
    let libfile = &task.file;
    let filename = libfile.path().to_string_lossy();
    let dimensions = task.params.dimensions;
    let dimension = cmp::max(dimensions.w, dimensions.h);
    let digest = task.params.digest();

    let rel_dest = cache.path_for_thumbnail(libfile.path(), libfile.id(), &digest)?;
    let dest = cache.cache_dir().to_path_buf().join(rel_dest);
    dbg_out!("Found preview entry {dest:?}");
    if dest.exists() {
        dbg_out!("found {filename} in the cache fs: {dest:?}");
        cache.hit(&filename, &digest);
        return ImageBitmap::from_file(dest).ok();
    }

    if let Ok(cache_item) = cache.get(&filename, &digest) {
        // cache hit
        dbg_out!("thumbnail for {:?} is cached!", filename);
        let target = if cache_item.target.is_relative() {
            cache.cache_dir().to_path_buf().join(cache_item.target)
        } else {
            cache_item.target
        };
        if target.exists() {
            return ImageBitmap::from_file(target).ok();
        } else {
            dbg_out!("File is missing");
        }
    }

    dbg_out!("creating preview for {:?}", filename);
    if let Some(cached_dir) = dest.parent() {
        if let Err(err) = fs::create_dir_all(cached_dir) {
            err_out!("Coudln't create directories for {:?}: {}", dest, err);
        }
    }

    // Run the pipeline
    if let Some(processor) = &task.processor {
        let sender = sender.clone();
        let rendering = task.params.clone();
        let cache_sender = cache.sender();
        let filename = filename.to_string();
        let id = task.file.id();
        on_err_out!(processor.send(RenderMsg::GetBitmap(Box::new(move |pix| {
            if let Err(err) = toolkit::thread_context().block_on(sender.send(ImageRendered(
                notification::ImageRendered {
                    id,
                    image: pix.clone(),
                },
            ))) {
                err_out!("Sending image rendered notification failed: {}", err);
            }
            on_err_out!(pix.save_png(&dest));
            on_err_out!(cache_sender.send(super::previewer::DbMessage::Put(
                filename.clone(),
                dimension,
                rendering.clone(),
                dest.to_string_lossy().to_string(),
            )));
        }))));
    } else {
        err_out!("no processor");
    }
    None
}

fn get_thumbnail(cache: &Cache, task: &Task, libfile: &LibFile) -> Option<Thumbnail> {
    let filename = libfile.path().to_string_lossy();
    let dimensions = task.params.dimensions;
    let dimension = cmp::max(dimensions.w, dimensions.h);
    let digest = task.params.digest();
    // true if we found a cache entry but no file.
    let mut is_missing = false;

    let rel_dest = cache.path_for_thumbnail(libfile.path(), libfile.id(), &digest)?;
    let dest = cache.cache_dir().to_path_buf().join(&rel_dest);
    if dest.exists() {
        cache.hit(&filename, &digest);
        return Thumbnail::load(dest).ok();
    }

    if let Ok(cache_item) = cache.get(&filename, &digest) {
        // cache hit
        dbg_out!("thumbnail for {:?} is cached!", filename);
        let target = if cache_item.target.is_relative() {
            cache.cache_dir().to_path_buf().join(cache_item.target)
        } else {
            cache_item.target
        };
        if target.exists() {
            return Thumbnail::load(dest).ok();
        } else {
            dbg_out!("File is missing");
            is_missing = true;
        }
    }

    dbg_out!("creating thumbnail for {:?}", filename);
    if let Some(cached_dir) = dest.parent() {
        if let Err(err) = fs::create_dir_all(cached_dir) {
            err_out!("Coudln't create directories for {:?}: {}", dest, err);
        }
    }

    let thumbnail = Thumbnail::thumbnail_file(
        libfile.path(),
        dimensions.w,
        dimensions.h,
        Some(libfile.orientation()),
    );
    if let Some(ref thumbnail) = thumbnail {
        thumbnail.save_png(&dest);
        if !is_missing {
            // We don't need to try to add it back
            // XXX maybe should we update the create date if it was recreated?
            cache.put(
                &filename,
                dimension,
                task.params.clone(),
                &rel_dest.to_string_lossy(),
            );
        }
    } else {
        dbg_out!("couldn't get the preview for {:?}", filename);
    }
    thumbnail
}

enum Request {
    Terminate,
    Task(Vec<Task>),
}

pub struct ThumbnailCache {
    /// Queue to send task
    queue_sender: std::sync::mpsc::Sender<Request>,
}

impl ThumbnailCache {
    pub fn new(dir: &Path, sender: LcChannel) -> Self {
        let (queue_sender, queue) = std::sync::mpsc::channel();
        let cache_dir = PathBuf::from(dir);
        on_err_out!(
            std::thread::Builder::new()
                .name("thumbnail cache".to_string())
                .spawn(move || {
                    Self::main(cache_dir, queue, sender);
                })
        );

        Self { queue_sender }
    }

    pub fn close(&self) {
        on_err_out!(self.queue_sender.send(Request::Terminate));
    }

    /// Build a path for the cache directory based on the catalog path.
    pub fn path_from_catalog(catalog_path: &Path) -> Option<PathBuf> {
        let mut cache_name = catalog_path.file_stem()?.to_os_string();
        cache_name.push(THUMBCACHE_SUFFIX);
        Some(std::path::PathBuf::from(catalog_path.parent()?).join(cache_name))
    }

    fn execute(task: &Task, cache: &Cache, sender: &LcChannel) {
        let libfile = &task.file;
        let id = libfile.id();
        let path = libfile.path();
        // We shall report if the file is missing.
        check_file_status(id, path, sender);

        match task.params.type_ {
            RenderType::Preview => {
                if let Some(pix) = get_preview(cache, task, sender) {
                    dbg_out!("Got the preview from the cache");
                    if let Err(err) = toolkit::thread_context().block_on(sender.send(
                        ImageRendered(notification::ImageRendered { id, image: pix }),
                    )) {
                        err_out!("Sending image rendered notification failed: {}", err);
                    }
                }
            }
            RenderType::Thumbnail => {
                if let Some(pix) = get_thumbnail(cache, task, libfile) {
                    // notify the thumbnail
                    if let Err(err) = toolkit::thread_context().block_on(sender.send(
                        ThumbnailLoaded(Box::new(notification::Thumbnail { id, pix })),
                    )) {
                        err_out!("Sending thumbnail notification failed: {}", err);
                    }
                } else {
                    err_out!("Failed to get thumbnail for {:?}", path);
                }
            }
        }
    }

    fn main(cache_dir: PathBuf, queue: std::sync::mpsc::Receiver<Request>, sender: LcChannel) {
        let cache = Cache::new(cache_dir);
        cache.initialize();
        dbg_out!("Cache database ready");
        while let Ok(tasks) = queue.recv() {
            match tasks {
                Request::Task(tasks) => {
                    dbg_out!("Parallel thumbnailing of {} files", tasks.len());
                    tasks.iter().for_each(|task| {
                        Self::execute(task, &cache, &sender);
                    })
                }
                Request::Terminate => break,
            }
        }
        dbg_out!("thumbnail cache thread terminating");
    }

    /// Request a render.
    pub fn request_render(
        &self,
        file: LibFile,
        params: RenderParams,
        processor: Option<RenderSender>,
    ) {
        on_err_out!(
            self.queue_sender
                .send(Request::Task(vec![Task::new_rendering(
                    file, params, processor
                )]))
        );
    }

    /// Request thumbnails.
    pub fn request(&self, fl: &[LibFile]) {
        on_err_out!(
            self.queue_sender.send(Request::Task(
                fl.iter()
                    .map(|f| Task::new_thumbnail(
                        f.clone(),
                        RenderParams::new_thumbnail(f.id(), Size { w: 160, h: 160 })
                    ))
                    .collect()
            ))
        );
    }
}
