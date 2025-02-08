/*
 * niepce - fwk/base/path_tree.rs
 *
 * Copyright (C) 2025 Hubert Figuière
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

#![warn(missing_docs)]

use std::collections::BTreeMap;

/// Trait that tree item need to implement.
pub trait PathTreeItem {
    /// ID type for each item.
    type Id;

    /// The item ID
    fn id(&self) -> Self::Id;
    /// The item path
    fn path(&self) -> String;
}

type Nodes<T> = BTreeMap<String, Node<T>>;

/// Get or insert a node from a `Nodes<T>`.
fn get_mut_or_insert<'a, T>(nodes: &'a mut Nodes<T>, component: &str) -> &'a mut Node<T> {
    // Figure out how to avoid the contains_key, but get_mut cause
    // twice mutable borrow.
    if nodes.contains_key(component) {
        nodes.get_mut(component).unwrap()
    } else {
        nodes.insert(component.into(), Node::default());
        nodes.get_mut(component).unwrap()
    }
}

/// Walk until it reaches 0 or the end.
fn walk<'a, T>(nodes: &'a Nodes<T>, count: &mut usize) -> Option<&'a Node<T>> {
    let mut iter = nodes.values();
    let mut node: Option<&Node<T>> = iter.next();
    while *count > 0 && node.is_some() {
        *count -= 1;
        if *count > 0 && !node.unwrap().nodes.is_empty() {
            node = walk(&node.unwrap().nodes, count);
        }
        if *count > 0 {
            node = iter.next();
        }
    }

    node
}

/// Result of an insertion.
struct Inserted<T: PathTreeItem> {
    /// The Id of the parent where this was inserted
    parent: Option<T::Id>,
    /// The Id of the node this replaces.
    old_id: Option<T::Id>,
}

/// A Path tree: a tree that allow accessing node with value `T` using
/// a path. Like a filesystem, or by its id.
///
/// Each node may contain a value. Each node can be addressed with a
/// path made out of components separated by `separator`, or by an `id`.
///
/// The node value is an Id to the node `by_id` map.
///
#[derive(Debug)]
pub struct PathTree<T: PathTreeItem> {
    separator: char,
    nodes: Nodes<T::Id>,
    by_id: BTreeMap<T::Id, T>,
}

/// A Tree node can contain value `T`.
#[derive(Debug)]
struct Node<T> {
    /// There may be no value.
    value: Option<T>,
    nodes: Nodes<T>,
}

// This avoid the derive `Default` which has stricter requirements.
impl<T> Default for Node<T> {
    fn default() -> Self {
        Node {
            value: None,
            nodes: Nodes::new(),
        }
    }
}

impl<T: Ord> Node<T> {
    /// Get the mutable node, or insert a new one.
    fn get_mut_or_insert(&mut self, component: &str) -> &mut Node<T> {
        get_mut_or_insert(&mut self.nodes, component)
    }
}

impl<T: PathTreeItem> PathTree<T> {
    /// Insert an item to its path, and return the parent id.
    pub fn push(&mut self, item: T) -> Option<T::Id>
    where
        <T as PathTreeItem>::Id: Ord + Copy,
    {
        self.insert_for_parent(&item.path(), item)
    }
}

impl<T: PathTreeItem> std::ops::Index<usize> for PathTree<T> {
    type Output = Option<T::Id>;

    fn index(&self, index: usize) -> &Option<T::Id> {
        let mut index = index + 1;
        let node = walk(&self.nodes, &mut index);
        assert!(node.is_some());
        &node.unwrap().value
    }
}

impl<T: PathTreeItem> PathTree<T> {
    /// Create a new `PathTree` with `separator`.
    pub fn new(separator: char) -> PathTree<T> {
        PathTree {
            separator,
            nodes: Nodes::default(),
            by_id: BTreeMap::new(),
        }
    }

    /// Return true it has no node.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Locate the node for `path`.
    fn get_node(&self, path: &str) -> Option<&Node<T::Id>> {
        let components = path.split(self.separator);
        let mut node: Option<&Node<T::Id>> = None;
        let mut nodes = &self.nodes;
        for component in components {
            node = nodes.get(component);
            if let Some(n) = node {
                nodes = &n.nodes;
            } else {
                return None;
            }
        }
        node
    }

    /// Get the children at `path`.
    pub fn children(&self, path: &str) -> Option<Vec<&T::Id>> {
        self.get_node(path).map(|node| {
            node.nodes
                .values()
                .filter_map(|node| node.value.as_ref())
                .collect::<Vec<_>>()
        })
    }

    /// There is a node at `path`. Even if value is `None`.
    pub fn contains_key(&self, path: &str) -> bool {
        self.get_node(path).is_some()
    }

    /// Insert a value at path and return its parent id and old id.
    fn insert_node_and_parent(&mut self, path: &str, value: &T) -> Option<Inserted<T>>
    where
        <T as PathTreeItem>::Id: Ord + Copy,
    {
        let mut components = path.split(self.separator);
        let component = components.next()?;
        let mut parent = None;
        let mut node = get_mut_or_insert(&mut self.nodes, component);
        for component in components {
            parent = node.value;
            node = node.get_mut_or_insert(component);
        }

        let old_id = node.value;

        let id = value.id();
        node.value = Some(id);

        Some(Inserted { parent, old_id })
    }

    /// Insert value at path and return older value.
    pub fn insert(&mut self, path: &str, value: T) -> Option<T>
    where
        <T as PathTreeItem>::Id: Ord + Copy,
    {
        let inserted = self.insert_node_and_parent(path, &value)?;

        self.insert_node(inserted.old_id, value)
    }

    /// Insert value at path and return parent ID.
    fn insert_for_parent(&mut self, path: &str, value: T) -> Option<T::Id>
    where
        <T as PathTreeItem>::Id: Ord + Copy,
    {
        let inserted = self.insert_node_and_parent(path, &value)?;

        self.insert_node(inserted.old_id, value);

        inserted.parent
    }

    /// Insert the value in the by_ maps and remove old_id if any.
    /// Returns the old_value
    fn insert_node(&mut self, old_id: Option<T::Id>, value: T) -> Option<T>
    where
        <T as PathTreeItem>::Id: Ord + Copy,
    {
        let mut old_value = None;

        if let Some(old_id) = old_id.as_ref() {
            old_value = self.by_id.remove(old_id);
        }
        // Note that we don't check there is already a value in there.
        // There actually shouldn't be one.
        let id = value.id();
        self.by_id.insert(id, value);

        old_value
    }

    /// Get item by `id`
    pub fn get_by_id(&self, id: T::Id) -> Option<&T>
    where
        <T as PathTreeItem>::Id: Ord,
    {
        self.by_id.get(&id)
    }

    /// Get the item at `path` if there is one.
    pub fn get(&self, path: &str) -> Option<&T>
    where
        <T as PathTreeItem>::Id: Ord,
    {
        self.get_node(path)
            .and_then(|v| v.value.as_ref())
            .and_then(|id| self.by_id.get(id))
    }
}

#[cfg(test)]
mod test {
    use super::{PathTree, PathTreeItem};

    #[derive(Debug, PartialEq)]
    struct TestItem {
        id: u32,
        path: String,
    }

    impl TestItem {
        fn new(id: u32, path: String) -> TestItem {
            TestItem { id, path }
        }
    }

    impl PathTreeItem for TestItem {
        type Id = u32;

        fn id(&self) -> Self::Id {
            self.id
        }

        fn path(&self) -> String {
            self.path.clone()
        }
    }

    #[test]
    fn test_tree() {
        let mut path_tree = PathTree::<TestItem>::new('/');

        path_tree.insert("usr", TestItem::new(21, "usr".into()));
        assert!(path_tree.contains_key("usr"));
        assert_eq!(path_tree[0], Some(21));

        let parent = path_tree.insert_for_parent("usr/bin", TestItem::new(42, "usr/bin".into()));
        // Test the parent is the right one.
        assert_eq!(parent, Some(21));

        assert!(path_tree.contains_key("usr"));
        assert!(!path_tree.contains_key("bin"));
        assert!(path_tree.contains_key("usr/bin"));
        assert_eq!(path_tree[1], Some(42));
        assert_eq!(path_tree.get("usr/bin").map(|item| item.id()), Some(42));

        let old_value = path_tree.insert("usr/lib", TestItem::new(43, "usr/lib".into()));
        // There was no old value since we didn't replace it.
        assert_eq!(old_value, None);

        assert!(path_tree.contains_key("usr/lib"));
        assert_eq!(path_tree[2], Some(43));
        assert_eq!(path_tree.get("usr/lib").map(|item| item.id()), Some(43));

        path_tree.insert("var/lib", TestItem::new(70, "var/lib".into()));

        // The node exist but has no value.
        assert!(path_tree.contains_key("var"));
        assert_eq!(path_tree[3], None);
        assert_eq!(path_tree.get("var"), None);

        assert!(path_tree.contains_key("var/lib"));
        assert_eq!(path_tree[4], Some(70));
        assert_eq!(path_tree.get("var/lib").map(|item| item.id()), Some(70));

        path_tree.insert("var/bin", TestItem::new(74, "var/bin".into()));

        assert!(path_tree.contains_key("var/bin"));
        assert_eq!(path_tree[4], Some(74));
        assert_eq!(path_tree.get("var/bin").map(|item| item.id()), Some(74));
        assert_eq!(path_tree[5], Some(70));
        assert_eq!(path_tree.get("var/lib").map(|item| item.id()), Some(70));

        let children = path_tree.children("var");
        assert!(children.is_some());
        let children = children.unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children, [&74, &70]);

        // Lets replace "usr/lib" with a different value
        let old_value = path_tree.insert("usr/lib", TestItem::new(69, "usr/lib".into()));
        // The old value is 43
        assert_eq!(old_value.map(|v| v.id()), Some(43));
        assert!(path_tree.contains_key("usr/lib"));
        // It is still in the same index.
        assert_eq!(path_tree[2], Some(69));
        assert_eq!(path_tree.get("usr/lib").map(|item| item.id()), Some(69));
        // The old value should have been removed.
        assert_eq!(path_tree.get_by_id(43), None);
        // The new one is at a different id.
        assert_eq!(
            path_tree.get_by_id(69).map(|v| v.path()),
            Some("usr/lib".to_string())
        );
    }
}
