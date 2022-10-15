/*
 * niepce - fwk/base/date.rs
 *
 * Copyright (C) 2017-2020 Hubert Figuière
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

use chrono::{Datelike, Timelike};

pub type Time = i64;
// XXX a tuple for the cxx bindings
#[derive(Clone, Copy, Debug)]
pub struct Date(pub chrono::DateTime<chrono::Utc>);

impl std::string::ToString for Date {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl std::ops::Deref for Date {
    type Target = chrono::DateTime<chrono::Utc>;

    fn deref(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.0
    }
}

pub fn xmp_date_from(d: &chrono::DateTime<chrono::Utc>) -> exempi::DateTime {
    let mut xmp_date = exempi::DateTime::new();
    xmp_date.set_date(d.year(), d.month() as i32, d.day() as i32);
    xmp_date.set_time(d.hour() as i32, d.minute() as i32, d.second() as i32);
    xmp_date.set_timezone(exempi::XmpTzSign::UTC, 0, 0);

    xmp_date
}
