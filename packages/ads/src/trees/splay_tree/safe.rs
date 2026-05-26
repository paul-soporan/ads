use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

use crate::traits::{
    core::{Map, OrderedMap},
    diagnostics::TreeDiagnostics,
};

#[derive(Debug)]
struct SplayNode<K, V> {
    key: K,
    value: V,
    left: Option<Rc<RefCell<SplayNode<K, V>>>>,
    right: Option<Rc<RefCell<SplayNode<K, V>>>>,
    parent: Option<Weak<RefCell<SplayNode<K, V>>>>,
}

impl<K, V> SplayNode<K, V> {
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
pub struct SplayNodeView<K, V> {
    node: Rc<RefCell<SplayNode<K, V>>>,
}

#[derive(Debug)]
pub struct SplayCursor<'a, K, V> {
    tree: &'a SplayTree<K, V>,
    node: Rc<RefCell<SplayNode<K, V>>>,
}

#[derive(Debug)]
pub struct SplayIter<'a, K, V> {
    next: Option<SplayCursor<'a, K, V>>,
}

impl<K, V> Clone for SplayNodeView<K, V> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl<'a, K, V> Clone for SplayCursor<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node.clone(),
        }
    }
}

impl<K, V> From<Rc<RefCell<SplayNode<K, V>>>> for SplayNodeView<K, V> {
    fn from(node: Rc<RefCell<SplayNode<K, V>>>) -> Self {
        Self { node }
    }
}

impl<K, V> SplayNodeView<K, V> {
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

impl<'a, K: Ord, V> SplayCursor<'a, K, V> {
    fn rc(&self) -> Rc<RefCell<SplayNode<K, V>>> {
        self.node.clone()
    }

    pub fn key(&self) -> Ref<'_, K> {
        Ref::map(self.node.borrow(), |node| &node.key)
    }

    pub fn value(&self) -> Ref<'_, V> {
        Ref::map(self.node.borrow(), |node| &node.value)
    }

    pub fn node_view(&self) -> SplayNodeView<K, V> {
        SplayNodeView::from(self.node.clone())
    }

    pub fn predecessor(&self) -> Option<Self> {
        let node = SplayTree::<K, V>::predecessor_node(&self.node)?;
        Some(Self {
            tree: self.tree,
            node,
        })
    }

    pub fn successor(&self) -> Option<Self> {
        let node = SplayTree::<K, V>::successor_node(&self.node)?;
        Some(Self {
            tree: self.tree,
            node,
        })
    }
}

#[derive(Debug)]
pub struct SplayTree<K, V> {
    root: Option<Rc<RefCell<SplayNode<K, V>>>>,
    len: usize,
}

impl<K, V> SplayTree<K, V> {
    pub fn new() -> Self {
        Self { root: None, len: 0 }
    }

