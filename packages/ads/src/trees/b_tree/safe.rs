use std::{
    cell::{Ref, RefCell},
    cmp::Ordering,
    rc::{Rc, Weak},
};

use crate::traits::{
    core::{Map, OrderedMap},
    diagnostics::TreeDiagnostics,
};

#[derive(Debug)]
struct BTreeNode<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
    children: Vec<Rc<RefCell<BTreeNode<K, V>>>>,
    parent: Option<Weak<RefCell<BTreeNode<K, V>>>>,
    is_leaf: bool,
}

impl<K, V> BTreeNode<K, V> {
    fn new(is_leaf: bool) -> Self {
        Self {
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
            parent: None,
            is_leaf,
        }
    }
}

#[derive(Debug)]
pub struct BTreeNodeView<K, V> {
    node: Rc<RefCell<BTreeNode<K, V>>>,
}

impl<K, V> Clone for BTreeNodeView<K, V> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl<K: Clone, V: Clone> BTreeNodeView<K, V> {
    pub fn keys(&self) -> Vec<K> {
        self.node.borrow().keys.clone()
    }

    pub fn entries(&self) -> Vec<(K, V)> {
        let node = self.node.borrow();
        node.keys
            .iter()
            .cloned()
            .zip(node.values.iter().cloned())
            .collect()
    }

    pub fn key_count(&self) -> usize {
        self.node.borrow().keys.len()
    }

    pub fn is_leaf(&self) -> bool {
        self.node.borrow().is_leaf
    }

    pub fn children(&self) -> Vec<BTreeNodeView<K, V>> {
        self.node
            .borrow()
            .children
            .iter()
            .map(|child| BTreeNodeView {
                node: child.clone(),
            })
            .collect()
    }

    pub fn parent(&self) -> Option<BTreeNodeView<K, V>> {
        self.node
            .borrow()
            .parent
            .as_ref()
            .and_then(|parent| parent.upgrade())
            .map(|parent| BTreeNodeView { node: parent })
    }
}

#[derive(Debug)]
pub struct BTree<K, V, const T: usize> {
    root: Option<Rc<RefCell<BTreeNode<K, V>>>>,
    len: usize,
}

#[derive(Debug)]
pub struct BTreeCursor<'a, K, V, const T: usize> {
    tree: &'a BTree<K, V, T>,
    node: Rc<RefCell<BTreeNode<K, V>>>,
    index: usize,
}

#[derive(Debug)]
pub struct BTreeIter<'a, K, V, const T: usize> {
    next: Option<BTreeCursor<'a, K, V, T>>,
}

impl<'a, K, V, const T: usize> Clone for BTreeCursor<'a, K, V, T> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            node: self.node.clone(),
            index: self.index,
        }
    }
}

impl<'a, K: Ord, V, const T: usize> BTreeCursor<'a, K, V, T> {
    pub fn key(&self) -> Ref<'_, K> {
        Ref::map(self.node.borrow(), |node| &node.keys[self.index])
    }

    pub fn value(&self) -> Ref<'_, V> {
        Ref::map(self.node.borrow(), |node| &node.values[self.index])
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn is_leaf(&self) -> bool {
        self.node.borrow().is_leaf
    }

    pub fn node_view(&self) -> BTreeNodeView<K, V> {
        BTreeNodeView {
            node: self.node.clone(),
        }
    }

    pub fn predecessor(&self) -> Option<Self> {
        let (node, index) = self.tree.predecessor_location(&self.node, self.index)?;
        Some(Self {
            tree: self.tree,
            node,
            index,
        })
    }

    pub fn successor(&self) -> Option<Self> {
        let (node, index) = self.tree.successor_location(&self.node, self.index)?;
        Some(Self {
            tree: self.tree,
            node,
            index,
        })
    }
}

impl<K: Ord, V, const T: usize> BTree<K, V, T> {
    pub fn new() -> Self {
        assert!(T >= 2, "minimum degree T must be >= 2");
        Self { root: None, len: 0 }
    }

    pub fn root_view(&self) -> Option<BTreeNodeView<K, V>> {
        self.root
            .as_ref()
            .map(|root| BTreeNodeView { node: root.clone() })
    }

