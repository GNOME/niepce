/*
 * niepce - fwk/base/propertybag.rs
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

use std::collections::BTreeMap;

use crate::base::propertyvalue::PropertyValue;

/// Marker trait for property index enum. This is only necessary for
/// few case of trait bound, but not for `PropertyBag<>`. Particularly
/// for the `Into<PropertyBag<u32>>`
pub trait PropertyIndex {}

/// A container for type properties whose order of addition
/// is kept
///
/// Insertion and lookup are same as for BTreeMap.
/// Removal is as long as lookup in a vector: O(n).
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyBag<Index> {
    pub bag: Vec<Index>,
    pub map: BTreeMap<Index, PropertyValue>,
}

impl<Index: Ord + Copy> Default for PropertyBag<Index> {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement a conversion from a `PropertyIndex` `PropertyBag` to one
/// for `u32`. This is due to a limitation in gtk widgets that can't
/// use generic type. The reverse isn't implemented.
impl<T> From<PropertyBag<T>> for PropertyBag<u32>
where
    T: PropertyIndex + Copy + Into<u32>,
{
    fn from(v: PropertyBag<T>) -> PropertyBag<u32> {
        PropertyBag {
            bag: v.bag.iter().map(|v| (*v).into()).collect::<Vec<_>>(),
            map: BTreeMap::from_iter(v.map.iter().map(|(k, v)| ((*k).into(), v.clone()))),
        }
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

    pub fn merge_mixed(&mut self, bag: Self) {
        // XXX manage MixedStringArray
        bag.map.iter().for_each(|(key, value)| {
            if self.map.get(key) != Some(value) {
                self.set_value(*key, PropertyValue::Mixed);
            }
        });
        let missing = self
            .bag
            .iter()
            .filter(|key| !bag.bag.contains(key))
            .copied()
            .collect::<Vec<_>>();
        missing.iter().for_each(|key| {
            self.set_value(*key, PropertyValue::Mixed);
        });
    }
}

#[cfg(test)]
mod test {
    use super::{PropertyBag, PropertyIndex, PropertyValue};

    #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
    #[repr(u32)]
    enum TestProperty {
        TestProp1,
        TestProp3,
        AnotherProp,
        UnsetProp,
        FifthProp,
    }

    impl PropertyIndex for TestProperty {}

    impl From<TestProperty> for u32 {
        fn from(tp: TestProperty) -> u32 {
            tp as u32
        }
    }

    fn create_test_bag() -> PropertyBag<TestProperty> {
        let mut bag = PropertyBag::<TestProperty>::new();

        bag.set_value(TestProperty::TestProp1, PropertyValue::String("foo".into()));
        bag.set_value(TestProperty::AnotherProp, PropertyValue::Int(42));
        bag.set_value(TestProperty::TestProp3, PropertyValue::Int(3));
        bag
    }

    #[test]
    fn test_property_bag() {
        let bag = create_test_bag();

        assert_eq!(bag.len(), 3);
        assert!(bag.get(&TestProperty::TestProp3).is_some());
        assert_eq!(bag.get(&TestProperty::UnsetProp), None);

        // Test the key order
        let keys = bag.keys().collect::<Vec<_>>();
        assert_eq!(keys.len(), 3);
        assert_eq!(*keys[0], TestProperty::TestProp1);
        assert_eq!(*keys[1], TestProperty::AnotherProp);
        assert_eq!(*keys[2], TestProperty::TestProp3);
    }

    #[test]
    fn test_from_property_index() {
        let bag = create_test_bag();
        let bag_len = bag.len();

        let bag2 = PropertyBag::<u32>::from(bag);
        assert_eq!(
            bag2.get(&(TestProperty::TestProp1.into())).unwrap(),
            &PropertyValue::String("foo".into())
        );
        assert_eq!(
            bag2.get(&(TestProperty::TestProp3.into())).unwrap(),
            &PropertyValue::Int(3)
        );
        assert_eq!(
            bag2.get(&(TestProperty::AnotherProp.into())).unwrap(),
            &PropertyValue::Int(42)
        );

        assert_eq!(bag2.len(), bag_len);
    }

    #[test]
    fn test_merge_mixed() {
        let mut bag = create_test_bag();

        let other = create_test_bag();

        bag.merge_mixed(other);
        assert_eq!(bag.len(), 3);
        assert_eq!(bag, create_test_bag());

        let mut another = create_test_bag();
        another.set_value(TestProperty::TestProp3, PropertyValue::Int(5));

        // Test that different values lead to Mixed.
        bag.merge_mixed(another.clone());
        assert_eq!(
            bag.get(&TestProperty::TestProp3),
            Some(&PropertyValue::Mixed),
            "Differing property should be Mixed"
        );

        // Test that a property in the source not in target lead to
        // Mixed
        another.set_value(
            TestProperty::UnsetProp,
            PropertyValue::String("unset".into()),
        );
        bag.merge_mixed(another.clone());
        assert_eq!(
            bag.get(&TestProperty::UnsetProp),
            Some(&PropertyValue::Mixed),
            "Property in source not target should be Mixed"
        );

        bag.set_value(TestProperty::FifthProp, PropertyValue::Int(14));

        bag.merge_mixed(another);
        assert_eq!(
            bag.get(&TestProperty::FifthProp),
            Some(&PropertyValue::Mixed),
            "Property in target and not source should be Mixed"
        );

        println!("bag: {bag:?}");
    }
}
