use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeColor {
    Red,
    Black,
}

#[derive(Debug)]
struct RbNode<T> {
    value: T,
    color: NodeColor,
    /// Number of nodes in this subtree (including self).
    size: usize,

    left: Option<Rc<RefCell<RbNode<T>>>>,
    right: Option<Rc<RefCell<RbNode<T>>>>,

    parent: Option<Weak<RefCell<RbNode<T>>>>,
}

impl<T> RbNode<T> {
    pub fn new(value: T) -> Self {
        Self {
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
pub struct RbNodeHandle<T> {
    node: Rc<RefCell<RbNode<T>>>,
}

impl<T> From<Rc<RefCell<RbNode<T>>>> for RbNodeHandle<T> {
    fn from(node: Rc<RefCell<RbNode<T>>>) -> Self {
        RbNodeHandle { node }
    }
}

impl<T> RbNodeHandle<T> {
    fn rc(&self) -> Rc<RefCell<RbNode<T>>> {
        self.node.clone()
    }

    fn into_inner(self) -> Rc<RefCell<RbNode<T>>> {
        self.node
    }

    pub fn value(&self) -> Ref<T> {
        Ref::map(self.node.borrow(), |node| &node.value)
    }

    pub fn color(&self) -> NodeColor {
        self.node.borrow().color
    }

    pub fn left(&self) -> Option<RbNodeHandle<T>> {
        self.node
            .borrow()
            .left
            .as_ref()
            .map(|left_rc| RbNodeHandle {
                node: left_rc.clone(),
            })
    }

    pub fn right(&self) -> Option<RbNodeHandle<T>> {
        self.node
            .borrow()
            .right
            .as_ref()
            .map(|right_rc| RbNodeHandle {
                node: right_rc.clone(),
            })
    }

    pub fn parent(&self) -> Option<RbNodeHandle<T>> {
        self.node
            .borrow()
            .parent
            .as_ref()
            .and_then(|weak_parent| weak_parent.upgrade())
            .map(|parent_rc| RbNodeHandle { node: parent_rc })
    }
}

#[derive(Debug)]
pub struct RedBlackTree<T> {
    root: Option<Rc<RefCell<RbNode<T>>>>,
}

impl<T> RedBlackTree<T> {
    pub fn new() -> Self {
        Self { root: None }
    }
}

impl<T: Ord> RedBlackTree<T> {
    pub fn root(&self) -> Option<RbNodeHandle<T>> {
        self.root.clone().map(RbNodeHandle::from)
    }

    fn leftmost(rc: Rc<RefCell<RbNode<T>>>) -> Rc<RefCell<RbNode<T>>> {
        let mut current = rc;
        loop {
            let next_left = current.borrow().left.clone();
            match next_left {
                Some(l) => current = l,
                None => return current,
            }
        }
    }

    fn rightmost(rc: Rc<RefCell<RbNode<T>>>) -> Rc<RefCell<RbNode<T>>> {
        let mut current = rc;
        loop {
            let next_right = current.borrow().right.clone();
            match next_right {
                Some(r) => current = r,
                None => return current,
            }
        }
    }

    pub fn search(&self, value: &T) -> Option<RbNodeHandle<T>> {
        let mut current = self.root.clone();

        while let Some(current_rc) = current {
            let current_borrow = current_rc.borrow();
            if value == &current_borrow.value {
                return Some(RbNodeHandle::from(current_rc.clone()));
            }

            current = if value < &current_borrow.value {
                current_borrow.left.clone()
            } else {
                current_borrow.right.clone()
            };
        }

        None
    }

    pub fn contains(&self, value: &T) -> bool {
        self.search(value).is_some()
    }

    pub fn min(&self) -> Option<RbNodeHandle<T>> {
        self.root
            .clone()
            .map(|root_rc| RbNodeHandle::from(Self::leftmost(root_rc)))
    }

    pub fn max(&self) -> Option<RbNodeHandle<T>> {
        self.root
            .clone()
            .map(|root_rc| RbNodeHandle::from(Self::rightmost(root_rc)))
    }

    pub fn predecessor(&self, handle: &RbNodeHandle<T>) -> Option<RbNodeHandle<T>> {
        let mut current = handle.rc();

        if let Some(left_rc) = &current.borrow().left {
            return Some(RbNodeHandle::from(Self::rightmost(left_rc.clone())));
        }

        loop {
            let parent_weak = current.borrow().parent.clone();
            let parent_rc = parent_weak.and_then(|w| w.upgrade())?;

            let parent_right = parent_rc.borrow().right.clone();
            if parent_right
                .as_ref()
                .map(|r| Rc::ptr_eq(r, &current))
                .unwrap_or(false)
            {
                return Some(RbNodeHandle::from(parent_rc));
            }

            current = parent_rc;
        }
    }

    pub fn successor(&self, handle: &RbNodeHandle<T>) -> Option<RbNodeHandle<T>> {
        let mut current = handle.rc();

        if let Some(right_rc) = &current.borrow().right {
            return Some(RbNodeHandle::from(Self::leftmost(right_rc.clone())));
        }

        loop {
            let parent_weak = current.borrow().parent.clone();
            let parent_rc = parent_weak.and_then(|w| w.upgrade())?;

            let parent_left = parent_rc.borrow().left.clone();
            if parent_left
                .as_ref()
                .map(|l| Rc::ptr_eq(l, &current))
                .unwrap_or(false)
            {
                return Some(RbNodeHandle::from(parent_rc));
            }

            current = parent_rc;
        }
    }

    pub fn predecessor_of_value(&self, value: &T) -> Option<RbNodeHandle<T>> {
        self.search(value).and_then(|h| self.predecessor(&h))
    }

    pub fn successor_of_value(&self, value: &T) -> Option<RbNodeHandle<T>> {
        self.search(value).and_then(|h| self.successor(&h))
    }

    fn subtree_size(node: &Option<Rc<RefCell<RbNode<T>>>>) -> usize {
        node.as_ref().map_or(0, |rc| rc.borrow().size)
    }

    fn update_size(rc: &Rc<RefCell<RbNode<T>>>) {
        let left_size = rc.borrow().left.as_ref().map_or(0, |l| l.borrow().size);
        let right_size = rc.borrow().right.as_ref().map_or(0, |r| r.borrow().size);
        rc.borrow_mut().size = 1 + left_size + right_size;
    }

    fn recompute_sizes_up(mut node: Option<Rc<RefCell<RbNode<T>>>>) {
        while let Some(rc) = node {
            Self::update_size(&rc);
            let next = rc.borrow().parent.as_ref().and_then(|w| w.upgrade());
            node = next;
        }
    }

    fn left_rotate(&mut self, x: &Rc<RefCell<RbNode<T>>>) {
        let y = x
            .borrow()
            .right
            .clone()
            .expect("y must exist for left_rotate");

        x.borrow_mut().right = y.borrow().left.clone();
        if let Some(y_left) = &y.borrow().left {
            y_left.borrow_mut().parent = Some(Rc::downgrade(x));
        }

        let x_parent = x.borrow().parent.clone();
        y.borrow_mut().parent = x_parent.clone();

        if let Some(p_weak) = x_parent {
            if let Some(p) = p_weak.upgrade() {
                let is_left = p.borrow().left.as_ref().is_some_and(|l| Rc::ptr_eq(l, x));
                if is_left {
                    p.borrow_mut().left = Some(y.clone());
                } else {
                    p.borrow_mut().right = Some(y.clone());
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

    fn right_rotate(&mut self, x: &Rc<RefCell<RbNode<T>>>) {
        let y = x
            .borrow()
            .left
            .clone()
            .expect("y must exist for right_rotate");

        x.borrow_mut().left = y.borrow().right.clone();
        if let Some(y_right) = &y.borrow().right {
            y_right.borrow_mut().parent = Some(Rc::downgrade(x));
        }

        let x_parent = x.borrow().parent.clone();
        y.borrow_mut().parent = x_parent.clone();

        if let Some(p_weak) = x_parent {
            if let Some(p) = p_weak.upgrade() {
                let is_right = p.borrow().right.as_ref().is_some_and(|r| Rc::ptr_eq(r, x));
                if is_right {
                    p.borrow_mut().right = Some(y.clone());
                } else {
                    p.borrow_mut().left = Some(y.clone());
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

    pub fn insert(&mut self, value: T) {
        let mut parent = None;
        let mut current = self.root.clone();

        while let Some(current_rc) = current {
            parent = Some(current_rc.clone());

            let current_borrow = current_rc.borrow();
            current = if value < current_borrow.value {
                current_borrow.left.clone()
            } else {
                current_borrow.right.clone()
            };
        }

        let new_node = Rc::new(RefCell::new(RbNode::new(value)));
        if let Some(parent_rc) = parent {
            new_node.borrow_mut().parent = Some(Rc::downgrade(&parent_rc));

            {
                let mut parent_borrow = parent_rc.borrow_mut();
                if new_node.borrow().value < parent_borrow.value {
                    parent_borrow.left = Some(new_node.clone());
                } else {
                    parent_borrow.right = Some(new_node.clone());
                }
            }

            let mut ancestor: Option<Rc<RefCell<RbNode<T>>>> = Some(parent_rc);
            while let Some(rc) = ancestor {
                rc.borrow_mut().size += 1;
                ancestor = rc.borrow().parent.as_ref().and_then(|w| w.upgrade());
            }
        } else {
            self.root = Some(new_node.clone());
        }

        self.insert_fixup(new_node);
    }

    fn insert_fixup(&mut self, mut z: Rc<RefCell<RbNode<T>>>) {
        while z
            .borrow()
            .parent
            .as_ref()
            .and_then(|w| w.upgrade())
            .is_some_and(|p| p.borrow().color == NodeColor::Red)
        {
            let z_parent = z.borrow().parent.as_ref().unwrap().upgrade().unwrap();
            let z_grandparent = z_parent
                .borrow()
                .parent
                .as_ref()
                .unwrap()
                .upgrade()
                .unwrap();

            let is_parent_left = z_grandparent
                .borrow()
                .left
                .as_ref()
                .is_some_and(|l| Rc::ptr_eq(l, &z_parent));

            if is_parent_left {
                let y = z_grandparent.borrow().right.clone();

                if y.as_ref()
                    .is_some_and(|uncle| uncle.borrow().color == NodeColor::Red)
                {
                    z_parent.borrow_mut().color = NodeColor::Black;
                    y.unwrap().borrow_mut().color = NodeColor::Black;
                    z_grandparent.borrow_mut().color = NodeColor::Red;
                    z = z_grandparent;
                } else {
                    if z_parent
                        .borrow()
                        .right
                        .as_ref()
                        .is_some_and(|r| Rc::ptr_eq(r, &z))
                    {
                        z = z_parent.clone();
                        self.left_rotate(&z);
                    }
                    let z_parent_new = z.borrow().parent.as_ref().unwrap().upgrade().unwrap();
                    let z_grandparent_new = z_parent_new
                        .borrow()
                        .parent
                        .as_ref()
                        .unwrap()
                        .upgrade()
                        .unwrap();
                    z_parent_new.borrow_mut().color = NodeColor::Black;
                    z_grandparent_new.borrow_mut().color = NodeColor::Red;
                    self.right_rotate(&z_grandparent_new);
                }
            } else {
                let y = z_grandparent.borrow().left.clone();

                if y.as_ref()
                    .is_some_and(|uncle| uncle.borrow().color == NodeColor::Red)
                {
                    z_parent.borrow_mut().color = NodeColor::Black;
                    y.unwrap().borrow_mut().color = NodeColor::Black;
                    z_grandparent.borrow_mut().color = NodeColor::Red;
                    z = z_grandparent;
                } else {
                    if z_parent
                        .borrow()
                        .left
                        .as_ref()
                        .is_some_and(|l| Rc::ptr_eq(l, &z))
                    {
                        z = z_parent.clone();
                        self.right_rotate(&z);
                    }
                    let z_parent_new = z.borrow().parent.as_ref().unwrap().upgrade().unwrap();
                    let z_grandparent_new = z_parent_new
                        .borrow()
                        .parent
                        .as_ref()
                        .unwrap()
                        .upgrade()
                        .unwrap();
                    z_parent_new.borrow_mut().color = NodeColor::Black;
                    z_grandparent_new.borrow_mut().color = NodeColor::Red;
                    self.left_rotate(&z_grandparent_new);
                }
            }
        }

        self.root.as_ref().unwrap().borrow_mut().color = NodeColor::Black;
    }

    fn transplant(
        &mut self,
        target: &Rc<RefCell<RbNode<T>>>,
        replacement: Option<Rc<RefCell<RbNode<T>>>>,
    ) {
        let target_parent_weak = target.borrow().parent.clone();

        if let Some(target_parent_rc) = target_parent_weak.as_ref().and_then(|w| w.upgrade()) {
            let is_left_child = target_parent_rc
                .borrow()
                .left
                .as_ref()
                .is_some_and(|l| Rc::ptr_eq(l, target));

            if is_left_child {
                target_parent_rc.borrow_mut().left = replacement.clone();
            } else {
                target_parent_rc.borrow_mut().right = replacement.clone();
            }
        } else {
            self.root = replacement.clone();
        }

        if let Some(replacement_rc) = replacement {
            replacement_rc.borrow_mut().parent = target_parent_weak;
        }
    }

    pub fn delete(&mut self, handle: RbNodeHandle<T>) -> Option<T> {
        let z = handle.into_inner();

        let y_original_color;
        let x: Option<Rc<RefCell<RbNode<T>>>>;
        let x_parent: Option<Rc<RefCell<RbNode<T>>>>;

        if z.borrow().left.is_none() {
            y_original_color = z.borrow().color;
            x = z.borrow_mut().right.take();
            x_parent = z.borrow().parent.as_ref().and_then(|w| w.upgrade());
            self.transplant(&z, x.clone());
        } else if z.borrow().right.is_none() {
            y_original_color = z.borrow().color;
            x = z.borrow_mut().left.take();
            x_parent = z.borrow().parent.as_ref().and_then(|w| w.upgrade());
            self.transplant(&z, x.clone());
        } else {
            let right_child = z.borrow_mut().right.take().unwrap();
            let y = Self::leftmost(right_child.clone());
            y_original_color = y.borrow().color;

            x = y.borrow().right.clone();

            if Rc::ptr_eq(&y, &right_child) {
                x_parent = Some(y.clone());
            } else {
                x_parent = y.borrow().parent.as_ref().and_then(|w| w.upgrade());
                self.transplant(&y, x.clone());
                y.borrow_mut().right = Some(right_child.clone());
                right_child.borrow_mut().parent = Some(Rc::downgrade(&y));
            }

            self.transplant(&z, Some(y.clone()));
            let left_child = z.borrow_mut().left.take();
            y.borrow_mut().left = left_child.clone();
            if let Some(yl) = left_child {
                yl.borrow_mut().parent = Some(Rc::downgrade(&y));
            }
            y.borrow_mut().color = z.borrow().color;
        }

        Self::recompute_sizes_up(x_parent.clone());

        if y_original_color == NodeColor::Black {
            self.delete_fixup(x, x_parent);
        }

        Some(
            Rc::try_unwrap(z)
                .unwrap_or_else(|_| {
                    unreachable!("Deleted node should only have one strong reference")
                })
                .into_inner()
                .value,
        )
    }

    fn delete_fixup(
        &mut self,
        mut x: Option<Rc<RefCell<RbNode<T>>>>,
        mut x_parent: Option<Rc<RefCell<RbNode<T>>>>,
    ) {
        while !match (&x, &self.root) {
            (Some(n), Some(r)) => Rc::ptr_eq(n, r),
            (None, None) => true,
            _ => false,
        } && x
            .as_ref()
            .is_none_or(|n| n.borrow().color == NodeColor::Black)
        {
            let xp = x_parent.clone().unwrap();

            let is_x_left = if let Some(ref x_rc) = x {
                xp.borrow()
                    .left
                    .as_ref()
                    .is_some_and(|l| Rc::ptr_eq(l, x_rc))
            } else {
                xp.borrow().left.is_none()
            };

            if is_x_left {
                let mut w = xp.borrow().right.clone().expect("Sibling must exist");
                if w.borrow().color == NodeColor::Red {
                    w.borrow_mut().color = NodeColor::Black;
                    xp.borrow_mut().color = NodeColor::Red;
                    self.left_rotate(&xp);
                    w = xp.borrow().right.clone().unwrap();
                }

                let wl_black = w
                    .borrow()
                    .left
                    .as_ref()
                    .is_none_or(|l| l.borrow().color == NodeColor::Black);
                let wr_black = w
                    .borrow()
                    .right
                    .as_ref()
                    .is_none_or(|r| r.borrow().color == NodeColor::Black);

                if wl_black && wr_black {
                    w.borrow_mut().color = NodeColor::Red;
                    x = Some(xp.clone());
                    x_parent = xp.borrow().parent.as_ref().and_then(|wp| wp.upgrade());
                } else {
                    if wr_black {
                        if let Some(wl) = w.borrow().left.clone() {
                            wl.borrow_mut().color = NodeColor::Black;
                        }
                        w.borrow_mut().color = NodeColor::Red;
                        self.right_rotate(&w);
                        w = xp.borrow().right.clone().unwrap();
                    }
                    w.borrow_mut().color = xp.borrow().color;
                    xp.borrow_mut().color = NodeColor::Black;
                    if let Some(wr) = w.borrow().right.clone() {
                        wr.borrow_mut().color = NodeColor::Black;
                    }
                    self.left_rotate(&xp);
                    x = self.root.clone();
                }
            } else {
                let mut w = xp.borrow().left.clone().expect("Sibling must exist");
                if w.borrow().color == NodeColor::Red {
                    w.borrow_mut().color = NodeColor::Black;
                    xp.borrow_mut().color = NodeColor::Red;
                    self.right_rotate(&xp);
                    w = xp.borrow().left.clone().unwrap();
                }

                let wr_black = w
                    .borrow()
                    .right
                    .as_ref()
                    .is_none_or(|r| r.borrow().color == NodeColor::Black);
                let wl_black = w
                    .borrow()
                    .left
                    .as_ref()
                    .is_none_or(|l| l.borrow().color == NodeColor::Black);

                if wr_black && wl_black {
                    w.borrow_mut().color = NodeColor::Red;
                    x = Some(xp.clone());
                    x_parent = xp.borrow().parent.as_ref().and_then(|wp| wp.upgrade());
                } else {
                    if wl_black {
                        if let Some(wr) = w.borrow().right.clone() {
                            wr.borrow_mut().color = NodeColor::Black;
                        }
                        w.borrow_mut().color = NodeColor::Red;
                        self.left_rotate(&w);
                        w = xp.borrow().left.clone().unwrap();
                    }
                    w.borrow_mut().color = xp.borrow().color;
                    xp.borrow_mut().color = NodeColor::Black;
                    if let Some(wl) = w.borrow().left.clone() {
                        wl.borrow_mut().color = NodeColor::Black;
                    }
                    self.right_rotate(&xp);
                    x = self.root.clone();
                }
            }
        }

        if let Some(x_node) = x {
            x_node.borrow_mut().color = NodeColor::Black;
        }
    }

    pub fn delete_value(&mut self, value: &T) -> Option<T> {
        self.search(value).and_then(|h| self.delete(h))
    }

    pub fn size(&self) -> usize {
        Self::subtree_size(&self.root)
    }

    pub fn select(&self, rank: usize) -> Option<RbNodeHandle<T>> {
        let mut current = self.root.clone()?;
        let mut k = rank;
        loop {
            let left_size = current
                .borrow()
                .left
                .as_ref()
                .map_or(0, |l| l.borrow().size);
            if k < left_size {
                let left = current.borrow().left.clone()?;
                current = left;
            } else if k == left_size {
                return Some(RbNodeHandle::from(current));
            } else {
                k -= left_size + 1;
                let right = current.borrow().right.clone()?;
                current = right;
            }
        }
    }
}

impl<T> Default for RedBlackTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_rb_properties<T: Ord + std::fmt::Debug>(tree: &RedBlackTree<T>) {
        if let Some(root) = &tree.root {
            assert_eq!(root.borrow().color, NodeColor::Black);
            check_node(root.clone());
        }
    }

    fn check_node<T: Ord + std::fmt::Debug>(node: Rc<RefCell<RbNode<T>>>) -> usize {
        let b = node.borrow();

        if b.color == NodeColor::Red {
            if let Some(left) = &b.left {
                assert_eq!(left.borrow().color, NodeColor::Black);
            }
            if let Some(right) = &b.right {
                assert_eq!(right.borrow().color, NodeColor::Black);
            }
        }

        let left_bh = b.left.as_ref().map_or(0, |l| check_node(l.clone()));
        let right_bh = b.right.as_ref().map_or(0, |r| check_node(r.clone()));

        assert_eq!(left_bh, right_bh);

        left_bh + if b.color == NodeColor::Black { 1 } else { 0 }
    }

    #[test]
    fn test_empty_tree() {
        let tree = RedBlackTree::<i32>::new();
        assert!(tree.min().is_none());
        assert!(tree.max().is_none());
        assert!(!tree.contains(&5));
        assert_rb_properties(&tree);
    }

    #[test]
    fn test_insert_and_contains() {
        //   5(B)
        //  /   \
        // 3(R) 7(R)
        let mut tree = RedBlackTree::new();
        tree.insert(5);
        tree.insert(3);
        tree.insert(7);

        assert!(tree.contains(&5));
        assert!(tree.contains(&3));
        assert!(tree.contains(&7));
        assert!(!tree.contains(&4));
        assert_rb_properties(&tree);
    }

    #[test]
    fn test_min_max() {
        let mut tree = RedBlackTree::new();
        let values = [5, 3, 7, 2, 4, 6, 8];
        for &v in &values {
            tree.insert(v);
        }

        assert_eq!(*tree.min().unwrap().value(), 2);
        assert_eq!(*tree.max().unwrap().value(), 8);
        assert_rb_properties(&tree);
    }

    #[test]
    fn test_predecessor_successor() {
        let mut tree = RedBlackTree::new();
        let values = [5, 3, 7, 2, 4, 6, 8];
        for &v in &values {
            tree.insert(v);
        }

        assert_eq!(*tree.successor_of_value(&2).unwrap().value(), 3);
        assert_eq!(*tree.successor_of_value(&4).unwrap().value(), 5);
        assert!(tree.successor_of_value(&8).is_none());

        assert_eq!(*tree.predecessor_of_value(&8).unwrap().value(), 7);
        assert_eq!(*tree.predecessor_of_value(&5).unwrap().value(), 4);
        assert!(tree.predecessor_of_value(&2).is_none());
    }

    #[test]
    fn test_insert_rotations_left() {
        // Insert: 10, 20, 30
        //
        // Result:
        //    20(B)
        //   /     \
        // 10(R)  30(R)
        let mut tree = RedBlackTree::new();
        tree.insert(10);
        tree.insert(20);
        tree.insert(30);

        assert_eq!(*tree.root().unwrap().value(), 20);
        assert_eq!(tree.root().unwrap().color(), NodeColor::Black);
        assert_rb_properties(&tree);
    }

    #[test]
    fn test_insert_rotations_right() {
        // Insert: 30, 20, 10
        //
        // Result:
        //    20(B)
        //   /     \
        // 10(R)  30(R)
        let mut tree = RedBlackTree::new();
        tree.insert(30);
        tree.insert(20);
        tree.insert(10);

        assert_eq!(*tree.root().unwrap().value(), 20);
        assert_rb_properties(&tree);
    }

    #[test]
    fn test_insert_rotations_left_right() {
        // Insert: 30, 10, 20
        //
        // Result:
        //    20(B)
        //   /     \
        // 10(R)  30(R)
        let mut tree = RedBlackTree::new();
        tree.insert(30);
        tree.insert(10);
        tree.insert(20);

        assert_eq!(*tree.root().unwrap().value(), 20);
        assert_rb_properties(&tree);
    }

    #[test]
    fn test_delete_leaf() {
        // Insert: 10, 5, 15
        //
        //    10(B)
        //   /     \
        // 5(R)   15(R)
        //
        // Delete 5 ->
        //
        //    10(B)
        //         \
        //         15(R)
        let mut tree = RedBlackTree::new();
        tree.insert(10);
        tree.insert(5);
        tree.insert(15);

        let deleted = tree.delete_value(&5);
        assert_eq!(deleted, Some(5));
        assert!(!tree.contains(&5));
        assert_rb_properties(&tree);
    }

    #[test]
    fn test_delete_node_with_two_children() {
        // Insert: 20, 10, 30
        //
        //    20(B)
        //   /     \
        // 10(R)  30(R)
        //
        // Delete 20 ->
        //
        //    30(B)
        //   /
        // 10(R)
        let mut tree = RedBlackTree::new();
        tree.insert(20);
        tree.insert(10);
        tree.insert(30);

        let deleted = tree.delete_value(&20);
        assert_eq!(deleted, Some(20));
        assert!(!tree.contains(&20));
        assert_eq!(*tree.root().unwrap().value(), 30);
        assert_rb_properties(&tree);
    }

    #[test]
    fn test_complex_tree_delete() {
        // Values: 10, 5, 15, 2, 7, 12, 20, 6, 8
        // Deletions test deep tree fixups and color changes.
        let mut tree = RedBlackTree::new();
        let values = [10, 5, 15, 2, 7, 12, 20, 6, 8];
        for &v in &values {
            tree.insert(v);
        }
        assert_rb_properties(&tree);

        assert_eq!(tree.delete_value(&5), Some(5));
        assert_rb_properties(&tree);

        assert_eq!(tree.delete_value(&10), Some(10));
        assert_rb_properties(&tree);
    }
}
