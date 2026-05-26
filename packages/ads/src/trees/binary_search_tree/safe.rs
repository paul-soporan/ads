use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

use crate::traits::{
    core::{Map, OrderedMap},
    diagnostics::TreeDiagnostics,
};

#[derive(Debug)]
struct BstNode<K, V> {
    key: K,
    value: V,
    left: Option<Rc<RefCell<BstNode<K, V>>>>,
    right: Option<Rc<RefCell<BstNode<K, V>>>>,
    parent: Option<Weak<RefCell<BstNode<K, V>>>>,
}

impl<K, V> BstNode<K, V> {
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            left: None,
            right: None,
            parent: None,
        }
    }
}

#[derive(Debug)]
pub struct BstNodeView<K, V> {
    node: Rc<RefCell<BstNode<K, V>>>,
}

#[derive(Debug)]
pub struct BstCursor<'a, K, V> {
    tree: &'a BinarySearchTree<K, V>,
    node: Rc<RefCell<BstNode<K, V>>>,
}

#[derive(Debug)]
pub struct BstIter<'a, K, V> {
    next: Option<BstCursor<'a, K, V>>,
}

impl<K, V> Clone for BstNodeView<K, V> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl<'a, K, V> Clone for BstCursor<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node.clone(),
        }
    }
}

impl<K, V> From<Rc<RefCell<BstNode<K, V>>>> for BstNodeView<K, V> {
    fn from(node: Rc<RefCell<BstNode<K, V>>>) -> Self {
        Self { node }
    }
}

impl<K, V> BstNodeView<K, V> {
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

impl<'a, K: Ord, V> BstCursor<'a, K, V> {
    fn rc(&self) -> Rc<RefCell<BstNode<K, V>>> {
        self.node.clone()
    }

    pub fn key(&self) -> Ref<'_, K> {
        Ref::map(self.node.borrow(), |node| &node.key)
    }

    pub fn value(&self) -> Ref<'_, V> {
        Ref::map(self.node.borrow(), |node| &node.value)
    }

    pub fn node_view(&self) -> BstNodeView<K, V> {
        BstNodeView::from(self.node.clone())
    }

    pub fn predecessor(&self) -> Option<Self> {
        let node = BinarySearchTree::<K, V>::predecessor_node(&self.node)?;
        Some(Self {
            tree: self.tree,
            node,
        })
    }

    pub fn successor(&self) -> Option<Self> {
        let node = BinarySearchTree::<K, V>::successor_node(&self.node)?;
        Some(Self {
            tree: self.tree,
            node,
        })
    }
}

#[derive(Debug)]
pub struct BinarySearchTree<K, V> {
    root: Option<Rc<RefCell<BstNode<K, V>>>>,
    len: usize,
}

impl<K, V> BinarySearchTree<K, V> {
    pub fn new() -> Self {
        Self { root: None, len: 0 }
    }

    pub fn root_view(&self) -> Option<BstNodeView<K, V>> {
        self.root.clone().map(BstNodeView::from)
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

    fn height_node(node: &Option<Rc<RefCell<BstNode<K, V>>>>) -> usize {
        match node {
            None => 0,
            Some(rc) => {
                let left = rc.borrow().left.clone();
                let right = rc.borrow().right.clone();
                1 + usize::max(Self::height_node(&left), Self::height_node(&right))
            }
        }
    }
}

impl<K: Ord, V> BinarySearchTree<K, V> {
    fn leftmost(mut node: Rc<RefCell<BstNode<K, V>>>) -> Rc<RefCell<BstNode<K, V>>> {
        loop {
            let next = node.borrow().left.clone();
            match next {
                Some(left) => node = left,
                None => return node,
            }
        }
    }

    fn rightmost(mut node: Rc<RefCell<BstNode<K, V>>>) -> Rc<RefCell<BstNode<K, V>>> {
        loop {
            let next = node.borrow().right.clone();
            match next {
                Some(right) => node = right,
                None => return node,
            }
        }
    }

    fn node_for_key(&self, key: &K) -> Option<Rc<RefCell<BstNode<K, V>>>> {
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

    pub fn cursor<'a>(&'a self, key: &K) -> Option<BstCursor<'a, K, V>> {
        self.node_for_key(key)
            .map(|node| BstCursor { tree: self, node })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.cursor(key).is_some()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<BstCursor<'a, K, V>> {
        self.root.clone().map(|root| BstCursor {
            tree: self,
            node: Self::leftmost(root),
        })
    }

