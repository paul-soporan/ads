use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

use crate::traits::{core::PriorityQueue, diagnostics::ForestDiagnostics};

#[derive(Debug)]
struct FibonacciNode<T> {
    value: Option<T>,
    degree: usize,
    marked: bool,
    parent: Option<Weak<RefCell<FibonacciNode<T>>>>,
    child: Option<Rc<RefCell<FibonacciNode<T>>>>,
    sibling: Option<Rc<RefCell<FibonacciNode<T>>>>,
}

impl<T> FibonacciNode<T> {
    fn new(value: T) -> Self {
        Self {
            value: Some(value),
            degree: 0,
            marked: false,
            parent: None,
            child: None,
            sibling: None,
        }
    }
}

#[derive(Debug)]
pub struct FibonacciNodeView<T> {
    node: Rc<RefCell<FibonacciNode<T>>>,
}

impl<T> From<Rc<RefCell<FibonacciNode<T>>>> for FibonacciNodeView<T> {
    fn from(node: Rc<RefCell<FibonacciNode<T>>>) -> Self {
        Self { node }
    }
}

#[derive(Debug)]
pub struct FibonacciNodeCursor<T> {
    node: Rc<RefCell<FibonacciNode<T>>>,
}

impl<T> From<Rc<RefCell<FibonacciNode<T>>>> for FibonacciNodeCursor<T> {
    fn from(node: Rc<RefCell<FibonacciNode<T>>>) -> Self {
        Self { node }
    }
}

impl<T> Clone for FibonacciNodeView<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl<T> Clone for FibonacciNodeCursor<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl<T> FibonacciNodeView<T> {
    pub fn value(&self) -> Ref<'_, T> {
        Ref::map(self.node.borrow(), |node| {
            node.value.as_ref().expect("node value should be present")
        })
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

impl<T> FibonacciNodeCursor<T> {
    fn rc(&self) -> Rc<RefCell<FibonacciNode<T>>> {
        self.node.clone()
    }

    pub fn value(&self) -> Ref<'_, T> {
        Ref::map(self.node.borrow(), |node| {
            node.value.as_ref().expect("node value should be present")
        })
    }

    pub fn node_view(&self) -> FibonacciNodeView<T> {
        FibonacciNodeView::from(self.node.clone())
    }
}

#[derive(Debug)]
pub struct FibonacciHeap<T> {
    head: Option<Rc<RefCell<FibonacciNode<T>>>>,
    len: usize,
}

impl<T: Ord> FibonacciHeap<T> {
    pub fn new() -> Self {
        Self { head: None, len: 0 }
    }

    pub fn head_view(&self) -> Option<FibonacciNodeView<T>> {
        self.head.clone().map(FibonacciNodeView::from)
    }

