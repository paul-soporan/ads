use std::cmp::Ordering;

use crate::traits::{
    core::{Map, OrderedMap},
    diagnostics::TreeDiagnostics,
};

#[derive(Debug)]
struct BTreeNode<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    children: Vec<usize>,
    parent: Option<usize>,
    is_leaf: bool,
}

impl<K, V> BTreeNode<K, V> {
    fn new(is_leaf: bool) -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            parent: None,
            is_leaf,
        }
    }
}

#[derive(Debug)]
pub struct BTreeNodeView<'a, K, V, const T: usize> {
    tree: &'a BTree<K, V, T>,
    node_idx: usize,
}

#[derive(Debug)]
pub struct BTreeCursor<'a, K, V, const T: usize> {
    tree: &'a BTree<K, V, T>,
    node_idx: usize,
    index: usize,
}

#[derive(Debug)]
pub struct BTreeIter<'a, K, V, const T: usize> {
    next: Option<BTreeCursor<'a, K, V, T>>,
}

impl<'a, K, V, const T: usize> Clone for BTreeNodeView<'a, K, V, T> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node_idx: self.node_idx,
        }
    }
}

impl<'a, K, V, const T: usize> Clone for BTreeCursor<'a, K, V, T> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node_idx: self.node_idx,
            index: self.index,
        }
    }
}

impl<'a, K, V, const T: usize> BTreeNodeView<'a, K, V, T> {
    fn node_ref(&self) -> &BTreeNode<K, V> {
        self.tree.nodes[self.node_idx]
            .as_ref()
            .expect("live arena node")
    }
}

impl<'a, K: Clone, V: Clone, const T: usize> BTreeNodeView<'a, K, V, T> {
    pub fn keys(&self) -> Vec<K> {
        self.node_ref().keys.clone()
    }

    pub fn entries(&self) -> Vec<(K, V)> {
        let n = self.node_ref();
        n.keys
            .iter()
            .cloned()
            .zip(n.values.iter().cloned())
            .collect()
    }

    pub fn key_count(&self) -> usize {
        self.node_ref().keys.len()
    }

    pub fn is_leaf(&self) -> bool {
        self.node_ref().is_leaf
    }

    pub fn children(&self) -> Vec<BTreeNodeView<'a, K, V, T>> {
        self.node_ref()
            .children
            .iter()
            .map(|child| BTreeNodeView {
                tree: self.tree,
                node_idx: *child,
            })
            .collect()
    }

    pub fn parent(&self) -> Option<BTreeNodeView<'a, K, V, T>> {
        self.node_ref().parent.map(|node_idx| BTreeNodeView {
            tree: self.tree,
            node_idx,
        })
    }
}

impl<'a, K: Ord, V, const T: usize> BTreeCursor<'a, K, V, T> {
    fn node_ref(&self) -> &BTreeNode<K, V> {
        self.tree.node_ref(self.node_idx)
    }

    pub fn key(&self) -> &K {
        &self.node_ref().keys[self.index]
    }

    pub fn value(&self) -> &V {
        &self.node_ref().values[self.index]
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn is_leaf(&self) -> bool {
        self.node_ref().is_leaf
    }

    pub fn node_view(&self) -> BTreeNodeView<'a, K, V, T> {
        BTreeNodeView {
            tree: self.tree,
            node_idx: self.node_idx,
        }
    }

    pub fn predecessor(&self) -> Option<Self> {
        let (node_idx, index) = self.tree.predecessor_location(self.node_idx, self.index)?;
        Some(Self {
            tree: self.tree,
            node_idx,
            index,
        })
    }

    pub fn successor(&self) -> Option<Self> {
        let (node_idx, index) = self.tree.successor_location(self.node_idx, self.index)?;
        Some(Self {
            tree: self.tree,
            node_idx,
            index,
        })
    }
}

#[derive(Debug)]
pub struct BTree<K, V, const T: usize> {
    root: Option<usize>,
    nodes: Vec<Option<BTreeNode<K, V>>>,
    free: Vec<usize>,
    len: usize,
}

impl<K: Ord, V, const T: usize> BTree<K, V, T> {
    pub fn new() -> Self {
        assert!(T >= 2, "minimum degree T must be >= 2");
        Self {
            root: None,
            nodes: Vec::new(),
            free: Vec::new(),
            len: 0,
        }
    }

