/*
 * niepce - fwk/base/propertybag.rs
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

use std::collections::BTreeMap;

use crate::base::propertyvalue::PropertyValue;

/// A container for type properties whose order of addition
/// is kept
///
/// Insertion and lookup are same as for BTreeMap.
/// Removal is as long as lookup in a vector: O(n).
#[derive(Clone)]
pub struct PropertyBag<Index> {
    pub bag: Vec<Index>,
    pub map: BTreeMap<Index, PropertyValue>,
}

impl<Index: Ord + Copy> Default for PropertyBag<Index> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Index: Ord + Copy> PropertyBag<Index> {
    pub fn new() -> Self {
        Self {
            bag: vec![],
            map: BTreeMap::new(),
        }
    }

    /// Return the keys in the order of the bag, i.e. in which they have been added.
    pub fn keys(&self) -> std::slice::Iter<'_, Index> {
        self.bag.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.bag.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bag.len()
    }

    pub fn get(&self, key: &Index) -> Option<&PropertyValue> {
        self.map.get(key)
    }

    pub fn contains_key(&self, key: &Index) -> bool {
        self.map.contains_key(key)
    }

    pub fn set_value(&mut self, key: Index, value: PropertyValue) -> bool {
        let ret = self.map.insert(key, value);
        if ret.is_some() {
            return true;
        }
        self.bag.push(key);
        false
    }
}
