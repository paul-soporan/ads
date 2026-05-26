use std::ptr;

use crate::traits::{
    core::{Map, OrderedMap},
    diagnostics::TreeDiagnostics,
};

#[derive(Debug)]
struct SplayNode<K, V> {
    key: K,
    value: V,
    left: *mut SplayNode<K, V>,
    right: *mut SplayNode<K, V>,
    parent: *mut SplayNode<K, V>,
}

impl<K, V> SplayNode<K, V> {
    fn new(key: K, value: V, parent: *mut SplayNode<K, V>) -> Self {
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
pub struct SplayNodeView<K, V> {
    tree: *const SplayTree<K, V>,
    node: *mut SplayNode<K, V>,
}

#[derive(Debug)]
pub struct SplayCursor<K, V> {
    tree: *const SplayTree<K, V>,
    node: *mut SplayNode<K, V>,
}

#[derive(Debug)]
pub struct SplayIter<'a, K, V> {
    tree: &'a SplayTree<K, V>,
    next: *mut SplayNode<K, V>,
}

impl<K, V> Clone for SplayNodeView<K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node,
        }
    }
}

impl<K, V> Clone for SplayCursor<K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node,
        }
    }
}

impl<K, V> SplayNodeView<K, V> {
    fn node_ref(&self) -> &SplayNode<K, V> {
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

impl<K: Ord, V> SplayCursor<K, V> {
    fn tree_ref(&self) -> &SplayTree<K, V> {
        // SAFETY: created from a live tree reference.
        unsafe { &*self.tree }
    }

    fn node_ref(&self) -> &SplayNode<K, V> {
        // SAFETY: created from a live node owned by the tree.
        unsafe { &*self.node }
    }

    pub fn key(&self) -> &K {
        &self.node_ref().key
    }

    pub fn value(&self) -> &V {
        &self.node_ref().value
    }

    pub fn node_view(&self) -> SplayNodeView<K, V> {
        SplayNodeView {
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
pub struct SplayTree<K, V> {
    root: *mut SplayNode<K, V>,
    len: usize,
}

impl<K, V> SplayTree<K, V> {
    pub fn new() -> Self {
        Self {
            root: ptr::null_mut(),
            len: 0,
        }
    }

    fn drop_subtree(node: *mut SplayNode<K, V>) {
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

    fn leftmost(mut node: *mut SplayNode<K, V>) -> *mut SplayNode<K, V> {
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

    fn rightmost(mut node: *mut SplayNode<K, V>) -> *mut SplayNode<K, V> {
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

    fn height_from(node: *mut SplayNode<K, V>) -> usize {
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

    pub fn root_view(&self) -> Option<SplayNodeView<K, V>> {
        (!self.root.is_null()).then(|| SplayNodeView {
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

    fn rotate_left(&mut self, x: *mut SplayNode<K, V>) {
        if x.is_null() {
            return;
        }

        // SAFETY: x is valid and y must be its right child.
        let y = unsafe { (*x).right };
        if y.is_null() {
            return;
        }

        // SAFETY: pointers are valid and part of this tree.
        unsafe {
            (*x).right = (*y).left;
            if !(*y).left.is_null() {
                (*(*y).left).parent = x;
            }

            (*y).parent = (*x).parent;
            if (*x).parent.is_null() {
                self.root = y;
            } else if (*(*x).parent).left == x {
                (*(*x).parent).left = y;
            } else {
                (*(*x).parent).right = y;
            }

            (*y).left = x;
            (*x).parent = y;
        }
    }

    fn rotate_right(&mut self, x: *mut SplayNode<K, V>) {
        if x.is_null() {
            return;
        }

        // SAFETY: x is valid and y must be its left child.
        let y = unsafe { (*x).left };
        if y.is_null() {
            return;
        }

        // SAFETY: pointers are valid and part of this tree.
        unsafe {
            (*x).left = (*y).right;
            if !(*y).right.is_null() {
                (*(*y).right).parent = x;
            }

            (*y).parent = (*x).parent;
            if (*x).parent.is_null() {
                self.root = y;
            } else if (*(*x).parent).left == x {
                (*(*x).parent).left = y;
            } else {
                (*(*x).parent).right = y;
            }

            (*y).right = x;
            (*x).parent = y;
        }
    }
}

impl<K: Ord, V> SplayTree<K, V> {
    fn splay(&mut self, x: *mut SplayNode<K, V>) {
        if x.is_null() {
            return;
        }

        // SAFETY: x is valid while in this tree.
        unsafe {
            while !(*x).parent.is_null() {
                let parent = (*x).parent;
                let grand = (*parent).parent;

                if grand.is_null() {
                    if (*parent).left == x {
                        self.rotate_right(parent);
                    } else {
                        self.rotate_left(parent);
                    }
                } else if (*grand).left == parent && (*parent).left == x {
                    self.rotate_right(grand);
                    self.rotate_right(parent);
                } else if (*grand).right == parent && (*parent).right == x {
                    self.rotate_left(grand);
                    self.rotate_left(parent);
                } else if (*grand).left == parent && (*parent).right == x {
                    self.rotate_left(parent);
                    self.rotate_right(grand);
                } else {
                    self.rotate_right(parent);
                    self.rotate_left(grand);
                }
            }
        }

        self.root = x;
    }

    fn find_node(&self, key: &K) -> *mut SplayNode<K, V> {
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

    fn find_node_with_last(&self, key: &K) -> (*mut SplayNode<K, V>, *mut SplayNode<K, V>) {
        let mut current = self.root;
        let mut last = ptr::null_mut();

        while !current.is_null() {
            last = current;
            // SAFETY: current is valid while reachable from root.
            let ord = unsafe { key.cmp(&(*current).key) };
            current = match ord {
                std::cmp::Ordering::Less => unsafe { (*current).left },
                std::cmp::Ordering::Greater => unsafe { (*current).right },
                std::cmp::Ordering::Equal => return (current, last),
            };
        }

        (ptr::null_mut(), last)
    }

    fn predecessor_node(&self, node: *mut SplayNode<K, V>) -> *mut SplayNode<K, V> {
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

    fn successor_node(&self, node: *mut SplayNode<K, V>) -> *mut SplayNode<K, V> {
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

    pub fn cursor<'a>(&'a self, key: &K) -> Option<SplayCursor<K, V>> {
        let node = self.find_node(key);
        (!node.is_null()).then(|| SplayCursor {
            tree: self as *const Self,
            node,
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        !self.find_node(key).is_null()
    }

    pub fn get_adaptive<'a>(&'a mut self, key: &K) -> Option<SplayNodeView<K, V>> {
        let (found, last) = self.find_node_with_last(key);
        if !found.is_null() {
            self.splay(found);
            return Some(SplayNodeView {
                tree: self as *const Self,
                node: found,
            });
        }

        if !last.is_null() {
            self.splay(last);
        }
        None
    }

    pub fn contains_adaptive(&mut self, key: &K) -> bool {
        self.get_adaptive(key).is_some()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<SplayCursor<K, V>> {
        let node = Self::leftmost(self.root);
        (!node.is_null()).then(|| SplayCursor {
            tree: self as *const Self,
            node,
        })
    }

    pub fn max_cursor<'a>(&'a self) -> Option<SplayCursor<K, V>> {
        let node = Self::rightmost(self.root);
        (!node.is_null()).then(|| SplayCursor {
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
                    self.splay(current);
                    return Some(old);
                }
            }
        }

        let new_node = Box::into_raw(Box::new(SplayNode::new(key, value, parent)));

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

        self.splay(new_node);
        self.len += 1;
        None
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let target = self.find_node(key);
        if target.is_null() {
            return None;
        }

        self.splay(target);

        // SAFETY: target is now root.
        let (left, right) = unsafe { ((*target).left, (*target).right) };

        if !left.is_null() {
            // SAFETY: left is detached root of left subtree.
            unsafe { (*left).parent = ptr::null_mut() };
        }
        if !right.is_null() {
            // SAFETY: right is detached root of right subtree.
            unsafe { (*right).parent = ptr::null_mut() };
        }

        if left.is_null() {
            self.root = right;
        } else {
            self.root = left;
            let max_left = Self::rightmost(self.root);
            self.splay(max_left);

            // SAFETY: max_left is root and has empty right child by construction.
            unsafe {
                (*max_left).right = right;
                if !right.is_null() {
                    (*right).parent = max_left;
                }
            }
        }

        self.len = self.len.saturating_sub(1);

        // SAFETY: target is detached from the tree and can be freed once.
        let boxed = unsafe { Box::from_raw(target) };
        let SplayNode { value, .. } = *boxed;
        Some(value)
    }

    pub fn iter<'a>(&'a self) -> SplayIter<'a, K, V> {
        SplayIter {
            tree: self,
            next: Self::leftmost(self.root),
        }
    }
}

impl<K, V> Drop for SplayTree<K, V> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<K: Ord, V> Map<K, V> for SplayTree<K, V> {
    type Cursor<'a>
        = SplayCursor<K, V>
    where
        Self: 'a;

    type View<'a>
        = SplayNodeView<K, V>
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

impl<K: Ord, V> OrderedMap<K, V> for SplayTree<K, V> {
    fn first_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.min_cursor()
    }

    fn last_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.max_cursor()
    }
}

impl<K: Ord, V> TreeDiagnostics for SplayTree<K, V> {
    type NodeCursor<'a>
        = SplayCursor<K, V>
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

impl<K: Ord, V> Default for SplayTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for SplayTree<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut tree = Self::new();
        for (key, value) in iter {
            tree.insert_entry(key, value);
        }
        tree
    }
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for SplayIter<'a, K, V> {
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

impl<'a, K: Ord + Clone, V: Clone> IntoIterator for &'a SplayTree<K, V> {
    type Item = (K, V);
    type IntoIter = SplayIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
