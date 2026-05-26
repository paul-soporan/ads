use std::ptr;

use crate::traits::{
    core::{Map, OrderedMap},
    diagnostics::TreeDiagnostics,
};

#[derive(Debug)]
struct BstNode<K, V> {
    key: K,
    value: V,
    left: *mut BstNode<K, V>,
    right: *mut BstNode<K, V>,
    parent: *mut BstNode<K, V>,
}

impl<K, V> BstNode<K, V> {
    fn new(key: K, value: V, parent: *mut BstNode<K, V>) -> Self {
        Self {
            key,
            value,
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            parent,
        }
    }
}

#[derive(Debug)]
pub struct BstNodeView<K, V> {
    tree: *const BinarySearchTree<K, V>,
    node: *mut BstNode<K, V>,
}

#[derive(Debug)]
pub struct BstCursor<K, V> {
    tree: *const BinarySearchTree<K, V>,
    node: *mut BstNode<K, V>,
}

#[derive(Debug)]
pub struct BstIter<'a, K, V> {
    tree: &'a BinarySearchTree<K, V>,
    next: *mut BstNode<K, V>,
}

impl<K, V> Clone for BstNodeView<K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node,
        }
    }
}

impl<K, V> Clone for BstCursor<K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node,
        }
    }
}

impl<K, V> BstNodeView<K, V> {
    fn node_ref(&self) -> &BstNode<K, V> {
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

impl<K: Ord, V> BstCursor<K, V> {
    fn tree_ref(&self) -> &BinarySearchTree<K, V> {
        // SAFETY: created from a live tree reference.
        unsafe { &*self.tree }
    }

    fn node_ref(&self) -> &BstNode<K, V> {
        // SAFETY: created from a live node owned by the tree.
        unsafe { &*self.node }
    }

    pub fn key(&self) -> &K {
        &self.node_ref().key
    }

    pub fn value(&self) -> &V {
        &self.node_ref().value
    }

    pub fn node_view(&self) -> BstNodeView<K, V> {
        BstNodeView {
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
pub struct BinarySearchTree<K, V> {
    root: *mut BstNode<K, V>,
    len: usize,
}

impl<K, V> BinarySearchTree<K, V> {
    pub fn new() -> Self {
        Self {
            root: ptr::null_mut(),
            len: 0,
        }
    }

    fn drop_subtree(node: *mut BstNode<K, V>) {
        if node.is_null() {
            return;
        }

        // SAFETY: node is valid and owned by the tree.
        let (left, right) = unsafe { ((*node).left, (*node).right) };
        Self::drop_subtree(left);
        Self::drop_subtree(right);

        // SAFETY: each node is dropped exactly once by post-order traversal.
        unsafe {
            drop(Box::from_raw(node));
        }
    }

    fn leftmost(mut node: *mut BstNode<K, V>) -> *mut BstNode<K, V> {
        while !node.is_null() {
            // SAFETY: node is a valid pointer while traversing tree links.
            let left = unsafe { (*node).left };
            if left.is_null() {
                return node;
            }
            node = left;
        }
        ptr::null_mut()
    }

    fn rightmost(mut node: *mut BstNode<K, V>) -> *mut BstNode<K, V> {
        while !node.is_null() {
            // SAFETY: node is a valid pointer while traversing tree links.
            let right = unsafe { (*node).right };
            if right.is_null() {
                return node;
            }
            node = right;
        }
        ptr::null_mut()
    }

    fn height_from(node: *mut BstNode<K, V>) -> usize {
        if node.is_null() {
            return 0;
        }

        // SAFETY: node is valid while reachable from tree root.
        unsafe {
            1 + usize::max(
                Self::height_from((*node).left),
                Self::height_from((*node).right),
            )
        }
    }

    pub fn root_view(&self) -> Option<BstNodeView<K, V>> {
        (!self.root.is_null()).then(|| BstNodeView {
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

impl<K: Ord, V> BinarySearchTree<K, V> {
    fn find_node(&self, key: &K) -> *mut BstNode<K, V> {
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

    fn predecessor_node(&self, node: *mut BstNode<K, V>) -> *mut BstNode<K, V> {
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

    fn successor_node(&self, node: *mut BstNode<K, V>) -> *mut BstNode<K, V> {
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

    fn transplant(&mut self, u: *mut BstNode<K, V>, v: *mut BstNode<K, V>) {
        // SAFETY: u is a valid node in this tree.
        let parent = unsafe { (*u).parent };

        if parent.is_null() {
            self.root = v;
        } else {
            // SAFETY: parent is valid.
            let is_left = unsafe { (*parent).left == u };
            if is_left {
                // SAFETY: parent is valid.
                unsafe { (*parent).left = v };
            } else {
                // SAFETY: parent is valid.
                unsafe { (*parent).right = v };
            }
        }

        if !v.is_null() {
            // SAFETY: v is valid and re-parented into this tree.
            unsafe { (*v).parent = parent };
        }
    }

    pub fn cursor<'a>(&'a self, key: &K) -> Option<BstCursor<K, V>> {
        let node = self.find_node(key);
        (!node.is_null()).then(|| BstCursor {
            tree: self as *const Self,
            node,
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        !self.find_node(key).is_null()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<BstCursor<K, V>> {
        let node = Self::leftmost(self.root);
        (!node.is_null()).then(|| BstCursor {
            tree: self as *const Self,
            node,
        })
    }

    pub fn max_cursor<'a>(&'a self) -> Option<BstCursor<K, V>> {
        let node = Self::rightmost(self.root);
        (!node.is_null()).then(|| BstCursor {
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

        let new_node = Box::into_raw(Box::new(BstNode::new(key, value, parent)));

        if parent.is_null() {
            self.root = new_node;
        } else {
            // SAFETY: parent is valid.
            let go_left = unsafe { (*new_node).key < (*parent).key };
            if go_left {
                // SAFETY: parent is valid.
                unsafe { (*parent).left = new_node };
            } else {
                // SAFETY: parent is valid.
                unsafe { (*parent).right = new_node };
            }
        }

        self.len += 1;
        None
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let target = self.find_node(key);
        if target.is_null() {
            return None;
        }

        // SAFETY: target is valid in this tree.
        let (left, right) = unsafe { ((*target).left, (*target).right) };

        if left.is_null() {
            self.transplant(target, right);
        } else if right.is_null() {
            self.transplant(target, left);
        } else {
            let succ = Self::leftmost(right);
            // SAFETY: succ and target are valid.
            let succ_parent = unsafe { (*succ).parent };

            if succ_parent != target {
                // SAFETY: succ is valid.
                let succ_right = unsafe { (*succ).right };
                self.transplant(succ, succ_right);

                // SAFETY: succ/target/right are valid.
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
        }

        self.len = self.len.saturating_sub(1);

        // SAFETY: target was detached from the tree; read out value and free once.
        let boxed = unsafe { Box::from_raw(target) };
        let BstNode { value, .. } = *boxed;
        Some(value)
    }

    pub fn iter<'a>(&'a self) -> BstIter<'a, K, V> {
        BstIter {
            tree: self,
            next: Self::leftmost(self.root),
        }
    }
}

impl<K, V> Drop for BinarySearchTree<K, V> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<K: Ord, V> Map<K, V> for BinarySearchTree<K, V> {
    type Cursor<'a>
        = BstCursor<K, V>
    where
        Self: 'a;

    type View<'a>
        = BstNodeView<K, V>
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

impl<K: Ord, V> OrderedMap<K, V> for BinarySearchTree<K, V> {
    fn first_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.min_cursor()
    }

    fn last_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.max_cursor()
    }
}

impl<K: Ord, V> TreeDiagnostics for BinarySearchTree<K, V> {
    type NodeCursor<'a>
        = BstCursor<K, V>
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

impl<K: Ord, V> Default for BinarySearchTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for BinarySearchTree<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut tree = Self::new();
        for (key, value) in iter {
            tree.insert_entry(key, value);
        }
        tree
    }
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for BstIter<'a, K, V> {
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

impl<'a, K: Ord + Clone, V: Clone> IntoIterator for &'a BinarySearchTree<K, V> {
    type Item = (K, V);
    type IntoIter = BstIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
