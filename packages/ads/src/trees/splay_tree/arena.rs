use crate::traits::{
    core::{Map, OrderedMap},
    diagnostics::TreeDiagnostics,
};

#[derive(Debug)]
struct SplayNode<K, V> {
    key: K,
    value: V,
    left: Option<usize>,
    right: Option<usize>,
    parent: Option<usize>,
}

impl<K, V> SplayNode<K, V> {
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
pub struct SplayNodeView<'a, K, V> {
    tree: &'a SplayTree<K, V>,
    node_idx: usize,
}

#[derive(Debug)]
pub struct SplayCursor<'a, K, V> {
    tree: &'a SplayTree<K, V>,
    node_idx: usize,
}

#[derive(Debug)]
pub struct SplayIter<'a, K, V> {
    tree: &'a SplayTree<K, V>,
    next: Option<usize>,
}

impl<'a, K, V> Clone for SplayNodeView<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node_idx: self.node_idx,
        }
    }
}

impl<'a, K, V> Clone for SplayCursor<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node_idx: self.node_idx,
        }
    }
}

impl<'a, K, V> SplayNodeView<'a, K, V> {
    fn node_ref(&self) -> &SplayNode<K, V> {
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

impl<'a, K: Ord, V> SplayCursor<'a, K, V> {
    fn node_ref(&self) -> &SplayNode<K, V> {
        self.tree.node_ref(self.node_idx)
    }

    pub fn key(&self) -> &K {
        &self.node_ref().key
    }

    pub fn value(&self) -> &V {
        &self.node_ref().value
    }

    pub fn node_view(&self) -> SplayNodeView<'a, K, V> {
        SplayNodeView {
            tree: self.tree,
            node_idx: self.node_idx,
        }
    }

    pub fn predecessor(&self) -> Option<Self> {
        self.tree
            .predecessor_node(self.node_idx)
            .map(|node_idx| Self {
                tree: self.tree,
                node_idx,
            })
    }

    pub fn successor(&self) -> Option<Self> {
        self.tree
            .successor_node(self.node_idx)
            .map(|node_idx| Self {
                tree: self.tree,
                node_idx,
            })
    }
}

#[derive(Debug)]
pub struct SplayTree<K, V> {
    root: Option<usize>,
    nodes: Vec<Option<SplayNode<K, V>>>,
    free: Vec<usize>,
    len: usize,
}

impl<K, V> SplayTree<K, V> {
    pub fn new() -> Self {
        Self {
            root: None,
            nodes: Vec::new(),
            free: Vec::new(),
            len: 0,
        }
    }

    fn node_ref(&self, idx: usize) -> &SplayNode<K, V> {
        self.nodes[idx].as_ref().expect("live arena node")
    }

    fn node_mut(&mut self, idx: usize) -> &mut SplayNode<K, V> {
        self.nodes[idx].as_mut().expect("live arena node")
    }

    fn alloc_node(&mut self, node: SplayNode<K, V>) -> usize {
        if let Some(idx) = self.free.pop() {
            self.nodes[idx] = Some(node);
            idx
        } else {
            self.nodes.push(Some(node));
            self.nodes.len() - 1
        }
    }

