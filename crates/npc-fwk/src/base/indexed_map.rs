/*
 * niepce - ncp_fwk/base/indexed_map.rs
 *
 * Copyright (C) 2022-2024 Hubert Figuière
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

use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::Index;
use std::slice::SliceIndex;

/// A HashMap with indexable entries.
///
/// This work by using a vector to order the keys in the hashmap.
///
/// Currently any operation that need to get the idx of a value from the id
/// is slow O(n) as it search linearly.
#[derive(Default)]
pub struct IndexedMap<K, V> {
    index: Vec<K>,
    map: HashMap<K, V>,
}

impl<K, V> IndexedMap<K, V> {
    /// Push a key / value tuple.
    /// If the key already exists, it replace with the new value
    /// but does not reorder. The old value is returned.
    #[must_use]
    pub fn push(&mut self, kv: (K, V)) -> Option<V>
    where
        K: std::cmp::Eq + std::hash::Hash + Clone,
    {
        let old = self.map.insert(kv.0.clone(), kv.1);
        if old.is_none() {
            self.index.push(kv.0);
        }

        old
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Get the iterator.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
    }

    /// Return the index of the `key`.
    /// This is currenly SLOW O(n)
    pub fn index_of<Q>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q> + std::cmp::Eq + std::cmp::PartialEq<Q> + std::hash::Hash,
        Q: std::cmp::Eq + std::hash::Hash + ?Sized,
    {
        self.index.iter().position(|item| item == key)
    }

    /// Contains the `key`.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + std::cmp::Eq + std::hash::Hash,
        Q: std::hash::Hash + Eq + ?Sized,
    {
        self.map.contains_key(key)
    }

    /// Get the value at `key`
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + std::cmp::Eq + std::hash::Hash,
        Q: std::cmp::Eq + std::hash::Hash + ?Sized,
    {
        self.map.get(key)
    }

    /// Remove value for `key`
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + std::cmp::Eq + std::cmp::PartialEq<Q> + std::hash::Hash,
        Q: std::cmp::Eq + std::hash::Hash + ?Sized,
    {
        // Currently this is slow O(n)
        if let Some(pos) = self.index_of(key) {
            self.index.remove(pos);
        }
        self.map.remove(key)
    }

    /// Remove value at `idx` and return it.
    /// If `idx` is out of bounds, returns `None`.
    pub fn remove_at(&mut self, idx: usize) -> Option<V>
    where
        K: std::cmp::Eq + std::hash::Hash,
    {
        if idx >= self.len() {
            return None;
        }

        let k = &self.index[idx];
        let old = self.map.remove(k);
        self.index.remove(idx);

        old
    }

    #[cfg(test)]
    /// Test only: check the length of both is consistent (equal).
    fn consistent_len(&self) -> bool {
        self.index.len() == self.map.len()
    }
}

impl<K, V, I> Index<I> for IndexedMap<K, V>
where
    I: SliceIndex<[K], Output = K>,
    K: std::cmp::Eq + std::hash::Hash,
{
    type Output = V;

    /// # Panic
    /// Will panic if the map doesn't contain the element (internal consistency)
    fn index(&self, index: I) -> &Self::Output {
        self.map.get(&self.index[index]).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::IndexedMap;

    #[test]
    fn test_indexed_map() {
        let mut indexed_map = IndexedMap::default();

        assert_eq!(indexed_map.len(), 0);
        assert!(indexed_map.consistent_len());

        let old = indexed_map.push((1, "Foo".to_string()));
        assert!(old.is_none());
        assert!(indexed_map.consistent_len());
        assert_eq!(indexed_map.len(), 1);

        let old = indexed_map.push((2, "Bar".to_string()));
        assert!(old.is_none());
        assert!(indexed_map.consistent_len());
        assert_eq!(indexed_map.len(), 2);

        let old = indexed_map.push((42, "Joke".to_string()));
        assert!(old.is_none());
        assert!(indexed_map.consistent_len());
        assert_eq!(indexed_map.len(), 3);

        assert_eq!(indexed_map[0], "Foo");
        assert_eq!(indexed_map[1], "Bar");
        assert_eq!(indexed_map[2], "Joke");
        assert_eq!(indexed_map.get(&1), Some(&"Foo".to_string()));
        assert_eq!(indexed_map.get(&2), Some(&"Bar".to_string()));
        assert_eq!(indexed_map.get(&42), Some(&"Joke".to_string()));
        let old = indexed_map.push((2, "Baz".to_string()));
        assert!(old.is_some());
        // Same number of elements
        assert!(indexed_map.consistent_len());
        assert_eq!(indexed_map.len(), 3);
        // The value was updated.
        assert_eq!(indexed_map.get(&2), Some(&"Baz".to_string()));
        // No reordering though.
        assert_eq!(indexed_map[1], "Baz");

        // Check the `index_of`
        assert_eq!(indexed_map.index_of(&42), Some(2));

        let old = indexed_map.remove_at(1);
        assert!(old.is_some());
        assert!(indexed_map.consistent_len());
        assert_eq!(indexed_map.len(), 2);
        assert_eq!(indexed_map.get(&2), None);
        assert_eq!(indexed_map[1], "Joke");

        let old = indexed_map.remove(&42);
        assert!(old.is_some());
        assert!(indexed_map.consistent_len());
        assert_eq!(indexed_map.len(), 1);
        assert_eq!(indexed_map.get(&42), None);
    }

    #[test]
    #[should_panic]
    fn test_panic_out_of_index() {
        let mut indexed_map = IndexedMap::default();

        assert_eq!(indexed_map.len(), 0);
        let old = indexed_map.push((1, "Foo".to_string()));
        assert!(old.is_none());
        assert_eq!(indexed_map.len(), 1);
        // There is no index 1, this should panic.
        let _value = &indexed_map[1];
    }
}
