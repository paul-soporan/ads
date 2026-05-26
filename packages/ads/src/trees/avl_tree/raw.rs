use std::ptr;

use crate::traits::{
    core::{Map, OrderedMap},
    diagnostics::TreeDiagnostics,
};

#[derive(Debug)]
struct AvlNode<K, V> {
    key: K,
    value: V,
    height: i32,
    left: *mut AvlNode<K, V>,
    right: *mut AvlNode<K, V>,
    parent: *mut AvlNode<K, V>,
}

impl<K, V> AvlNode<K, V> {
    fn new(key: K, value: V, parent: *mut AvlNode<K, V>) -> Self {
        Self {
            key,
            value,
            height: 1,
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            parent,
        }
    }
}

#[derive(Debug)]
pub struct AvlNodeView<K, V> {
    tree: *const AvlTree<K, V>,
    node: *mut AvlNode<K, V>,
}

#[derive(Debug)]
pub struct AvlCursor<K, V> {
    tree: *const AvlTree<K, V>,
    node: *mut AvlNode<K, V>,
}

#[derive(Debug)]
pub struct AvlIter<'a, K, V> {
    tree: &'a AvlTree<K, V>,
    next: *mut AvlNode<K, V>,
}

impl<K, V> Clone for AvlNodeView<K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node,
        }
    }
}

impl<K, V> Clone for AvlCursor<K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node,
        }
    }
}

impl<K, V> AvlNodeView<K, V> {
    fn node_ref(&self) -> &AvlNode<K, V> {
        // SAFETY: created from a live node owned by the tree.
        unsafe { &*self.node }
    }

    pub fn key(&self) -> &K {
        &self.node_ref().key
    }

    pub fn value(&self) -> &V {
        &self.node_ref().value
    }

    pub fn left(&self) -> Option<Self> {
        let left = self.node_ref().left;
        (!left.is_null()).then(|| Self {
            tree: self.tree,
            node: left,
        })
    }

    pub fn right(&self) -> Option<Self> {
        let right = self.node_ref().right;
        (!right.is_null()).then(|| Self {
            tree: self.tree,
            node: right,
        })
    }

    pub fn parent(&self) -> Option<Self> {
        let parent = self.node_ref().parent;
        (!parent.is_null()).then(|| Self {
            tree: self.tree,
            node: parent,
        })
    }
}

impl<K: Ord, V> AvlCursor<K, V> {
    fn tree_ref(&self) -> &AvlTree<K, V> {
        // SAFETY: created from a live tree reference.
        unsafe { &*self.tree }
    }

    fn node_ref(&self) -> &AvlNode<K, V> {
        // SAFETY: created from a live node owned by the tree.
        unsafe { &*self.node }
    }

    pub fn key(&self) -> &K {
        &self.node_ref().key
    }

    pub fn value(&self) -> &V {
        &self.node_ref().value
    }

    pub fn node_view(&self) -> AvlNodeView<K, V> {
        AvlNodeView {
            tree: self.tree,
            node: self.node,
        }
    }

    pub fn predecessor(&self) -> Option<Self> {
        let prev = self.tree_ref().predecessor_node(self.node);
        (!prev.is_null()).then(|| Self {
            tree: self.tree,
            node: prev,
        })
    }

    pub fn successor(&self) -> Option<Self> {
        let next = self.tree_ref().successor_node(self.node);
        (!next.is_null()).then(|| Self {
            tree: self.tree,
            node: next,
        })
    }
}

#[derive(Debug)]
pub struct AvlTree<K, V> {
    root: *mut AvlNode<K, V>,
    len: usize,
}

impl<K, V> AvlTree<K, V> {
    pub fn new() -> Self {
        Self {
            root: ptr::null_mut(),
            len: 0,
        }
    }

    fn drop_subtree(node: *mut AvlNode<K, V>) {
        if node.is_null() {
            return;
        }

        // SAFETY: node is valid and owned by this tree.
        let (left, right) = unsafe { ((*node).left, (*node).right) };
        Self::drop_subtree(left);
        Self::drop_subtree(right);

        // SAFETY: each node is dropped exactly once by post-order traversal.
        unsafe {
            drop(Box::from_raw(node));
        }
    }

