use crate::traits::{
    core::{Map, OrderedMap},
    diagnostics::TreeDiagnostics,
};

#[derive(Debug)]
struct BstNode<K, V> {
    key: K,
    value: V,
    left: Option<usize>,
    right: Option<usize>,
    parent: Option<usize>,
}

impl<K, V> BstNode<K, V> {
    fn new(key: K, value: V, parent: Option<usize>) -> Self {
        Self {
            key,
            value,
            left: None,
            right: None,
            parent,
        }
    }
}

#[derive(Debug)]
pub struct BstNodeView<'a, K, V> {
    tree: &'a BinarySearchTree<K, V>,
    node_idx: usize,
}

#[derive(Debug)]
pub struct BstCursor<'a, K, V> {
    tree: &'a BinarySearchTree<K, V>,
    node_idx: usize,
}

#[derive(Debug)]
pub struct BstIter<'a, K, V> {
    tree: &'a BinarySearchTree<K, V>,
    next: Option<usize>,
}

impl<'a, K, V> Clone for BstNodeView<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node_idx: self.node_idx,
        }
    }
}

impl<'a, K, V> Clone for BstCursor<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node_idx: self.node_idx,
        }
    }
}

impl<'a, K, V> BstNodeView<'a, K, V> {
    fn node_ref(&self) -> &BstNode<K, V> {
        self.tree.node_ref(self.node_idx)
    }

    pub fn key(&self) -> &K {
        &self.node_ref().key
    }

