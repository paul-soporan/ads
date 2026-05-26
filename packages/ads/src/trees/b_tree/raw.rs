#![allow(dangerous_implicit_autorefs)]

use std::{cmp::Ordering, ptr};

use crate::traits::{
    core::{Map, OrderedMap},
    diagnostics::TreeDiagnostics,
};

#[derive(Debug)]
struct BTreeNode<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    children: Vec<*mut BTreeNode<K, V>>,
    parent: *mut BTreeNode<K, V>,
    is_leaf: bool,
}

impl<K, V> BTreeNode<K, V> {
    fn new(is_leaf: bool) -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            parent: ptr::null_mut(),
            is_leaf,
        }
    }
}

#[derive(Debug)]
pub struct BTreeNodeView<K, V, const T: usize> {
    tree: *const BTree<K, V, T>,
    node: *mut BTreeNode<K, V>,
}

#[derive(Debug)]
pub struct BTreeCursor<K, V, const T: usize> {
    tree: *const BTree<K, V, T>,
    node: *mut BTreeNode<K, V>,
    index: usize,
}

#[derive(Debug)]
pub struct BTreeIter<'a, K, V, const T: usize> {
    _tree: &'a BTree<K, V, T>,
    next: Option<BTreeCursor<K, V, T>>,
}

impl<K, V, const T: usize> Clone for BTreeNodeView<K, V, T> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node,
        }
    }
}

impl<K, V, const T: usize> Clone for BTreeCursor<K, V, T> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node,
            index: self.index,
        }
    }
}

impl<K, V, const T: usize> BTreeNodeView<K, V, T> {
    fn node_ref(&self) -> &BTreeNode<K, V> {
        // SAFETY: created from a live tree node.
        unsafe { &*self.node }
    }
}

impl<K: Clone, V: Clone, const T: usize> BTreeNodeView<K, V, T> {
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

    pub fn children(&self) -> Vec<BTreeNodeView<K, V, T>> {
        self.node_ref()
            .children
            .iter()
            .copied()
            .map(|child| BTreeNodeView {
                tree: self.tree,
                node: child,
            })
            .collect()
    }

    pub fn parent(&self) -> Option<BTreeNodeView<K, V, T>> {
        let p = self.node_ref().parent;
        (!p.is_null()).then(|| BTreeNodeView {
            tree: self.tree,
            node: p,
        })
    }
}

impl<K: Ord, V, const T: usize> BTreeCursor<K, V, T> {
    fn tree_ref(&self) -> &BTree<K, V, T> {
        // SAFETY: created from a live tree reference.
        unsafe { &*self.tree }
    }

    fn node_ref(&self) -> &BTreeNode<K, V> {
        // SAFETY: created from a live tree node.
        unsafe { &*self.node }
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

    pub fn node_view(&self) -> BTreeNodeView<K, V, T> {
        BTreeNodeView {
            tree: self.tree,
            node: self.node,
        }
    }

    pub fn predecessor(&self) -> Option<Self> {
        let (node, index) = self
            .tree_ref()
            .predecessor_location(self.node, self.index)?;
        Some(Self {
            tree: self.tree,
            node,
            index,
        })
    }

    pub fn successor(&self) -> Option<Self> {
        let (node, index) = self.tree_ref().successor_location(self.node, self.index)?;
        Some(Self {
            tree: self.tree,
            node,
            index,
        })
    }
}

#[derive(Debug)]
pub struct BTree<K, V, const T: usize> {
    root: *mut BTreeNode<K, V>,
    len: usize,
}

impl<K: Ord, V, const T: usize> BTree<K, V, T> {
    pub fn new() -> Self {
        assert!(T >= 2, "minimum degree T must be >= 2");
        Self {
            root: ptr::null_mut(),
            len: 0,
        }
    }

    fn alloc_node(&self, node: BTreeNode<K, V>) -> *mut BTreeNode<K, V> {
        Box::into_raw(Box::new(node))
    }

    fn drop_subtree(node: *mut BTreeNode<K, V>) {
        if node.is_null() {
            return;
        }

        // SAFETY: node is valid and owned by this tree.
        let mut children = unsafe { std::mem::take(&mut (*node).children) };
        for child in children.drain(..) {
            Self::drop_subtree(child);
        }

        // SAFETY: dropped exactly once.
        unsafe {
            drop(Box::from_raw(node));
        }
    }