    pub fn roots(&self) -> Vec<FibonacciNodeView<T>> {
        let mut roots = Vec::new();
        let mut current = self.head.clone();
        while let Some(node) = current {
            roots.push(FibonacciNodeView::from(node.clone()));
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

    pub fn search(&self, value: &T) -> Option<FibonacciNodeCursor<T>> {
        let mut stack = Vec::new();
        let mut current = self.head.clone();

        while let Some(node) = current {
            stack.push(node.clone());
            current = node.borrow().sibling.clone();
        }

        while let Some(node) = stack.pop() {
            if node
                .borrow()
                .value
                .as_ref()
                .is_some_and(|current| current == value)
            {
                return Some(FibonacciNodeCursor::from(node));
            }

            if node
                .borrow()
                .value
                .as_ref()
                .is_some_and(|current| current < value)
            {
                let mut child = node.borrow().child.clone();
                while let Some(c) = child {
                    stack.push(c.clone());
                    child = c.borrow().sibling.clone();
                }
            }
        }

        None
    }

    pub fn min(&self) -> Option<FibonacciNodeCursor<T>> {
        let mut min_node = None;
        let mut current = self.head.clone();

        while let Some(node) = current {
            let is_smaller = min_node
                .as_ref()
                .is_none_or(|min: &Rc<RefCell<FibonacciNode<T>>>| {
                    node.borrow()
                        .value
                        .as_ref()
                        .expect("node value should be present")
                        < min
                            .borrow()
                            .value
                            .as_ref()
                            .expect("node value should be present")
                });

            if is_smaller {
                min_node = Some(node.clone());
            }

            current = node.borrow().sibling.clone();
        }

        min_node.map(FibonacciNodeCursor::from)
    }

    pub fn insert(&mut self, value: T) {
        let mut singleton = Self {
            head: Some(Rc::new(RefCell::new(FibonacciNode::new(value)))),
            len: 1,
        };
        self.merge(&mut singleton);
    }

    fn link(child_root: Rc<RefCell<FibonacciNode<T>>>, parent_root: Rc<RefCell<FibonacciNode<T>>>) {
        {
            let mut child_mut = child_root.borrow_mut();
            child_mut.parent = Some(Rc::downgrade(&parent_root));
            child_mut.marked = false;
        }
        let parent_child = parent_root.borrow_mut().child.take();
        child_root.borrow_mut().sibling = parent_child;
        parent_root.borrow_mut().child = Some(child_root);
        parent_root.borrow_mut().degree += 1;
    }

    fn prepend_root_list(
        first: Option<Rc<RefCell<FibonacciNode<T>>>>,
        second: Option<Rc<RefCell<FibonacciNode<T>>>>,
    ) -> Option<Rc<RefCell<FibonacciNode<T>>>> {
        match (first, second) {
            (None, None) => None,
            (Some(head), None) | (None, Some(head)) => Some(head),
            (Some(first_head), Some(second_head)) => {
                let mut tail = second_head.clone();
                loop {
                    let next = tail.borrow().sibling.clone();
                    if let Some(next_node) = next {
                        tail = next_node;
                    } else {
                        break;
                    }
                }
                tail.borrow_mut().sibling = Some(first_head);
                Some(second_head)
            }
        }
    }

    fn roots_from_list(
        mut head: Option<Rc<RefCell<FibonacciNode<T>>>>,
    ) -> Vec<Rc<RefCell<FibonacciNode<T>>>> {
        let mut roots = Vec::new();
        while let Some(node) = head {
            let next = node.borrow_mut().sibling.take();
            node.borrow_mut().parent = None;
            roots.push(node.clone());
            head = next;
        }
        roots
    }

    fn list_from_roots(
        roots: Vec<Rc<RefCell<FibonacciNode<T>>>>,
    ) -> Option<Rc<RefCell<FibonacciNode<T>>>> {
        let mut head: Option<Rc<RefCell<FibonacciNode<T>>>> = None;
        let mut tail: Option<Rc<RefCell<FibonacciNode<T>>>> = None;

        for node in roots {
            node.borrow_mut().sibling = None;
            if let Some(tail_node) = tail {
                tail_node.borrow_mut().sibling = Some(node.clone());
                tail = Some(node);
            } else {
                head = Some(node.clone());
                tail = Some(node);
            }
        }

        head
    }

    fn consolidate(
        head: Option<Rc<RefCell<FibonacciNode<T>>>>,
    ) -> Option<Rc<RefCell<FibonacciNode<T>>>> {
        let mut buckets: Vec<Option<Rc<RefCell<FibonacciNode<T>>>>> = Vec::new();

        for root in Self::roots_from_list(head) {
            let mut current = root;
            loop {
                let degree = current.borrow().degree;
                if buckets.len() <= degree {
                    buckets.resize_with(degree + 1, || None);
                }

                if let Some(other) = buckets[degree].take() {
                    let (parent, child) = if current
                        .borrow()
                        .value
                        .as_ref()
                        .expect("node value should be present")
                        <= other
                            .borrow()
                            .value
                            .as_ref()
                            .expect("node value should be present")
                    {
                        (current, other)
                    } else {
                        (other, current)
                    };
                    Self::link(child, parent.clone());
                    current = parent;
                } else {
                    buckets[degree] = Some(current);
                    break;
                }
            }
        }

        let roots: Vec<_> = buckets.into_iter().flatten().collect();
        Self::list_from_roots(roots)
    }

    fn push_root(&mut self, node: Rc<RefCell<FibonacciNode<T>>>) {
        {
            let mut node_mut = node.borrow_mut();
            node_mut.parent = None;
            node_mut.marked = false;
            node_mut.sibling = self.head.take();
        }
        self.head = Some(node);
    }

    fn detach_from_parent(
        parent: &Rc<RefCell<FibonacciNode<T>>>,
        child: &Rc<RefCell<FibonacciNode<T>>>,
    ) {
        let mut prev: Option<Rc<RefCell<FibonacciNode<T>>>> = None;
        let mut current = parent.borrow().child.clone();

        while let Some(node) = current {
            if Rc::ptr_eq(&node, child) {
                let next = node.borrow_mut().sibling.take();
                if let Some(prev_node) = prev {
                    prev_node.borrow_mut().sibling = next;
                } else {
                    parent.borrow_mut().child = next;
                }
                let mut parent_mut = parent.borrow_mut();
                parent_mut.degree = parent_mut.degree.saturating_sub(1);
                break;
            }

            prev = Some(node.clone());
            current = node.borrow().sibling.clone();
        }
    }

    fn cut(&mut self, node: Rc<RefCell<FibonacciNode<T>>>, parent: Rc<RefCell<FibonacciNode<T>>>) {
        Self::detach_from_parent(&parent, &node);
        self.push_root(node);
    }

    fn cascading_cut(&mut self, mut node: Rc<RefCell<FibonacciNode<T>>>) {
        loop {
            let parent = {
                let b = node.borrow();
                b.parent.as_ref().and_then(|p| p.upgrade())
            };

            let Some(parent_rc) = parent else {
                break;
            };

            let was_marked = node.borrow().marked;
            if !was_marked {
                node.borrow_mut().marked = true;
                break;
            }

            self.cut(node.clone(), parent_rc.clone());
            node = parent_rc;
        }
    }

    fn remove_root(&mut self, target: &Rc<RefCell<FibonacciNode<T>>>) -> bool {
        let mut prev: Option<Rc<RefCell<FibonacciNode<T>>>> = None;
        let mut current = self.head.clone();

        while let Some(node) = current {
            if Rc::ptr_eq(&node, target) {
                let next = node.borrow_mut().sibling.take();
                if let Some(prev_node) = prev {
                    prev_node.borrow_mut().sibling = next;
                } else {
                    self.head = next;
                }
                return true;
            }

            prev = Some(node.clone());
            current = node.borrow().sibling.clone();
        }

        false
    }

    pub fn merge(&mut self, other: &mut Self) {
        let total_len = self.len + other.len;
        let merged = Self::prepend_root_list(self.head.take(), other.head.take());
        self.head = Self::consolidate(merged);
        self.len = total_len;
        other.len = 0;
    }

    pub fn extract_min(&mut self) -> Option<T> {
        let old_len = self.len;
        if self.head.is_none() {
            return None;
        }

        let mut min_node = self.head.as_ref().expect("head").clone();
        let mut min_prev = None;

        {
            let mut current = min_node.borrow().sibling.clone();
            let mut prev = Some(min_node.clone());

            while let Some(node) = current {
                if node
                    .borrow()
                    .value
                    .as_ref()
                    .expect("node value should be present")
                    < min_node
                        .borrow()
                        .value
                        .as_ref()
                        .expect("node value should be present")
                {
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

        let merged = Self::prepend_root_list(self.head.take(), new_head);
        self.head = Self::consolidate(merged);
        self.len = old_len.saturating_sub(1);

        let removed_value = min_node
            .borrow_mut()
            .value
            .take()
            .expect("extracted node value should be present");
        Some(removed_value)
    }

    pub fn decrease_key(&mut self, handle: FibonacciNodeCursor<T>, new_value: T) {
        if new_value > *handle.value() {
            panic!("decrease_key received a larger replacement value");
        }

        let node = handle.node;
        node.borrow_mut().value = Some(new_value);

        let parent = {
            let b = node.borrow();
            b.parent.as_ref().and_then(|p| p.upgrade())
        };

        if let Some(parent_rc) = parent
            && node
                .borrow()
                .value
                .as_ref()
                .expect("node value should be present")
                < parent_rc
                    .borrow()
                    .value
                    .as_ref()
                    .expect("node value should be present")
        {
            self.cut(node.clone(), parent_rc.clone());
            self.cascading_cut(parent_rc);
        }
    }

    pub fn delete(&mut self, handle: FibonacciNodeCursor<T>) -> Option<T> {
        let old_len = self.len;
        let target = handle.rc();

        let parent = {
            let b = target.borrow();
            b.parent.as_ref().and_then(|p| p.upgrade())
        };
        if let Some(parent_rc) = parent {
            self.cut(target.clone(), parent_rc.clone());
            self.cascading_cut(parent_rc);
        }

        if !self.remove_root(&target) {
            return None;
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

        let merged = Self::prepend_root_list(self.head.take(), new_head);
        self.head = Self::consolidate(merged);
        self.len = old_len.saturating_sub(1);

        let removed_value = target
            .borrow_mut()
            .value
            .take()
            .expect("deleted node value should be present");
        Some(removed_value)
    }

    pub fn delete_value(&mut self, value: &T) -> Option<T> {
        let cursor = self.search(value)?;
        self.delete(cursor)
    }
}

impl<T: Ord> PriorityQueue<T> for FibonacciHeap<T> {
    type Cursor<'a>
        = FibonacciNodeCursor<T>
    where
        Self: 'a;

    type View<'a>
        = FibonacciNodeView<T>
    where
        Self: 'a;

    fn push(&mut self, value: T) {
        Self::insert(self, value)
    }

    fn pop(&mut self) -> Option<T> {
        Self::extract_min(self)
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

    fn remove_cursor<'a>(&mut self, cursor: Self::Cursor<'a>) -> Option<T>
    where
        T: 'a,
    {
        self.delete(cursor)
    }

    fn clear(&mut self) {
        Self::clear(self)
    }

    fn len(&self) -> usize {
        Self::len(self)
    }
}

impl<T: Ord> ForestDiagnostics for FibonacciHeap<T> {
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

impl<T: Ord> Default for FibonacciHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> FromIterator<T> for FibonacciHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut heap = Self::new();
        for value in iter {
            heap.insert(value);
        }
        heap
    }
}
