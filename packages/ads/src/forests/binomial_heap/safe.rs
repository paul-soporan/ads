use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

use crate::traits::{core::PriorityQueue, diagnostics::ForestDiagnostics};

#[derive(Debug)]
struct BinomialNode<T> {
    value: T,
    degree: usize,
    parent: Option<Weak<RefCell<BinomialNode<T>>>>,
    child: Option<Rc<RefCell<BinomialNode<T>>>>,
    sibling: Option<Rc<RefCell<BinomialNode<T>>>>,
}

impl<T> BinomialNode<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            degree: 0,
            parent: None,
            child: None,
            sibling: None,
        }
    }
}

#[derive(Debug)]
pub struct BinomialNodeView<T> {
    node: Rc<RefCell<BinomialNode<T>>>,
}

impl<T> From<Rc<RefCell<BinomialNode<T>>>> for BinomialNodeView<T> {
    fn from(node: Rc<RefCell<BinomialNode<T>>>) -> Self {
        Self { node }
    }
}

#[derive(Debug)]
pub struct BinomialNodeCursor<T> {
    node: Rc<RefCell<BinomialNode<T>>>,
}

impl<T> From<Rc<RefCell<BinomialNode<T>>>> for BinomialNodeCursor<T> {
    fn from(node: Rc<RefCell<BinomialNode<T>>>) -> Self {
        Self { node }
    }
}

impl<T> Clone for BinomialNodeView<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl<T> Clone for BinomialNodeCursor<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl<T> BinomialNodeView<T> {
    pub fn value(&self) -> Ref<'_, T> {
        Ref::map(self.node.borrow(), |node| &node.value)
    }

    pub fn degree(&self) -> usize {
        self.node.borrow().degree
    }

    pub fn child(&self) -> Option<Self> {
        self.node
            .borrow()
            .child
            .as_ref()
            .map(|child| Self::from(child.clone()))
    }

    pub fn sibling(&self) -> Option<Self> {
        self.node
            .borrow()
            .sibling
            .as_ref()
            .map(|sibling| Self::from(sibling.clone()))
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

impl<T> BinomialNodeCursor<T> {
    fn rc(&self) -> Rc<RefCell<BinomialNode<T>>> {
        self.node.clone()
    }

    pub fn value(&self) -> Ref<'_, T> {
        Ref::map(self.node.borrow(), |node| &node.value)
    }

    pub fn node_view(&self) -> BinomialNodeView<T> {
        BinomialNodeView::from(self.node.clone())
    }
}

#[derive(Debug)]
pub struct BinomialHeap<T> {
    head: Option<Rc<RefCell<BinomialNode<T>>>>,
    len: usize,
}

impl<T: Ord> BinomialHeap<T> {
    pub fn new() -> Self {
        Self { head: None, len: 0 }
    }

    pub fn head_view(&self) -> Option<BinomialNodeView<T>> {
        self.head.clone().map(BinomialNodeView::from)
    }

    pub fn roots(&self) -> Vec<BinomialNodeView<T>> {
        let mut roots = Vec::new();
        let mut current = self.head.clone();
        while let Some(node) = current {
            roots.push(BinomialNodeView::from(node.clone()));
            current = node.borrow().sibling.clone();
        }
        roots
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.head = None;
        self.len = 0;
    }

    pub fn search(&self, value: &T) -> Option<BinomialNodeCursor<T>> {
        let mut stack = Vec::new();
        let mut current = self.head.clone();

        while let Some(node) = current {
            stack.push(node.clone());
            current = node.borrow().sibling.clone();
        }

        while let Some(node) = stack.pop() {
            let node_borrow = node.borrow();
            if &node_borrow.value == value {
                return Some(BinomialNodeCursor::from(node.clone()));
            }

            if &node_borrow.value < value {
                let mut child = node_borrow.child.clone();
                while let Some(c) = child {
                    stack.push(c.clone());
                    child = c.borrow().sibling.clone();
                }
            }
        }

        None
    }

    pub fn min(&self) -> Option<BinomialNodeCursor<T>> {
        let mut min_node = None;
        let mut current = self.head.clone();

        while let Some(node) = current {
            let is_smaller = min_node
                .as_ref()
                .is_none_or(|min: &Rc<RefCell<BinomialNode<T>>>| {
                    node.borrow().value < min.borrow().value
                });

            if is_smaller {
                min_node = Some(node.clone());
            }

            current = node.borrow().sibling.clone();
        }

        min_node.map(BinomialNodeCursor::from)
    }

