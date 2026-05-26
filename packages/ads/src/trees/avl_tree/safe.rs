use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

use crate::traits::{
    core::{Map, OrderedMap},
    diagnostics::TreeDiagnostics,
};

#[derive(Debug)]
struct AvlNode<K, V> {
    key: K,
    value: V,
    height: usize,
    left: Option<Rc<RefCell<AvlNode<K, V>>>>,
    right: Option<Rc<RefCell<AvlNode<K, V>>>>,
    parent: Option<Weak<RefCell<AvlNode<K, V>>>>,
}

impl<K, V> AvlNode<K, V> {
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            height: 1,
            left: None,
            right: None,
            parent: None,
        }
    }
}

#[derive(Debug)]
pub struct AvlNodeView<K, V> {
    node: Rc<RefCell<AvlNode<K, V>>>,
}

#[derive(Debug)]
pub struct AvlCursor<'a, K, V> {
    tree: &'a AvlTree<K, V>,
    node: Rc<RefCell<AvlNode<K, V>>>,
}

#[derive(Debug)]
pub struct AvlIter<'a, K, V> {
    next: Option<AvlCursor<'a, K, V>>,
}

impl<K, V> Clone for AvlNodeView<K, V> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl<'a, K, V> Clone for AvlCursor<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node.clone(),
        }
    }
}

impl<K, V> From<Rc<RefCell<AvlNode<K, V>>>> for AvlNodeView<K, V> {
    fn from(node: Rc<RefCell<AvlNode<K, V>>>) -> Self {
        Self { node }
    }
}

impl<K, V> AvlNodeView<K, V> {
    pub fn key(&self) -> Ref<'_, K> {
        Ref::map(self.node.borrow(), |node| &node.key)
    }

    pub fn value(&self) -> Ref<'_, V> {
        Ref::map(self.node.borrow(), |node| &node.value)
    }

    pub fn left(&self) -> Option<Self> {
        self.node
            .borrow()
            .left
            .as_ref()
            .map(|left| Self::from(left.clone()))
    }

    pub fn right(&self) -> Option<Self> {
        self.node
            .borrow()
            .right
            .as_ref()
            .map(|right| Self::from(right.clone()))
    }

    pub fn parent(&self) -> Option<Self> {
        self.node
            .borrow()
            .parent
            .as_ref()
            .and_then(|parent| parent.upgrade())
            .map(Self::from)
    }
}

impl<'a, K: Ord, V> AvlCursor<'a, K, V> {
    fn rc(&self) -> Rc<RefCell<AvlNode<K, V>>> {
        self.node.clone()
    }

    pub fn key(&self) -> Ref<'_, K> {
        Ref::map(self.node.borrow(), |node| &node.key)
    }

    pub fn value(&self) -> Ref<'_, V> {
        Ref::map(self.node.borrow(), |node| &node.value)
    }

    pub fn node_view(&self) -> AvlNodeView<K, V> {
        AvlNodeView::from(self.node.clone())
    }

    pub fn predecessor(&self) -> Option<Self> {
        let node = AvlTree::<K, V>::predecessor_node(&self.node)?;
        Some(Self {
            tree: self.tree,
            node,
        })
    }

    pub fn successor(&self) -> Option<Self> {
        let node = AvlTree::<K, V>::successor_node(&self.node)?;
        Some(Self {
            tree: self.tree,
            node,
        })
    }
}

#[derive(Debug)]
pub struct AvlTree<K, V> {
    root: Option<Rc<RefCell<AvlNode<K, V>>>>,
    len: usize,
}

impl<K, V> AvlTree<K, V> {
    pub fn new() -> Self {
        Self { root: None, len: 0 }
    }

    pub fn root_view(&self) -> Option<AvlNodeView<K, V>> {
        self.root.clone().map(AvlNodeView::from)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.len = 0;
    }
}

impl<K: Ord, V> AvlTree<K, V> {
    fn node_height(node: &Option<Rc<RefCell<AvlNode<K, V>>>>) -> usize {
        node.as_ref().map_or(0, |node| node.borrow().height)
    }

