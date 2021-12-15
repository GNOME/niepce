/*
 * niepce - fwk/base/date.rs
 *
 * Copyright (C) 2017-2021 Hubert Figuière
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

use std::ffi::CString;

use chrono::{Datelike, Timelike};

pub type Time = i64;
pub type Date = chrono::DateTime<chrono::FixedOffset>;

/// Convert an `exempi2::DateTime` to a `chrono::DateTime<FixedOffset>`
pub fn xmp_date_from(d: &Date) -> exempi2::DateTime {
    use exempi2::TzSign;

    let mut xmp_date = exempi2::DateTime::new();
    xmp_date.set_date(d.year(), d.month() as i32, d.day() as i32);
    xmp_date.set_time(d.hour() as i32, d.minute() as i32, d.second() as i32);
    let offset = d.offset().local_minus_utc();
    let sign = if offset == 0 {
        TzSign::UTC
    } else if offset > 0 {
        TzSign::East
    } else {
        TzSign::West
    };
    let offset = offset.abs();
    xmp_date.set_timezone(sign, offset / 3600, offset / 60);

    xmp_date
}

/// Delete a %Date object
///
/// # Safety
/// Dereference the raw pointer.
#[no_mangle]
pub unsafe extern "C" fn fwk_date_delete(date: *mut Date) {
    Box::from_raw(date);
}

#[no_mangle]
pub extern "C" fn fwk_date_to_string(date: &Date) -> *mut libc::c_char {
    CString::new(date.to_string().as_bytes())
        .unwrap()
        .into_raw()
}

#[cfg(test)]
mod test {
    use chrono::TimeZone;

    use super::xmp_date_from;

    #[test]
    fn test_xmp_date_from() {
        let date = chrono::FixedOffset::west(5 * 3600)
            .ymd(2021, 12, 25)
            .and_hms(10, 42, 12);
        let xmp_date = xmp_date_from(&date);
        assert_eq!(xmp_date.year(), 2021);
        assert_eq!(xmp_date.month(), 12);
        assert_eq!(xmp_date.day(), 25);

        assert_eq!(xmp_date.hour(), 10);

        assert_eq!(xmp_date.tz_hours(), 5);
        assert_eq!(xmp_date.tz_sign(), exempi2::TzSign::West);
    }
}