    pub fn insert(&mut self, value: T) {
        let mut singleton = Self {
            head: Some(Rc::new(RefCell::new(BinomialNode::new(value)))),
            len: 1,
        };
        self.merge(&mut singleton);
    }

    fn link(child_root: Rc<RefCell<BinomialNode<T>>>, parent_root: Rc<RefCell<BinomialNode<T>>>) {
        child_root.borrow_mut().parent = Some(Rc::downgrade(&parent_root));
        let parent_child = parent_root.borrow_mut().child.take();
        child_root.borrow_mut().sibling = parent_child;
        parent_root.borrow_mut().child = Some(child_root);
        parent_root.borrow_mut().degree += 1;
    }

    fn merge_root_lists(
        mut first: Option<Rc<RefCell<BinomialNode<T>>>>,
        mut second: Option<Rc<RefCell<BinomialNode<T>>>>,
    ) -> Option<Rc<RefCell<BinomialNode<T>>>> {
        let mut head = None;
        let mut tail: Option<Rc<RefCell<BinomialNode<T>>>> = None;

        while first.is_some() && second.is_some() {
            let n1 = first.as_ref().unwrap().clone();
            let n2 = second.as_ref().unwrap().clone();

            let take_first = n1.borrow().degree <= n2.borrow().degree;
            let next_node = if take_first {
                first = n1.borrow_mut().sibling.take();
                n1
            } else {
                second = n2.borrow_mut().sibling.take();
                n2
            };

            if let Some(tail_node) = &tail {
                tail_node.borrow_mut().sibling = Some(next_node.clone());
            } else {
                head = Some(next_node.clone());
            }
            tail = Some(next_node);
        }

        let remaining = if first.is_some() { first } else { second };
        if let Some(rem) = remaining {
            if let Some(tail_node) = &tail {
                tail_node.borrow_mut().sibling = Some(rem);
            } else {
                head = Some(rem);
            }
        }

        head
    }

    pub fn merge(&mut self, other: &mut Self) {
        let total_len = self.len + other.len;

        let first = self.head.take();
        let second = other.head.take();

        if first.is_none() {
            self.head = second;
            self.len = total_len;
            other.len = 0;
            return;
        }

        if second.is_none() {
            self.head = first;
            self.len = total_len;
            other.len = 0;
            return;
        }

        let mut real_head = Self::merge_root_lists(first, second);
        
        let mut prev = None;
        let mut x = real_head.as_ref().unwrap().clone();
        let mut next = x.borrow().sibling.clone();

        while let Some(n) = next {
            let n_next = n.borrow().sibling.clone();
            let x_degree = x.borrow().degree;
            let n_degree = n.borrow().degree;
            let n_next_degree = n_next.as_ref().map(|sibling| sibling.borrow().degree);

            if x_degree != n_degree || n_next_degree == Some(x_degree) {
                prev = Some(x.clone());
                x = n.clone();
            } else if x.borrow().value <= n.borrow().value {
                x.borrow_mut().sibling = n_next;
                Self::link(n.clone(), x.clone());
            } else {
                if let Some(prev_node) = &prev {
                    prev_node.borrow_mut().sibling = Some(n.clone());
                } else {
                    real_head = Some(n.clone());
                }
                Self::link(x.clone(), n.clone());
                x = n;
            }

            next = x.borrow().sibling.clone();
        }

        self.head = real_head;
        self.len = total_len;
        other.len = 0;
    }

    pub fn extract_min(&mut self) -> Option<T> {
        if self.head.is_none() {
            return None;
        }

        let mut min_node = self.head.as_ref().unwrap().clone();
        let mut min_prev = None;

        {
            let mut current = min_node.borrow().sibling.clone();
            let mut prev = Some(min_node.clone());

            while let Some(node) = current {
                if node.borrow().value < min_node.borrow().value {
                    min_node = node.clone();
                    min_prev = prev.clone();
                }
                prev = Some(node.clone());
                current = node.borrow().sibling.clone();
            }
        }

        if let Some(prev) = min_prev {
            prev.borrow_mut().sibling = min_node.borrow_mut().sibling.take();
        } else {
            self.head = min_node.borrow_mut().sibling.take();
        }

        let mut child = min_node.borrow_mut().child.take();
        let mut new_head = None;
        while let Some(c) = child {
            let next = c.borrow_mut().sibling.take();
            c.borrow_mut().parent = None;
            c.borrow_mut().sibling = new_head;
            new_head = Some(c.clone());
            child = next;
        }

        let mut child_heap = Self {
            head: new_head,
            len: 0,
        };
        let old_len = self.len;
        self.merge(&mut child_heap);
        self.len = old_len.saturating_sub(1);

        Rc::try_unwrap(min_node)
            .map_err(|_| "Rc::try_unwrap failed in extract_min")
            .ok()
            .map(|cell| cell.into_inner().value)
    }

