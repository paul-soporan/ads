use std::ptr;

use crate::traits::{
    core::{Map, OrderedMap},
    diagnostics::TreeDiagnostics,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeColor {
    Red,
    Black,
}

#[derive(Debug)]
struct RbNode<K, V> {
    key: K,
    value: V,
    color: NodeColor,
    size: usize,
    left: *mut RbNode<K, V>,
    right: *mut RbNode<K, V>,
    parent: *mut RbNode<K, V>,
}

impl<K, V> RbNode<K, V> {
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            color: NodeColor::Red,
            size: 1,
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            parent: ptr::null_mut(),
        }
    }
}

#[derive(Debug)]
pub struct RbNodeView<K, V> {
    tree: *const RedBlackTree<K, V>,
    node: *mut RbNode<K, V>,
}

#[derive(Debug)]
pub struct RbCursor<K, V> {
    tree: *const RedBlackTree<K, V>,
    node: *mut RbNode<K, V>,
}

#[derive(Debug)]
pub struct RbIter<'a, K, V> {
    tree: &'a RedBlackTree<K, V>,
    next: *mut RbNode<K, V>,
}

impl<K, V> Clone for RbNodeView<K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node,
        }
    }
}

impl<K, V> Clone for RbCursor<K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node,
        }
    }
}

impl<K, V> RbNodeView<K, V> {
    fn node_ref(&self) -> &RbNode<K, V> {
        // SAFETY: created from a live tree node.
        unsafe { &*self.node }
    }

    pub fn key(&self) -> &K {
        &self.node_ref().key
    }

    pub fn value(&self) -> &V {
        &self.node_ref().value
    }