    pub fn root_view(&self) -> Option<SplayNodeView<K, V>> {
        self.root.clone().map(SplayNodeView::from)
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

    fn height_node(node: &Option<Rc<RefCell<SplayNode<K, V>>>>) -> usize {
        match node {
            None => 0,
            Some(rc) => {
                let left = rc.borrow().left.clone();
                let right = rc.borrow().right.clone();
                1 + usize::max(Self::height_node(&left), Self::height_node(&right))
            }
        }
    }

    fn parent(node: &Rc<RefCell<SplayNode<K, V>>>) -> Option<Rc<RefCell<SplayNode<K, V>>>> {
        node.borrow().parent.as_ref().and_then(Weak::upgrade)
    }

    fn leftmost(mut node: Rc<RefCell<SplayNode<K, V>>>) -> Rc<RefCell<SplayNode<K, V>>> {
        loop {
            let next = node.borrow().left.clone();
            match next {
                Some(left) => node = left,
                None => return node,
            }
        }
    }

    fn rightmost(mut node: Rc<RefCell<SplayNode<K, V>>>) -> Rc<RefCell<SplayNode<K, V>>> {
        loop {
            let next = node.borrow().right.clone();
            match next {
                Some(right) => node = right,
                None => return node,
            }
        }
    }

    fn is_left_child(
        child: &Rc<RefCell<SplayNode<K, V>>>,
        parent: &Rc<RefCell<SplayNode<K, V>>>,
    ) -> bool {
        parent
            .borrow()
            .left
            .as_ref()
            .is_some_and(|left| Rc::ptr_eq(left, child))
    }

    fn replace_parent_child(
        &mut self,
        parent: Option<Rc<RefCell<SplayNode<K, V>>>>,
        old_child: &Rc<RefCell<SplayNode<K, V>>>,
        new_child: Option<Rc<RefCell<SplayNode<K, V>>>>,
    ) {
        if let Some(parent_rc) = parent {
            if Self::is_left_child(old_child, &parent_rc) {
                parent_rc.borrow_mut().left = new_child.clone();
            } else {
                parent_rc.borrow_mut().right = new_child.clone();
            }
            if let Some(new_rc) = new_child {
                new_rc.borrow_mut().parent = Some(Rc::downgrade(&parent_rc));
            }
        } else {
            self.root = new_child.clone();
            if let Some(new_rc) = new_child {
                new_rc.borrow_mut().parent = None;
            }
        }
    }

    fn rotate_left(&mut self, x: Rc<RefCell<SplayNode<K, V>>>) {
        let y = x
            .borrow_mut()
            .right
            .take()
            .expect("rotate_left requires right child");

        let y_left = y.borrow_mut().left.take();
        {
            let mut x_borrow = x.borrow_mut();
            x_borrow.right = y_left.clone();
            if let Some(ref node) = y_left {
                node.borrow_mut().parent = Some(Rc::downgrade(&x));
            }
        }

        let x_parent = Self::parent(&x);
        self.replace_parent_child(x_parent, &x, Some(y.clone()));

        y.borrow_mut().left = Some(x.clone());
        x.borrow_mut().parent = Some(Rc::downgrade(&y));
    }

    fn rotate_right(&mut self, x: Rc<RefCell<SplayNode<K, V>>>) {
        let y = x
            .borrow_mut()
            .left
            .take()
            .expect("rotate_right requires left child");

        let y_right = y.borrow_mut().right.take();
        {
            let mut x_borrow = x.borrow_mut();
            x_borrow.left = y_right.clone();
            if let Some(ref node) = y_right {
                node.borrow_mut().parent = Some(Rc::downgrade(&x));
            }
        }

        let x_parent = Self::parent(&x);
        self.replace_parent_child(x_parent, &x, Some(y.clone()));

        y.borrow_mut().right = Some(x.clone());
        x.borrow_mut().parent = Some(Rc::downgrade(&y));
    }
}

impl<K: Ord, V> SplayTree<K, V> {
    fn splay(&mut self, node: Rc<RefCell<SplayNode<K, V>>>) {
        while let Some(parent) = Self::parent(&node) {
            if let Some(grand) = Self::parent(&parent) {
                let node_is_left = Self::is_left_child(&node, &parent);
                let parent_is_left = Self::is_left_child(&parent, &grand);

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
            } else if Self::is_left_child(&node, &parent) {
                self.rotate_right(parent);
            } else {
                self.rotate_left(parent);
            }
        }

        self.root = Some(node);
    }

    fn node_for_key(&self, key: &K) -> Option<Rc<RefCell<SplayNode<K, V>>>> {
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

    fn node_for_key_with_last(
        &self,
        key: &K,
    ) -> (
        Option<Rc<RefCell<SplayNode<K, V>>>>,
        Option<Rc<RefCell<SplayNode<K, V>>>>,
    ) {
        let mut current = self.root.clone();
        let mut last = None;

        while let Some(current_rc) = current {
            last = Some(current_rc.clone());

            let ordering = {
                let borrowed = current_rc.borrow();
                key.cmp(&borrowed.key)
            };

            match ordering {
                std::cmp::Ordering::Less => {
                    current = current_rc.borrow().left.clone();
                }
                std::cmp::Ordering::Greater => {
                    current = current_rc.borrow().right.clone();
                }
                std::cmp::Ordering::Equal => {
                    return (Some(current_rc), last);
                }
            }
        }

        (None, last)
    }

    pub fn cursor<'a>(&'a self, key: &K) -> Option<SplayCursor<'a, K, V>> {
        self.node_for_key(key)
            .map(|node| SplayCursor { tree: self, node })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.cursor(key).is_some()
    }