    fn update_height(node: &Rc<RefCell<AvlNode<K, V>>>) {
        let (left, right) = {
            let b = node.borrow();
            (b.left.clone(), b.right.clone())
        };

        let left_height = Self::node_height(&left);
        let right_height = Self::node_height(&right);
        node.borrow_mut().height = 1 + usize::max(left_height, right_height);
    }

    fn balance_factor(node: &Rc<RefCell<AvlNode<K, V>>>) -> isize {
        let (left, right) = {
            let b = node.borrow();
            (b.left.clone(), b.right.clone())
        };

        Self::node_height(&left) as isize - Self::node_height(&right) as isize
    }

    fn leftmost(mut node: Rc<RefCell<AvlNode<K, V>>>) -> Rc<RefCell<AvlNode<K, V>>> {
        loop {
            let next = node.borrow().left.clone();
            match next {
                Some(left) => node = left,
                None => return node,
            }
        }
    }

    fn rightmost(mut node: Rc<RefCell<AvlNode<K, V>>>) -> Rc<RefCell<AvlNode<K, V>>> {
        loop {
            let next = node.borrow().right.clone();
            match next {
                Some(right) => node = right,
                None => return node,
            }
        }
    }

    fn predecessor_node(node: &Rc<RefCell<AvlNode<K, V>>>) -> Option<Rc<RefCell<AvlNode<K, V>>>> {
        let mut current = node.clone();

        if let Some(left) = current.borrow().left.clone() {
            return Some(Self::rightmost(left));
        }

        loop {
            let parent = current.borrow().parent.clone().and_then(|p| p.upgrade())?;

            let parent_right = parent.borrow().right.clone();
            if parent_right
                .as_ref()
                .is_some_and(|right| Rc::ptr_eq(right, &current))
            {
                return Some(parent);
            }

            current = parent;
        }
    }

    fn successor_node(node: &Rc<RefCell<AvlNode<K, V>>>) -> Option<Rc<RefCell<AvlNode<K, V>>>> {
        let mut current = node.clone();

        if let Some(right) = current.borrow().right.clone() {
            return Some(Self::leftmost(right));
        }

        loop {
            let parent = current.borrow().parent.clone().and_then(|p| p.upgrade())?;

            let parent_left = parent.borrow().left.clone();
            if parent_left
                .as_ref()
                .is_some_and(|left| Rc::ptr_eq(left, &current))
            {
                return Some(parent);
            }

            current = parent;
        }
    }

    fn node_for_key(&self, key: &K) -> Option<Rc<RefCell<AvlNode<K, V>>>> {
        let mut current = self.root.clone();

        while let Some(current_rc) = current {
            let current_borrow = current_rc.borrow();
            if key == &current_borrow.key {
                return Some(current_rc.clone());
            }

            current = if key < &current_borrow.key {
                current_borrow.left.clone()
            } else {
                current_borrow.right.clone()
            };
        }

        None
    }

    fn rotate_left(&mut self, x: Rc<RefCell<AvlNode<K, V>>>) -> Rc<RefCell<AvlNode<K, V>>> {
        let y = x
            .borrow_mut()
            .right
            .take()
            .expect("right child must exist for left rotation");

        let y_left = y.borrow_mut().left.take();
        x.borrow_mut().right = y_left.clone();
        if let Some(left_child) = y_left {
            left_child.borrow_mut().parent = Some(Rc::downgrade(&x));
        }

        let x_parent = x.borrow().parent.clone().and_then(|parent| parent.upgrade());
        if let Some(parent) = x_parent {
            let is_left = parent
                .borrow()
                .left
                .as_ref()
                .is_some_and(|left| Rc::ptr_eq(left, &x));
            if is_left {
                parent.borrow_mut().left = Some(y.clone());
            } else {
                parent.borrow_mut().right = Some(y.clone());
            }
            y.borrow_mut().parent = Some(Rc::downgrade(&parent));
        } else {
            y.borrow_mut().parent = None;
            self.root = Some(y.clone());
        }

        y.borrow_mut().left = Some(x.clone());
        x.borrow_mut().parent = Some(Rc::downgrade(&y));

        Self::update_height(&x);
        Self::update_height(&y);

        y
    }