    pub fn value(&self) -> &V {
        &self.node_ref().value
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

impl<'a, K: Ord, V> BstCursor<'a, K, V> {
    fn node_ref(&self) -> &BstNode<K, V> {
        self.tree.node_ref(self.node_idx)
    }

    pub fn key(&self) -> &K {
        &self.node_ref().key
    }

    pub fn value(&self) -> &V {
        &self.node_ref().value
    }

    pub fn node_view(&self) -> BstNodeView<'a, K, V> {
        BstNodeView {
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
pub struct BinarySearchTree<K, V> {
    root: Option<usize>,
    nodes: Vec<Option<BstNode<K, V>>>,
    free: Vec<usize>,
    len: usize,
}

impl<K, V> BinarySearchTree<K, V> {
    pub fn new() -> Self {
        Self {
            root: None,
            nodes: Vec::new(),
            free: Vec::new(),
            len: 0,
        }
    }

    fn node_ref(&self, idx: usize) -> &BstNode<K, V> {
        self.nodes[idx].as_ref().expect("live arena node")
    }

    fn node_mut(&mut self, idx: usize) -> &mut BstNode<K, V> {
        self.nodes[idx].as_mut().expect("live arena node")
    }

    fn alloc_node(&mut self, node: BstNode<K, V>) -> usize {
        if let Some(idx) = self.free.pop() {
            self.nodes[idx] = Some(node);
            idx
        } else {
            self.nodes.push(Some(node));
            self.nodes.len() - 1
        }
    }

    fn take_node(&mut self, idx: usize) -> BstNode<K, V> {
        let node = self.nodes[idx].take().expect("live arena node");
        self.free.push(idx);
        node
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

    fn height_from(&self, node: Option<usize>) -> usize {
        let Some(idx) = node else {
            return 0;
        };

        let left = self.node_ref(idx).left;
        let right = self.node_ref(idx).right;
        1 + usize::max(self.height_from(left), self.height_from(right))
    }

    pub fn root_view(&self) -> Option<BstNodeView<'_, K, V>> {
        self.root.map(|node_idx| BstNodeView {
            tree: self,
            node_idx,
        })
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
}

impl<K: Ord, V> BinarySearchTree<K, V> {
    fn find_node(&self, key: &K) -> Option<usize> {
        let mut current = self.root;
        while let Some(idx) = current {
            let node = self.node_ref(idx);
            current = match key.cmp(&node.key) {
                std::cmp::Ordering::Less => node.left,
                std::cmp::Ordering::Greater => node.right,
                std::cmp::Ordering::Equal => return Some(idx),
            };
        }
        None
    }

    fn predecessor_node(&self, node_idx: usize) -> Option<usize> {
        let node = self.node_ref(node_idx);
        if let Some(left) = node.left {
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
        let node = self.node_ref(node_idx);
        if let Some(right) = node.right {
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

    fn transplant(&mut self, u: usize, v: Option<usize>) {
        let parent = self.node_ref(u).parent;
        if let Some(pidx) = parent {
            if self.node_ref(pidx).left == Some(u) {
                self.node_mut(pidx).left = v;
            } else {
                self.node_mut(pidx).right = v;
            }
        } else {
            self.root = v;
        }

        if let Some(vidx) = v {
            self.node_mut(vidx).parent = parent;
        }
    }

    pub fn cursor<'a>(&'a self, key: &K) -> Option<BstCursor<'a, K, V>> {
        self.find_node(key).map(|node_idx| BstCursor {
            tree: self,
            node_idx,
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.find_node(key).is_some()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<BstCursor<'a, K, V>> {
        self.leftmost(self.root).map(|node_idx| BstCursor {
            tree: self,
            node_idx,
        })
    }

    pub fn max_cursor<'a>(&'a self) -> Option<BstCursor<'a, K, V>> {
        self.rightmost(self.root).map(|node_idx| BstCursor {
            tree: self,
            node_idx,
        })
    }

    pub fn insert_entry(&mut self, key: K, value: V) -> Option<V> {
        let mut parent = None;
        let mut current = self.root;

        while let Some(idx) = current {
            parent = Some(idx);
            let node = self.node_ref(idx);
            current = match key.cmp(&node.key) {
                std::cmp::Ordering::Less => node.left,
                std::cmp::Ordering::Greater => node.right,
                std::cmp::Ordering::Equal => {
                    let old = std::mem::replace(&mut self.node_mut(idx).value, value);
                    return Some(old);
                }
            };
        }

        let new_idx = self.alloc_node(BstNode::new(key, value, parent));
        if let Some(pidx) = parent {
            let go_left = self.node_ref(new_idx).key < self.node_ref(pidx).key;
            if go_left {
                self.node_mut(pidx).left = Some(new_idx);
            } else {
                self.node_mut(pidx).right = Some(new_idx);
            }
        } else {
            self.root = Some(new_idx);
        }

        self.len += 1;
        None
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let target = self.find_node(key)?;

        let (left, right) = {
            let node = self.node_ref(target);
            (node.left, node.right)
        };

        if left.is_none() {
            self.transplant(target, right);
        } else if right.is_none() {
            self.transplant(target, left);
        } else {
            let succ = self.leftmost(right).expect("right subtree has minimum");
            let succ_parent = self.node_ref(succ).parent;

            if succ_parent != Some(target) {
                let succ_right = self.node_ref(succ).right;
                self.transplant(succ, succ_right);
                self.node_mut(succ).right = right;
                if let Some(ridx) = right {
                    self.node_mut(ridx).parent = Some(succ);
                }
            }

            self.transplant(target, Some(succ));
            self.node_mut(succ).left = left;
            if let Some(lidx) = left {
                self.node_mut(lidx).parent = Some(succ);
            }
        }

        self.len = self.len.saturating_sub(1);
        let removed = self.take_node(target);
        Some(removed.value)
    }

    pub fn iter<'a>(&'a self) -> BstIter<'a, K, V> {
        BstIter {
            tree: self,
            next: self.leftmost(self.root),
        }
    }
}

impl<K: Ord, V> Map<K, V> for BinarySearchTree<K, V> {
    type Cursor<'a>
        = BstCursor<'a, K, V>
    where
        Self: 'a;

    type View<'a>
        = BstNodeView<'a, K, V>
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
        = BstCursor<'a, K, V>
    where
        Self: 'a;

    fn height(&self) -> usize {
        self.height_from(self.root)
    }

    fn node_count(&self) -> usize {
        self.len
    }

    fn node_height<'a>(&'a self, cursor: &Self::NodeCursor<'a>) -> usize {
        self.height_from(Some(cursor.node_idx))
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
        let node_idx = self.next?;
        let node = self.tree.node_ref(node_idx);
        let item = (node.key.clone(), node.value.clone());
        self.next = self.tree.successor_node(node_idx);
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