    pub fn max_cursor<'a>(&'a self) -> Option<BstCursor<'a, K, V>> {
        self.root.clone().map(|root| BstCursor {
            tree: self,
            node: Self::rightmost(root),
        })
    }

    fn predecessor_node(node: &Rc<RefCell<BstNode<K, V>>>) -> Option<Rc<RefCell<BstNode<K, V>>>> {
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

    fn successor_node(node: &Rc<RefCell<BstNode<K, V>>>) -> Option<Rc<RefCell<BstNode<K, V>>>> {
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

        let new_node = Rc::new(RefCell::new(BstNode::new(key, value)));
        if let Some(parent_rc) = parent {
            new_node.borrow_mut().parent = Some(Rc::downgrade(&parent_rc));
            let mut parent_borrow = parent_rc.borrow_mut();
            if new_node.borrow().key < parent_borrow.key {
                parent_borrow.left = Some(new_node);
            } else {
                parent_borrow.right = Some(new_node);
            }
        } else {
            self.root = Some(new_node);
        }

        self.len += 1;
        None
    }

    fn transplant(
        &mut self,
        target: &Rc<RefCell<BstNode<K, V>>>,
        replacement: Option<Rc<RefCell<BstNode<K, V>>>>,
    ) {
        let target_parent = target.borrow().parent.clone();

        if let Some(parent_rc) = target_parent.as_ref().and_then(|parent| parent.upgrade()) {
            let is_left_child = parent_rc
                .borrow()
                .left
                .as_ref()
                .is_some_and(|left| Rc::ptr_eq(left, target));

            if is_left_child {
                parent_rc.borrow_mut().left = replacement.clone();
            } else {
                parent_rc.borrow_mut().right = replacement.clone();
            }
        } else {
            self.root = replacement.clone();
        }

        if let Some(replacement_rc) = replacement {
            replacement_rc.borrow_mut().parent = target_parent;
        }
    }

    fn remove_node_internal(&mut self, target: Rc<RefCell<BstNode<K, V>>>) -> Option<V> {
        let has_left = target.borrow().left.is_some();
        let has_right = target.borrow().right.is_some();

        if has_left && has_right {
            let right_child = target
                .borrow_mut()
                .right
                .take()
                .expect("right child exists");
            let successor = Self::leftmost(right_child.clone());

            if !Rc::ptr_eq(&successor, &right_child) {
                let successor_right = successor.borrow_mut().right.take();
                self.transplant(&successor, successor_right);

                successor.borrow_mut().right = Some(right_child.clone());
                right_child.borrow_mut().parent = Some(Rc::downgrade(&successor));
            }

            self.transplant(&target, Some(successor.clone()));

            let left_child = target.borrow_mut().left.take();
            successor.borrow_mut().left = left_child.clone();
            if let Some(left) = left_child {
                left.borrow_mut().parent = Some(Rc::downgrade(&successor));
            }
        } else {
            let child = if has_left {
                target.borrow_mut().left.take()
            } else {
                target.borrow_mut().right.take()
            };

            self.transplant(&target, child);
        }

        self.len = self.len.saturating_sub(1);

        Some(
            Rc::try_unwrap(target)
                .unwrap_or_else(|_| unreachable!("deleted node should be uniquely owned"))
                .into_inner()
                .value,
        )
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let target = self.node_for_key(key)?;
        self.remove_node_internal(target)
    }

    pub fn iter<'a>(&'a self) -> BstIter<'a, K, V> {
        BstIter {
            next: self.min_cursor(),
        }
    }
}

impl<K: Ord, V> Map<K, V> for BinarySearchTree<K, V> {
    type Cursor<'a>
        = BstCursor<'a, K, V>
    where
        Self: 'a;

    type View<'a>
        = BstNodeView<K, V>
    where
        Self: 'a;

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        BinarySearchTree::insert_entry(self, key, value)
    }

    fn cursor<'a>(&'a self, key: &K) -> Option<Self::Cursor<'a>> {
        BinarySearchTree::cursor(self, key)
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        cursor.node_view()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        BinarySearchTree::remove_key(self, key)
    }

    fn contains_key(&self, key: &K) -> bool {
        BinarySearchTree::contains_key(self, key)
    }

    fn clear(&mut self) {
        BinarySearchTree::clear(self)
    }

    fn len(&self) -> usize {
        BinarySearchTree::len(self)
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
        Self::height_node(&self.root)
    }

    fn node_count(&self) -> usize {
        self.len
    }

    fn node_height<'a>(&'a self, cursor: &Self::NodeCursor<'a>) -> usize {
        Self::height_node(&Some(cursor.rc()))
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
        let current = self.next.take()?;
        self.next = current.successor();

        let (key, value) = {
            let node = current.node.borrow();
            (node.key.clone(), node.value.clone())
        };

        Some((key, value))
    }
}

impl<'a, K: Ord + Clone, V: Clone> IntoIterator for &'a BinarySearchTree<K, V> {
    type Item = (K, V);
    type IntoIter = BstIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