    fn rotate_right(&mut self, x: Rc<RefCell<AvlNode<K, V>>>) -> Rc<RefCell<AvlNode<K, V>>> {
        let y = x
            .borrow_mut()
            .left
            .take()
            .expect("left child must exist for right rotation");

        let y_right = y.borrow_mut().right.take();
        x.borrow_mut().left = y_right.clone();
        if let Some(right_child) = y_right {
            right_child.borrow_mut().parent = Some(Rc::downgrade(&x));
        }

        let x_parent = x.borrow().parent.clone().and_then(|parent| parent.upgrade());
        if let Some(parent) = x_parent {
            let is_left = parent
                .borrow()
                .left
                .as_ref()
                .is_some_and(|left| Rc::ptr_eq(left, &x));
            if is_left {
                parent.borrow_mut().left = Some(y.clone());
            } else {
                parent.borrow_mut().right = Some(y.clone());
            }
            y.borrow_mut().parent = Some(Rc::downgrade(&parent));
        } else {
            y.borrow_mut().parent = None;
            self.root = Some(y.clone());
        }

        y.borrow_mut().right = Some(x.clone());
        x.borrow_mut().parent = Some(Rc::downgrade(&y));

        Self::update_height(&x);
        Self::update_height(&y);

        y
    }

    fn rebalance_upwards(&mut self, mut current: Option<Rc<RefCell<AvlNode<K, V>>>>) {
        while let Some(node) = current {
            Self::update_height(&node);
            let balance = Self::balance_factor(&node);

            if balance > 1 {
                let left = node.borrow().left.clone().expect("left child must exist");
                if Self::balance_factor(&left) < 0 {
                    self.rotate_left(left);
                }

                let rotated_root = self.rotate_right(node);
                current = rotated_root
                    .borrow()
                    .parent
                    .as_ref()
                    .and_then(|parent| parent.upgrade());
            } else if balance < -1 {
                let right = node
                    .borrow()
                    .right
                    .clone()
                    .expect("right child must exist");
                if Self::balance_factor(&right) > 0 {
                    self.rotate_right(right);
                }

                let rotated_root = self.rotate_left(node);
                current = rotated_root
                    .borrow()
                    .parent
                    .as_ref()
                    .and_then(|parent| parent.upgrade());
            } else {
                current = node
                    .borrow()
                    .parent
                    .as_ref()
                    .and_then(|parent| parent.upgrade());
            }
        }
    }

