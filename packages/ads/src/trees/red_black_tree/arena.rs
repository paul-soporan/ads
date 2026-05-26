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
    left: Option<usize>,
    right: Option<usize>,
    parent: Option<usize>,
}

impl<K, V> RbNode<K, V> {
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            color: NodeColor::Red,
            size: 1,
            left: None,
            right: None,
            parent: None,
        }
    }
}

#[derive(Debug)]
pub struct RbNodeView<'a, K, V> {
    tree: &'a RedBlackTree<K, V>,
    node_idx: usize,
}

#[derive(Debug)]
pub struct RbCursor<'a, K, V> {
    tree: &'a RedBlackTree<K, V>,
    node_idx: usize,
}

#[derive(Debug)]
pub struct RbIter<'a, K, V> {
    tree: &'a RedBlackTree<K, V>,
    next: Option<usize>,
}

impl<'a, K, V> Clone for RbNodeView<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node_idx: self.node_idx,
        }
    }
}

impl<'a, K, V> Clone for RbCursor<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node_idx: self.node_idx,
        }
    }
}

impl<'a, K, V> RbNodeView<'a, K, V> {
    fn node_ref(&self) -> &RbNode<K, V> {
        self.tree.node_ref(self.node_idx)
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
        self.node_ref().left.map(|node_idx| Self {
            tree: self.tree,
            node_idx,
        })
    }

    pub fn right(&self) -> Option<Self> {
        self.node_ref().right.map(|node_idx| Self {
            tree: self.tree,
            node_idx,
        })
    }

    pub fn parent(&self) -> Option<Self> {
        self.node_ref().parent.map(|node_idx| Self {
            tree: self.tree,
            node_idx,
        })
    }
}

impl<'a, K: Ord, V> RbCursor<'a, K, V> {
    fn node_ref(&self) -> &RbNode<K, V> {
        self.tree.node_ref(self.node_idx)
    }

    pub fn key(&self) -> &K {
        &self.node_ref().key
    }

    pub fn value(&self) -> &V {
        &self.node_ref().value
    }

    pub fn node_view(&self) -> RbNodeView<'a, K, V> {
        RbNodeView {
            tree: self.tree,
            node_idx: self.node_idx,
        }
    }

    pub fn predecessor(&self) -> Option<Self> {
        self.tree.predecessor_node(self.node_idx).map(|node_idx| Self {
            tree: self.tree,
            node_idx,
        })
    }

    pub fn successor(&self) -> Option<Self> {
        self.tree.successor_node(self.node_idx).map(|node_idx| Self {
            tree: self.tree,
            node_idx,
        })
    }
}

#[derive(Debug)]
pub struct RedBlackTree<K, V> {
    root: Option<usize>,
    nodes: Vec<Option<RbNode<K, V>>>,
    free: Vec<usize>,
}

impl<K, V> RedBlackTree<K, V> {
    pub fn new() -> Self {
        Self {
            root: None,
            nodes: Vec::new(),
            free: Vec::new(),
        }
    }

    fn node_ref(&self, idx: usize) -> &RbNode<K, V> {
        self.nodes[idx].as_ref().expect("live arena node")
    }

    fn node_mut(&mut self, idx: usize) -> &mut RbNode<K, V> {
        self.nodes[idx].as_mut().expect("live arena node")
    }

    fn alloc_node(&mut self, node: RbNode<K, V>) -> usize {
        if let Some(idx) = self.free.pop() {
            self.nodes[idx] = Some(node);
            idx
        } else {
            self.nodes.push(Some(node));
            self.nodes.len() - 1
        }
    }

    fn take_node(&mut self, idx: usize) -> RbNode<K, V> {
        let node = self.nodes[idx].take().expect("live arena node");
        self.free.push(idx);
        node
    }

    fn node_size(&self, node: Option<usize>) -> usize {
        node.map_or(0, |idx| self.node_ref(idx).size)
    }

