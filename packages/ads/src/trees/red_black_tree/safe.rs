use std::{
    cell::{Ref, RefCell},
    cmp::Ordering,
    rc::{Rc, Weak},
};

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
    left: Option<Rc<RefCell<RbNode<K, V>>>>,
    right: Option<Rc<RefCell<RbNode<K, V>>>>,
    parent: Option<Weak<RefCell<RbNode<K, V>>>>,
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
pub struct RbNodeView<K, V> {
    node: Rc<RefCell<RbNode<K, V>>>,
}

#[derive(Debug)]
pub struct RbCursor<'a, K, V> {
    tree: &'a RedBlackTree<K, V>,
    node: Rc<RefCell<RbNode<K, V>>>,
}

#[derive(Debug)]
pub struct RbIter<'a, K, V> {
    next: Option<RbCursor<'a, K, V>>,
}

impl<K, V> Clone for RbNodeView<K, V> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl<'a, K, V> Clone for RbCursor<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node.clone(),
        }
    }
}

impl<K, V> From<Rc<RefCell<RbNode<K, V>>>> for RbNodeView<K, V> {
    fn from(node: Rc<RefCell<RbNode<K, V>>>) -> Self {
        Self { node }
    }
}

impl<K, V> RbNodeView<K, V> {
    pub fn key(&self) -> Ref<'_, K> {
        Ref::map(self.node.borrow(), |node| &node.key)
    }

    pub fn value(&self) -> Ref<'_, V> {
        Ref::map(self.node.borrow(), |node| &node.value)
    }

    pub fn color(&self) -> NodeColor {
        self.node.borrow().color
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

impl<'a, K: Ord, V> RbCursor<'a, K, V> {
    fn rc(&self) -> Rc<RefCell<RbNode<K, V>>> {
        self.node.clone()
    }

    pub fn key(&self) -> Ref<'_, K> {
        Ref::map(self.node.borrow(), |node| &node.key)
    }

    pub fn value(&self) -> Ref<'_, V> {
        Ref::map(self.node.borrow(), |node| &node.value)
    }

    pub fn node_view(&self) -> RbNodeView<K, V> {
        RbNodeView::from(self.node.clone())
    }

    pub fn predecessor(&self) -> Option<Self> {
        let node = RedBlackTree::<K, V>::predecessor_node(&self.node)?;
        Some(Self {
            tree: self.tree,
            node,
        })
    }

    pub fn successor(&self) -> Option<Self> {
        let node = RedBlackTree::<K, V>::successor_node(&self.node)?;
        Some(Self {
            tree: self.tree,
            node,
        })
    }
}

#[derive(Debug)]
pub struct RedBlackTree<K, V> {
    root: Option<Rc<RefCell<RbNode<K, V>>>>,
}

impl<K, V> RedBlackTree<K, V> {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn root_view(&self) -> Option<RbNodeView<K, V>> {
        self.root.clone().map(RbNodeView::from)
    }

    pub fn len(&self) -> usize {
        self.root.as_ref().map_or(0, |root| root.borrow().size)
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn clear(&mut self) {
        self.root = None;
    }

    fn height_node(node: &Option<Rc<RefCell<RbNode<K, V>>>>) -> usize {
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

impl<K: Ord, V> RedBlackTree<K, V> {
    fn leftmost(mut node: Rc<RefCell<RbNode<K, V>>>) -> Rc<RefCell<RbNode<K, V>>> {
        loop {
            let next = node.borrow().left.clone();
            match next {
                Some(left) => node = left,
                None => return node,
            }
        }
    }

    fn rightmost(mut node: Rc<RefCell<RbNode<K, V>>>) -> Rc<RefCell<RbNode<K, V>>> {
        loop {
            let next = node.borrow().right.clone();
            match next {
                Some(right) => node = right,
                None => return node,
            }
        }
    }

    fn node_for_key(&self, key: &K) -> Option<Rc<RefCell<RbNode<K, V>>>> {
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

    pub fn cursor<'a>(&'a self, key: &K) -> Option<RbCursor<'a, K, V>> {
        self.node_for_key(key)
            .map(|node| RbCursor { tree: self, node })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.cursor(key).is_some()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<RbCursor<'a, K, V>> {
        self.root.clone().map(|root| RbCursor {
            tree: self,
            node: Self::leftmost(root),
        })
    }

    pub fn max_cursor<'a>(&'a self) -> Option<RbCursor<'a, K, V>> {
        self.root.clone().map(|root| RbCursor {
            tree: self,
            node: Self::rightmost(root),
        })
    }

