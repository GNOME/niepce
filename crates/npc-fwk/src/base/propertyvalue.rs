/*
 * niepce - fwk/base/propertyvalue.rs
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

use super::date::Date;

#[derive(Clone, Debug)]
pub enum PropertyValue {
    Empty,
    Int(i32),
    String(String),
    StringArray(Vec<String>),
    Date(Date),
}

unsafe impl Send for PropertyValue {}

impl PropertyValue {
    pub fn is_empty(&self) -> bool {
        matches!(*self, PropertyValue::Empty)
    }

    pub fn is_integer(&self) -> bool {
        matches!(*self, PropertyValue::Int(_))
    }

    pub fn is_date(&self) -> bool {
        matches!(*self, PropertyValue::Date(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(*self, PropertyValue::String(_))
    }

    pub fn integer_unchecked(&self) -> i32 {
        match *self {
            PropertyValue::Int(i) => i,
            _ => panic!("value is not Int"),
        }
    }

    pub fn date_unchecked(&self) -> Box<Date> {
        match *self {
            PropertyValue::Date(ref d) => Box::new(*d),
            _ => panic!("value is not Date"),
        }
    }

    pub fn string_unchecked(&self) -> &str {
        match *self {
            PropertyValue::String(ref s) => s,
            _ => panic!("value is not a String"),
        }
    }

    /// Add a string a StringArray %PropertyValue
    ///
    /// Will panic if the type is incorrect.
    pub fn add_string_unchecked(&mut self, string: &str) {
        match *self {
            PropertyValue::StringArray(ref mut sa) => {
                sa.push(string.to_string());
            }
            _ => panic!("value is not a StringArray"),
        }
    }

    pub fn string_array_unchecked(&self) -> &[String] {
        match *self {
            PropertyValue::StringArray(ref sa) => sa,
            _ => panic!("value is not a StringArray"),
        }
    }
}

/// Create a new String %PropertyValue from a string
pub fn property_value_new_str(v: &str) -> Box<PropertyValue> {
    Box::new(PropertyValue::String(v.to_string()))
}

pub fn property_value_new_int(v: i32) -> Box<PropertyValue> {
    Box::new(PropertyValue::Int(v))
}

pub fn property_value_new_date(v: &Date) -> Box<PropertyValue> {
    Box::new(PropertyValue::Date(*v))
}

pub fn property_value_new_string_array() -> Box<PropertyValue> {
    Box::new(PropertyValue::StringArray(vec![]))
}
