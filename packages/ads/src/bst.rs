use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

#[derive(Debug)]
struct BstNode<T> {
    value: T,

    left: Option<Rc<RefCell<BstNode<T>>>>,
    right: Option<Rc<RefCell<BstNode<T>>>>,

    parent: Option<Weak<RefCell<BstNode<T>>>>,
}

impl<T> BstNode<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            left: None,
            right: None,
            parent: None,
        }
    }
}

#[derive(Debug)]
pub struct BstNodeHandle<T> {
    node: Rc<RefCell<BstNode<T>>>,
}

impl<T> From<Rc<RefCell<BstNode<T>>>> for BstNodeHandle<T> {
    fn from(node: Rc<RefCell<BstNode<T>>>) -> Self {
        BstNodeHandle { node }
    }
}

impl<T> BstNodeHandle<T> {
    fn rc(&self) -> Rc<RefCell<BstNode<T>>> {
        self.node.clone()
    }

    fn into_inner(self) -> Rc<RefCell<BstNode<T>>> {
        self.node
    }

    pub fn value(&self) -> Ref<T> {
        Ref::map(self.node.borrow(), |node| &node.value)
    }

    pub fn left(&self) -> Option<BstNodeHandle<T>> {
        self.node
            .borrow()
            .left
            .as_ref()
            .map(|left_rc| BstNodeHandle {
                node: left_rc.clone(),
            })
    }

    pub fn right(&self) -> Option<BstNodeHandle<T>> {
        self.node
            .borrow()
            .right
            .as_ref()
            .map(|right_rc| BstNodeHandle {
                node: right_rc.clone(),
            })
    }

    pub fn parent(&self) -> Option<BstNodeHandle<T>> {
        self.node
            .borrow()
            .parent
            .as_ref()
            .and_then(|weak_parent| weak_parent.upgrade())
            .map(|parent_rc| BstNodeHandle { node: parent_rc })
    }
}

#[derive(Debug)]
pub struct BinarySearchTree<T> {
    root: Option<Rc<RefCell<BstNode<T>>>>,
}

impl<T> BinarySearchTree<T> {
    pub fn new() -> Self {
        Self { root: None }
    }
}

impl<T: Ord> BinarySearchTree<T> {
    fn leftmost(rc: Rc<RefCell<BstNode<T>>>) -> Rc<RefCell<BstNode<T>>> {
        let mut current = rc;
        loop {
            let next_left = current.borrow().left.clone();
            match next_left {
                Some(l) => current = l,
                None => return current,
            }
        }
    }

    fn rightmost(rc: Rc<RefCell<BstNode<T>>>) -> Rc<RefCell<BstNode<T>>> {
        let mut current = rc;
        loop {
            let next_right = current.borrow().right.clone();
            match next_right {
                Some(r) => current = r,
                None => return current,
            }
        }
    }