    fn update_size(&mut self, node: Option<usize>) {
        let Some(idx) = node else {
            return;
        };

        let (left, right) = {
            let n = self.node_ref(idx);
            (n.left, n.right)
        };
        self.node_mut(idx).size = 1 + self.node_size(left) + self.node_size(right);
    }

    fn recompute_sizes_up(&mut self, mut node: Option<usize>) {
        while let Some(idx) = node {
            self.update_size(Some(idx));
            node = self.node_ref(idx).parent;
        }
    }

    fn color_of(&self, node: Option<usize>) -> NodeColor {
        node.map_or(NodeColor::Black, |idx| self.node_ref(idx).color)
    }

    fn height_from(&self, node: Option<usize>) -> usize {
        let Some(idx) = node else {
            return 0;
        };
        let n = self.node_ref(idx);
        1 + usize::max(self.height_from(n.left), self.height_from(n.right))
    }

    fn leftmost(&self, mut node: Option<usize>) -> Option<usize> {
        while let Some(idx) = node {
            let left = self.node_ref(idx).left;
            if left.is_none() {
                return Some(idx);
            }
            node = left;
        }
        None
    }

    fn rightmost(&self, mut node: Option<usize>) -> Option<usize> {
        while let Some(idx) = node {
            let right = self.node_ref(idx).right;
            if right.is_none() {
                return Some(idx);
            }
            node = right;
        }
        None
    }

    pub fn root_view(&self) -> Option<RbNodeView<'_, K, V>> {
        self.root.map(|node_idx| RbNodeView {
            tree: self,
            node_idx,
        })
    }

    pub fn len(&self) -> usize {
        self.node_size(self.root)
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.nodes.clear();
        self.free.clear();
    }
}

impl<K: Ord, V> RedBlackTree<K, V> {
    fn find_node(&self, key: &K) -> Option<usize> {
        let mut current = self.root;
        while let Some(idx) = current {
            let n = self.node_ref(idx);
            current = match key.cmp(&n.key) {
                std::cmp::Ordering::Less => n.left,
                std::cmp::Ordering::Greater => n.right,
                std::cmp::Ordering::Equal => return Some(idx),
            };
        }
        None
    }

    fn predecessor_node(&self, node_idx: usize) -> Option<usize> {
        let n = self.node_ref(node_idx);
        if let Some(left) = n.left {
            return self.rightmost(Some(left));
        }

        let mut current = node_idx;
        let mut parent = self.node_ref(current).parent;
        while let Some(pidx) = parent {
            if self.node_ref(pidx).right == Some(current) {
                return Some(pidx);
            }
            current = pidx;
            parent = self.node_ref(current).parent;
        }
        None
    }

    fn successor_node(&self, node_idx: usize) -> Option<usize> {
        let n = self.node_ref(node_idx);
        if let Some(right) = n.right {
            return self.leftmost(Some(right));
        }

        let mut current = node_idx;
        let mut parent = self.node_ref(current).parent;
        while let Some(pidx) = parent {
            if self.node_ref(pidx).left == Some(current) {
                return Some(pidx);
            }
            current = pidx;
            parent = self.node_ref(current).parent;
        }
        None
    }

    fn replace_parent_child(
        &mut self,
        parent: Option<usize>,
        old_child: usize,
        new_child: Option<usize>,
    ) {
        if let Some(pidx) = parent {
            if self.node_ref(pidx).left == Some(old_child) {
                self.node_mut(pidx).left = new_child;
            } else {
                self.node_mut(pidx).right = new_child;
            }
        } else {
            self.root = new_child;
        }

        if let Some(cidx) = new_child {
            self.node_mut(cidx).parent = parent;
        }
    }

    fn left_rotate(&mut self, x: usize) {
        let y = self.node_ref(x).right.expect("left_rotate needs right child");
        let y_left = self.node_ref(y).left;
        let x_parent = self.node_ref(x).parent;

        self.node_mut(x).right = y_left;
        if let Some(yl) = y_left {
            self.node_mut(yl).parent = Some(x);
        }

        self.node_mut(y).left = Some(x);
        self.node_mut(y).parent = x_parent;
        self.node_mut(x).parent = Some(y);

        self.replace_parent_child(x_parent, x, Some(y));
        self.update_size(Some(x));
        self.update_size(Some(y));
    }