    fn leftmost(mut node: *mut AvlNode<K, V>) -> *mut AvlNode<K, V> {
        while !node.is_null() {
            // SAFETY: node is valid while traversing links.
            let left = unsafe { (*node).left };
            if left.is_null() {
                return node;
            }
            node = left;
        }
        ptr::null_mut()
    }

    fn rightmost(mut node: *mut AvlNode<K, V>) -> *mut AvlNode<K, V> {
        while !node.is_null() {
            // SAFETY: node is valid while traversing links.
            let right = unsafe { (*node).right };
            if right.is_null() {
                return node;
            }
            node = right;
        }
        ptr::null_mut()
    }

    fn node_height(node: *mut AvlNode<K, V>) -> i32 {
        if node.is_null() {
            0
        } else {
            // SAFETY: node is valid while reachable from root.
            unsafe { (*node).height }
        }
    }

    fn update_height(node: *mut AvlNode<K, V>) {
        if node.is_null() {
            return;
        }

        // SAFETY: node is valid while reachable from root.
        unsafe {
            let lh = Self::node_height((*node).left);
            let rh = Self::node_height((*node).right);
            (*node).height = 1 + lh.max(rh);
        }
    }

    fn balance(node: *mut AvlNode<K, V>) -> i32 {
        if node.is_null() {
            0
        } else {
            // SAFETY: node is valid while reachable from root.
            unsafe { Self::node_height((*node).left) - Self::node_height((*node).right) }
        }
    }

    fn height_from(node: *mut AvlNode<K, V>) -> usize {
        if node.is_null() {
            return 0;
        }

        // SAFETY: node is valid while reachable from root.
        unsafe {
            1 + usize::max(
                Self::height_from((*node).left),
                Self::height_from((*node).right),
            )
        }
    }

    pub fn root_view(&self) -> Option<AvlNodeView<K, V>> {
        (!self.root.is_null()).then(|| AvlNodeView {
            tree: self as *const Self,
            node: self.root,
        })
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
}

impl<K: Ord, V> AvlTree<K, V> {
    fn find_node(&self, key: &K) -> *mut AvlNode<K, V> {
        let mut current = self.root;
        while !current.is_null() {
            // SAFETY: current is valid while reachable from root.
            let ord = unsafe { key.cmp(&(*current).key) };
            current = match ord {
                std::cmp::Ordering::Less => unsafe { (*current).left },
                std::cmp::Ordering::Greater => unsafe { (*current).right },
                std::cmp::Ordering::Equal => return current,
            };
        }
        ptr::null_mut()
    }

    fn predecessor_node(&self, node: *mut AvlNode<K, V>) -> *mut AvlNode<K, V> {
        if node.is_null() {
            return ptr::null_mut();
        }

        // SAFETY: node is valid.
        let left = unsafe { (*node).left };
        if !left.is_null() {
            return Self::rightmost(left);
        }

        let mut current = node;
        // SAFETY: current is valid while climbing parent links.
        let mut parent = unsafe { (*current).parent };
        while !parent.is_null() {
            // SAFETY: parent/current are valid linked nodes.
            let is_right = unsafe { (*parent).right == current };
            if is_right {
                return parent;
            }
            current = parent;
            // SAFETY: current is valid.
            parent = unsafe { (*current).parent };
        }

        ptr::null_mut()
    }

    fn successor_node(&self, node: *mut AvlNode<K, V>) -> *mut AvlNode<K, V> {
        if node.is_null() {
            return ptr::null_mut();
        }

        // SAFETY: node is valid.
        let right = unsafe { (*node).right };
        if !right.is_null() {
            return Self::leftmost(right);
        }

        let mut current = node;
        // SAFETY: current is valid while climbing parent links.
        let mut parent = unsafe { (*current).parent };
        while !parent.is_null() {
            // SAFETY: parent/current are valid linked nodes.
            let is_left = unsafe { (*parent).left == current };
            if is_left {
                return parent;
            }
            current = parent;
            // SAFETY: current is valid.
            parent = unsafe { (*current).parent };
        }

        ptr::null_mut()
    }

    fn replace_parent_child(
        &mut self,
        parent: *mut AvlNode<K, V>,
        old_child: *mut AvlNode<K, V>,
        new_child: *mut AvlNode<K, V>,
    ) {
        if parent.is_null() {
            self.root = new_child;
        } else {
            // SAFETY: parent is valid.
            let is_left = unsafe { (*parent).left == old_child };
            if is_left {
                // SAFETY: parent is valid.
                unsafe { (*parent).left = new_child };
            } else {
                // SAFETY: parent is valid.
                unsafe { (*parent).right = new_child };
            }
        }

        if !new_child.is_null() {
            // SAFETY: new_child is valid.
            unsafe { (*new_child).parent = parent };
        }
    }