    fn node_ref(&self, idx: usize) -> &BTreeNode<K, V> {
        self.nodes[idx].as_ref().expect("live arena node")
    }

    fn node_mut(&mut self, idx: usize) -> &mut BTreeNode<K, V> {
        self.nodes[idx].as_mut().expect("live arena node")
    }

    fn alloc_node(&mut self, node: BTreeNode<K, V>) -> usize {
        if let Some(idx) = self.free.pop() {
            self.nodes[idx] = Some(node);
            idx
        } else {
            self.nodes.push(Some(node));
            self.nodes.len() - 1
        }
    }

    fn take_node(&mut self, idx: usize) -> BTreeNode<K, V> {
        let node = self.nodes[idx].take().expect("live arena node");
        self.free.push(idx);
        node
    }

    pub fn root_view(&self) -> Option<BTreeNodeView<'_, K, V, T>> {
        self.root.map(|node_idx| BTreeNodeView {
            tree: self,
            node_idx,
        })
    }

    pub fn degree(&self) -> usize {
        T
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.nodes.clear();
        self.free.clear();
        self.len = 0;
    }

    fn search_node(&self, key: &K) -> Option<(usize, usize)> {
        let mut current = self.root?;

        loop {
            let idx = {
                let n = self.node_ref(current);
                match n.keys.binary_search(key) {
                    Ok(i) => return Some((current, i)),
                    Err(i) => i,
                }
            };

            if self.node_ref(current).is_leaf {
                return None;
            }

            current = self.node_ref(current).children[idx];
        }
    }

    pub fn cursor<'a>(&'a self, key: &K) -> Option<BTreeCursor<'a, K, V, T>> {
        self.search_node(key).map(|(node_idx, index)| BTreeCursor {
            tree: self,
            node_idx,
            index,
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.search_node(key).is_some()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<BTreeCursor<'a, K, V, T>> {
        let mut current = self.root?;

        loop {
            if self.node_ref(current).is_leaf {
                let len = self.node_ref(current).keys.len();
                if len == 0 {
                    return None;
                }
                return Some(BTreeCursor {
                    tree: self,
                    node_idx: current,
                    index: 0,
                });
            }

            current = self.node_ref(current).children[0];
        }
    }

    pub fn max_cursor<'a>(&'a self) -> Option<BTreeCursor<'a, K, V, T>> {
        let mut current = self.root?;

        loop {
            if self.node_ref(current).is_leaf {
                let len = self.node_ref(current).keys.len();
                if len == 0 {
                    return None;
                }
                return Some(BTreeCursor {
                    tree: self,
                    node_idx: current,
                    index: len - 1,
                });
            }

            let last = self.node_ref(current).children.len().saturating_sub(1);
            current = self.node_ref(current).children[last];
        }
    }

    fn predecessor_location(&self, node_idx: usize, index: usize) -> Option<(usize, usize)> {
        if !self.node_ref(node_idx).is_leaf {
            let mut current = self.node_ref(node_idx).children[index];
            loop {
                if self.node_ref(current).is_leaf {
                    let len = self.node_ref(current).keys.len();
                    return Some((current, len.saturating_sub(1)));
                }
                let last = self.node_ref(current).children.len().saturating_sub(1);
                current = self.node_ref(current).children[last];
            }
        }

        if index > 0 {
            return Some((node_idx, index - 1));
        }

        let mut current = node_idx;
        loop {
            let parent = self.node_ref(current).parent?;
            let child_idx = self
                .node_ref(parent)
                .children
                .iter()
                .position(|child| *child == current)?;

            if child_idx > 0 {
                return Some((parent, child_idx - 1));
            }

            current = parent;
        }
    }

    fn successor_location(&self, node_idx: usize, index: usize) -> Option<(usize, usize)> {
        if !self.node_ref(node_idx).is_leaf {
            let mut current = self.node_ref(node_idx).children[index + 1];
            loop {
                if self.node_ref(current).is_leaf {
                    return Some((current, 0));
                }
                current = self.node_ref(current).children[0];
            }
        }

        if index + 1 < self.node_ref(node_idx).keys.len() {
            return Some((node_idx, index + 1));
        }

        let mut current = node_idx;
        loop {
            let parent = self.node_ref(current).parent?;
            let child_idx = self
                .node_ref(parent)
                .children
                .iter()
                .position(|child| *child == current)?;

            if child_idx < self.node_ref(parent).keys.len() {
                return Some((parent, child_idx));
            }

            current = parent;
        }
    }

    pub fn insert_entry(&mut self, key: K, value: V) -> Option<V> {
        let root = match self.root {
            Some(root) => root,
            None => {
                let mut root = BTreeNode::new(true);
                root.keys.push(key);
                root.values.push(value);
                self.root = Some(self.alloc_node(root));
                self.len += 1;
                return None;
            }
        };

        let replaced = if self.node_ref(root).keys.len() == 2 * T - 1 {
            let new_root = self.alloc_node(BTreeNode::new(false));
            self.node_mut(new_root).children.push(root);
            self.node_mut(root).parent = Some(new_root);
            self.root = Some(new_root);

            self.split_child(new_root, 0, root);
            self.insert_non_full(new_root, key, value)
        } else {
            self.insert_non_full(root, key, value)
        };

        if replaced.is_none() {
            self.len += 1;
        }

        replaced
    }

    fn split_child(&mut self, parent: usize, index: usize, child: usize) {
        let is_leaf = self.node_ref(child).is_leaf;
        let mut new_right = BTreeNode::new(is_leaf);
        new_right.parent = Some(parent);

        let mut keys_to_move = self.node_mut(child).keys.split_off(T);
        let mut values_to_move = self.node_mut(child).values.split_off(T);
        let median = self.node_mut(child).keys.pop().expect("median key exists");
        let median_value = self
            .node_mut(child)
            .values
            .pop()
            .expect("median value exists");
        new_right.keys.append(&mut keys_to_move);
        new_right.values.append(&mut values_to_move);

        if !is_leaf {
            let mut children_to_move = self.node_mut(child).children.split_off(T);
            new_right.children.append(&mut children_to_move);
        }

        let new_right_idx = self.alloc_node(new_right);
        if !self.node_ref(new_right_idx).is_leaf {
            let moved = self.node_ref(new_right_idx).children.clone();
            for c in moved {
                self.node_mut(c).parent = Some(new_right_idx);
            }
        }

        self.node_mut(parent)
            .children
            .insert(index + 1, new_right_idx);
        self.node_mut(parent).keys.insert(index, median);
        self.node_mut(parent).values.insert(index, median_value);
    }

    fn insert_non_full(&mut self, mut node: usize, key: K, value: V) -> Option<V> {
        let value = value;
        loop {
            if self.node_ref(node).is_leaf {
                let index = match self.node_ref(node).keys.binary_search(&key) {
                    Ok(i) => {
                        return Some(std::mem::replace(&mut self.node_mut(node).values[i], value));
                    }
                    Err(i) => i,
                };
                self.node_mut(node).keys.insert(index, key);
                self.node_mut(node).values.insert(index, value);
                return None;
            }

            let mut child_index = match self.node_ref(node).keys.binary_search(&key) {
                Ok(i) => {
                    return Some(std::mem::replace(&mut self.node_mut(node).values[i], value));
                }
                Err(i) => i,
            };

            let child = self.node_ref(node).children[child_index];
            let child_is_full = self.node_ref(child).keys.len() == 2 * T - 1;

            if child_is_full {
                self.split_child(node, child_index, child);
                let split_cmp = {
                    let k = &self.node_ref(node).keys[child_index];
                    if key > *k {
                        Ordering::Greater
                    } else if key == *k {
                        Ordering::Equal
                    } else {
                        Ordering::Less
                    }
                };

                match split_cmp {
                    Ordering::Greater => child_index += 1,
                    Ordering::Equal => {
                        return Some(std::mem::replace(
                            &mut self.node_mut(node).values[child_index],
                            value,
                        ));
                    }
                    Ordering::Less => {}
                }
            }

            node = self.node_ref(node).children[child_index];
        }
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let (target_node, target_index) = self.search_node(key)?;

        let deleted_value;
        let leaf_to_fix;

        if self.node_ref(target_node).is_leaf {
            self.node_mut(target_node).keys.remove(target_index);
            deleted_value = self.node_mut(target_node).values.remove(target_index);
            leaf_to_fix = target_node;
        } else {
            let left_child = self.node_ref(target_node).children[target_index];
            let mut current = left_child;

            loop {
                if self.node_ref(current).is_leaf {
                    break;
                }
                let last = self.node_ref(current).children.len().saturating_sub(1);
                current = self.node_ref(current).children[last];
            }

            let predecessor_key = self
                .node_mut(current)
                .keys
                .pop()
                .expect("predecessor key exists");
            let predecessor_value = self
                .node_mut(current)
                .values
                .pop()
                .expect("predecessor value exists");

            self.node_mut(target_node).keys[target_index] = predecessor_key;
            deleted_value = std::mem::replace(
                &mut self.node_mut(target_node).values[target_index],
                predecessor_value,
            );
            leaf_to_fix = current;
        }

        self.fix_underflow(leaf_to_fix);
        self.len = self.len.saturating_sub(1);
        Some(deleted_value)
    }

    fn fix_underflow(&mut self, mut node: usize) {
        loop {
            if self.node_ref(node).parent.is_none() {
                if self.node_ref(node).keys.is_empty() {
                    if self.node_ref(node).is_leaf {
                        self.root = None;
                        let _ = self.take_node(node);
                    } else {
                        let new_root = self.node_mut(node).children.remove(0);
                        self.node_mut(new_root).parent = None;
                        self.root = Some(new_root);
                        let _ = self.take_node(node);
                    }
                }
                return;
            }

            if self.node_ref(node).keys.len() >= T - 1 {
                return;
            }

            let parent = self.node_ref(node).parent.expect("parent exists");
            let index = self
                .node_ref(parent)
                .children
                .iter()
                .position(|child| *child == node)
                .expect("node in parent children");

            let has_left = index > 0;
            let has_right = index + 1 < self.node_ref(parent).children.len();
            let mut borrowed = false;

            if has_left {
                let left_sibling = self.node_ref(parent).children[index - 1];
                if self.node_ref(left_sibling).keys.len() >= T {
                    let left_key = self.node_mut(left_sibling).keys.pop().expect("left key");
                    let left_value = self
                        .node_mut(left_sibling)
                        .values
                        .pop()
                        .expect("left value");
                    let sep_key =
                        std::mem::replace(&mut self.node_mut(parent).keys[index - 1], left_key);
                    let sep_value =
                        std::mem::replace(&mut self.node_mut(parent).values[index - 1], left_value);
                    self.node_mut(node).keys.insert(0, sep_key);
                    self.node_mut(node).values.insert(0, sep_value);

                    if !self.node_ref(node).is_leaf {
                        let left_child = self
                            .node_mut(left_sibling)
                            .children
                            .pop()
                            .expect("left child");
                        self.node_mut(left_child).parent = Some(node);
                        self.node_mut(node).children.insert(0, left_child);
                    }

                    borrowed = true;
                }
            }

            if !borrowed && has_right {
                let right_sibling = self.node_ref(parent).children[index + 1];
                if self.node_ref(right_sibling).keys.len() >= T {
                    let right_key = self.node_mut(right_sibling).keys.remove(0);
                    let right_value = self.node_mut(right_sibling).values.remove(0);
                    let sep_key =
                        std::mem::replace(&mut self.node_mut(parent).keys[index], right_key);
                    let sep_value =
                        std::mem::replace(&mut self.node_mut(parent).values[index], right_value);
                    self.node_mut(node).keys.push(sep_key);
                    self.node_mut(node).values.push(sep_value);

                    if !self.node_ref(node).is_leaf {
                        let right_child = self.node_mut(right_sibling).children.remove(0);
                        self.node_mut(right_child).parent = Some(node);
                        self.node_mut(node).children.push(right_child);
                    }

                    borrowed = true;
                }
            }

            if borrowed {
                return;
            }

            if has_left {
                let left_sibling = self.node_ref(parent).children[index - 1];
                let sep_key = self.node_mut(parent).keys.remove(index - 1);
                let sep_value = self.node_mut(parent).values.remove(index - 1);
                self.node_mut(parent).children.remove(index);

                self.node_mut(left_sibling).keys.push(sep_key);
                self.node_mut(left_sibling).values.push(sep_value);

                let mut node_keys = std::mem::take(&mut self.node_mut(node).keys);
                let mut node_values = std::mem::take(&mut self.node_mut(node).values);
                self.node_mut(left_sibling).keys.append(&mut node_keys);
                self.node_mut(left_sibling).values.append(&mut node_values);

                if !self.node_ref(node).is_leaf {
                    let moved_children = std::mem::take(&mut self.node_mut(node).children);
                    for child in &moved_children {
                        self.node_mut(*child).parent = Some(left_sibling);
                    }
                    self.node_mut(left_sibling).children.extend(moved_children);
                }

                let _ = self.take_node(node);
                node = parent;
            } else if has_right {
                let right_sibling = self.node_ref(parent).children[index + 1];
                let sep_key = self.node_mut(parent).keys.remove(index);
                let sep_value = self.node_mut(parent).values.remove(index);
                self.node_mut(parent).children.remove(index + 1);

                self.node_mut(node).keys.push(sep_key);
                self.node_mut(node).values.push(sep_value);

                let mut right_keys = std::mem::take(&mut self.node_mut(right_sibling).keys);
                let mut right_values = std::mem::take(&mut self.node_mut(right_sibling).values);
                self.node_mut(node).keys.append(&mut right_keys);
                self.node_mut(node).values.append(&mut right_values);

                if !self.node_ref(node).is_leaf {
                    let moved_children = std::mem::take(&mut self.node_mut(right_sibling).children);
                    for child in &moved_children {
                        self.node_mut(*child).parent = Some(node);
                    }
                    self.node_mut(node).children.extend(moved_children);
                }

                let _ = self.take_node(right_sibling);
                node = parent;
            } else {
                unreachable!("non-root node must have sibling");
            }
        }
    }

    pub fn iter<'a>(&'a self) -> BTreeIter<'a, K, V, T> {
        BTreeIter {
            next: self.min_cursor(),
        }
    }

    fn node_count_recursive(&self, node: Option<usize>) -> usize {
        let Some(idx) = node else {
            return 0;
        };

        let children = self.node_ref(idx).children.clone();
        1 + children
            .iter()
            .map(|child| self.node_count_recursive(Some(*child)))
            .sum::<usize>()
    }

    fn height_recursive(&self, node: Option<usize>) -> usize {
        let Some(idx) = node else {
            return 0;
        };

        if self.node_ref(idx).is_leaf {
            return 1;
        }

        let children = self.node_ref(idx).children.clone();
        1 + children
            .iter()
            .map(|child| self.height_recursive(Some(*child)))
            .max()
            .unwrap_or(0)
    }
}

