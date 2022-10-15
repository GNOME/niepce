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

use libc::c_char;
use std::ffi::{CStr, CString};

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

/// Create a new String %PropertyValue from a C string
///
/// # Safety
/// Dereference the pointer (C string)
#[no_mangle]
pub unsafe extern "C" fn fwk_property_value_new_str(v: *const c_char) -> *mut PropertyValue {
    let cstr = CStr::from_ptr(v);
    let value = Box::new(PropertyValue::String(cstr.to_string_lossy().into_owned()));
    Box::into_raw(value)
}

#[no_mangle]
pub extern "C" fn fwk_property_value_new_int(v: i32) -> *mut PropertyValue {
    let value = Box::new(PropertyValue::Int(v));
    Box::into_raw(value)
}

#[no_mangle]
pub extern "C" fn fwk_property_value_new_date(v: &Date) -> *mut PropertyValue {
    let value = Box::new(PropertyValue::Date(*v));
    Box::into_raw(value)
}

#[no_mangle]
pub extern "C" fn fwk_property_value_new_string_array() -> *mut PropertyValue {
    let value = Box::new(PropertyValue::StringArray(vec![]));
    Box::into_raw(value)
}

/// Delete the %PropertyValue object
///
/// # Safety
/// Dereference the pointer.
#[no_mangle]
pub unsafe extern "C" fn fwk_property_value_delete(v: *mut PropertyValue) {
    if !v.is_null() {
        drop(Box::from_raw(v));
    }
}

#[no_mangle]
pub extern "C" fn fwk_property_value_is_empty(v: &PropertyValue) -> bool {
    matches!(*v, PropertyValue::Empty)
}

#[no_mangle]
pub extern "C" fn fwk_property_value_is_integer(v: &PropertyValue) -> bool {
    matches!(*v, PropertyValue::Int(_))
}

#[no_mangle]
pub extern "C" fn fwk_property_value_get_integer(v: &PropertyValue) -> i32 {
    match *v {
        PropertyValue::Int(i) => i,
        _ => panic!("value is not Int"),
    }
}

#[no_mangle]
pub extern "C" fn fwk_property_value_is_date(v: &PropertyValue) -> bool {
    matches!(*v, PropertyValue::Date(_))
}

#[no_mangle]
pub extern "C" fn fwk_property_value_get_date(v: &PropertyValue) -> *mut Date {
    match *v {
        PropertyValue::Date(ref d) => Box::into_raw(Box::new(*d)),
        _ => panic!("value is not Date"),
    }
}

#[no_mangle]
pub extern "C" fn fwk_property_value_is_string(v: &PropertyValue) -> bool {
    matches!(*v, PropertyValue::String(_))
}

#[no_mangle]
pub extern "C" fn fwk_property_value_get_string(v: &PropertyValue) -> *mut c_char {
    match *v {
        PropertyValue::String(ref s) => CString::new(s.as_bytes()).unwrap().into_raw(),
        _ => panic!("value is not a String"),
    }
}

/// Add a string a StringArray %PropertyValue
///
/// Will panic if the type is incorrect.
///
/// # Safety
/// Dereference the pointer (C string)
#[no_mangle]
pub unsafe extern "C" fn fwk_property_value_add_string(v: &mut PropertyValue, cstr: *const c_char) {
    match *v {
        PropertyValue::StringArray(ref mut sa) => {
            sa.push(CStr::from_ptr(cstr).to_string_lossy().into_owned());
        }
        _ => panic!("value is not a StringArray"),
    }
}

#[no_mangle]
pub extern "C" fn fwk_property_value_count_string_array(v: &PropertyValue) -> usize {
    match *v {
        PropertyValue::StringArray(ref sa) => sa.len(),
        _ => panic!("value is not a StringArray"),
    }
}

#[no_mangle]
pub extern "C" fn fwk_property_value_get_string_at(v: &PropertyValue, idx: usize) -> *mut c_char {
    match *v {
        PropertyValue::StringArray(ref sa) => CString::new(sa[idx].as_bytes()).unwrap().into_raw(),
        _ => panic!("value is not a StringArray"),
    }
}
