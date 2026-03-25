use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

#[derive(Debug)]
struct BTreeNode<T> {
    keys: Vec<T>,
    children: Vec<Rc<RefCell<BTreeNode<T>>>>,
    parent: Option<Weak<RefCell<BTreeNode<T>>>>,
    is_leaf: bool,
}

impl<T> BTreeNode<T> {
    pub fn new(is_leaf: bool) -> Self {
        Self {
            keys: Vec::new(),
            children: Vec::new(),
            parent: None,
            is_leaf,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BTreeNodeView<T> {
    node: Rc<RefCell<BTreeNode<T>>>,
}

impl<T: Clone> BTreeNodeView<T> {
    pub fn keys(&self) -> Vec<T> {
        self.node.borrow().keys.clone()
    }

    pub fn is_leaf(&self) -> bool {
        self.node.borrow().is_leaf
    }

    pub fn children(&self) -> Vec<BTreeNodeView<T>> {
        self.node
            .borrow()
            .children
            .iter()
            .map(|c| BTreeNodeView { node: c.clone() })
            .collect()
    }
}

#[derive(Debug)]
pub struct BTreeCursor<T> {
    node: Rc<RefCell<BTreeNode<T>>>,
    index: usize,
}

impl<T> Clone for BTreeCursor<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            index: self.index,
        }
    }
}

impl<T> BTreeCursor<T> {
    pub fn value(&self) -> Ref<T> {
        Ref::map(self.node.borrow(), |node| &node.keys[self.index])
    }

    pub fn is_leaf(&self) -> bool {
        self.node.borrow().is_leaf
    }
}

#[derive(Debug)]
pub struct BTree<T> {
    root: Option<Rc<RefCell<BTreeNode<T>>>>,
    t: usize,
}

impl<T: Ord> BTree<T> {
    pub fn new(t: usize) -> Self {
        assert!(t >= 2, "Minimum degree t must be >= 2");
        Self { root: None, t }
    }

    pub fn root_view(&self) -> Option<BTreeNodeView<T>> {
        self.root
            .as_ref()
            .map(|r| BTreeNodeView { node: r.clone() })
    }

    pub fn degree(&self) -> usize {
        self.t
    }

    fn search_node(&self, value: &T) -> Option<(Rc<RefCell<BTreeNode<T>>>, usize)> {
        let mut current = self.root.clone()?;

        loop {
            let idx = {
                let current_borrow = current.borrow();
                match current_borrow.keys.binary_search(value) {
                    Ok(i) => return Some((current.clone(), i)),
                    Err(i) => i,
                }
            };

            let is_leaf = current.borrow().is_leaf;
            if is_leaf {
                return None;
            } else {
                let next = current.borrow().children[idx].clone();
                current = next;
            }
        }
    }

    pub fn search(&self, value: &T) -> Option<BTreeCursor<T>> {
        self.search_node(value)
            .map(|(node, index)| BTreeCursor { node, index })
    }

    pub fn contains(&self, value: &T) -> bool {
        self.search_node(value).is_some()
    }

    pub fn min(&self) -> Option<BTreeCursor<T>> {
        let mut current = self.root.clone()?;
        loop {
            let is_leaf = current.borrow().is_leaf;
            if is_leaf {
                if current.borrow().keys.is_empty() {
                    return None;
                }
                return Some(BTreeCursor {
                    node: current,
                    index: 0,
                });
            }
            let next = current.borrow().children[0].clone();
            current = next;
        }
    }

    pub fn max(&self) -> Option<BTreeCursor<T>> {
        let mut current = self.root.clone()?;
        loop {
            let is_leaf = current.borrow().is_leaf;
            if is_leaf {
                let len = current.borrow().keys.len();
                if len == 0 {
                    return None;
                }
                return Some(BTreeCursor {
                    node: current,
                    index: len - 1,
                });
            }
            let last_idx = current.borrow().children.len() - 1;
            let next = current.borrow().children[last_idx].clone();
            current = next;
        }
    }

