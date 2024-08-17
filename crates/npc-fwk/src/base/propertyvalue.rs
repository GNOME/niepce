/*
 * niepce - fwk/base/propertyvalue.rs
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

use crate::glib;

use super::date::Date;

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "PropertyValue")]
pub enum PropertyValue {
    Empty,
    Int(i32),
    String(String),
    StringArray(Vec<String>),
    Date(Date),
}

impl From<i32> for PropertyValue {
    fn from(value: i32) -> PropertyValue {
        Self::Int(value)
    }
}

impl From<&str> for PropertyValue {
    fn from(value: &str) -> PropertyValue {
        Self::String(value.into())
    }
}

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

    pub fn integer(&self) -> Option<i32> {
        match *self {
            PropertyValue::Int(i) => Some(i),
            _ => None,
        }
    }

    pub fn date(&self) -> Option<&Date> {
        match *self {
            PropertyValue::Date(ref d) => Some(d),
            _ => None,
        }
    }

    pub fn string(&self) -> Option<&str> {
        match *self {
            PropertyValue::String(ref s) => Some(s),
            _ => None,
        }
    }

    pub fn string_unchecked(&self) -> &str {
        match *self {
            PropertyValue::String(ref s) => s,
            _ => panic!("value is not a String"),
        }
    }

    pub fn string_array(&self) -> Option<&[String]> {
        match *self {
            PropertyValue::StringArray(ref sa) => Some(sa),
            _ => None,
        }
    }
}
