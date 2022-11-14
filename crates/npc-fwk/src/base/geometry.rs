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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Size {
    pub w: u32,
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

    pub fn scale(&self, scale: f64) -> Size {
        Size {
            w: (self.w as f64 * scale) as u32,
            h: (self.h as f64 * scale) as u32,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Size;

    #[test]
    fn test_fit_into_square() {
        let size = Size { w: 320, h: 120 };
        assert_eq!(size.fit_into_square(160), Size { w: 160, h: 60 });
        let size = Size { w: 120, h: 320 };
        assert_eq!(size.fit_into_square(160), Size { w: 60, h: 160 });

        // already small enough
        let size = Size { w: 160, h: 120 };
        assert_eq!(size.fit_into_square(160), Size { w: 160, h: 120 });
    }
}