    fn rotate_left(&mut self, x: *mut AvlNode<K, V>) -> *mut AvlNode<K, V> {
        // SAFETY: x is valid and has right child for left rotation.
        let y = unsafe { (*x).right };
        // SAFETY: y is valid for this rotation.
        let y_left = unsafe { (*y).left };

        // SAFETY: rewiring local subtree pointers preserves tree ownership.
        unsafe {
            (*x).right = y_left;
            if !y_left.is_null() {
                (*y_left).parent = x;
            }

            let x_parent = (*x).parent;
            (*y).left = x;
            (*y).parent = x_parent;
            (*x).parent = y;

            self.replace_parent_child(x_parent, x, y);
        }

        Self::update_height(x);
        Self::update_height(y);
        y
    }

    fn rotate_right(&mut self, x: *mut AvlNode<K, V>) -> *mut AvlNode<K, V> {
        // SAFETY: x is valid and has left child for right rotation.
        let y = unsafe { (*x).left };
        // SAFETY: y is valid for this rotation.
        let y_right = unsafe { (*y).right };

        // SAFETY: rewiring local subtree pointers preserves tree ownership.
        unsafe {
            (*x).left = y_right;
            if !y_right.is_null() {
                (*y_right).parent = x;
            }

            let x_parent = (*x).parent;
            (*y).right = x;
            (*y).parent = x_parent;
            (*x).parent = y;

            self.replace_parent_child(x_parent, x, y);
        }

        Self::update_height(x);
        Self::update_height(y);
        y
    }

    fn rebalance_upwards(&mut self, mut current: *mut AvlNode<K, V>) {
        while !current.is_null() {
            Self::update_height(current);
            let b = Self::balance(current);

            if b > 1 {
                // SAFETY: current is valid.
                let left = unsafe { (*current).left };
                if Self::balance(left) < 0 {
                    self.rotate_left(left);
                }
                let new_root = self.rotate_right(current);
                // SAFETY: new_root is valid.
                current = unsafe { (*new_root).parent };
                continue;
            }

            if b < -1 {
                // SAFETY: current is valid.
                let right = unsafe { (*current).right };
                if Self::balance(right) > 0 {
                    self.rotate_right(right);
                }
                let new_root = self.rotate_left(current);
                // SAFETY: new_root is valid.
                current = unsafe { (*new_root).parent };
                continue;
            }

            // SAFETY: current is valid.
            current = unsafe { (*current).parent };
        }
    }

    fn transplant(&mut self, u: *mut AvlNode<K, V>, v: *mut AvlNode<K, V>) {
        // SAFETY: u is a valid node.
        let parent = unsafe { (*u).parent };
        self.replace_parent_child(parent, u, v);
    }