    fn right_rotate(&mut self, x: usize) {
        let y = self.node_ref(x).left.expect("right_rotate needs left child");
        let y_right = self.node_ref(y).right;
        let x_parent = self.node_ref(x).parent;

        self.node_mut(x).left = y_right;
        if let Some(yr) = y_right {
            self.node_mut(yr).parent = Some(x);
        }

        self.node_mut(y).right = Some(x);
        self.node_mut(y).parent = x_parent;
        self.node_mut(x).parent = Some(y);

        self.replace_parent_child(x_parent, x, Some(y));
        self.update_size(Some(x));
        self.update_size(Some(y));
    }

    pub fn cursor<'a>(&'a self, key: &K) -> Option<RbCursor<'a, K, V>> {
        self.find_node(key).map(|node_idx| RbCursor {
            tree: self,
            node_idx,
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.find_node(key).is_some()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<RbCursor<'a, K, V>> {
        self.leftmost(self.root).map(|node_idx| RbCursor {
            tree: self,
            node_idx,
        })
    }

    pub fn max_cursor<'a>(&'a self) -> Option<RbCursor<'a, K, V>> {
        self.rightmost(self.root).map(|node_idx| RbCursor {
            tree: self,
            node_idx,
        })
    }

    pub fn insert_entry(&mut self, key: K, value: V) -> Option<V> {
        let mut parent = None;
        let mut current = self.root;

        while let Some(idx) = current {
            parent = Some(idx);
            let n = self.node_ref(idx);
            current = match key.cmp(&n.key) {
                std::cmp::Ordering::Less => n.left,
                std::cmp::Ordering::Greater => n.right,
                std::cmp::Ordering::Equal => {
                    let old = std::mem::replace(&mut self.node_mut(idx).value, value);
                    return Some(old);
                }
            };
        }

        let z = self.alloc_node(RbNode::new(key, value));
        self.node_mut(z).parent = parent;

        if let Some(pidx) = parent {
            let go_left = self.node_ref(z).key < self.node_ref(pidx).key;
            if go_left {
                self.node_mut(pidx).left = Some(z);
            } else {
                self.node_mut(pidx).right = Some(z);
            }
        } else {
            self.root = Some(z);
        }

        let mut anc = parent;
        while let Some(a) = anc {
            self.node_mut(a).size += 1;
            anc = self.node_ref(a).parent;
        }

        self.insert_fixup(z);
        None
    }

    fn insert_fixup(&mut self, mut z: usize) {
        while self
            .node_ref(z)
            .parent
            .is_some_and(|p| self.color_of(Some(p)) == NodeColor::Red)
        {
            let p = self.node_ref(z).parent.expect("parent exists");
            let g = self.node_ref(p).parent.expect("grandparent exists");
            let parent_is_left = self.node_ref(g).left == Some(p);

            if parent_is_left {
                let u = self.node_ref(g).right;
                if self.color_of(u) == NodeColor::Red {
                    self.node_mut(p).color = NodeColor::Black;
                    if let Some(uidx) = u {
                        self.node_mut(uidx).color = NodeColor::Black;
                    }
                    self.node_mut(g).color = NodeColor::Red;
                    z = g;
                } else {
                    if self.node_ref(p).right == Some(z) {
                        z = p;
                        self.left_rotate(z);
                    }
                    let p2 = self.node_ref(z).parent.expect("parent exists");
                    let g2 = self.node_ref(p2).parent.expect("grandparent exists");
                    self.node_mut(p2).color = NodeColor::Black;
                    self.node_mut(g2).color = NodeColor::Red;
                    self.right_rotate(g2);
                }
            } else {
                let u = self.node_ref(g).left;
                if self.color_of(u) == NodeColor::Red {
                    self.node_mut(p).color = NodeColor::Black;
                    if let Some(uidx) = u {
                        self.node_mut(uidx).color = NodeColor::Black;
                    }
                    self.node_mut(g).color = NodeColor::Red;
                    z = g;
                } else {
                    if self.node_ref(p).left == Some(z) {
                        z = p;
                        self.right_rotate(z);
                    }
                    let p2 = self.node_ref(z).parent.expect("parent exists");
                    let g2 = self.node_ref(p2).parent.expect("grandparent exists");
                    self.node_mut(p2).color = NodeColor::Black;
                    self.node_mut(g2).color = NodeColor::Red;
                    self.left_rotate(g2);
                }
            }
        }

        if let Some(root) = self.root {
            self.node_mut(root).color = NodeColor::Black;
        }
    }

    fn transplant(&mut self, u: usize, v: Option<usize>) {
        let parent = self.node_ref(u).parent;
        self.replace_parent_child(parent, u, v);
    }

    fn delete_fixup(&mut self, mut x: Option<usize>, mut x_parent: Option<usize>) {
        while x != self.root && self.color_of(x) == NodeColor::Black {
            let Some(parent) = x_parent else { break };
            let x_is_left = self.node_ref(parent).left == x;

            if x_is_left {
                let mut w = self.node_ref(parent).right;
                if self.color_of(w) == NodeColor::Red {
                    if let Some(wi) = w {
                        self.node_mut(wi).color = NodeColor::Black;
                    }
                    self.node_mut(parent).color = NodeColor::Red;
                    self.left_rotate(parent);
                    w = self.node_ref(parent).right;
                }

                let w_left = w.and_then(|wi| self.node_ref(wi).left);
                let w_right = w.and_then(|wi| self.node_ref(wi).right);

                if self.color_of(w_left) == NodeColor::Black
                    && self.color_of(w_right) == NodeColor::Black
                {
                    if let Some(wi) = w {
                        self.node_mut(wi).color = NodeColor::Red;
                    }
                    x = Some(parent);
                    x_parent = self.node_ref(parent).parent;
                } else {
                    if self.color_of(w_right) == NodeColor::Black {
                        if let Some(wl) = w_left {
                            self.node_mut(wl).color = NodeColor::Black;
                        }
                        if let Some(wi) = w {
                            self.node_mut(wi).color = NodeColor::Red;
                            self.right_rotate(wi);
                        }
                        w = self.node_ref(parent).right;
                    }

                    if let Some(wi) = w {
                        self.node_mut(wi).color = self.node_ref(parent).color;
                    }
                    self.node_mut(parent).color = NodeColor::Black;
                    if let Some(wr) = w.and_then(|wi| self.node_ref(wi).right) {
                        self.node_mut(wr).color = NodeColor::Black;
                    }
                    self.left_rotate(parent);
                    x = self.root;
                    x_parent = None;
                }
            } else {
                let mut w = self.node_ref(parent).left;
                if self.color_of(w) == NodeColor::Red {
                    if let Some(wi) = w {
                        self.node_mut(wi).color = NodeColor::Black;
                    }
                    self.node_mut(parent).color = NodeColor::Red;
                    self.right_rotate(parent);
                    w = self.node_ref(parent).left;
                }

                let w_right = w.and_then(|wi| self.node_ref(wi).right);
                let w_left = w.and_then(|wi| self.node_ref(wi).left);

                if self.color_of(w_right) == NodeColor::Black
                    && self.color_of(w_left) == NodeColor::Black
                {
                    if let Some(wi) = w {
                        self.node_mut(wi).color = NodeColor::Red;
                    }
                    x = Some(parent);
                    x_parent = self.node_ref(parent).parent;
                } else {
                    if self.color_of(w_left) == NodeColor::Black {
                        if let Some(wr) = w_right {
                            self.node_mut(wr).color = NodeColor::Black;
                        }
                        if let Some(wi) = w {
                            self.node_mut(wi).color = NodeColor::Red;
                            self.left_rotate(wi);
                        }
                        w = self.node_ref(parent).left;
                    }

                    if let Some(wi) = w {
                        self.node_mut(wi).color = self.node_ref(parent).color;
                    }
                    self.node_mut(parent).color = NodeColor::Black;
                    if let Some(wl) = w.and_then(|wi| self.node_ref(wi).left) {
                        self.node_mut(wl).color = NodeColor::Black;
                    }
                    self.right_rotate(parent);
                    x = self.root;
                    x_parent = None;
                }
            }
        }

        if let Some(xi) = x {
            self.node_mut(xi).color = NodeColor::Black;
        }
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let z = self.find_node(key)?;

        let mut y = z;
        let mut y_original_color = self.color_of(Some(y));
        let x: Option<usize>;
        let x_parent: Option<usize>;

        let (z_left, z_right) = {
            let n = self.node_ref(z);
            (n.left, n.right)
        };

        if z_left.is_none() {
            x = z_right;
            x_parent = self.node_ref(z).parent;
            self.transplant(z, z_right);
        } else if z_right.is_none() {
            x = z_left;
            x_parent = self.node_ref(z).parent;
            self.transplant(z, z_left);
        } else {
            y = self.leftmost(z_right).expect("right subtree has minimum");
            y_original_color = self.color_of(Some(y));
            x = self.node_ref(y).right;

            if self.node_ref(y).parent == Some(z) {
                x_parent = Some(y);
                if let Some(xi) = x {
                    self.node_mut(xi).parent = Some(y);
                }
            } else {
                x_parent = self.node_ref(y).parent;
                let y_right = self.node_ref(y).right;
                self.transplant(y, y_right);
                self.node_mut(y).right = z_right;
                if let Some(zr) = z_right {
                    self.node_mut(zr).parent = Some(y);
                }
            }

            self.transplant(z, Some(y));
            self.node_mut(y).left = z_left;
            if let Some(zl) = z_left {
                self.node_mut(zl).parent = Some(y);
            }
            self.node_mut(y).color = self.node_ref(z).color;
            self.update_size(Some(y));
        }

        self.recompute_sizes_up(x_parent);

        if y_original_color == NodeColor::Black {
            self.delete_fixup(x, x_parent);
        }

        let removed = self.take_node(z);
        Some(removed.value)
    }

    pub fn size(&self) -> usize {
        self.node_size(self.root)
    }

    pub fn select<'a>(&'a self, rank: usize) -> Option<RbCursor<'a, K, V>> {
        let mut current = self.root;
        let mut k = rank;

        while let Some(idx) = current {
            let left = self.node_ref(idx).left;
            let left_size = self.node_size(left);
            if k < left_size {
                current = left;
            } else if k == left_size {
                return Some(RbCursor {
                    tree: self,
                    node_idx: idx,
                });
            } else {
                k -= left_size + 1;
                current = self.node_ref(idx).right;
            }
        }

        None
    }

    pub fn iter<'a>(&'a self) -> RbIter<'a, K, V> {
        RbIter {
            tree: self,
            next: self.leftmost(self.root),
        }
    }
}

impl<K: Ord, V> Map<K, V> for RedBlackTree<K, V> {
    type Cursor<'a>
        = RbCursor<'a, K, V>
    where
        Self: 'a;

    type View<'a>
        = RbNodeView<'a, K, V>
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
        = RbCursor<'a, K, V>
    where
        Self: 'a;

    fn height(&self) -> usize {
        self.height_from(self.root)
    }

    fn node_count(&self) -> usize {
        self.node_size(self.root)
    }

    fn node_height<'a>(&'a self, cursor: &Self::NodeCursor<'a>) -> usize {
        self.height_from(Some(cursor.node_idx))
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
        let idx = self.next?;
        let node = self.tree.node_ref(idx);
        let item = (node.key.clone(), node.value.clone());
        self.next = self.tree.successor_node(idx);
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