    pub fn cursor<'a>(&'a self, key: &K) -> Option<AvlCursor<'a, K, V>> {
        self.node_for_key(key)
            .map(|node| AvlCursor { tree: self, node })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.cursor(key).is_some()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<AvlCursor<'a, K, V>> {
        self.root.clone().map(|root| AvlCursor {
            tree: self,
            node: Self::leftmost(root),
        })
    }

    pub fn max_cursor<'a>(&'a self) -> Option<AvlCursor<'a, K, V>> {
        self.root.clone().map(|root| AvlCursor {
            tree: self,
            node: Self::rightmost(root),
        })
    }

    pub fn insert_entry(&mut self, key: K, value: V) -> Option<V> {
        let mut parent = None;
        let mut current = self.root.clone();

        while let Some(current_rc) = current {
            parent = Some(current_rc.clone());
            let current_borrow = current_rc.borrow();

            if key == current_borrow.key {
                drop(current_borrow);
                return Some(std::mem::replace(&mut current_rc.borrow_mut().value, value));
            }

            current = if key < current_borrow.key {
                current_borrow.left.clone()
            } else {
                current_borrow.right.clone()
            };
        }

        let new_node = Rc::new(RefCell::new(AvlNode::new(key, value)));

        if let Some(parent_node) = parent.clone() {
            new_node.borrow_mut().parent = Some(Rc::downgrade(&parent_node));

            if new_node.borrow().key < parent_node.borrow().key {
                parent_node.borrow_mut().left = Some(new_node);
            } else {
                parent_node.borrow_mut().right = Some(new_node);
            }

            self.len += 1;
            self.rebalance_upwards(Some(parent_node));
        } else {
            self.root = Some(new_node);
            self.len += 1;
        }

        None
    }

    fn remove_node_with_at_most_one_child(
        &mut self,
        target: Rc<RefCell<AvlNode<K, V>>>,
    ) -> Option<(V, Option<Rc<RefCell<AvlNode<K, V>>>>)> {
        let parent = target
            .borrow()
            .parent
            .clone()
            .and_then(|parent| parent.upgrade());

        let child = {
            let mut target_mut = target.borrow_mut();
            if target_mut.left.is_some() {
                target_mut.left.take()
            } else {
                target_mut.right.take()
            }
        };

        if let Some(child_node) = child.as_ref() {
            child_node.borrow_mut().parent = parent.as_ref().map(Rc::downgrade);
        }

        if let Some(parent_node) = parent.as_ref() {
            let is_left = parent_node
                .borrow()
                .left
                .as_ref()
                .is_some_and(|left| Rc::ptr_eq(left, &target));

            if is_left {
                parent_node.borrow_mut().left = child.clone();
            } else {
                parent_node.borrow_mut().right = child.clone();
            }
        } else {
            self.root = child.clone();
        }

        let rebalance_start = parent.or(child);
        let removed_value = Rc::try_unwrap(target)
            .unwrap_or_else(|_| unreachable!("deleted node should be uniquely owned"))
            .into_inner()
            .value;

        Some((removed_value, rebalance_start))
    }

    fn remove_node_internal(
        &mut self,
        target: Rc<RefCell<AvlNode<K, V>>>,
    ) -> Option<(V, Option<Rc<RefCell<AvlNode<K, V>>>>)> {
        if target.borrow().left.is_some() && target.borrow().right.is_some() {
            let right = target
                .borrow()
                .right
                .clone()
                .expect("right child exists");
            let successor = Self::leftmost(right);

            {
                let mut target_mut = target.borrow_mut();
                let mut successor_mut = successor.borrow_mut();
                std::mem::swap(&mut target_mut.key, &mut successor_mut.key);
                std::mem::swap(&mut target_mut.value, &mut successor_mut.value);
            }

            self.remove_node_with_at_most_one_child(successor)
        } else {
            self.remove_node_with_at_most_one_child(target)
        }
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let target = self.node_for_key(key)?;
        let (removed_value, rebalance_start) = self.remove_node_internal(target)?;

        self.len = self.len.saturating_sub(1);

        if let Some(start) = rebalance_start {
            self.rebalance_upwards(Some(start));
        }

        Some(removed_value)
    }

    pub fn iter<'a>(&'a self) -> AvlIter<'a, K, V> {
        AvlIter {
            next: self.min_cursor(),
        }
    }
}

impl<K: Ord, V> Map<K, V> for AvlTree<K, V> {
    type Cursor<'a>
        = AvlCursor<'a, K, V>
    where
        Self: 'a;

    type View<'a>
        = AvlNodeView<K, V>
    where
        Self: 'a;

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        AvlTree::insert_entry(self, key, value)
    }

    fn cursor<'a>(&'a self, key: &K) -> Option<Self::Cursor<'a>> {
        AvlTree::cursor(self, key)
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        cursor.node_view()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        AvlTree::remove_key(self, key)
    }

    fn contains_key(&self, key: &K) -> bool {
        AvlTree::contains_key(self, key)
    }

    fn clear(&mut self) {
        AvlTree::clear(self)
    }

    fn len(&self) -> usize {
        AvlTree::len(self)
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
        = AvlCursor<'a, K, V>
    where
        Self: 'a;

    fn height(&self) -> usize {
        Self::node_height(&self.root)
    }

    fn node_count(&self) -> usize {
        self.len
    }

    fn node_height<'a>(&'a self, cursor: &Self::NodeCursor<'a>) -> usize {
        Self::node_height(&Some(cursor.rc()))
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
        let current = self.next.take()?;
        self.next = current.successor();

        let (key, value) = {
            let node = current.node.borrow();
            (node.key.clone(), node.value.clone())
        };

        Some((key, value))
    }
}

impl<'a, K: Ord + Clone, V: Clone> IntoIterator for &'a AvlTree<K, V> {
    type Item = (K, V);
    type IntoIter = AvlIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
