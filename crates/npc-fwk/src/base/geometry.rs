/*
 * niepce - fwk/base/geometry.rs
 *
 * Copyright (C) 2022 Hubert Figuière
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// A size
pub struct Size {
    /// The width
    pub w: u32,
    /// The height
    pub h: u32,
}

impl Size {
    /// Resize self size to fit in a square of `dim` size preserving the aspect ratio.
    pub fn fit_into_square(&self, dim: u32) -> Size {
        if self.w <= dim && self.h <= dim {
            return *self;
        }
        let scale = if self.w > self.h {
            (dim as f64) / (self.w as f64)
        } else {
            (dim as f64) / (self.h as f64)
        };

        self.scale(scale)
    }

    /// Scale the size by `scale`.
    pub fn scale(&self, scale: f64) -> Size {
        Size {
            w: (self.w as f64 * scale) as u32,
            h: (self.h as f64 * scale) as u32,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// A rectangle
pub struct Rect {
    /// x coordinate
    pub x: u32,
    /// y coordinate
    pub y: u32,
    /// The width
    pub w: u32,
    /// The height
    pub h: u32,
}

impl Rect {
    /// New rectangle with `x`, `y`, `w`, `h`.
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    fn scale(&mut self, scale: f64) {
        self.w = (self.w as f64 * scale) as u32;
        self.h = (self.h as f64 * scale) as u32;
    }

    /// Fit `self` in `dest`. The new rectangle will be in
    /// whole in the destination.
    pub fn fit_into(&self, dest: &Rect) -> Rect {
        let mut result = self.clone();

        let in_w = self.w as f64;
        let in_h = self.h as f64;
        let scale_w = dest.w as f64 / in_w;
        dbg_out!("w {} in_w {}", dest.w, in_w);
        let scale_h = dest.h as f64 / in_h;
        dbg_out!("h {} in_h {}", dest.h, in_h);
        dbg_out!("scale w {} h {}", scale_w, scale_h);
        let scale = scale_w.min(scale_h);

        result.scale(scale);
        if scale_w <= scale_h {
            result.w = dest.w;
        }
        if scale_w >= scale_h {
            result.h = dest.h;
        }

        result
    }

    /// Fill `self` in `dest`. That mean dest will crop `self` so that
    /// there is no space left.
    pub fn fill_into(&self, dest: &Rect) -> Rect {
        // the smallest dimension
        let mut result = self.clone();

        let in_w = self.w as f64;
        let in_h = self.h as f64;
        let scale_w = dest.w as f64 / in_w;
        dbg_out!("w {} in_w {}", dest.w, in_w);
        let scale_h = dest.h as f64 / in_h;
        dbg_out!("h {} in_h {}", dest.h, in_h);
        dbg_out!("scale w {} h {}", scale_w, scale_h);
        let scale = scale_w.max(scale_h);

        result.scale(scale);
        if scale_w >= scale_h {
            result.w = dest.w;
        }
        if scale_w <= scale_h {
            result.h = dest.h;
        }

        result
    }
}

#[cfg(test)]
mod test {
    use super::{Rect, Size};

    #[test]
    fn test_size_fit_into_square() {
        let size = Size { w: 320, h: 120 };
        assert_eq!(size.fit_into_square(160), Size { w: 160, h: 60 });
        let size = Size { w: 120, h: 320 };
        assert_eq!(size.fit_into_square(160), Size { w: 60, h: 160 });

        // already small enough
        let size = Size { w: 160, h: 120 };
        assert_eq!(size.fit_into_square(160), Size { w: 160, h: 120 });
    }

    #[test]
    fn test_rect_fit_into() {
        let dest1 = Rect::new(0, 0, 640, 480);
        let dest2 = Rect::new(0, 0, 480, 640);

        let source1 = Rect::new(0, 0, 2000, 1000);
        let source2 = Rect::new(0, 0, 1000, 2000);

        // FIT
        let result = source1.fit_into(&dest1);
        assert_eq!(result, Rect::new(0, 0, 640, 320));
        let result = source1.fit_into(&dest2);
        assert_eq!(result.w, 480);

        let result = source2.fit_into(&dest1);
        assert_eq!(result.h, 480);
        let result = source2.fit_into(&dest2);
        assert_eq!(result, Rect::new(0, 0, 320, 640));
    }

    #[test]
    fn test_rect_fill_into() {
        let dest1 = Rect::new(0, 0, 640, 480);
        let dest2 = Rect::new(0, 0, 480, 640);

        let source1 = Rect::new(0, 0, 2000, 1000);
        let source2 = Rect::new(0, 0, 1000, 2000);
        // FILL
        let result = source1.fill_into(&dest1);
        assert_eq!(result.h, 480);
        let result = source1.fill_into(&dest2);
        assert_eq!(result, Rect::new(0, 0, 1280, 640));

        let result = source2.fill_into(&dest1);
        assert_eq!(result, Rect::new(0, 0, 640, 1280));
        let result = source2.fill_into(&dest2);
        assert_eq!(result.w, 480);
    }
}