    pub fn search(&self, value: &T) -> Option<BstNodeHandle<T>> {
        let mut current = self.root.clone();

        while let Some(current_rc) = current {
            let current_borrow = current_rc.borrow();
            if value == &current_borrow.value {
                return Some(BstNodeHandle::from(current_rc.clone()));
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

    pub fn min(&self) -> Option<BstNodeHandle<T>> {
        self.root
            .clone()
            .map(|root_rc| BstNodeHandle::from(Self::leftmost(root_rc)))
    }

    pub fn max(&self) -> Option<BstNodeHandle<T>> {
        self.root
            .clone()
            .map(|root_rc| BstNodeHandle::from(Self::rightmost(root_rc)))
    }

    pub fn predecessor(&self, handle: &BstNodeHandle<T>) -> Option<BstNodeHandle<T>> {
        let mut current = handle.rc();

        if let Some(left_rc) = &current.borrow().left {
            return Some(BstNodeHandle::from(Self::rightmost(left_rc.clone())));
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
                return Some(BstNodeHandle::from(parent_rc));
            }

            current = parent_rc;
        }
    }

    pub fn successor(&self, handle: &BstNodeHandle<T>) -> Option<BstNodeHandle<T>> {
        let mut current = handle.rc();

        if let Some(right_rc) = &current.borrow().right {
            return Some(BstNodeHandle::from(Self::leftmost(right_rc.clone())));
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
                return Some(BstNodeHandle::from(parent_rc));
            }

            current = parent_rc;
        }
    }

    pub fn predecessor_of_value(&self, value: &T) -> Option<BstNodeHandle<T>> {
        self.search(value).and_then(|h| self.predecessor(&h))
    }

    pub fn successor_of_value(&self, value: &T) -> Option<BstNodeHandle<T>> {
        self.search(value).and_then(|h| self.successor(&h))
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

        let new_node = Rc::new(RefCell::new(BstNode::new(value)));
        if let Some(parent_rc) = parent {
            new_node.borrow_mut().parent = Some(Rc::downgrade(&parent_rc));

            let mut parent_borrow = parent_rc.borrow_mut();
            if new_node.borrow().value < parent_borrow.value {
                parent_borrow.left = Some(new_node.clone());
            } else {
                parent_borrow.right = Some(new_node.clone());
            }
        } else {
            self.root = Some(new_node);
        }
    }

    fn transplant(
        &mut self,
        target: &Rc<RefCell<BstNode<T>>>,
        replacement: Option<Rc<RefCell<BstNode<T>>>>,
    ) {
        let target_parent_weak = target.borrow().parent.clone();

        if let Some(target_parent_rc) = target_parent_weak.as_ref().and_then(|w| w.upgrade()) {
            let is_left_child = target_parent_rc
                .borrow()
                .left
                .as_ref()
                .map(|l| Rc::ptr_eq(l, target))
                .unwrap_or(false);

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

    pub fn delete(&mut self, handle: BstNodeHandle<T>) -> Option<T> {
        let target_rc = handle.into_inner();

        let has_left = target_rc.borrow().left.is_some();
        let has_right = target_rc.borrow().right.is_some();

        if has_left && has_right {
            let right_child = target_rc.borrow_mut().right.take().unwrap();
            let successor_rc = Self::leftmost(right_child.clone());

            if !Rc::ptr_eq(&successor_rc, &right_child) {
                let successor_right = successor_rc.borrow_mut().right.take();

                self.transplant(&successor_rc, successor_right);

                successor_rc.borrow_mut().right = Some(right_child.clone());
                right_child.borrow_mut().parent = Some(Rc::downgrade(&successor_rc));
            }

            self.transplant(&target_rc, Some(successor_rc.clone()));

            let left_child = target_rc.borrow_mut().left.take();
            successor_rc.borrow_mut().left = left_child.clone();
            if let Some(left) = left_child {
                left.borrow_mut().parent = Some(Rc::downgrade(&successor_rc));
            }
        } else {
            let child_rc = if has_left {
                target_rc.borrow_mut().left.take()
            } else {
                target_rc.borrow_mut().right.take()
            };

            self.transplant(&target_rc, child_rc);
        }

        Some(
            Rc::try_unwrap(target_rc)
                .unwrap_or_else(|_| {
                    unreachable!("Deleted node should only have one strong reference")
                })
                .into_inner()
                .value,
        )
    }

    pub fn delete_value(&mut self, value: &T) -> Option<T> {
        self.search(value).and_then(|h| self.delete(h))
    }
}

impl<T> Default for BinarySearchTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let bst = BinarySearchTree::<i32>::new();
        assert!(bst.min().is_none());
        assert!(bst.max().is_none());
        assert!(!bst.contains(&5));
    }

    #[test]
    fn test_insert_and_contains() {
        let mut bst = BinarySearchTree::new();
        bst.insert(5);
        bst.insert(3);
        bst.insert(7);

        assert!(bst.contains(&5));
        assert!(bst.contains(&3));
        assert!(bst.contains(&7));
        assert!(!bst.contains(&4));
    }

    #[test]
    fn test_min_max() {
        let mut bst = BinarySearchTree::new();
        bst.insert(5);
        bst.insert(3);
        bst.insert(7);
        bst.insert(2);
        bst.insert(4);
        bst.insert(6);
        bst.insert(8);

        assert_eq!(*bst.min().unwrap().value(), 2);
        assert_eq!(*bst.max().unwrap().value(), 8);
    }

    #[test]
    fn test_predecessor_successor() {
        let mut bst = BinarySearchTree::new();
        //       5
        //     /   \
        //    3     7
        //   / \   / \
        //  2   4 6   8
        bst.insert(5);
        bst.insert(3);
        bst.insert(7);
        bst.insert(2);
        bst.insert(4);
        bst.insert(6);
        bst.insert(8);

        assert_eq!(*bst.successor_of_value(&2).unwrap().value(), 3);
        assert_eq!(*bst.successor_of_value(&3).unwrap().value(), 4);
        assert_eq!(*bst.successor_of_value(&4).unwrap().value(), 5);
        assert_eq!(*bst.successor_of_value(&5).unwrap().value(), 6);
        assert_eq!(*bst.successor_of_value(&7).unwrap().value(), 8);
        assert!(bst.successor_of_value(&8).is_none());

        assert_eq!(*bst.predecessor_of_value(&8).unwrap().value(), 7);
        assert_eq!(*bst.predecessor_of_value(&7).unwrap().value(), 6);
        assert_eq!(*bst.predecessor_of_value(&6).unwrap().value(), 5);
        assert_eq!(*bst.predecessor_of_value(&5).unwrap().value(), 4);
        assert_eq!(*bst.predecessor_of_value(&3).unwrap().value(), 2);
        assert!(bst.predecessor_of_value(&2).is_none());
    }

    #[test]
    fn test_delete_leaf() {
        let mut bst = BinarySearchTree::new();
        bst.insert(5);
        bst.insert(3);

        let deleted = bst.delete_value(&3);
        assert_eq!(deleted, Some(3));
        assert!(!bst.contains(&3));
        assert!(bst.contains(&5));
    }

    #[test]
    fn test_delete_node_with_one_child() {
        let mut bst = BinarySearchTree::new();
        bst.insert(5);
        bst.insert(3);
        bst.insert(2);

        let deleted = bst.delete_value(&3);
        assert_eq!(deleted, Some(3));
        assert!(!bst.contains(&3));
        assert!(bst.contains(&2));
        assert_eq!(*bst.min().unwrap().value(), 2);

        assert_eq!(*bst.successor_of_value(&2).unwrap().value(), 5);
        assert_eq!(*bst.predecessor_of_value(&5).unwrap().value(), 2);
    }

    #[test]
    fn test_delete_node_with_two_children() {
        let mut bst = BinarySearchTree::new();
        //       5
        //     /   \
        //    3     7
        //   / \
        //  2   4
        bst.insert(5);
        bst.insert(3);
        bst.insert(7);
        bst.insert(2);
        bst.insert(4);

        let deleted = bst.delete_value(&3);
        assert_eq!(deleted, Some(3));
        assert!(!bst.contains(&3));
        assert!(bst.contains(&4));
        assert!(bst.contains(&2));

        assert_eq!(*bst.successor_of_value(&2).unwrap().value(), 4);
        assert_eq!(*bst.predecessor_of_value(&5).unwrap().value(), 4);
    }

    #[test]
    fn test_delete_root() {
        let mut bst = BinarySearchTree::new();
        bst.insert(5);
        bst.insert(3);
        bst.insert(7);

        let deleted = bst.delete_value(&5);
        assert_eq!(deleted, Some(5));
        assert!(!bst.contains(&5));
        assert!(bst.contains(&3));
        assert!(bst.contains(&7));

        assert_eq!(*bst.successor_of_value(&3).unwrap().value(), 7);
        assert!(bst.predecessor_of_value(&3).is_none());
    }

    #[test]
    fn test_delete_non_existent() {
        let mut bst = BinarySearchTree::new();
        bst.insert(5);
        assert_eq!(bst.delete_value(&10), None);
    }

    #[test]
    fn test_complex_delete() {
        let mut bst = BinarySearchTree::new();
        // Structure:
        //        10
        //      /    \
        //     5      15
        //   /  \    /  \
        //  2    7  12  20
        //      / \
        //     6   8
        let values = [10, 5, 15, 2, 7, 12, 20, 6, 8];
        for &v in &values {
            bst.insert(v);
        }

        let deleted = bst.delete_value(&5);
        assert_eq!(deleted, Some(5));

        assert_eq!(*bst.successor_of_value(&2).unwrap().value(), 6);
        assert_eq!(*bst.predecessor_of_value(&7).unwrap().value(), 6);
        assert_eq!(*bst.successor_of_value(&8).unwrap().value(), 10);
    }
}