    pub fn decrease_key(&mut self, handle: BinomialNodeCursor<T>, new_value: T) {
        if new_value > *handle.value() {
            panic!("decrease_key received a larger replacement value");
        }

        handle.node.borrow_mut().value = new_value;
        let mut current = handle.node;

        loop {
            let parent = {
                let b = current.borrow();
                b.parent.as_ref().and_then(|p| p.upgrade())
            };

            if let Some(parent_rc) = parent {
                if current.borrow().value < parent_rc.borrow().value {
                    let mut parent_mut = parent_rc.borrow_mut();
                    let mut current_mut = current.borrow_mut();
                    std::mem::swap(&mut parent_mut.value, &mut current_mut.value);
                    drop(parent_mut);
                    drop(current_mut);
                    current = parent_rc;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub fn delete(&mut self, handle: BinomialNodeCursor<T>) -> Option<T> {
        let mut current = handle.rc();
        drop(handle);

        loop {
            let parent = {
                let b = current.borrow();
                b.parent.as_ref().and_then(|p| p.upgrade())
            };

            if let Some(parent_rc) = parent {
                let mut parent_mut = parent_rc.borrow_mut();
                let mut current_mut = current.borrow_mut();
                std::mem::swap(&mut parent_mut.value, &mut current_mut.value);
                drop(parent_mut);
                drop(current_mut);
                current = parent_rc;
            } else {
                break;
            }
        }

        let target = current;

        {
            let mut prev: Option<Rc<RefCell<BinomialNode<T>>>> = None;
            let mut curr = self.head.clone();

            while let Some(node) = curr {
                if Rc::ptr_eq(&node, &target) {
                    if let Some(prev_node) = prev {
                        prev_node.borrow_mut().sibling = node.borrow_mut().sibling.take();
                    } else {
                        self.head = node.borrow_mut().sibling.take();
                    }
                    break;
                }
                prev = Some(node.clone());
                curr = node.borrow().sibling.clone();
            }
        }

        let mut child = target.borrow_mut().child.take();
        let mut new_head = None;
        while let Some(c) = child {
            let next = c.borrow_mut().sibling.take();
            c.borrow_mut().parent = None;
            c.borrow_mut().sibling = new_head;
            new_head = Some(c.clone());
            child = next;
        }

        let mut child_heap = Self {
            head: new_head,
            len: 0,
        };
        let old_len = self.len;
        self.merge(&mut child_heap);
        self.len = old_len.saturating_sub(1);

        Rc::try_unwrap(target)
            .map_err(|_| "Rc::try_unwrap failed in delete")
            .ok()
            .map(|cell| cell.into_inner().value)
    }

    pub fn delete_value(&mut self, value: &T) -> Option<T> {
        let cursor = self.search(value)?;
        self.delete(cursor)
    }
}

impl<T: Ord> PriorityQueue<T> for BinomialHeap<T> {
    type Cursor<'a> = BinomialNodeCursor<T> where Self: 'a;
    type View<'a> = BinomialNodeView<T> where Self: 'a;

    fn push(&mut self, value: T) {
        self.insert(value)
    }

    fn pop(&mut self) -> Option<T> {
        self.extract_min()
    }

    fn peek<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.min()
    }

    fn cursor<'a>(&'a self, value: &T) -> Option<Self::Cursor<'a>> {
        self.search(value)
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        cursor.node_view()
    }

    fn remove_cursor<'a>(&mut self, cursor: Self::Cursor<'a>) -> Option<T> where T: 'a {
        self.delete(cursor)
    }

    fn merge(&mut self, other: &mut Self) {
        Self::merge(self, other)
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl<T: Ord> ForestDiagnostics for BinomialHeap<T> {
    fn root_count(&self) -> usize {
        let mut count = 0;
        let mut current = self.head.clone();
        while let Some(node) = current {
            count += 1;
            current = node.borrow().sibling.clone();
        }
        count
    }

    fn node_count(&self) -> usize {
        self.len
    }

    fn max_root_degree(&self) -> usize {
        let mut current = self.head.clone();
        let mut max_degree = 0;
        while let Some(node) = current {
            max_degree = max_degree.max(node.borrow().degree);
            current = node.borrow().sibling.clone();
        }
        max_degree
    }
}

impl<T: Ord> Default for BinomialHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> FromIterator<T> for BinomialHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut heap = Self::new();
        for value in iter {
            heap.insert(value);
        }
        heap
    }
}
