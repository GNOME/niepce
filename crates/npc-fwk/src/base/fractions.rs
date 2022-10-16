/*
 * niepce - fwk/base/fractions.rs
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

use std::f64;

pub fn parse_fraction(value: &str) -> Option<(i64, i64)> {
    let numbers: Vec<i64> = value
        .split('/')
        .map(|s| s.parse::<i64>().unwrap_or(0))
        .collect();
    if numbers.len() != 2 {
        return None;
    }
    if numbers[1] == 0 {
        return None;
    }
    Some((numbers[0], numbers[1]))
}

pub fn fraction_to_decimal(value: &str) -> Option<f64> {
    parse_fraction(value).map(|(n, d)| n as f64 / d as f64)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_fraction_works() {
        let f = super::parse_fraction("1/4");
        assert!(f.is_some());
        assert_eq!(f.unwrap(), (1, 4));

        let f = super::parse_fraction("foobar");
        assert!(f.is_none());

        let f = super::parse_fraction("1/0");
        assert!(f.is_none());

        let f = super::parse_fraction("1/0/1");
        assert!(f.is_none());
    }

    #[test]
    fn fraction_to_decimal_works() {
        use super::fraction_to_decimal;

        let f = fraction_to_decimal("1/4");
        assert!(f.is_some());
        assert_eq!(f.unwrap(), 0.25);

        let f = fraction_to_decimal("foobar");
        assert!(f.is_none());

        let f = fraction_to_decimal("1/0");
        assert!(f.is_none());

        let f = fraction_to_decimal("1/0/1");
        assert!(f.is_none());
    }
}