    fn take_node(&mut self, idx: usize) -> SplayNode<K, V> {
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

    pub fn root_view(&self) -> Option<SplayNodeView<'_, K, V>> {
        self.root.map(|node_idx| SplayNodeView {
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

        if let Some(nidx) = new_child {
            self.node_mut(nidx).parent = parent;
        }
    }

    fn rotate_left(&mut self, x: usize) {
        let y = self
            .node_ref(x)
            .right
            .expect("rotate_left requires right child");

        let y_left = self.node_ref(y).left;
        self.node_mut(x).right = y_left;
        if let Some(yl) = y_left {
            self.node_mut(yl).parent = Some(x);
        }

        let x_parent = self.node_ref(x).parent;
        self.replace_parent_child(x_parent, x, Some(y));

        self.node_mut(y).left = Some(x);
        self.node_mut(x).parent = Some(y);
    }

    fn rotate_right(&mut self, x: usize) {
        let y = self
            .node_ref(x)
            .left
            .expect("rotate_right requires left child");

        let y_right = self.node_ref(y).right;
        self.node_mut(x).left = y_right;
        if let Some(yr) = y_right {
            self.node_mut(yr).parent = Some(x);
        }

        let x_parent = self.node_ref(x).parent;
        self.replace_parent_child(x_parent, x, Some(y));

        self.node_mut(y).right = Some(x);
        self.node_mut(x).parent = Some(y);
    }
}

impl<K: Ord, V> SplayTree<K, V> {
    fn splay(&mut self, x: usize) {
        while let Some(parent) = self.node_ref(x).parent {
            if let Some(grand) = self.node_ref(parent).parent {
                let node_is_left = self.node_ref(parent).left == Some(x);
                let parent_is_left = self.node_ref(grand).left == Some(parent);

                if node_is_left && parent_is_left {
                    self.rotate_right(grand);
                    self.rotate_right(parent);
                } else if !node_is_left && !parent_is_left {
                    self.rotate_left(grand);
                    self.rotate_left(parent);
                } else if node_is_left && !parent_is_left {
                    self.rotate_right(parent);
                    self.rotate_left(grand);
                } else {
                    self.rotate_left(parent);
                    self.rotate_right(grand);
                }
            } else if self.node_ref(parent).left == Some(x) {
                self.rotate_right(parent);
            } else {
                self.rotate_left(parent);
            }
        }

        self.root = Some(x);
    }

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

    fn find_node_with_last(&self, key: &K) -> (Option<usize>, Option<usize>) {
        let mut current = self.root;
        let mut last = None;

        while let Some(idx) = current {
            last = Some(idx);
            let node = self.node_ref(idx);
            current = match key.cmp(&node.key) {
                std::cmp::Ordering::Less => node.left,
                std::cmp::Ordering::Greater => node.right,
                std::cmp::Ordering::Equal => return (Some(idx), last),
            };
        }

        (None, last)
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

    pub fn cursor<'a>(&'a self, key: &K) -> Option<SplayCursor<'a, K, V>> {
        self.find_node(key).map(|node_idx| SplayCursor {
            tree: self,
            node_idx,
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.find_node(key).is_some()
    }

    pub fn get_adaptive<'a>(&'a mut self, key: &K) -> Option<SplayNodeView<'a, K, V>> {
        let (found, last) = self.find_node_with_last(key);
        match found {
            Some(node_idx) => {
                self.splay(node_idx);
                Some(SplayNodeView {
                    tree: self,
                    node_idx,
                })
            }
            None => {
                if let Some(last_idx) = last {
                    self.splay(last_idx);
                }
                None
            }
        }
    }

    pub fn contains_adaptive(&mut self, key: &K) -> bool {
        self.get_adaptive(key).is_some()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<SplayCursor<'a, K, V>> {
        self.leftmost(self.root).map(|node_idx| SplayCursor {
            tree: self,
            node_idx,
        })
    }

    pub fn max_cursor<'a>(&'a self) -> Option<SplayCursor<'a, K, V>> {
        self.rightmost(self.root).map(|node_idx| SplayCursor {
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
                    self.splay(idx);
                    return Some(old);
                }
            };
        }

        let new_idx = self.alloc_node(SplayNode::new(key, value, parent));
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

        self.splay(new_idx);
        self.len += 1;
        None
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let target = self.find_node(key)?;
        self.splay(target);

        let left = self.node_ref(target).left;
        let right = self.node_ref(target).right;

        if let Some(lidx) = left {
            self.node_mut(lidx).parent = None;
        }
        if let Some(ridx) = right {
            self.node_mut(ridx).parent = None;
        }

        if left.is_none() {
            self.root = right;
        } else {
            self.root = left;
            let max_left = self
                .rightmost(self.root)
                .expect("left subtree has rightmost");
            self.splay(max_left);

            self.node_mut(max_left).right = right;
            if let Some(ridx) = right {
                self.node_mut(ridx).parent = Some(max_left);
            }
        }

        self.len = self.len.saturating_sub(1);
        let removed = self.take_node(target);
        Some(removed.value)
    }

    pub fn iter<'a>(&'a self) -> SplayIter<'a, K, V> {
        SplayIter {
            tree: self,
            next: self.leftmost(self.root),
        }
    }
}

impl<K: Ord, V> Map<K, V> for SplayTree<K, V> {
    type Cursor<'a>
        = SplayCursor<'a, K, V>
    where
        Self: 'a;

    type View<'a>
        = SplayNodeView<'a, K, V>
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
        = SplayCursor<'a, K, V>
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
        let node_idx = self.next?;
        let node = self.tree.node_ref(node_idx);
        let item = (node.key.clone(), node.value.clone());
        self.next = self.tree.successor_node(node_idx);
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