    pub fn predecessor(&self, handle: &BTreeCursor<T>) -> Option<BTreeCursor<T>> {
        let node = handle.node.clone();
        let is_leaf = node.borrow().is_leaf;

        if !is_leaf {
            let mut current = node.borrow().children[handle.index].clone();
            loop {
                let curr_is_leaf = current.borrow().is_leaf;
                if curr_is_leaf {
                    let len = current.borrow().keys.len();
                    return Some(BTreeCursor {
                        node: current.clone(),
                        index: len - 1,
                    });
                }
                let last_idx = current.borrow().children.len() - 1;
                let next = current.borrow().children[last_idx].clone();
                current = next;
            }
        } else if handle.index > 0 {
            return Some(BTreeCursor {
                node,
                index: handle.index - 1,
            });
        } else {
            let mut current = node;
            loop {
                let parent_weak = current.borrow().parent.clone()?;
                let parent = parent_weak.upgrade()?;
                let child_idx = parent
                    .borrow()
                    .children
                    .iter()
                    .position(|c| Rc::ptr_eq(c, &current))?;

                if child_idx > 0 {
                    return Some(BTreeCursor {
                        node: parent,
                        index: child_idx - 1,
                    });
                }
                current = parent;
            }
        }
    }

    pub fn successor(&self, handle: &BTreeCursor<T>) -> Option<BTreeCursor<T>> {
        let node = handle.node.clone();
        let is_leaf = node.borrow().is_leaf;

        if !is_leaf {
            let mut current = node.borrow().children[handle.index + 1].clone();
            loop {
                let curr_is_leaf = current.borrow().is_leaf;
                if curr_is_leaf {
                    return Some(BTreeCursor {
                        node: current.clone(),
                        index: 0,
                    });
                }
                let next = current.borrow().children[0].clone();
                current = next;
            }
        } else {
            let keys_len = node.borrow().keys.len();
            if handle.index + 1 < keys_len {
                Some(BTreeCursor {
                    node,
                    index: handle.index + 1,
                })
            } else {
                let mut current = node;
                loop {
                    let parent_weak = current.borrow().parent.clone()?;
                    let parent = parent_weak.upgrade()?;
                    let child_idx = parent
                        .borrow()
                        .children
                        .iter()
                        .position(|c| Rc::ptr_eq(c, &current))?;

                    if child_idx < parent.borrow().keys.len() {
                        return Some(BTreeCursor {
                            node: parent,
                            index: child_idx,
                        });
                    }
                    current = parent;
                }
            }
        }
    }

    pub fn predecessor_of_value(&self, value: &T) -> Option<BTreeCursor<T>> {
        self.search(value).and_then(|h| self.predecessor(&h))
    }

    pub fn successor_of_value(&self, value: &T) -> Option<BTreeCursor<T>> {
        self.search(value).and_then(|h| self.successor(&h))
    }

    pub fn insert(&mut self, value: T) {
        let root_rc = match self.root.clone() {
            Some(r) => r,
            None => {
                let mut root = BTreeNode::new(true);
                root.keys.push(value);
                self.root = Some(Rc::new(RefCell::new(root)));
                return;
            }
        };

        let is_full = root_rc.borrow().keys.len() == 2 * self.t - 1;
        if is_full {
            let new_root = BTreeNode::new(false);
            let new_root_rc = Rc::new(RefCell::new(new_root));
            new_root_rc.borrow_mut().children.push(root_rc.clone());
            root_rc.borrow_mut().parent = Some(Rc::downgrade(&new_root_rc));
            self.root = Some(new_root_rc.clone());

            self.split_child(new_root_rc.clone(), 0, root_rc);
            self.insert_non_full(new_root_rc, value);
        } else {
            self.insert_non_full(root_rc, value);
        }
    }