    fn predecessor_node(node: &Rc<RefCell<RbNode<K, V>>>) -> Option<Rc<RefCell<RbNode<K, V>>>> {
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

    fn successor_node(node: &Rc<RefCell<RbNode<K, V>>>) -> Option<Rc<RefCell<RbNode<K, V>>>> {
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

    fn subtree_size(node: &Option<Rc<RefCell<RbNode<K, V>>>>) -> usize {
        node.as_ref().map_or(0, |rc| rc.borrow().size)
    }

    fn update_size(node: &Rc<RefCell<RbNode<K, V>>>) {
        let left_size = node
            .borrow()
            .left
            .as_ref()
            .map_or(0, |left| left.borrow().size);
        let right_size = node
            .borrow()
            .right
            .as_ref()
            .map_or(0, |right| right.borrow().size);
        node.borrow_mut().size = 1 + left_size + right_size;
    }

    fn recompute_sizes_up(mut node: Option<Rc<RefCell<RbNode<K, V>>>>) {
        while let Some(rc) = node {
            Self::update_size(&rc);
            node = rc
                .borrow()
                .parent
                .as_ref()
                .and_then(|parent| parent.upgrade());
        }
    }

    fn left_rotate(&mut self, x: &Rc<RefCell<RbNode<K, V>>>) {
        let y = x
            .borrow()
            .right
            .clone()
            .expect("right child must exist for left rotation");

        x.borrow_mut().right = y.borrow().left.clone();
        if let Some(y_left) = y.borrow().left.clone() {
            y_left.borrow_mut().parent = Some(Rc::downgrade(x));
        }

        let x_parent = x.borrow().parent.clone();
        y.borrow_mut().parent = x_parent.clone();

        if let Some(parent_weak) = x_parent {
            if let Some(parent) = parent_weak.upgrade() {
                let is_left = parent
                    .borrow()
                    .left
                    .as_ref()
                    .is_some_and(|left| Rc::ptr_eq(left, x));
                if is_left {
                    parent.borrow_mut().left = Some(y.clone());
                } else {
                    parent.borrow_mut().right = Some(y.clone());
                }
            }
        } else {
            self.root = Some(y.clone());
        }

        y.borrow_mut().left = Some(x.clone());
        x.borrow_mut().parent = Some(Rc::downgrade(&y));

        Self::update_size(x);
        Self::update_size(&y);
    }

    fn right_rotate(&mut self, x: &Rc<RefCell<RbNode<K, V>>>) {
        let y = x
            .borrow()
            .left
            .clone()
            .expect("left child must exist for right rotation");

        x.borrow_mut().left = y.borrow().right.clone();
        if let Some(y_right) = y.borrow().right.clone() {
            y_right.borrow_mut().parent = Some(Rc::downgrade(x));
        }

        let x_parent = x.borrow().parent.clone();
        y.borrow_mut().parent = x_parent.clone();

        if let Some(parent_weak) = x_parent {
            if let Some(parent) = parent_weak.upgrade() {
                let is_right = parent
                    .borrow()
                    .right
                    .as_ref()
                    .is_some_and(|right| Rc::ptr_eq(right, x));
                if is_right {
                    parent.borrow_mut().right = Some(y.clone());
                } else {
                    parent.borrow_mut().left = Some(y.clone());
                }
            }
        } else {
            self.root = Some(y.clone());
        }

        y.borrow_mut().right = Some(x.clone());
        x.borrow_mut().parent = Some(Rc::downgrade(&y));

        Self::update_size(x);
        Self::update_size(&y);
    }

    pub fn insert_entry(&mut self, key: K, value: V) -> Option<V> {
        let mut parent = None;
        let mut current = self.root.clone();

        while let Some(current_rc) = current {
            parent = Some(current_rc.clone());
            let direction = {
                let current_borrow = current_rc.borrow();
                if key == current_borrow.key {
                    Ordering::Equal
                } else if key < current_borrow.key {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            };

            match direction {
                Ordering::Equal => {
                    return Some(std::mem::replace(&mut current_rc.borrow_mut().value, value));
                }
                Ordering::Less => {
                    current = current_rc.borrow().left.clone();
                }
                Ordering::Greater => {
                    current = current_rc.borrow().right.clone();
                }
            }
        }

        let new_node = Rc::new(RefCell::new(RbNode::new(key, value)));

        if let Some(parent_rc) = parent {
            new_node.borrow_mut().parent = Some(Rc::downgrade(&parent_rc));

            {
                let mut parent_borrow = parent_rc.borrow_mut();
                if new_node.borrow().key < parent_borrow.key {
                    parent_borrow.left = Some(new_node.clone());
                } else {
                    parent_borrow.right = Some(new_node.clone());
                }
            }

            let mut ancestor = Some(parent_rc);
            while let Some(rc) = ancestor {
                rc.borrow_mut().size += 1;
                ancestor = rc
                    .borrow()
                    .parent
                    .as_ref()
                    .and_then(|parent| parent.upgrade());
            }
        } else {
            self.root = Some(new_node.clone());
        }

        self.insert_fixup(new_node);
        None
    }

    fn insert_fixup(&mut self, mut z: Rc<RefCell<RbNode<K, V>>>) {
        while z
            .borrow()
            .parent
            .as_ref()
            .and_then(|parent| parent.upgrade())
            .is_some_and(|parent| parent.borrow().color == NodeColor::Red)
        {
            let z_parent = z
                .borrow()
                .parent
                .as_ref()
                .and_then(|parent| parent.upgrade())
                .expect("parent exists");
            let z_grandparent = z_parent
                .borrow()
                .parent
                .as_ref()
                .and_then(|parent| parent.upgrade())
                .expect("grandparent exists");

            let parent_is_left = z_grandparent
                .borrow()
                .left
                .as_ref()
                .is_some_and(|left| Rc::ptr_eq(left, &z_parent));

            if parent_is_left {
                let uncle = z_grandparent.borrow().right.clone();

                if uncle
                    .as_ref()
                    .is_some_and(|u| u.borrow().color == NodeColor::Red)
                {
                    z_parent.borrow_mut().color = NodeColor::Black;
                    uncle.expect("uncle").borrow_mut().color = NodeColor::Black;
                    z_grandparent.borrow_mut().color = NodeColor::Red;
                    z = z_grandparent;
                } else {
                    if z_parent
                        .borrow()
                        .right
                        .as_ref()
                        .is_some_and(|right| Rc::ptr_eq(right, &z))
                    {
                        z = z_parent.clone();
                        self.left_rotate(&z);
                    }

                    let parent_new = z
                        .borrow()
                        .parent
                        .as_ref()
                        .and_then(|parent| parent.upgrade())
                        .expect("parent exists");
                    let grandparent_new = parent_new
                        .borrow()
                        .parent
                        .as_ref()
                        .and_then(|parent| parent.upgrade())
                        .expect("grandparent exists");

                    parent_new.borrow_mut().color = NodeColor::Black;
                    grandparent_new.borrow_mut().color = NodeColor::Red;
                    self.right_rotate(&grandparent_new);
                }
            } else {
                let uncle = z_grandparent.borrow().left.clone();

                if uncle
                    .as_ref()
                    .is_some_and(|u| u.borrow().color == NodeColor::Red)
                {
                    z_parent.borrow_mut().color = NodeColor::Black;
                    uncle.expect("uncle").borrow_mut().color = NodeColor::Black;
                    z_grandparent.borrow_mut().color = NodeColor::Red;
                    z = z_grandparent;
                } else {
                    if z_parent
                        .borrow()
                        .left
                        .as_ref()
                        .is_some_and(|left| Rc::ptr_eq(left, &z))
                    {
                        z = z_parent.clone();
                        self.right_rotate(&z);
                    }

                    let parent_new = z
                        .borrow()
                        .parent
                        .as_ref()
                        .and_then(|parent| parent.upgrade())
                        .expect("parent exists");
                    let grandparent_new = parent_new
                        .borrow()
                        .parent
                        .as_ref()
                        .and_then(|parent| parent.upgrade())
                        .expect("grandparent exists");

                    parent_new.borrow_mut().color = NodeColor::Black;
                    grandparent_new.borrow_mut().color = NodeColor::Red;
                    self.left_rotate(&grandparent_new);
                }
            }
        }

        if let Some(root) = self.root.as_ref() {
            root.borrow_mut().color = NodeColor::Black;
        }
    }

    fn transplant(
        &mut self,
        target: &Rc<RefCell<RbNode<K, V>>>,
        replacement: Option<Rc<RefCell<RbNode<K, V>>>>,
    ) {
        let target_parent = target.borrow().parent.clone();

        if let Some(parent_rc) = target_parent.as_ref().and_then(|parent| parent.upgrade()) {
            let is_left = parent_rc
                .borrow()
                .left
                .as_ref()
                .is_some_and(|left| Rc::ptr_eq(left, target));

            if is_left {
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

    fn remove_node_internal(&mut self, z: Rc<RefCell<RbNode<K, V>>>) -> Option<V> {
        let y_original_color;
        let x: Option<Rc<RefCell<RbNode<K, V>>>>;
        let x_parent: Option<Rc<RefCell<RbNode<K, V>>>>;

        if z.borrow().left.is_none() {
            y_original_color = z.borrow().color;
            x = z.borrow_mut().right.take();
            x_parent = z
                .borrow()
                .parent
                .as_ref()
                .and_then(|parent| parent.upgrade());
            self.transplant(&z, x.clone());
        } else if z.borrow().right.is_none() {
            y_original_color = z.borrow().color;
            x = z.borrow_mut().left.take();
            x_parent = z
                .borrow()
                .parent
                .as_ref()
                .and_then(|parent| parent.upgrade());
            self.transplant(&z, x.clone());
        } else {
            let right_child = z.borrow_mut().right.take().expect("right child exists");
            let y = Self::leftmost(right_child.clone());
            y_original_color = y.borrow().color;

            x = y.borrow().right.clone();

            if Rc::ptr_eq(&y, &right_child) {
                x_parent = Some(y.clone());
            } else {
                x_parent = y
                    .borrow()
                    .parent
                    .as_ref()
                    .and_then(|parent| parent.upgrade());
                self.transplant(&y, x.clone());
                y.borrow_mut().right = Some(right_child.clone());
                right_child.borrow_mut().parent = Some(Rc::downgrade(&y));
            }

            self.transplant(&z, Some(y.clone()));
            let left_child = z.borrow_mut().left.take();
            y.borrow_mut().left = left_child.clone();
            if let Some(left) = left_child {
                left.borrow_mut().parent = Some(Rc::downgrade(&y));
            }
            y.borrow_mut().color = z.borrow().color;
            Self::update_size(&y);
        }

        Self::recompute_sizes_up(x_parent.clone());

        if y_original_color == NodeColor::Black {
            self.delete_fixup(x, x_parent);
        }

        Some(
            Rc::try_unwrap(z)
                .unwrap_or_else(|_| unreachable!("deleted node should be uniquely owned"))
                .into_inner()
                .value,
        )
    }

    fn delete_fixup(
        &mut self,
        mut x: Option<Rc<RefCell<RbNode<K, V>>>>,
        mut x_parent: Option<Rc<RefCell<RbNode<K, V>>>>,
    ) {
        while !match (&x, &self.root) {
            (Some(node), Some(root)) => Rc::ptr_eq(node, root),
            (None, None) => true,
            _ => false,
        } && x
            .as_ref()
            .is_none_or(|node| node.borrow().color == NodeColor::Black)
        {
            let parent = x_parent.clone().expect("parent exists during fixup");

            let x_is_left = if let Some(ref x_rc) = x {
                parent
                    .borrow()
                    .left
                    .as_ref()
                    .is_some_and(|left| Rc::ptr_eq(left, x_rc))
            } else {
                parent.borrow().left.is_none()
            };

            if x_is_left {
                let mut sibling = parent.borrow().right.clone().expect("sibling exists");
                if sibling.borrow().color == NodeColor::Red {
                    sibling.borrow_mut().color = NodeColor::Black;
                    parent.borrow_mut().color = NodeColor::Red;
                    self.left_rotate(&parent);
                    sibling = parent.borrow().right.clone().expect("sibling exists");
                }

                let left_black = sibling
                    .borrow()
                    .left
                    .as_ref()
                    .is_none_or(|left| left.borrow().color == NodeColor::Black);
                let right_black = sibling
                    .borrow()
                    .right
                    .as_ref()
                    .is_none_or(|right| right.borrow().color == NodeColor::Black);

                if left_black && right_black {
                    sibling.borrow_mut().color = NodeColor::Red;
                    x = Some(parent.clone());
                    x_parent = parent.borrow().parent.as_ref().and_then(|p| p.upgrade());
                } else {
                    if right_black {
                        if let Some(sibling_left) = sibling.borrow().left.clone() {
                            sibling_left.borrow_mut().color = NodeColor::Black;
                        }
                        sibling.borrow_mut().color = NodeColor::Red;
                        self.right_rotate(&sibling);
                        sibling = parent.borrow().right.clone().expect("sibling exists");
                    }

                    sibling.borrow_mut().color = parent.borrow().color;
                    parent.borrow_mut().color = NodeColor::Black;
                    if let Some(sibling_right) = sibling.borrow().right.clone() {
                        sibling_right.borrow_mut().color = NodeColor::Black;
                    }
                    self.left_rotate(&parent);
                    x = self.root.clone();
                }
            } else {
                let mut sibling = parent.borrow().left.clone().expect("sibling exists");
                if sibling.borrow().color == NodeColor::Red {
                    sibling.borrow_mut().color = NodeColor::Black;
                    parent.borrow_mut().color = NodeColor::Red;
                    self.right_rotate(&parent);
                    sibling = parent.borrow().left.clone().expect("sibling exists");
                }

                let right_black = sibling
                    .borrow()
                    .right
                    .as_ref()
                    .is_none_or(|right| right.borrow().color == NodeColor::Black);
                let left_black = sibling
                    .borrow()
                    .left
                    .as_ref()
                    .is_none_or(|left| left.borrow().color == NodeColor::Black);

                if right_black && left_black {
                    sibling.borrow_mut().color = NodeColor::Red;
                    x = Some(parent.clone());
                    x_parent = parent.borrow().parent.as_ref().and_then(|p| p.upgrade());
                } else {
                    if left_black {
                        if let Some(sibling_right) = sibling.borrow().right.clone() {
                            sibling_right.borrow_mut().color = NodeColor::Black;
                        }
                        sibling.borrow_mut().color = NodeColor::Red;
                        self.left_rotate(&sibling);
                        sibling = parent.borrow().left.clone().expect("sibling exists");
                    }

                    sibling.borrow_mut().color = parent.borrow().color;
                    parent.borrow_mut().color = NodeColor::Black;
                    if let Some(sibling_left) = sibling.borrow().left.clone() {
                        sibling_left.borrow_mut().color = NodeColor::Black;
                    }
                    self.right_rotate(&parent);
                    x = self.root.clone();
                }
            }
        }

        if let Some(node) = x {
            node.borrow_mut().color = NodeColor::Black;
        }
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let target = self.node_for_key(key)?;
        self.remove_node_internal(target)
    }

    pub fn size(&self) -> usize {
        Self::subtree_size(&self.root)
    }

    pub fn select<'a>(&'a self, rank: usize) -> Option<RbCursor<'a, K, V>> {
        let mut current = self.root.clone()?;
        let mut k = rank;

        loop {
            let left_size = current
                .borrow()
                .left
                .as_ref()
                .map_or(0, |left| left.borrow().size);

            if k < left_size {
                let left = current.borrow().left.clone()?;
                current = left;
            } else if k == left_size {
                return Some(RbCursor {
                    tree: self,
                    node: current,
                });
            } else {
                k -= left_size + 1;
                let right = current.borrow().right.clone()?;
                current = right;
            }
        }
    }

    pub fn iter<'a>(&'a self) -> RbIter<'a, K, V> {
        RbIter {
            next: self.min_cursor(),
        }
    }
}

impl<K: Ord, V> Map<K, V> for RedBlackTree<K, V> {
    type Cursor<'a>
        = RbCursor<'a, K, V>
    where
        Self: 'a;

    type View<'a>
        = RbNodeView<K, V>
    where
        Self: 'a;

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        RedBlackTree::insert_entry(self, key, value)
    }

    fn cursor<'a>(&'a self, key: &K) -> Option<Self::Cursor<'a>> {
        RedBlackTree::cursor(self, key)
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        cursor.node_view()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        RedBlackTree::remove_key(self, key)
    }

    fn contains_key(&self, key: &K) -> bool {
        RedBlackTree::contains_key(self, key)
    }

    fn clear(&mut self) {
        RedBlackTree::clear(self)
    }

    fn len(&self) -> usize {
        RedBlackTree::size(self)
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
        Self::height_node(&self.root)
    }

    fn node_count(&self) -> usize {
        self.root.as_ref().map_or(0, |root| root.borrow().size)
    }

    fn node_height<'a>(&'a self, cursor: &Self::NodeCursor<'a>) -> usize {
        Self::height_node(&Some(cursor.rc()))
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
        let current = self.next.take()?;
        self.next = current.successor();

        let (key, value) = {
            let node = current.node.borrow();
            (node.key.clone(), node.value.clone())
        };

        Some((key, value))
    }
}

impl<'a, K: Ord + Clone, V: Clone> IntoIterator for &'a RedBlackTree<K, V> {
    type Item = (K, V);
    type IntoIter = RbIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