    pub fn cursor<'a>(&'a self, key: &K) -> Option<AvlCursor<K, V>> {
        let node = self.find_node(key);
        (!node.is_null()).then(|| AvlCursor {
            tree: self as *const Self,
            node,
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        !self.find_node(key).is_null()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<AvlCursor<K, V>> {
        let node = Self::leftmost(self.root);
        (!node.is_null()).then(|| AvlCursor {
            tree: self as *const Self,
            node,
        })
    }

    pub fn max_cursor<'a>(&'a self) -> Option<AvlCursor<K, V>> {
        let node = Self::rightmost(self.root);
        (!node.is_null()).then(|| AvlCursor {
            tree: self as *const Self,
            node,
        })
    }

    pub fn insert_entry(&mut self, key: K, value: V) -> Option<V> {
        let mut parent = ptr::null_mut();
        let mut current = self.root;

        while !current.is_null() {
            parent = current;
            // SAFETY: current is valid.
            match key.cmp(unsafe { &(*current).key }) {
                std::cmp::Ordering::Less => {
                    // SAFETY: current is valid.
                    current = unsafe { (*current).left };
                }
                std::cmp::Ordering::Greater => {
                    // SAFETY: current is valid.
                    current = unsafe { (*current).right };
                }
                std::cmp::Ordering::Equal => {
                    // SAFETY: current is valid.
                    let old = unsafe { std::mem::replace(&mut (*current).value, value) };
                    return Some(old);
                }
            }
        }

        let node = Box::into_raw(Box::new(AvlNode::new(key, value, parent)));
        if parent.is_null() {
            self.root = node;
        } else {
            // SAFETY: parent/node are valid.
            let go_left = unsafe { (*node).key < (*parent).key };
            if go_left {
                // SAFETY: parent is valid.
                unsafe { (*parent).left = node };
            } else {
                // SAFETY: parent is valid.
                unsafe { (*parent).right = node };
            }
        }

        self.len += 1;
        self.rebalance_upwards(parent);
        None
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let target = self.find_node(key);
        if target.is_null() {
            return None;
        }

        // SAFETY: target is valid.
        let (left, right, parent) = unsafe { ((*target).left, (*target).right, (*target).parent) };

        let rebalance_start = if left.is_null() {
            self.transplant(target, right);
            parent
        } else if right.is_null() {
            self.transplant(target, left);
            parent
        } else {
            let succ = Self::leftmost(right);
            // SAFETY: succ is valid.
            let succ_parent = unsafe { (*succ).parent };

            if succ_parent != target {
                // SAFETY: succ is valid.
                let succ_right = unsafe { (*succ).right };
                self.transplant(succ, succ_right);

                // SAFETY: succ/right are valid.
                unsafe {
                    (*succ).right = right;
                    (*right).parent = succ;
                }
            }

            self.transplant(target, succ);
            // SAFETY: succ/left are valid.
            unsafe {
                (*succ).left = left;
                (*left).parent = succ;
            }

            Self::update_height(succ);
            if succ_parent == target { succ } else { succ_parent }
        };

        self.len = self.len.saturating_sub(1);
        // SAFETY: target has been detached and is uniquely owned here.
        let boxed = unsafe { Box::from_raw(target) };
        let AvlNode { value, .. } = *boxed;

        self.rebalance_upwards(rebalance_start);
        Some(value)
    }

    pub fn iter<'a>(&'a self) -> AvlIter<'a, K, V> {
        AvlIter {
            tree: self,
            next: Self::leftmost(self.root),
        }
    }
}

impl<K, V> Drop for AvlTree<K, V> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<K: Ord, V> Map<K, V> for AvlTree<K, V> {
    type Cursor<'a>
        = AvlCursor<K, V>
    where
        Self: 'a;

    type View<'a>
        = AvlNodeView<K, V>
    where
        Self: 'a;

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        Self::insert_entry(self, key, value)
    }

    fn cursor<'a>(&'a self, key: &K) -> Option<Self::Cursor<'a>> {
        Self::cursor(self, key)
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        cursor.node_view()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        Self::remove_key(self, key)
    }

    fn contains_key(&self, key: &K) -> bool {
        Self::contains_key(self, key)
    }

    fn clear(&mut self) {
        Self::clear(self)
    }

    fn len(&self) -> usize {
        Self::len(self)
    }
}

impl<K: Ord, V> OrderedMap<K, V> for AvlTree<K, V> {
    fn first_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.min_cursor()
    }

    fn last_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.max_cursor()
    }
}

impl<K: Ord, V> TreeDiagnostics for AvlTree<K, V> {
    type NodeCursor<'a>
        = AvlCursor<K, V>
    where
        Self: 'a;

    fn height(&self) -> usize {
        Self::height_from(self.root)
    }

    fn node_count(&self) -> usize {
        self.len
    }

    fn node_height<'a>(&'a self, cursor: &Self::NodeCursor<'a>) -> usize {
        Self::height_from(cursor.node)
    }
}

impl<K: Ord, V> Default for AvlTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for AvlTree<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut tree = Self::new();
        for (key, value) in iter {
            tree.insert_entry(key, value);
        }
        tree
    }
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for AvlIter<'a, K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        }

        // SAFETY: next is valid while tree is immutably borrowed.
        let item = unsafe { ((*self.next).key.clone(), (*self.next).value.clone()) };
        self.next = self.tree.successor_node(self.next);
        Some(item)
    }
}

impl<'a, K: Ord + Clone, V: Clone> IntoIterator for &'a AvlTree<K, V> {
    type Item = (K, V);
    type IntoIter = AvlIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