impl<K: Ord, V, const T: usize> Map<K, V> for BTree<K, V, T> {
    type Cursor<'a>
        = BTreeCursor<'a, K, V, T>
    where
        Self: 'a;

    type View<'a>
        = BTreeNodeView<'a, K, V, T>
    where
        Self: 'a;

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        BTree::insert_entry(self, key, value)
    }

    fn cursor<'a>(&'a self, key: &K) -> Option<Self::Cursor<'a>> {
        BTree::cursor(self, key)
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        cursor.node_view()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        BTree::remove_key(self, key)
    }

    fn contains_key(&self, key: &K) -> bool {
        BTree::contains_key(self, key)
    }

    fn clear(&mut self) {
        BTree::clear(self)
    }

    fn len(&self) -> usize {
        BTree::len(self)
    }
}

impl<K: Ord, V, const T: usize> OrderedMap<K, V> for BTree<K, V, T> {
    fn first_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.min_cursor()
    }

    fn last_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.max_cursor()
    }
}

impl<K: Ord, V, const T: usize> TreeDiagnostics for BTree<K, V, T> {
    type NodeCursor<'a>
        = BTreeCursor<'a, K, V, T>
    where
        Self: 'a;

    fn height(&self) -> usize {
        self.height_recursive(self.root)
    }

    fn node_count(&self) -> usize {
        self.node_count_recursive(self.root)
    }

    fn node_height<'a>(&'a self, cursor: &Self::NodeCursor<'a>) -> usize {
        self.height_recursive(Some(cursor.node_idx))
    }
}

impl<K: Ord, V, const T: usize> Default for BTree<K, V, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V, const T: usize> FromIterator<(K, V)> for BTree<K, V, T> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut tree = Self::new();
        for (key, value) in iter {
            tree.insert_entry(key, value);
        }
        tree
    }
}

impl<'a, K: Ord + Clone, V: Clone, const T: usize> Iterator for BTreeIter<'a, K, V, T> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next.take()?;
        self.next = current.successor();

        let key = current.key().clone();
        let value = current.value().clone();
        Some((key, value))
    }
}

impl<'a, K: Ord + Clone, V: Clone, const T: usize> IntoIterator for &'a BTree<K, V, T> {
    type Item = (K, V);
    type IntoIter = BTreeIter<'a, K, V, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