    pub fn color(&self) -> NodeColor {
        self.node_ref().color
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

impl<K: Ord, V> RbCursor<K, V> {
    fn tree_ref(&self) -> &RedBlackTree<K, V> {
        // SAFETY: created from a live tree reference.
        unsafe { &*self.tree }
    }

    fn node_ref(&self) -> &RbNode<K, V> {
        // SAFETY: created from a live tree node.
        unsafe { &*self.node }
    }

    pub fn key(&self) -> &K {
        &self.node_ref().key
    }

    pub fn value(&self) -> &V {
        &self.node_ref().value
    }

    pub fn node_view(&self) -> RbNodeView<K, V> {
        RbNodeView {
            tree: self.tree,
            node: self.node,
        }
    }

    pub fn predecessor(&self) -> Option<Self> {
        let pred = self.tree_ref().predecessor_node(self.node);
        (!pred.is_null()).then(|| Self {
            tree: self.tree,
            node: pred,
        })
    }

    pub fn successor(&self) -> Option<Self> {
        let succ = self.tree_ref().successor_node(self.node);
        (!succ.is_null()).then(|| Self {
            tree: self.tree,
            node: succ,
        })
    }
}

#[derive(Debug)]
pub struct RedBlackTree<K, V> {
    root: *mut RbNode<K, V>,
}

impl<K, V> RedBlackTree<K, V> {
    pub fn new() -> Self {
        Self {
            root: ptr::null_mut(),
        }
    }

    fn drop_subtree(node: *mut RbNode<K, V>) {
        if node.is_null() {
            return;
        }

        // SAFETY: node is valid and owned by this tree.
        let (left, right) = unsafe { ((*node).left, (*node).right) };
        Self::drop_subtree(left);
        Self::drop_subtree(right);

        // SAFETY: dropped exactly once by post-order traversal.
        unsafe {
            drop(Box::from_raw(node));
        }
    }

    fn node_size(node: *mut RbNode<K, V>) -> usize {
        if node.is_null() {
            0
        } else {
            // SAFETY: node is valid while reachable from root.
            unsafe { (*node).size }
        }
    }

    fn update_size(node: *mut RbNode<K, V>) {
        if node.is_null() {
            return;
        }

        // SAFETY: node and children are valid while reachable from root.
        unsafe {
            (*node).size = 1 + Self::node_size((*node).left) + Self::node_size((*node).right);
        }
    }

    fn recompute_sizes_up(mut node: *mut RbNode<K, V>) {
        while !node.is_null() {
            Self::update_size(node);
            // SAFETY: node is valid.
            node = unsafe { (*node).parent };
        }
    }

    fn color_of(node: *mut RbNode<K, V>) -> NodeColor {
        if node.is_null() {
            NodeColor::Black
        } else {
            // SAFETY: node is valid.
            unsafe { (*node).color }
        }
    }

    fn height_from(node: *mut RbNode<K, V>) -> usize {
        if node.is_null() {
            return 0;
        }

        // SAFETY: node is valid.
        unsafe {
            1 + usize::max(
                Self::height_from((*node).left),
                Self::height_from((*node).right),
            )
        }
    }

    fn leftmost(mut node: *mut RbNode<K, V>) -> *mut RbNode<K, V> {
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

    fn rightmost(mut node: *mut RbNode<K, V>) -> *mut RbNode<K, V> {
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

    fn find_node(&self, key: &K) -> *mut RbNode<K, V>
    where
        K: Ord,
    {
        let mut current = self.root;
        while !current.is_null() {
            // SAFETY: current is valid.
            let ord = unsafe { key.cmp(&(*current).key) };
            current = match ord {
                std::cmp::Ordering::Less => unsafe { (*current).left },
                std::cmp::Ordering::Greater => unsafe { (*current).right },
                std::cmp::Ordering::Equal => return current,
            };
        }
        ptr::null_mut()
    }

    fn predecessor_node(&self, node: *mut RbNode<K, V>) -> *mut RbNode<K, V>
    where
        K: Ord,
    {
        if node.is_null() {
            return ptr::null_mut();
        }

        // SAFETY: node is valid.
        let left = unsafe { (*node).left };
        if !left.is_null() {
            return Self::rightmost(left);
        }

        let mut current = node;
        // SAFETY: current is valid.
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

    fn successor_node(&self, node: *mut RbNode<K, V>) -> *mut RbNode<K, V>
    where
        K: Ord,
    {
        if node.is_null() {
            return ptr::null_mut();
        }

        // SAFETY: node is valid.
        let right = unsafe { (*node).right };
        if !right.is_null() {
            return Self::leftmost(right);
        }

        let mut current = node;
        // SAFETY: current is valid.
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
        parent: *mut RbNode<K, V>,
        old_child: *mut RbNode<K, V>,
        new_child: *mut RbNode<K, V>,
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
            // SAFETY: child is valid.
            unsafe { (*new_child).parent = parent };
        }
    }

    fn left_rotate(&mut self, x: *mut RbNode<K, V>) {
        // SAFETY: x valid and has right child.
        let y = unsafe { (*x).right };
        // SAFETY: y valid.
        let y_left = unsafe { (*y).left };

        // SAFETY: local subtree rewiring.
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

        Self::update_size(x);
        Self::update_size(y);
    }

    fn right_rotate(&mut self, x: *mut RbNode<K, V>) {
        // SAFETY: x valid and has left child.
        let y = unsafe { (*x).left };
        // SAFETY: y valid.
        let y_right = unsafe { (*y).right };

        // SAFETY: local subtree rewiring.
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

        Self::update_size(x);
        Self::update_size(y);
    }

    pub fn root_view(&self) -> Option<RbNodeView<K, V>> {
        (!self.root.is_null()).then(|| RbNodeView {
            tree: self as *const Self,
            node: self.root,
        })
    }

    pub fn len(&self) -> usize {
        Self::node_size(self.root)
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_null()
    }

    pub fn clear(&mut self) {
        Self::drop_subtree(self.root);
        self.root = ptr::null_mut();
    }
}

impl<K: Ord, V> RedBlackTree<K, V> {
    pub fn cursor<'a>(&'a self, key: &K) -> Option<RbCursor<K, V>> {
        let node = self.find_node(key);
        (!node.is_null()).then(|| RbCursor {
            tree: self as *const Self,
            node,
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        !self.find_node(key).is_null()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<RbCursor<K, V>> {
        let node = Self::leftmost(self.root);
        (!node.is_null()).then(|| RbCursor {
            tree: self as *const Self,
            node,
        })
    }

    pub fn max_cursor<'a>(&'a self) -> Option<RbCursor<K, V>> {
        let node = Self::rightmost(self.root);
        (!node.is_null()).then(|| RbCursor {
            tree: self as *const Self,
            node,
        })
    }

    pub fn insert_entry(&mut self, key: K, value: V) -> Option<V> {
        let mut parent = ptr::null_mut();
        let mut current = self.root;

        while !current.is_null() {
            parent = current;
            // SAFETY: current valid.
            match key.cmp(unsafe { &(*current).key }) {
                std::cmp::Ordering::Less => {
                    // SAFETY: current valid.
                    current = unsafe { (*current).left };
                }
                std::cmp::Ordering::Greater => {
                    // SAFETY: current valid.
                    current = unsafe { (*current).right };
                }
                std::cmp::Ordering::Equal => {
                    // SAFETY: current valid.
                    let old = unsafe { std::mem::replace(&mut (*current).value, value) };
                    return Some(old);
                }
            }
        }

        let z = Box::into_raw(Box::new(RbNode::new(key, value)));
        // SAFETY: z is newly allocated.
        unsafe { (*z).parent = parent };

        if parent.is_null() {
            self.root = z;
        } else {
            // SAFETY: parent/z valid.
            let go_left = unsafe { (*z).key < (*parent).key };
            if go_left {
                // SAFETY: parent valid.
                unsafe { (*parent).left = z };
            } else {
                // SAFETY: parent valid.
                unsafe { (*parent).right = z };
            }
        }

        // Update subtree sizes on insertion path.
        let mut anc = parent;
        while !anc.is_null() {
            // SAFETY: anc valid.
            unsafe { (*anc).size += 1 };
            // SAFETY: anc valid.
            anc = unsafe { (*anc).parent };
        }

        self.insert_fixup(z);
        None
    }

    fn insert_fixup(&mut self, mut z: *mut RbNode<K, V>) {
        while {
            // SAFETY: z valid.
            let p = unsafe { (*z).parent };
            !p.is_null() && Self::color_of(p) == NodeColor::Red
        } {
            // SAFETY: z and ancestors valid in tree.
            let p = unsafe { (*z).parent };
            // SAFETY: parent is red, so grandparent exists in valid RB tree.
            let g = unsafe { (*p).parent };

            // SAFETY: g valid.
            let parent_is_left = unsafe { (*g).left == p };

            if parent_is_left {
                // SAFETY: g valid.
                let u = unsafe { (*g).right };
                if Self::color_of(u) == NodeColor::Red {
                    // SAFETY: pointers valid.
                    unsafe {
                        (*p).color = NodeColor::Black;
                        (*u).color = NodeColor::Black;
                        (*g).color = NodeColor::Red;
                    }
                    z = g;
                } else {
                    // SAFETY: p valid.
                    if unsafe { (*p).right == z } {
                        z = p;
                        self.left_rotate(z);
                    }

                    // SAFETY: z valid.
                    let p2 = unsafe { (*z).parent };
                    // SAFETY: p2 valid.
                    let g2 = unsafe { (*p2).parent };
                    // SAFETY: pointers valid.
                    unsafe {
                        (*p2).color = NodeColor::Black;
                        (*g2).color = NodeColor::Red;
                    }
                    self.right_rotate(g2);
                }
            } else {
                // SAFETY: g valid.
                let u = unsafe { (*g).left };
                if Self::color_of(u) == NodeColor::Red {
                    // SAFETY: pointers valid.
                    unsafe {
                        (*p).color = NodeColor::Black;
                        (*u).color = NodeColor::Black;
                        (*g).color = NodeColor::Red;
                    }
                    z = g;
                } else {
                    // SAFETY: p valid.
                    if unsafe { (*p).left == z } {
                        z = p;
                        self.right_rotate(z);
                    }

                    // SAFETY: z valid.
                    let p2 = unsafe { (*z).parent };
                    // SAFETY: p2 valid.
                    let g2 = unsafe { (*p2).parent };
                    // SAFETY: pointers valid.
                    unsafe {
                        (*p2).color = NodeColor::Black;
                        (*g2).color = NodeColor::Red;
                    }
                    self.left_rotate(g2);
                }
            }
        }

        if !self.root.is_null() {
            // SAFETY: root valid.
            unsafe { (*self.root).color = NodeColor::Black };
        }
    }

    fn transplant(&mut self, u: *mut RbNode<K, V>, v: *mut RbNode<K, V>) {
        // SAFETY: u valid.
        let parent = unsafe { (*u).parent };
        self.replace_parent_child(parent, u, v);
    }

    fn delete_fixup(
        &mut self,
        mut x: *mut RbNode<K, V>,
        mut x_parent: *mut RbNode<K, V>,
    ) {
        while x != self.root && Self::color_of(x) == NodeColor::Black {
            if x_parent.is_null() {
                break;
            }

            // SAFETY: x_parent valid.
            let x_is_left = unsafe { (*x_parent).left == x };

            if x_is_left {
                // SAFETY: x_parent valid.
                let mut w = unsafe { (*x_parent).right };

                if Self::color_of(w) == NodeColor::Red {
                    // SAFETY: pointers valid.
                    unsafe {
                        (*w).color = NodeColor::Black;
                        (*x_parent).color = NodeColor::Red;
                    }
                    self.left_rotate(x_parent);
                    // SAFETY: x_parent valid.
                    w = unsafe { (*x_parent).right };
                }

                // SAFETY: w may be null.
                let w_left = if w.is_null() { ptr::null_mut() } else { unsafe { (*w).left } };
                let w_right = if w.is_null() { ptr::null_mut() } else { unsafe { (*w).right } };

                if Self::color_of(w_left) == NodeColor::Black
                    && Self::color_of(w_right) == NodeColor::Black
                {
                    if !w.is_null() {
                        // SAFETY: w valid.
                        unsafe { (*w).color = NodeColor::Red };
                    }
                    x = x_parent;
                    // SAFETY: x valid unless null.
                    x_parent = if x.is_null() {
                        ptr::null_mut()
                    } else {
                        unsafe { (*x).parent }
                    };
                } else {
                    if Self::color_of(w_right) == NodeColor::Black {
                        if !w_left.is_null() {
                            // SAFETY: w_left valid.
                            unsafe { (*w_left).color = NodeColor::Black };
                        }
                        if !w.is_null() {
                            // SAFETY: w valid.
                            unsafe { (*w).color = NodeColor::Red };
                            self.right_rotate(w);
                        }
                        // SAFETY: x_parent valid.
                        w = unsafe { (*x_parent).right };
                    }

                    if !w.is_null() {
                        // SAFETY: w and parent valid.
                        unsafe {
                            (*w).color = (*x_parent).color;
                        }
                    }
                    // SAFETY: parent valid.
                    unsafe { (*x_parent).color = NodeColor::Black };
                    // SAFETY: w may be null.
                    let w_right2 = if w.is_null() {
                        ptr::null_mut()
                    } else {
                        unsafe { (*w).right }
                    };
                    if !w_right2.is_null() {
                        // SAFETY: child valid.
                        unsafe { (*w_right2).color = NodeColor::Black };
                    }
                    self.left_rotate(x_parent);
                    x = self.root;
                    x_parent = ptr::null_mut();
                }
            } else {
                // SAFETY: x_parent valid.
                let mut w = unsafe { (*x_parent).left };

                if Self::color_of(w) == NodeColor::Red {
                    // SAFETY: pointers valid.
                    unsafe {
                        (*w).color = NodeColor::Black;
                        (*x_parent).color = NodeColor::Red;
                    }
                    self.right_rotate(x_parent);
                    // SAFETY: x_parent valid.
                    w = unsafe { (*x_parent).left };
                }

                // SAFETY: w may be null.
                let w_right = if w.is_null() { ptr::null_mut() } else { unsafe { (*w).right } };
                let w_left = if w.is_null() { ptr::null_mut() } else { unsafe { (*w).left } };

                if Self::color_of(w_right) == NodeColor::Black
                    && Self::color_of(w_left) == NodeColor::Black
                {
                    if !w.is_null() {
                        // SAFETY: w valid.
                        unsafe { (*w).color = NodeColor::Red };
                    }
                    x = x_parent;
                    // SAFETY: x valid unless null.
                    x_parent = if x.is_null() {
                        ptr::null_mut()
                    } else {
                        unsafe { (*x).parent }
                    };
                } else {
                    if Self::color_of(w_left) == NodeColor::Black {
                        if !w_right.is_null() {
                            // SAFETY: child valid.
                            unsafe { (*w_right).color = NodeColor::Black };
                        }
                        if !w.is_null() {
                            // SAFETY: w valid.
                            unsafe { (*w).color = NodeColor::Red };
                            self.left_rotate(w);
                        }
                        // SAFETY: x_parent valid.
                        w = unsafe { (*x_parent).left };
                    }

                    if !w.is_null() {
                        // SAFETY: w and parent valid.
                        unsafe {
                            (*w).color = (*x_parent).color;
                        }
                    }
                    // SAFETY: x_parent valid.
                    unsafe { (*x_parent).color = NodeColor::Black };
                    // SAFETY: w may be null.
                    let w_left2 = if w.is_null() {
                        ptr::null_mut()
                    } else {
                        unsafe { (*w).left }
                    };
                    if !w_left2.is_null() {
                        // SAFETY: child valid.
                        unsafe { (*w_left2).color = NodeColor::Black };
                    }
                    self.right_rotate(x_parent);
                    x = self.root;
                    x_parent = ptr::null_mut();
                }
            }
        }

        if !x.is_null() {
            // SAFETY: x valid.
            unsafe { (*x).color = NodeColor::Black };
        }
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let z = self.find_node(key);
        if z.is_null() {
            return None;
        }

        let mut y = z;
        let mut y_original_color = Self::color_of(y);
        let x: *mut RbNode<K, V>;
        let x_parent: *mut RbNode<K, V>;

        // SAFETY: z valid.
        let z_left = unsafe { (*z).left };
        let z_right = unsafe { (*z).right };

        if z_left.is_null() {
            x = z_right;
            // SAFETY: z valid.
            x_parent = unsafe { (*z).parent };
            self.transplant(z, z_right);
        } else if z_right.is_null() {
            x = z_left;
            // SAFETY: z valid.
            x_parent = unsafe { (*z).parent };
            self.transplant(z, z_left);
        } else {
            y = Self::leftmost(z_right);
            y_original_color = Self::color_of(y);
            // SAFETY: y valid.
            x = unsafe { (*y).right };

            // SAFETY: y valid.
            if unsafe { (*y).parent } == z {
                x_parent = y;
                if !x.is_null() {
                    // SAFETY: x valid.
                    unsafe { (*x).parent = y };
                }
            } else {
                // SAFETY: y valid.
                x_parent = unsafe { (*y).parent };
                // SAFETY: y valid.
                let y_right = unsafe { (*y).right };
                self.transplant(y, y_right);
                // SAFETY: y/z_right valid.
                unsafe {
                    (*y).right = z_right;
                    (*z_right).parent = y;
                }
            }

            self.transplant(z, y);
            // SAFETY: y/z_left valid.
            unsafe {
                (*y).left = z_left;
                (*z_left).parent = y;
                (*y).color = (*z).color;
            }
            Self::update_size(y);
        }

        Self::recompute_sizes_up(x_parent);

        if y_original_color == NodeColor::Black {
            self.delete_fixup(x, x_parent);
        }

        // SAFETY: z detached and uniquely owned here.
        let boxed = unsafe { Box::from_raw(z) };
        let RbNode { value, .. } = *boxed;
        Some(value)
    }

    pub fn size(&self) -> usize {
        Self::node_size(self.root)
    }

    pub fn select<'a>(&'a self, rank: usize) -> Option<RbCursor<K, V>> {
        let mut current = self.root;
        let mut k = rank;

        while !current.is_null() {
            // SAFETY: current valid.
            let left = unsafe { (*current).left };
            let left_size = Self::node_size(left);
            if k < left_size {
                current = left;
            } else if k == left_size {
                return Some(RbCursor {
                    tree: self as *const Self,
                    node: current,
                });
            } else {
                k -= left_size + 1;
                // SAFETY: current valid.
                current = unsafe { (*current).right };
            }
        }

        None
    }

    pub fn iter<'a>(&'a self) -> RbIter<'a, K, V> {
        RbIter {
            tree: self,
            next: Self::leftmost(self.root),
        }
    }
}

impl<K, V> Drop for RedBlackTree<K, V> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<K: Ord, V> Map<K, V> for RedBlackTree<K, V> {
    type Cursor<'a>
        = RbCursor<K, V>
    where
        Self: 'a;

    type View<'a>
        = RbNodeView<K, V>
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
        Self::size(self)
    }
}

impl<K: Ord, V> OrderedMap<K, V> for RedBlackTree<K, V> {
    fn first_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.min_cursor()
    }

    fn last_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.max_cursor()
    }
}

impl<K: Ord, V> TreeDiagnostics for RedBlackTree<K, V> {
    type NodeCursor<'a>
        = RbCursor<K, V>
    where
        Self: 'a;

    fn height(&self) -> usize {
        Self::height_from(self.root)
    }

    fn node_count(&self) -> usize {
        Self::node_size(self.root)
    }

    fn node_height<'a>(&'a self, cursor: &Self::NodeCursor<'a>) -> usize {
        Self::height_from(cursor.node)
    }
}

impl<K: Ord, V> Default for RedBlackTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for RedBlackTree<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut tree = Self::new();
        for (key, value) in iter {
            tree.insert_entry(key, value);
        }
        tree
    }
}

impl<'a, K: Ord + Clone, V: Clone> Iterator for RbIter<'a, K, V> {
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

impl<'a, K: Ord + Clone, V: Clone> IntoIterator for &'a RedBlackTree<K, V> {
    type Item = (K, V);
    type IntoIter = RbIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