    fn split_child(
        &mut self,
        parent: Rc<RefCell<BTreeNode<T>>>,
        i: usize,
        child: Rc<RefCell<BTreeNode<T>>>,
    ) {
        let t = self.t;
        let mut z = BTreeNode::new(child.borrow().is_leaf);
        z.parent = Some(Rc::downgrade(&parent));

        let mut child_mut = child.borrow_mut();

        let mut keys_to_move = child_mut.keys.split_off(t);
        let median_key = child_mut.keys.pop().unwrap();

        z.keys.append(&mut keys_to_move);

        if !child_mut.is_leaf {
            let mut children_to_move = child_mut.children.split_off(t);
            z.children.append(&mut children_to_move);
        }
        drop(child_mut);

        let z_rc = Rc::new(RefCell::new(z));

        if !z_rc.borrow().is_leaf {
            for c in &z_rc.borrow().children {
                c.borrow_mut().parent = Some(Rc::downgrade(&z_rc));
            }
        }

        let mut parent_mut = parent.borrow_mut();
        parent_mut.children.insert(i + 1, z_rc);
        parent_mut.keys.insert(i, median_key);
    }

    fn insert_non_full(&mut self, mut x: Rc<RefCell<BTreeNode<T>>>, k: T) {
        loop {
            let is_leaf = x.borrow().is_leaf;
            if is_leaf {
                let mut x_mut = x.borrow_mut();
                let idx = match x_mut.keys.binary_search(&k) {
                    Ok(i) => i,
                    Err(i) => i,
                };
                x_mut.keys.insert(idx, k);
                return;
            } else {
                let mut idx = {
                    let x_borrow = x.borrow();
                    match x_borrow.keys.binary_search(&k) {
                        Ok(i) => i + 1, // Go to right child on duplicate to maintain stability
                        Err(i) => i,
                    }
                };

                let child = x.borrow().children[idx].clone();
                let is_full = child.borrow().keys.len() == 2 * self.t - 1;

                if is_full {
                    self.split_child(x.clone(), idx, child.clone());
                    let x_borrow = x.borrow();
                    if k > x_borrow.keys[idx] {
                        idx += 1;
                    }
                }

                let next_child = x.borrow().children[idx].clone();
                x = next_child;
            }
        }
    }

    pub fn delete_value(&mut self, value: &T) -> Option<T> {
        let (target_node, target_idx) = self.search_node(value)?;

        let is_leaf = target_node.borrow().is_leaf;
        let leaf_to_fix;
        let deleted_value;

        if is_leaf {
            deleted_value = target_node.borrow_mut().keys.remove(target_idx);
            leaf_to_fix = target_node.clone();
        } else {
            let left_child = target_node.borrow().children[target_idx].clone();
            let mut curr = left_child;
            loop {
                let is_leaf_curr = curr.borrow().is_leaf;
                if is_leaf_curr {
                    break;
                }
                let last_idx = curr.borrow().children.len() - 1;
                let next = curr.borrow().children[last_idx].clone();
                curr = next;
            }

            let pred_value = curr.borrow_mut().keys.pop().unwrap();
            deleted_value =
                std::mem::replace(&mut target_node.borrow_mut().keys[target_idx], pred_value);
            leaf_to_fix = curr;
        }

        self.fix_underflow(leaf_to_fix);
        Some(deleted_value)
    }

