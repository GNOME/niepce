/*
 * niepce - fwk/toolkit/mimetype.rs
 *
 * Copyright (C) 2017-2025 Hubert Figuière
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

use std::convert::AsRef;
use std::path::Path;

use crate::gio;

/// MType image format
#[derive(PartialEq, Copy, Clone, Debug, Eq)]
pub enum ImgFormat {
    /// Digital camera raw
    Raw,
    /// JPEG
    Jpeg,
    /// HEIF
    Heif,
    /// AVIF
    Avif,
}

/// Mime type
#[derive(PartialEq, Copy, Clone, Debug, Eq)]
pub enum MType {
    None,
    /// Still Image
    Image(ImgFormat),
    /// Movie
    Movie,
    /// XMP side car
    Xmp,
    /// Thumbnail file (like Canon THM).
    Thumbnail,
}

#[derive(Debug)]
pub struct MimeType(MType);

/// Guess the type from the gio type string
pub fn guess_type(gmtype: &str) -> MType {
    if gio::content_type_is_a(gmtype, "image/*") {
        if gio::content_type_is_a(gmtype, "image/x-dcraw") {
            return MType::Image(ImgFormat::Raw);
        } else if gio::content_type_is_a(gmtype, "image/heif") {
            return MType::Image(ImgFormat::Heif);
        } else if gio::content_type_is_a(gmtype, "image/avif") {
            return MType::Image(ImgFormat::Avif);
        }
        return MType::Image(ImgFormat::Jpeg);
    } else if gio::content_type_is_a(gmtype, "video/*") {
        return MType::Movie;
    }
    MType::None
}

/// Guess the type from a file
pub fn guess_type_for_file<P: AsRef<Path>>(p: P) -> MType {
    let path = p.as_ref();
    if let Some(ext) = path.extension().map(|e| e.to_ascii_lowercase()) {
        let extensions = libopenraw::extensions();
        if extensions.iter().any(|e| std::ffi::OsStr::new(e) == ext) {
            return MType::Image(ImgFormat::Raw);
        }
        match ext.to_str() {
            Some("xmp") => return MType::Xmp,
            Some("thm") => return MType::Thumbnail,
            _ => {}
        }
    }
    let guess = mime_guess::from_path(path);
    if !guess.is_empty() {
        for mime in guess.iter_raw() {
            let t = guess_type(mime);
            if t != MType::None {
                return t;
            }
        }
    }

    // alternative
    let (content_type, _) = gio::content_type_guess(path.to_str(), None);

    guess_type(content_type.as_str())
}

impl MimeType {
    pub fn new<P: AsRef<Path>>(filename: P) -> MimeType {
        MimeType(guess_type_for_file(filename))
    }

    pub fn mime_type(&self) -> MType {
        self.0
    }

    pub fn is_image(&self) -> bool {
        if let MType::Image(_) = self.0 {
            return true;
        }
        false
    }

    pub fn is_digicam_raw(&self) -> bool {
        if let MType::Image(ref b) = self.0 {
            return *b == ImgFormat::Raw;
        }
        false
    }

    pub fn is_xmp(&self) -> bool {
        self.0 == MType::Xmp
    }

    pub fn is_movie(&self) -> bool {
        self.0 == MType::Movie
    }

    pub fn is_unknown(&self) -> bool {
        self.0 == MType::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_type_works() {
        let mimetype = MimeType::new("/foo/bar/img_0001.cr2");
        assert_eq!(
            guess_type_for_file("/foo/bar/img_0001.cr2"),
            MType::Image(ImgFormat::Raw)
        );
        assert!(mimetype.is_image());
        assert!(mimetype.is_digicam_raw());
    }

    #[test]
    fn mime_type_detect_erf() {
        let mimetype = MimeType::new("/foo/bar/img_0001.erf");
        assert_eq!(
            guess_type_for_file("/foo/bar/img_0001.erf"),
            MType::Image(ImgFormat::Raw)
        );
        assert!(mimetype.is_image());
        assert!(mimetype.is_digicam_raw());
    }

    #[test]
    fn mime_type_detect_nrw() {
        let mimetype = MimeType::new("/foo/bar/img_0001.nrw");
        assert_eq!(
            guess_type_for_file("/foo/bar/img_0001.NRW"),
            MType::Image(ImgFormat::Raw)
        );
        assert!(mimetype.is_image());
        assert!(mimetype.is_digicam_raw());
    }

    #[test]
    fn mime_type_unknown() {
        let mimetype = MimeType::new("");
        assert!(mimetype.is_unknown());
    }
}
