/*
 * niepce - fwk/base/date.rs
 *
 * Copyright (C) 2017-2024 Hubert Figuière
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

use chrono::{Datelike, Offset, TimeZone, Timelike, Utc};

pub type Time = i64;
// XXX a new type for the cxx bindings
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub struct Date(pub chrono::DateTime<chrono::FixedOffset>);

impl Date {
    /// Now as a `Date`.
    pub fn now() -> Self {
        let dt = chrono::Local::now();
        Self(dt.with_timezone(dt.offset()))
    }

    /// Create a `Date` from a `SystemTime`.
    pub fn from_system_time(t: std::time::SystemTime) -> Self {
        let dt = chrono::DateTime::<Utc>::from(t);
        Self(dt.with_timezone(&dt.offset().fix()))
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for Date {
    type Target = chrono::DateTime<chrono::FixedOffset>;

    fn deref(&self) -> &chrono::DateTime<chrono::FixedOffset> {
        &self.0
    }
}

impl From<exempi2::DateTime> for Date {
    fn from(d: exempi2::DateTime) -> Date {
        Self::from(&d)
    }
}

impl From<&exempi2::DateTime> for Date {
    fn from(d: &exempi2::DateTime) -> Date {
        use exempi2::TzSign;

        let tz = if d.has_tz() {
            match d.tz_sign() {
                TzSign::UTC => chrono::FixedOffset::east_opt(0),
                TzSign::East => {
                    chrono::FixedOffset::east_opt(d.tz_hours() * 3600 + d.tz_minutes() * 60)
                }
                TzSign::West => {
                    chrono::FixedOffset::west_opt(d.tz_hours() * 3600 + d.tz_minutes() * 60)
                }
            }
        } else {
            chrono::FixedOffset::east_opt(0)
        }
        .expect("date conversion error");
        let result = tz.with_ymd_and_hms(
            d.year(),
            d.month() as u32,
            d.day() as u32,
            d.hour() as u32,
            d.minute() as u32,
            d.second() as u32,
        );
        Date(result.single().expect("date conversion error"))
    }
}

impl From<Date> for exempi2::DateTime {
    /// Convert a `Date` (newtype for `chrono::DateTime<FixedOffset>`)
    /// to an `exempi2::DateTime`
    fn from(d: Date) -> exempi2::DateTime {
        use exempi2::TzSign;

        let mut xmp_date = exempi2::DateTime::new();
        xmp_date.set_date(d.year(), d.month() as i32, d.day() as i32);
        xmp_date.set_time(d.hour() as i32, d.minute() as i32, d.second() as i32);
        let offset = d.offset().local_minus_utc();
        let sign = match offset {
            0 => TzSign::UTC,
            1.. => TzSign::East,
            _ => TzSign::West,
        };
        let offset = offset.abs();
        xmp_date.set_timezone(sign, offset / 3600, offset / 60);

        xmp_date
    }
}

#[cfg(test)]
mod test {
    use chrono::TimeZone;

    use super::Date;

    #[test]
    fn test_xmp_date_west_from() {
        let date = chrono::FixedOffset::west_opt(5 * 3600)
            .and_then(|tz| tz.with_ymd_and_hms(2021, 12, 25, 10, 42, 12).single())
            .unwrap();
        let xmp_date: exempi2::DateTime = Date(date).into();
        assert_eq!(xmp_date.year(), 2021);
        assert_eq!(xmp_date.month(), 12);
        assert_eq!(xmp_date.day(), 25);

        assert_eq!(xmp_date.hour(), 10);

        assert_eq!(xmp_date.tz_hours(), 5);
        assert_eq!(xmp_date.tz_sign(), exempi2::TzSign::West);
    }

    #[test]
    fn test_xmp_date_east_from() {
        let date = chrono::FixedOffset::east_opt(5 * 3600)
            .and_then(|tz| tz.with_ymd_and_hms(2021, 12, 25, 10, 42, 12).single())
            .unwrap();
        let xmp_date: exempi2::DateTime = Date(date).into();
        assert_eq!(xmp_date.year(), 2021);
        assert_eq!(xmp_date.month(), 12);
        assert_eq!(xmp_date.day(), 25);

        assert_eq!(xmp_date.hour(), 10);

        assert_eq!(xmp_date.tz_hours(), 5);
        assert_eq!(xmp_date.tz_sign(), exempi2::TzSign::East);
    }
}