    fn fix_underflow(&mut self, mut node: Rc<RefCell<BTreeNode<T>>>) {
        loop {
            let is_root = node.borrow().parent.is_none();
            if is_root {
                let mut node_mut = node.borrow_mut();
                if node_mut.keys.is_empty() {
                    if !node_mut.is_leaf {
                        let new_root = node_mut.children.remove(0);
                        new_root.borrow_mut().parent = None;
                        self.root = Some(new_root);
                    } else {
                        self.root = None;
                    }
                }
                return;
            }

            let keys_len = node.borrow().keys.len();
            if keys_len >= self.t - 1 {
                return;
            }

            let parent_weak = node.borrow().parent.clone().unwrap();
            let parent = parent_weak.upgrade().expect("Parent should exist");

            let idx = parent
                .borrow()
                .children
                .iter()
                .position(|c| Rc::ptr_eq(c, &node))
                .unwrap();

            let has_left = idx > 0;
            let has_right = idx + 1 < parent.borrow().children.len();

            let mut borrowed = false;

            if has_left {
                let left_sibling = parent.borrow().children[idx - 1].clone();
                if left_sibling.borrow().keys.len() >= self.t {
                    let mut p_mut = parent.borrow_mut();
                    let mut ls_mut = left_sibling.borrow_mut();
                    let mut n_mut = node.borrow_mut();

                    let separator =
                        std::mem::replace(&mut p_mut.keys[idx - 1], ls_mut.keys.pop().unwrap());
                    n_mut.keys.insert(0, separator);

                    if !n_mut.is_leaf {
                        let ls_child = ls_mut.children.pop().unwrap();
                        ls_child.borrow_mut().parent = Some(Rc::downgrade(&node));
                        n_mut.children.insert(0, ls_child);
                    }
                    borrowed = true;
                }
            }

            if !borrowed && has_right {
                let right_sibling = parent.borrow().children[idx + 1].clone();
                if right_sibling.borrow().keys.len() >= self.t {
                    let mut p_mut = parent.borrow_mut();
                    let mut rs_mut = right_sibling.borrow_mut();
                    let mut n_mut = node.borrow_mut();

                    let separator = std::mem::replace(&mut p_mut.keys[idx], rs_mut.keys.remove(0));
                    n_mut.keys.push(separator);

                    if !n_mut.is_leaf {
                        let rs_child = rs_mut.children.remove(0);
                        rs_child.borrow_mut().parent = Some(Rc::downgrade(&node));
                        n_mut.children.push(rs_child);
                    }
                    borrowed = true;
                }
            }

            if borrowed {
                return;
            }

            if has_left {
                let left_sibling = parent.borrow().children[idx - 1].clone();
                let mut p_mut = parent.borrow_mut();
                let separator = p_mut.keys.remove(idx - 1);
                p_mut.children.remove(idx);

                let mut ls_mut = left_sibling.borrow_mut();
                let mut n_mut = node.borrow_mut();

                ls_mut.keys.push(separator);
                ls_mut.keys.append(&mut n_mut.keys);

                if !n_mut.is_leaf {
                    for child in &n_mut.children {
                        child.borrow_mut().parent = Some(Rc::downgrade(&left_sibling));
                    }
                    ls_mut.children.append(&mut n_mut.children);
                }

                drop(n_mut);
                drop(ls_mut);
                drop(p_mut);

                node = parent;
            } else if has_right {
                let right_sibling = parent.borrow().children[idx + 1].clone();
                let mut p_mut = parent.borrow_mut();
                let separator = p_mut.keys.remove(idx);
                p_mut.children.remove(idx + 1);

                let mut rs_mut = right_sibling.borrow_mut();
                let mut n_mut = node.borrow_mut();

                n_mut.keys.push(separator);
                n_mut.keys.append(&mut rs_mut.keys);

                if !n_mut.is_leaf {
                    for child in &rs_mut.children {
                        child.borrow_mut().parent = Some(Rc::downgrade(&node));
                    }
                    n_mut.children.append(&mut rs_mut.children);
                }

                drop(n_mut);
                drop(rs_mut);
                drop(p_mut);

                node = parent;
            } else {
                unreachable!("Node must have at least one sibling if it's not the root");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_btree_properties<T: Ord + std::fmt::Debug>(tree: &BTree<T>) {
        if let Some(root) = &tree.root {
            check_node(root.clone(), tree.t, true);
        }
    }

    fn check_node<T: Ord + std::fmt::Debug>(
        node: Rc<RefCell<BTreeNode<T>>>,
        t: usize,
        is_root: bool,
    ) {
        let b = node.borrow();

        let keys_len = b.keys.len();
        if !is_root {
            assert!(keys_len >= t - 1, "Node underflow");
        }
        assert!(keys_len <= 2 * t - 1, "Node overflow");

        for i in 0..keys_len.saturating_sub(1) {
            assert!(b.keys[i] <= b.keys[i + 1], "Keys not sorted");
        }

        if !b.is_leaf {
            assert_eq!(b.children.len(), keys_len + 1, "Invalid children count");
            for child in &b.children {
                let parent_ptr = child.borrow().parent.as_ref().unwrap().upgrade().unwrap();
                assert!(Rc::ptr_eq(&parent_ptr, &node), "Invalid parent pointer");
                check_node(child.clone(), t, false);
            }
        }
    }

    #[test]
    fn test_empty_tree() {
        let tree = BTree::<i32>::new(2);
        assert!(tree.min().is_none());
        assert!(tree.max().is_none());
        assert!(!tree.contains(&5));
        assert_btree_properties(&tree);
    }

    #[test]
    fn test_insert_and_contains() {
        let mut tree = BTree::new(2);
        tree.insert(10);
        tree.insert(20);
        tree.insert(5);
        tree.insert(6);
        tree.insert(12);

        assert!(tree.contains(&10));
        assert!(tree.contains(&20));
        assert!(tree.contains(&12));
        assert!(!tree.contains(&15));
        assert_btree_properties(&tree);
    }

    #[test]
    fn test_min_max() {
        let mut tree = BTree::new(3);
        let values = [5, 3, 7, 2, 4, 6, 8, 1, 9, 10, 15];
        for &v in &values {
            tree.insert(v);
        }

        assert_eq!(*tree.min().unwrap().value(), 1);
        assert_eq!(*tree.max().unwrap().value(), 15);
        assert_btree_properties(&tree);
    }

    #[test]
    fn test_predecessor_successor() {
        let mut tree = BTree::new(2);
        let values = [10, 20, 30, 40, 50, 60, 70, 80, 90];
        for &v in &values {
            tree.insert(v);
        }

        assert_eq!(*tree.successor_of_value(&30).unwrap().value(), 40);
        assert_eq!(*tree.successor_of_value(&40).unwrap().value(), 50);
        assert!(tree.successor_of_value(&90).is_none());

        assert_eq!(*tree.predecessor_of_value(&90).unwrap().value(), 80);
        assert_eq!(*tree.predecessor_of_value(&40).unwrap().value(), 30);
        assert!(tree.predecessor_of_value(&10).is_none());
    }

    #[test]
    fn test_delete_leaf() {
        let mut tree = BTree::new(2);
        tree.insert(10);
        tree.insert(20);
        tree.insert(30);

        let deleted = tree.delete_value(&20);
        assert_eq!(deleted, Some(20));
        assert!(!tree.contains(&20));
        assert!(tree.contains(&10));
        assert_btree_properties(&tree);
    }

    #[test]
    fn test_delete_internal_and_merge() {
        let mut tree = BTree::new(2);
        let values = [10, 20, 30, 40, 50, 60];
        for &v in &values {
            tree.insert(v);
        }

        let deleted = tree.delete_value(&40);
        assert_eq!(deleted, Some(40));
        assert!(!tree.contains(&40));
        assert_btree_properties(&tree);

        let deleted2 = tree.delete_value(&20);
        assert_eq!(deleted2, Some(20));
        assert_btree_properties(&tree);
    }

    #[test]
    fn test_complex_tree_delete() {
        let mut tree = BTree::new(3);
        for i in 1..=50 {
            tree.insert(i);
        }
        assert_btree_properties(&tree);

        for i in (1..=50).step_by(3) {
            assert_eq!(tree.delete_value(&i), Some(i));
            assert_btree_properties(&tree);
        }

        for i in 1..=50 {
            if i % 3 == 1 {
                assert!(!tree.contains(&i));
            } else {
                assert!(tree.contains(&i));
            }
        }
    }
}