    pub fn get_adaptive<'a>(&'a mut self, key: &K) -> Option<SplayNodeView<K, V>> {
        let (found, last) = self.node_for_key_with_last(key);
        match found {
            Some(node) => {
                self.splay(node.clone());
                Some(SplayNodeView::from(node))
            }
            None => {
                if let Some(last_node) = last {
                    self.splay(last_node);
                }
                None
            }
        }
    }

    pub fn contains_adaptive(&mut self, key: &K) -> bool {
        self.get_adaptive(key).is_some()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<SplayCursor<'a, K, V>> {
        self.root.clone().map(|root| SplayCursor {
            tree: self,
            node: Self::leftmost(root),
        })
    }

    pub fn max_cursor<'a>(&'a self) -> Option<SplayCursor<'a, K, V>> {
        self.root.clone().map(|root| SplayCursor {
            tree: self,
            node: Self::rightmost(root),
        })
    }

    fn predecessor_node(
        node: &Rc<RefCell<SplayNode<K, V>>>,
    ) -> Option<Rc<RefCell<SplayNode<K, V>>>> {
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

    fn successor_node(node: &Rc<RefCell<SplayNode<K, V>>>) -> Option<Rc<RefCell<SplayNode<K, V>>>> {
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
        if self.root.is_none() {
            self.root = Some(Rc::new(RefCell::new(SplayNode::new(key, value))));
            self.len = 1;
            return None;
        }

        let mut current = self.root.clone().expect("root exists");
        loop {
            let ordering = {
                let borrowed = current.borrow();
                key.cmp(&borrowed.key)
            };

            match ordering {
                std::cmp::Ordering::Less => {
                    let left = current.borrow().left.clone();
                    if let Some(next) = left {
                        current = next;
                    } else {
                        let new_node = Rc::new(RefCell::new(SplayNode::new(key, value)));
                        new_node.borrow_mut().parent = Some(Rc::downgrade(&current));
                        current.borrow_mut().left = Some(new_node.clone());
                        self.len += 1;
                        self.splay(new_node);
                        return None;
                    }
                }
                std::cmp::Ordering::Greater => {
                    let right = current.borrow().right.clone();
                    if let Some(next) = right {
                        current = next;
                    } else {
                        let new_node = Rc::new(RefCell::new(SplayNode::new(key, value)));
                        new_node.borrow_mut().parent = Some(Rc::downgrade(&current));
                        current.borrow_mut().right = Some(new_node.clone());
                        self.len += 1;
                        self.splay(new_node);
                        return None;
                    }
                }
                std::cmp::Ordering::Equal => {
                    let old = std::mem::replace(&mut current.borrow_mut().value, value);
                    self.splay(current);
                    return Some(old);
                }
            }
        }
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let target = self.node_for_key(key)?;
        self.splay(target.clone());

        let root = self.root.take().expect("splayed root exists");
        let left = root.borrow_mut().left.take();
        let right = root.borrow_mut().right.take();

        if let Some(ref left_rc) = left {
            left_rc.borrow_mut().parent = None;
        }
        if let Some(ref right_rc) = right {
            right_rc.borrow_mut().parent = None;
        }

        if let Some(left_root) = left {
            self.root = Some(left_root.clone());
            let max_left = Self::rightmost(left_root);
            self.splay(max_left.clone());
            max_left.borrow_mut().right = right.clone();
            if let Some(right_root) = right {
                right_root.borrow_mut().parent = Some(Rc::downgrade(&max_left));
            }
        } else {
            self.root = right;
        }

        self.len = self.len.saturating_sub(1);

        root.borrow_mut().parent = None;
        drop(target);
        let value = Rc::try_unwrap(root)
            .unwrap_or_else(|_| unreachable!("removed root should be uniquely owned"))
            .into_inner()
            .value;
        Some(value)
    }

    pub fn iter<'a>(&'a self) -> SplayIter<'a, K, V> {
        SplayIter {
            next: self.min_cursor(),
        }
    }
}

impl<K: Ord, V> Map<K, V> for SplayTree<K, V> {
    type Cursor<'a>
        = SplayCursor<'a, K, V>
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
        = SplayCursor<'a, K, V>
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
        let current = self.next.take()?;
        self.next = current.successor();

        let (key, value) = {
            let node = current.node.borrow();
            (node.key.clone(), node.value.clone())
        };

        Some((key, value))
    }
}

impl<'a, K: Ord + Clone, V: Clone> IntoIterator for &'a SplayTree<K, V> {
    type Item = (K, V);
    type IntoIter = SplayIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