    pub fn degree(&self) -> usize {
        T
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

    fn search_node(&self, key: &K) -> Option<(Rc<RefCell<BTreeNode<K, V>>>, usize)> {
        let mut current = self.root.clone()?;

        loop {
            let idx = {
                let current_borrow = current.borrow();
                match current_borrow.keys.binary_search(key) {
                    Ok(i) => return Some((current.clone(), i)),
                    Err(i) => i,
                }
            };

            if current.borrow().is_leaf {
                return None;
            }

            let next = current.borrow().children[idx].clone();
            current = next;
        }
    }

    pub fn cursor<'a>(&'a self, key: &K) -> Option<BTreeCursor<'a, K, V, T>> {
        self.search_node(key).map(|(node, index)| BTreeCursor {
            tree: self,
            node,
            index,
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.search_node(key).is_some()
    }

    pub fn min_cursor<'a>(&'a self) -> Option<BTreeCursor<'a, K, V, T>> {
        let mut current = self.root.clone()?;

        loop {
            if current.borrow().is_leaf {
                if current.borrow().keys.is_empty() {
                    return None;
                }
                return Some(BTreeCursor {
                    tree: self,
                    node: current,
                    index: 0,
                });
            }

            let next = current.borrow().children[0].clone();
            current = next;
        }
    }

    pub fn max_cursor<'a>(&'a self) -> Option<BTreeCursor<'a, K, V, T>> {
        let mut current = self.root.clone()?;

        loop {
            if current.borrow().is_leaf {
                let len = current.borrow().keys.len();
                if len == 0 {
                    return None;
                }
                return Some(BTreeCursor {
                    tree: self,
                    node: current,
                    index: len - 1,
                });
            }

            let last_index = current.borrow().children.len().saturating_sub(1);
            let next = current.borrow().children[last_index].clone();
            current = next;
        }
    }

    fn predecessor_location(
        &self,
        node: &Rc<RefCell<BTreeNode<K, V>>>,
        index: usize,
    ) -> Option<(Rc<RefCell<BTreeNode<K, V>>>, usize)> {
        let node = node.clone();

        if !node.borrow().is_leaf {
            let mut current = node.borrow().children[index].clone();
            loop {
                if current.borrow().is_leaf {
                    let len = current.borrow().keys.len();
                    return Some((current, len.saturating_sub(1)));
                }

                let last = current.borrow().children.len().saturating_sub(1);
                let next = current.borrow().children[last].clone();
                current = next;
            }
        }

        if index > 0 {
            return Some((node, index - 1));
        }

        let mut current = node;
        loop {
            let parent = current.borrow().parent.clone()?.upgrade()?;
            let child_idx = parent
                .borrow()
                .children
                .iter()
                .position(|child| Rc::ptr_eq(child, &current))?;

            if child_idx > 0 {
                return Some((parent, child_idx - 1));
            }

            current = parent;
        }
    }

    fn successor_location(
        &self,
        node: &Rc<RefCell<BTreeNode<K, V>>>,
        index: usize,
    ) -> Option<(Rc<RefCell<BTreeNode<K, V>>>, usize)> {
        let node = node.clone();

        if !node.borrow().is_leaf {
            let mut current = node.borrow().children[index + 1].clone();
            loop {
                if current.borrow().is_leaf {
                    return Some((current, 0));
                }

                let next = current.borrow().children[0].clone();
                current = next;
            }
        }

        if index + 1 < node.borrow().keys.len() {
            return Some((node, index + 1));
        }

        let mut current = node;
        loop {
            let parent = current.borrow().parent.clone()?.upgrade()?;
            let child_idx = parent
                .borrow()
                .children
                .iter()
                .position(|child| Rc::ptr_eq(child, &current))?;

            if child_idx < parent.borrow().keys.len() {
                return Some((parent, child_idx));
            }

            current = parent;
        }
    }

    pub fn insert_entry(&mut self, key: K, value: V) -> Option<V> {
        let root = match self.root.clone() {
            Some(root) => root,
            None => {
                let mut root = BTreeNode::new(true);
                root.keys.push(key);
                root.values.push(value);
                self.root = Some(Rc::new(RefCell::new(root)));
                self.len += 1;
                return None;
            }
        };

        let replaced = if root.borrow().keys.len() == 2 * T - 1 {
            let new_root = Rc::new(RefCell::new(BTreeNode::new(false)));
            new_root.borrow_mut().children.push(root.clone());
            root.borrow_mut().parent = Some(Rc::downgrade(&new_root));
            self.root = Some(new_root.clone());

            self.split_child(new_root.clone(), 0, root);
            self.insert_non_full(new_root, key, value)
        } else {
            self.insert_non_full(root, key, value)
        };

        if replaced.is_none() {
            self.len += 1;
        }

        replaced
    }

    fn split_child(
        &mut self,
        parent: Rc<RefCell<BTreeNode<K, V>>>,
        index: usize,
        child: Rc<RefCell<BTreeNode<K, V>>>,
    ) {
        let mut new_right = BTreeNode::new(child.borrow().is_leaf);
        new_right.parent = Some(Rc::downgrade(&parent));

        let mut child_mut = child.borrow_mut();

        let mut keys_to_move = child_mut.keys.split_off(T);
        let mut values_to_move = child_mut.values.split_off(T);
        let median = child_mut.keys.pop().expect("median key exists");
        let median_value = child_mut.values.pop().expect("median value exists");
        new_right.keys.append(&mut keys_to_move);
        new_right.values.append(&mut values_to_move);

        if !child_mut.is_leaf {
            let mut children_to_move = child_mut.children.split_off(T);
            new_right.children.append(&mut children_to_move);
        }

        drop(child_mut);

        let new_right_rc = Rc::new(RefCell::new(new_right));
        if !new_right_rc.borrow().is_leaf {
            for child in &new_right_rc.borrow().children {
                child.borrow_mut().parent = Some(Rc::downgrade(&new_right_rc));
            }
        }

        let mut parent_mut = parent.borrow_mut();
        parent_mut.children.insert(index + 1, new_right_rc);
        parent_mut.keys.insert(index, median);
        parent_mut.values.insert(index, median_value);
    }

    fn insert_non_full(
        &mut self,
        mut node: Rc<RefCell<BTreeNode<K, V>>>,
        key: K,
        value: V,
    ) -> Option<V> {
        let value = value;
        loop {
            if node.borrow().is_leaf {
                let mut node_mut = node.borrow_mut();
                let index = match node_mut.keys.binary_search(&key) {
                    Ok(i) => {
                        return Some(std::mem::replace(&mut node_mut.values[i], value));
                    }
                    Err(i) => i,
                };
                node_mut.keys.insert(index, key);
                node_mut.values.insert(index, value);
                return None;
            }

            let mut child_index = {
                let mut node_borrow = node.borrow_mut();
                match node_borrow.keys.binary_search(&key) {
                    Ok(i) => {
                        return Some(std::mem::replace(&mut node_borrow.values[i], value));
                    }
                    Err(i) => i,
                }
            };

            let child = node.borrow().children[child_index].clone();
            let child_is_full = child.borrow().keys.len() == 2 * T - 1;

            if child_is_full {
                self.split_child(node.clone(), child_index, child);
                let split_cmp = {
                    let node_borrow = node.borrow();
                    if key > node_borrow.keys[child_index] {
                        Ordering::Greater
                    } else if key == node_borrow.keys[child_index] {
                        Ordering::Equal
                    } else {
                        Ordering::Less
                    }
                };

                match split_cmp {
                    Ordering::Greater => child_index += 1,
                    Ordering::Equal => {
                        return Some(std::mem::replace(
                            &mut node.borrow_mut().values[child_index],
                            value,
                        ));
                    }
                    Ordering::Less => {}
                }
            }

            let next = node.borrow().children[child_index].clone();
            node = next;
        }
    }

    pub fn remove_key(&mut self, key: &K) -> Option<V> {
        let (target_node, target_index) = self.search_node(key)?;

        let deleted_value;
        let leaf_to_fix;

        if target_node.borrow().is_leaf {
            {
                let mut target_mut = target_node.borrow_mut();
                target_mut.keys.remove(target_index);
                deleted_value = target_mut.values.remove(target_index);
            }
            leaf_to_fix = target_node;
        } else {
            let left_child = target_node.borrow().children[target_index].clone();
            let mut current = left_child;

            loop {
                if current.borrow().is_leaf {
                    break;
                }
                let last = current.borrow().children.len().saturating_sub(1);
                let next = current.borrow().children[last].clone();
                current = next;
            }

            let (predecessor_key, predecessor_value) = {
                let mut predecessor_mut = current.borrow_mut();
                let predecessor_key = predecessor_mut.keys.pop().expect("predecessor key exists");
                let predecessor_value = predecessor_mut
                    .values
                    .pop()
                    .expect("predecessor value exists");
                (predecessor_key, predecessor_value)
            };

            {
                let mut target_mut = target_node.borrow_mut();
                target_mut.keys[target_index] = predecessor_key;
                deleted_value =
                    std::mem::replace(&mut target_mut.values[target_index], predecessor_value);
            }
            leaf_to_fix = current;
        }

        self.fix_underflow(leaf_to_fix);
        self.len = self.len.saturating_sub(1);
        Some(deleted_value)
    }

    fn fix_underflow(&mut self, mut node: Rc<RefCell<BTreeNode<K, V>>>) {
        loop {
            if node.borrow().parent.is_none() {
                let mut node_mut = node.borrow_mut();
                if node_mut.keys.is_empty() {
                    if node_mut.is_leaf {
                        self.root = None;
                    } else {
                        let new_root = node_mut.children.remove(0);
                        new_root.borrow_mut().parent = None;
                        self.root = Some(new_root);
                    }
                }
                return;
            }

            if node.borrow().keys.len() >= T - 1 {
                return;
            }

            let parent = node
                .borrow()
                .parent
                .clone()
                .and_then(|p| p.upgrade())
                .expect("parent exists");

            let index = parent
                .borrow()
                .children
                .iter()
                .position(|child| Rc::ptr_eq(child, &node))
                .expect("node must exist in parent children");

            let has_left = index > 0;
            let has_right = index + 1 < parent.borrow().children.len();
            let mut borrowed = false;

            if has_left {
                let left_sibling = parent.borrow().children[index - 1].clone();
                if left_sibling.borrow().keys.len() >= T {
                    let mut parent_mut = parent.borrow_mut();
                    let mut left_mut = left_sibling.borrow_mut();
                    let mut node_mut = node.borrow_mut();

                    let separator_key = std::mem::replace(
                        &mut parent_mut.keys[index - 1],
                        left_mut.keys.pop().expect("left key"),
                    );
                    let separator_value = std::mem::replace(
                        &mut parent_mut.values[index - 1],
                        left_mut.values.pop().expect("left value"),
                    );
                    node_mut.keys.insert(0, separator_key);
                    node_mut.values.insert(0, separator_value);

                    if !node_mut.is_leaf {
                        let left_child = left_mut.children.pop().expect("left child");
                        left_child.borrow_mut().parent = Some(Rc::downgrade(&node));
                        node_mut.children.insert(0, left_child);
                    }

                    borrowed = true;
                }
            }

            if !borrowed && has_right {
                let right_sibling = parent.borrow().children[index + 1].clone();
                if right_sibling.borrow().keys.len() >= T {
                    let mut parent_mut = parent.borrow_mut();
                    let mut right_mut = right_sibling.borrow_mut();
                    let mut node_mut = node.borrow_mut();

                    let separator_key =
                        std::mem::replace(&mut parent_mut.keys[index], right_mut.keys.remove(0));
                    let separator_value = std::mem::replace(
                        &mut parent_mut.values[index],
                        right_mut.values.remove(0),
                    );
                    node_mut.keys.push(separator_key);
                    node_mut.values.push(separator_value);

                    if !node_mut.is_leaf {
                        let right_child = right_mut.children.remove(0);
                        right_child.borrow_mut().parent = Some(Rc::downgrade(&node));
                        node_mut.children.push(right_child);
                    }

                    borrowed = true;
                }
            }

            if borrowed {
                return;
            }

            if has_left {
                let left_sibling = parent.borrow().children[index - 1].clone();
                let mut parent_mut = parent.borrow_mut();
                let separator_key = parent_mut.keys.remove(index - 1);
                let separator_value = parent_mut.values.remove(index - 1);
                parent_mut.children.remove(index);

                let mut left_mut = left_sibling.borrow_mut();
                let mut node_mut = node.borrow_mut();

                left_mut.keys.push(separator_key);
                left_mut.values.push(separator_value);
                left_mut.keys.append(&mut node_mut.keys);
                left_mut.values.append(&mut node_mut.values);

                if !node_mut.is_leaf {
                    for child in &node_mut.children {
                        child.borrow_mut().parent = Some(Rc::downgrade(&left_sibling));
                    }
                    left_mut.children.append(&mut node_mut.children);
                }

                drop(node_mut);
                drop(left_mut);
                drop(parent_mut);

                node = parent;
            } else if has_right {
                let right_sibling = parent.borrow().children[index + 1].clone();
                let mut parent_mut = parent.borrow_mut();
                let separator_key = parent_mut.keys.remove(index);
                let separator_value = parent_mut.values.remove(index);
                parent_mut.children.remove(index + 1);

                let mut right_mut = right_sibling.borrow_mut();
                let mut node_mut = node.borrow_mut();

                node_mut.keys.push(separator_key);
                node_mut.values.push(separator_value);
                node_mut.keys.append(&mut right_mut.keys);
                node_mut.values.append(&mut right_mut.values);

                if !node_mut.is_leaf {
                    for child in &right_mut.children {
                        child.borrow_mut().parent = Some(Rc::downgrade(&node));
                    }
                    node_mut.children.append(&mut right_mut.children);
                }

                drop(node_mut);
                drop(right_mut);
                drop(parent_mut);

                node = parent;
            } else {
                unreachable!("non-root node must have at least one sibling");
            }
        }
    }

    pub fn iter<'a>(&'a self) -> BTreeIter<'a, K, V, T> {
        BTreeIter {
            next: self.min_cursor(),
        }
    }

    fn node_count_recursive(node: &Option<Rc<RefCell<BTreeNode<K, V>>>>) -> usize {
        match node {
            None => 0,
            Some(rc) => {
                let children = rc.borrow().children.clone();
                1 + children
                    .iter()
                    .map(|child| Self::node_count_recursive(&Some(child.clone())))
                    .sum::<usize>()
            }
        }
    }

    fn height_recursive(node: &Option<Rc<RefCell<BTreeNode<K, V>>>>) -> usize {
        let Some(rc) = node else {
            return 0;
        };

        if rc.borrow().is_leaf {
            return 1;
        }

        1 + rc
            .borrow()
            .children
            .iter()
            .map(|child| Self::height_recursive(&Some(child.clone())))
            .max()
            .unwrap_or(0)
    }

}

