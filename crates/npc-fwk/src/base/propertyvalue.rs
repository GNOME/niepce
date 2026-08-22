/*
 * niepce - fwk/base/propertyvalue.rs
 *
 * Copyright (C) 2017-2026 Hubert Figuière
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

/// PropertyValue, a type checked value type. It is also glib boxed
/// to allow passing it into glib properties.
#[derive(Clone, Debug, Default, PartialEq, glib::Boxed)]
#[boxed_type(name = "PropertyValue")]
pub enum PropertyValue {
    /// There is no value.
    #[default]
    Empty,
    /// The property is unset. This signal it should be removed.
    Unset,
    /// Integer value.
    Int(i32),
    /// String value.
    String(String),
    /// String array.
    StringArray(Vec<String>),
    /// Date object.
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

    pub fn is_unset(&self) -> bool {
        matches!(*self, PropertyValue::Unset)
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

    /// Return a string without checking.
    ///
    /// # Panic
    /// Will panic the property value isn't a string.
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

#[cfg(test)]
mod test {
    use super::PropertyValue;

    #[test]
    fn test_property_value() {
        let value = PropertyValue::String("a string".into());

        assert_eq!(value.string_unchecked(), "a string");
        assert!(value.is_string());
        assert!(!value.is_empty());
        assert!(!value.is_unset());
        assert!(!value.is_integer());
        assert!(!value.is_date());
        assert_eq!(value.string(), Some("a string"));
        assert_eq!(value.integer(), None);
        assert_eq!(value.date(), None);

        let value = PropertyValue::Int(42);

        assert!(!value.is_string());
        assert!(!value.is_empty());
        assert!(!value.is_unset());
        assert!(value.is_integer());
        assert!(!value.is_date());
        assert_eq!(value.string(), None);
        assert_eq!(value.integer(), Some(42));
        assert_eq!(value.date(), None);
    }

    #[should_panic]
    #[test]
    fn test_property_value_that_should_panic() {
        let value = PropertyValue::Int(42);

        value.string_unchecked();
    }
}