    pub fn root_view(&self) -> Option<BTreeNodeView<K, V, T>> {
        (!self.root.is_null()).then(|| BTreeNodeView {
            tree: self as *const Self,
            node: self.root,
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
        Self::drop_subtree(self.root);
        self.root = ptr::null_mut();
        self.len = 0;
    }

    fn search_node(&self, key: &K) -> Option<(*mut BTreeNode<K, V>, usize)> {
        if self.root.is_null() {
            return None;
        }
        let mut current = self.root;

        loop {
            // SAFETY: current valid.
            let idx = unsafe {
                match (*current).keys.binary_search(key) {
                    Ok(i) => return Some((current, i)),
                    Err(i) => i,
                }
            };

            // SAFETY: current valid.
            if unsafe { (*current).is_leaf } {
                return None;
            }

            // SAFETY: current valid and internal.
            current = unsafe { (*current).children[idx] };
        }
    }

    pub fn cursor<'a>(&'a self, key: &K) -> Option<BTreeCursor<K, V, T>> {
        self.search_node(key).map(|(node, index)| BTreeCursor {
            tree: self as *const Self,
            node,
            index,
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.search_node(key).is_some()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<BTreeCursor<K, V, T>> {
        if self.root.is_null() {
            return None;
        }
        let mut current = self.root;

        loop {
            // SAFETY: current valid.
            if unsafe { (*current).is_leaf } {
                // SAFETY: current valid.
                let len = unsafe { (*current).keys.len() };
                if len == 0 {
                    return None;
                }
                return Some(BTreeCursor {
                    tree: self as *const Self,
                    node: current,
                    index: 0,
                });
            }

            // SAFETY: internal node has at least one child.
            current = unsafe { (*current).children[0] };
        }
    }

    pub fn max_cursor<'a>(&'a self) -> Option<BTreeCursor<K, V, T>> {
        if self.root.is_null() {
            return None;
        }
        let mut current = self.root;

        loop {
            // SAFETY: current valid.
            if unsafe { (*current).is_leaf } {
                // SAFETY: current valid.
                let len = unsafe { (*current).keys.len() };
                if len == 0 {
                    return None;
                }
                return Some(BTreeCursor {
                    tree: self as *const Self,
                    node: current,
                    index: len - 1,
                });
            }

            // SAFETY: current valid.
            let last = unsafe { (*current).children.len().saturating_sub(1) };
            // SAFETY: index in bounds for internal node.
            current = unsafe { (*current).children[last] };
        }
    }

    fn predecessor_location(
        &self,
        node: *mut BTreeNode<K, V>,
        index: usize,
    ) -> Option<(*mut BTreeNode<K, V>, usize)> {
        if node.is_null() {
            return None;
        }

        // SAFETY: node valid.
        if unsafe { !(*node).is_leaf } {
            // SAFETY: child index valid for internal node.
            let mut current = unsafe { (*node).children[index] };
            loop {
                // SAFETY: current valid.
                if unsafe { (*current).is_leaf } {
                    // SAFETY: current valid.
                    let len = unsafe { (*current).keys.len() };
                    return Some((current, len.saturating_sub(1)));
                }
                // SAFETY: current valid.
                let last = unsafe { (*current).children.len().saturating_sub(1) };
                // SAFETY: internal node has child at last.
                current = unsafe { (*current).children[last] };
            }
        }

        // SAFETY: node valid.
        if index > 0 {
            return Some((node, index - 1));
        }

        let mut current = node;
        loop {
            // SAFETY: current valid.
            let parent = unsafe { (*current).parent };
            if parent.is_null() {
                return None;
            }

            // SAFETY: parent valid.
            let child_idx = unsafe {
                (*parent)
                    .children
                    .iter()
                    .position(|child| *child == current)
            }?;
            if child_idx > 0 {
                return Some((parent, child_idx - 1));
            }
            current = parent;
        }
    }

    fn successor_location(
        &self,
        node: *mut BTreeNode<K, V>,
        index: usize,
    ) -> Option<(*mut BTreeNode<K, V>, usize)> {
        if node.is_null() {
            return None;
        }

        // SAFETY: node valid.
        if unsafe { !(*node).is_leaf } {
            // SAFETY: child index valid.
            let mut current = unsafe { (*node).children[index + 1] };
            loop {
                // SAFETY: current valid.
                if unsafe { (*current).is_leaf } {
                    return Some((current, 0));
                }
                // SAFETY: internal node has child 0.
                current = unsafe { (*current).children[0] };
            }
        }

        // SAFETY: node valid.
        if unsafe { index + 1 < (*node).keys.len() } {
            return Some((node, index + 1));
        }

        let mut current = node;
        loop {
            // SAFETY: current valid.
            let parent = unsafe { (*current).parent };
            if parent.is_null() {
                return None;
            }

            // SAFETY: parent valid.
            let child_idx = unsafe {
                (*parent)
                    .children
                    .iter()
                    .position(|child| *child == current)
            }?;
            // SAFETY: parent valid.
            let parent_keys = unsafe { (*parent).keys.len() };
            if child_idx < parent_keys {
                return Some((parent, child_idx));
            }
            current = parent;
        }
    }

    pub fn insert_entry(&mut self, key: K, value: V) -> Option<V> {
        if self.root.is_null() {
            let mut root = BTreeNode::new(true);
            root.keys.push(key);
            root.values.push(value);
            self.root = self.alloc_node(root);
            self.len += 1;
            return None;
        }

        // SAFETY: root valid.
        let root = self.root;
        let replaced = if unsafe { (*root).keys.len() } == 2 * T - 1 {
            let new_root = self.alloc_node(BTreeNode::new(false));
            // SAFETY: pointers valid.
            unsafe {
                (*new_root).children.push(root);
                (*root).parent = new_root;
            }
            self.root = new_root;

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

    fn split_child(
        &mut self,
        parent: *mut BTreeNode<K, V>,
        index: usize,
        child: *mut BTreeNode<K, V>,
    ) {
        // SAFETY: child valid.
        let is_leaf = unsafe { (*child).is_leaf };
        let mut new_right = BTreeNode::new(is_leaf);
        new_right.parent = parent;

        // SAFETY: child valid.
        let mut keys_to_move = unsafe { (*child).keys.split_off(T) };
        // SAFETY: child valid.
        let mut values_to_move = unsafe { (*child).values.split_off(T) };
        // SAFETY: child has median.
        let median = unsafe { (*child).keys.pop().expect("median key exists") };
        // SAFETY: child has median value.
        let median_value = unsafe { (*child).values.pop().expect("median value exists") };

        new_right.keys.append(&mut keys_to_move);
        new_right.values.append(&mut values_to_move);

        if !is_leaf {
            // SAFETY: child valid.
            let mut children_to_move = unsafe { (*child).children.split_off(T) };
            new_right.children.append(&mut children_to_move);
        }

        let new_right_ptr = self.alloc_node(new_right);

        // SAFETY: new_right_ptr valid.
        if unsafe { !(*new_right_ptr).is_leaf } {
            // SAFETY: children valid.
            let children = unsafe { (*new_right_ptr).children.clone() };
            for c in children {
                // SAFETY: child pointer valid.
                unsafe { (*c).parent = new_right_ptr };
            }
        }

        // SAFETY: parent valid.
        unsafe {
            (*parent).children.insert(index + 1, new_right_ptr);
            (*parent).keys.insert(index, median);
            (*parent).values.insert(index, median_value);
        }
    }

    fn insert_non_full(&mut self, mut node: *mut BTreeNode<K, V>, key: K, value: V) -> Option<V> {
        let value = value;
        loop {
            // SAFETY: node valid.
            if unsafe { (*node).is_leaf } {
                // SAFETY: node valid.
                let index = unsafe {
                    match (*node).keys.binary_search(&key) {
                        Ok(i) => {
                            return Some(std::mem::replace(&mut (*node).values[i], value));
                        }
                        Err(i) => i,
                    }
                };
                // SAFETY: node valid.
                unsafe {
                    (*node).keys.insert(index, key);
                    (*node).values.insert(index, value);
                }
                return None;
            }

            // SAFETY: node valid.
            let mut child_index = unsafe {
                match (*node).keys.binary_search(&key) {
                    Ok(i) => {
                        return Some(std::mem::replace(&mut (*node).values[i], value));
                    }
                    Err(i) => i,
                }
            };

            // SAFETY: node valid and internal.
            let child = unsafe { (*node).children[child_index] };
            // SAFETY: child valid.
            let child_is_full = unsafe { (*child).keys.len() == 2 * T - 1 };

            if child_is_full {
                self.split_child(node, child_index, child);
                let split_cmp = {
                    // SAFETY: node valid.
                    let k = unsafe { &(*node).keys[child_index] };
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
                        // SAFETY: node valid.
                        return Some(unsafe {
                            std::mem::replace(&mut (*node).values[child_index], value)
                        });
                    }
                    Ordering::Less => {}
                }
            }

            // SAFETY: node valid and internal.
            node = unsafe { (*node).children[child_index] };
        }
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let (target_node, target_index) = self.search_node(key)?;

        let deleted_value;
        let leaf_to_fix;

        // SAFETY: target valid.
        if unsafe { (*target_node).is_leaf } {
            // SAFETY: target valid.
            unsafe {
                (*target_node).keys.remove(target_index);
                deleted_value = (*target_node).values.remove(target_index);
            }
            leaf_to_fix = target_node;
        } else {
            // SAFETY: target valid internal.
            let left_child = unsafe { (*target_node).children[target_index] };
            let mut current = left_child;

            loop {
                // SAFETY: current valid.
                if unsafe { (*current).is_leaf } {
                    break;
                }
                // SAFETY: current valid internal.
                let last = unsafe { (*current).children.len().saturating_sub(1) };
                // SAFETY: index valid.
                current = unsafe { (*current).children[last] };
            }

            let (pred_key, pred_value) = {
                // SAFETY: current leaf has at least one key.
                unsafe {
                    let k = (*current).keys.pop().expect("predecessor key exists");
                    let v = (*current).values.pop().expect("predecessor value exists");
                    (k, v)
                }
            };

            // SAFETY: target valid.
            unsafe {
                (*target_node).keys[target_index] = pred_key;
                deleted_value =
                    std::mem::replace(&mut (*target_node).values[target_index], pred_value);
            }
            leaf_to_fix = current;
        }

        self.fix_underflow(leaf_to_fix);
        self.len = self.len.saturating_sub(1);
        Some(deleted_value)
    }

    fn fix_underflow(&mut self, mut node: *mut BTreeNode<K, V>) {
        loop {
            // SAFETY: node valid.
            let parent = unsafe { (*node).parent };
            if parent.is_null() {
                // SAFETY: node valid root.
                let keys_empty = unsafe { (*node).keys.is_empty() };
                if keys_empty {
                    // SAFETY: node valid.
                    let is_leaf = unsafe { (*node).is_leaf };
                    if is_leaf {
                        self.root = ptr::null_mut();
                        // SAFETY: node valid and detached.
                        unsafe { drop(Box::from_raw(node)) };
                    } else {
                        // SAFETY: node valid.
                        let new_root = unsafe { (*node).children.remove(0) };
                        // SAFETY: child valid.
                        unsafe { (*new_root).parent = ptr::null_mut() };
                        self.root = new_root;
                        // SAFETY: old root detached.
                        unsafe { drop(Box::from_raw(node)) };
                    }
                }
                return;
            }

            // SAFETY: node valid.
            if unsafe { (*node).keys.len() } >= T - 1 {
                return;
            }

            // SAFETY: parent valid.
            let index = unsafe {
                (*parent)
                    .children
                    .iter()
                    .position(|child| *child == node)
                    .expect("node in parent children")
            };

            // SAFETY: parent valid.
            let has_left = index > 0;
            // SAFETY: parent valid.
            let has_right = unsafe { index + 1 < (*parent).children.len() };
            let mut borrowed = false;

            if has_left {
                // SAFETY: parent valid.
                let left_sibling = unsafe { (*parent).children[index - 1] };
                // SAFETY: sibling valid.
                if unsafe { (*left_sibling).keys.len() } >= T {
                    // rotate from left sibling
                    // SAFETY: pointers valid.
                    unsafe {
                        let sep_key = std::mem::replace(
                            &mut (*parent).keys[index - 1],
                            (*left_sibling).keys.pop().expect("left key"),
                        );
                        let sep_val = std::mem::replace(
                            &mut (*parent).values[index - 1],
                            (*left_sibling).values.pop().expect("left value"),
                        );
                        (*node).keys.insert(0, sep_key);
                        (*node).values.insert(0, sep_val);

                        if !(*node).is_leaf {
                            let left_child = (*left_sibling).children.pop().expect("left child");
                            (*left_child).parent = node;
                            (*node).children.insert(0, left_child);
                        }
                    }
                    borrowed = true;
                }
            }

            if !borrowed && has_right {
                // SAFETY: parent valid.
                let right_sibling = unsafe { (*parent).children[index + 1] };
                // SAFETY: sibling valid.
                if unsafe { (*right_sibling).keys.len() } >= T {
                    // rotate from right sibling
                    // SAFETY: pointers valid.
                    unsafe {
                        let sep_key = std::mem::replace(
                            &mut (*parent).keys[index],
                            (*right_sibling).keys.remove(0),
                        );
                        let sep_val = std::mem::replace(
                            &mut (*parent).values[index],
                            (*right_sibling).values.remove(0),
                        );
                        (*node).keys.push(sep_key);
                        (*node).values.push(sep_val);

                        if !(*node).is_leaf {
                            let right_child = (*right_sibling).children.remove(0);
                            (*right_child).parent = node;
                            (*node).children.push(right_child);
                        }
                    }
                    borrowed = true;
                }
            }

            if borrowed {
                return;
            }

            if has_left {
                // SAFETY: parent valid.
                let left_sibling = unsafe { (*parent).children[index - 1] };

                // SAFETY: pointers valid.
                unsafe {
                    let sep_key = (*parent).keys.remove(index - 1);
                    let sep_val = (*parent).values.remove(index - 1);
                    (*parent).children.remove(index);

                    (*left_sibling).keys.push(sep_key);
                    (*left_sibling).values.push(sep_val);
                    (*left_sibling).keys.append(&mut (*node).keys);
                    (*left_sibling).values.append(&mut (*node).values);

                    if !(*node).is_leaf {
                        for child in &(*node).children {
                            (*(*child)).parent = left_sibling;
                        }
                        (*left_sibling).children.append(&mut (*node).children);
                    }
                }

                // SAFETY: node removed from tree.
                unsafe { drop(Box::from_raw(node)) };
                node = parent;
            } else if has_right {
                // SAFETY: parent valid.
                let right_sibling = unsafe { (*parent).children[index + 1] };

                // SAFETY: pointers valid.
                unsafe {
                    let sep_key = (*parent).keys.remove(index);
                    let sep_val = (*parent).values.remove(index);
                    (*parent).children.remove(index + 1);

                    (*node).keys.push(sep_key);
                    (*node).values.push(sep_val);
                    (*node).keys.append(&mut (*right_sibling).keys);
                    (*node).values.append(&mut (*right_sibling).values);

                    if !(*node).is_leaf {
                        for child in &(*right_sibling).children {
                            (*(*child)).parent = node;
                        }
                        (*node).children.append(&mut (*right_sibling).children);
                    }
                }

                // SAFETY: sibling removed from tree.
                unsafe { drop(Box::from_raw(right_sibling)) };
                node = parent;
            } else {
                unreachable!("non-root node must have a sibling");
            }
        }
    }

    pub fn iter<'a>(&'a self) -> BTreeIter<'a, K, V, T> {
        BTreeIter {
            _tree: self,
            next: self.min_cursor(),
        }
    }

    fn node_count_recursive(node: *mut BTreeNode<K, V>) -> usize {
        if node.is_null() {
            return 0;
        }
        // SAFETY: node valid.
        let children = unsafe { (*node).children.clone() };
        1 + children
            .iter()
            .map(|child| Self::node_count_recursive(*child))
            .sum::<usize>()
    }

    fn height_recursive(node: *mut BTreeNode<K, V>) -> usize {
        if node.is_null() {
            return 0;
        }
        // SAFETY: node valid.
        if unsafe { (*node).is_leaf } {
            return 1;
        }

        // SAFETY: node valid.
        let children = unsafe { (*node).children.clone() };
        1 + children
            .iter()
            .map(|child| Self::height_recursive(*child))
            .max()
            .unwrap_or(0)
    }
}

impl<K, V, const T: usize> Drop for BTree<K, V, T> {
    fn drop(&mut self) {
        fn drop_subtree<K, V>(node: *mut BTreeNode<K, V>) {
            if node.is_null() {
                return;
            }

            // SAFETY: node is valid and owned by this tree.
            let mut children = unsafe { std::mem::take(&mut (*node).children) };
            for child in children.drain(..) {
                drop_subtree(child);
            }

            // SAFETY: dropped exactly once.
            unsafe {
                drop(Box::from_raw(node));
            }
        }

        drop_subtree(self.root);
        self.root = ptr::null_mut();
        self.len = 0;
    }
}

impl<K: Ord, V, const T: usize> Map<K, V> for BTree<K, V, T> {
    type Cursor<'a>
        = BTreeCursor<K, V, T>
    where
        Self: 'a;

    type View<'a>
        = BTreeNodeView<K, V, T>
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
        = BTreeCursor<K, V, T>
    where
        Self: 'a;

    fn height(&self) -> usize {
        Self::height_recursive(self.root)
    }

    fn node_count(&self) -> usize {
        Self::node_count_recursive(self.root)
    }

    fn node_height<'a>(&'a self, cursor: &Self::NodeCursor<'a>) -> usize {
        Self::height_recursive(cursor.node)
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