impl<K: Ord, V, const T: usize> Map<K, V> for BTree<K, V, T> {
    type Cursor<'a>
        = BTreeCursor<'a, K, V, T>
    where
        Self: 'a;

    type View<'a>
        = BTreeNodeView<K, V>
    where
        Self: 'a;

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        BTree::insert_entry(self, key, value)
    }

    fn cursor<'a>(&'a self, key: &K) -> Option<Self::Cursor<'a>> {
        BTree::cursor(self, key)
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        cursor.node_view()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        BTree::remove_key(self, key)
    }

    fn contains_key(&self, key: &K) -> bool {
        BTree::contains_key(self, key)
    }

    fn clear(&mut self) {
        BTree::clear(self)
    }

    fn len(&self) -> usize {
        BTree::len(self)
    }
}

impl<K: Ord, V, const T: usize> OrderedMap<K, V> for BTree<K, V, T> {
    fn first_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.min_cursor()
    }

    fn last_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.max_cursor()
    }
}

impl<K: Ord, V, const T: usize> TreeDiagnostics for BTree<K, V, T> {
    type NodeCursor<'a>
        = BTreeCursor<'a, K, V, T>
    where
        Self: 'a;

    fn height(&self) -> usize {
        Self::height_recursive(&self.root)
    }

    fn node_count(&self) -> usize {
        Self::node_count_recursive(&self.root)
    }

    fn node_height<'a>(&'a self, cursor: &Self::NodeCursor<'a>) -> usize {
        Self::height_recursive(&Some(cursor.node.clone()))
    }
}

impl<K: Ord, V, const T: usize> Default for BTree<K, V, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V, const T: usize> FromIterator<(K, V)> for BTree<K, V, T> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut tree = Self::new();
        for (key, value) in iter {
            tree.insert_entry(key, value);
        }
        tree
    }
}

impl<'a, K: Ord + Clone, V: Clone, const T: usize> Iterator for BTreeIter<'a, K, V, T> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next.take()?;
        self.next = current.successor();

        let (key, value) = {
            let node = current.node.borrow();
            (
                node.keys[current.index].clone(),
                node.values[current.index].clone(),
            )
        };

        Some((key, value))
    }
}

impl<'a, K: Ord + Clone, V: Clone, const T: usize> IntoIterator for &'a BTree<K, V, T> {
    type Item = (K, V);
    type IntoIter = BTreeIter<'a, K, V, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
