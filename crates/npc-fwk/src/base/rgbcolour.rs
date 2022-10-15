/*
 * niepce - fwk/base/rgbcolour.rs
 *
 * Copyright (C) 2017-2022 Hubert Figuière
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

use libc::c_char;
use std::ffi::CString;
use std::num::ParseIntError;
use std::str::FromStr;

#[repr(C)]
#[derive(Clone, Default)]
pub struct RgbColour {
    pub r: u16,
    pub g: u16,
    pub b: u16,
}

#[derive(Debug)]
pub enum ColourParseError {
    /// No Error.
    None,
    /// Parse Error.
    ParseError,
    /// Error parsing one of the 3 int components.
    ParseIntError,
}

impl From<ParseIntError> for ColourParseError {
    fn from(_: ParseIntError) -> ColourParseError {
        ColourParseError::ParseIntError
    }
}

impl RgbColour {
    pub fn new(r: u16, g: u16, b: u16) -> RgbColour {
        RgbColour { r, g, b }
    }
}

impl FromStr for RgbColour {
    type Err = ColourParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let components: Vec<&str> = s.split(' ').collect();
        if components.len() != 3 {
            return Err(ColourParseError::ParseError);
        }
        let r = components[0].parse::<u16>()?;
        let g = components[1].parse::<u16>()?;
        let b = components[2].parse::<u16>()?;
        Ok(RgbColour::new(r, g, b))
    }
}

impl ToString for RgbColour {
    fn to_string(&self) -> String {
        format!("{} {} {}", self.r, self.g, self.b)
    }
}

impl From<RgbColour> for gdk4::RGBA {
    fn from(v: RgbColour) -> gdk4::RGBA {
        gdk4::RGBA::new(
            v.r as f32 / 65535_f32,
            v.g as f32 / 65535_f32,
            v.b as f32 / 65535_f32,
            1.0,
        )
    }
}

#[no_mangle]
pub extern "C" fn fwk_rgbcolour_to_string(c: &RgbColour) -> *mut c_char {
    CString::new(c.to_string().as_bytes()).unwrap().into_raw()
}

/// Delete the %RgbColour object
///
/// # Safety
/// Dereference the pointer.
#[no_mangle]
pub unsafe extern "C" fn fwk_rgbcolour_delete(c: *mut RgbColour) {
    drop(Box::from_raw(c));
}

#[no_mangle]
pub extern "C" fn fwk_rgbcolour_component(c: &RgbColour, idx: i32) -> u16 {
    match idx {
        0 => c.r,
        1 => c.g,
        2 => c.b,
        _ => unreachable!(),
    }
}

#[no_mangle]
pub extern "C" fn fwk_rgbcolour_new(r: u16, g: u16, b: u16) -> *mut RgbColour {
    Box::into_raw(Box::new(RgbColour::new(r, g, b)))
}

#[no_mangle]
pub extern "C" fn fwk_rgbcolour_clone(c: &RgbColour) -> *mut RgbColour {
    Box::into_raw(Box::new(c.clone()))
}
