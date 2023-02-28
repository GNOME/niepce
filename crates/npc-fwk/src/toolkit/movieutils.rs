/*
 * niepce - npc-fwk/toolkit/movieutils.rs
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
use std::path::Path;
use std::process::Command;

pub fn thumbnail_movie<S, D>(source: S, w: u32, h: u32, dest: D) -> bool
where
    S: AsRef<Path> + std::fmt::Debug,
    D: AsRef<Path>,
{
    Command::new("totem-video-thumbnailer")
        .arg("-s")
        .arg(format!("{}", cmp::max(w, h)))
        .arg(source.as_ref().as_os_str())
        .arg(dest.as_ref().as_os_str())
        .status()
        .map(|s| s.success())
        .unwrap_or_else(|e| {
            err_out!("Failed to thumbnail {:?}: {:?}", source, e);
            false
        })
}
