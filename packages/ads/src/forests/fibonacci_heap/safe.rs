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
    prev: Option<Weak<RefCell<FibonacciNode<T>>>>,
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
            prev: None,
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
    pub(crate) fn identity(&self) -> usize {
        self.node.as_ptr() as usize
    }

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
    tail: Option<Rc<RefCell<FibonacciNode<T>>>>,
    min_node: Option<Rc<RefCell<FibonacciNode<T>>>>,
    len: usize,
}

impl<T: Ord> FibonacciHeap<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            min_node: None,
            len: 0,
        }
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
        self.tail = None;
        self.min_node = None;
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
            let node_borrow = node.borrow();
            let node_value = node_borrow.value.as_ref().expect("node value");

            if node_value == value {
                return Some(FibonacciNodeCursor::from(node.clone()));
            }

            if node_value < value {
                let mut child = node_borrow.child.clone();
                while let Some(c) = child {
                    stack.push(c.clone());
                    child = c.borrow().sibling.clone();
                }
            }
        }

        None
    }

    pub fn min(&self) -> Option<FibonacciNodeCursor<T>> {
        self.min_node.clone().map(FibonacciNodeCursor::from)
    }

    fn update_min_node(&mut self, candidate: Rc<RefCell<FibonacciNode<T>>>) {
        if let Some(min) = &self.min_node {
            if candidate.borrow().value.as_ref().expect("value") < min.borrow().value.as_ref().expect("value") {
                self.min_node = Some(candidate);
            }
        } else {
            self.min_node = Some(candidate);
        }
    }

    pub fn insert(&mut self, value: T) {
        let node = Rc::new(RefCell::new(FibonacciNode::new(value)));
        self.update_min_node(node.clone());

        if let Some(tail_node) = self.tail.take() {
            node.borrow_mut().prev = Some(Rc::downgrade(&tail_node));
            tail_node.borrow_mut().sibling = Some(node.clone());
            self.tail = Some(node);
        } else {
            self.head = Some(node.clone());
            self.tail = Some(node);
        }
        self.len += 1;
    }

    fn link(child_root: Rc<RefCell<FibonacciNode<T>>>, parent_root: Rc<RefCell<FibonacciNode<T>>>) {
        {
            let mut child_mut = child_root.borrow_mut();
            child_mut.parent = Some(Rc::downgrade(&parent_root));
            child_mut.marked = false;
            child_mut.sibling = parent_root.borrow().child.clone();
            if let Some(next) = &child_mut.sibling {
                next.borrow_mut().prev = Some(Rc::downgrade(&child_root));
            }
            child_mut.prev = None;
        }
        parent_root.borrow_mut().child = Some(child_root);
        parent_root.borrow_mut().degree += 1;
    }

    fn consolidate(&mut self) {
        let mut buckets: Vec<Option<Rc<RefCell<FibonacciNode<T>>>>> = Vec::new();

        let mut current = self.head.take();
        self.tail = None;

        while let Some(root) = current {
            let next = root.borrow_mut().sibling.take();
            root.borrow_mut().prev = None;
            root.borrow_mut().parent = None;

            let mut current_node = root;
            loop {
                let degree = current_node.borrow().degree;
                if buckets.len() <= degree {
                    buckets.resize_with(degree + 1, || None);
                }

                if let Some(other) = buckets[degree].take() {
                    let (parent, child) = if current_node.borrow().value < other.borrow().value {
                        (current_node, other)
                    } else {
                        (other, current_node)
                    };
                    Self::link(child, parent.clone());
                    current_node = parent;
                } else {
                    buckets[degree] = Some(current_node);
                    break;
                }
            }
            current = next;
        }

        self.head = None;
        self.tail = None;
        self.min_node = None;

        for root_opt in buckets {
            if let Some(root) = root_opt {
                self.update_min_node(root.clone());
                if let Some(tail_node) = self.tail.take() {
                    root.borrow_mut().prev = Some(Rc::downgrade(&tail_node));
                    tail_node.borrow_mut().sibling = Some(root.clone());
                    self.tail = Some(root);
                } else {
                    self.head = Some(root.clone());
                    self.tail = Some(root);
                }
            }
        }
    }

    fn detach_node(
        head: &mut Option<Rc<RefCell<FibonacciNode<T>>>>,
        tail: &mut Option<Rc<RefCell<FibonacciNode<T>>>>,
        node: &Rc<RefCell<FibonacciNode<T>>>,
    ) {
        let next = node.borrow_mut().sibling.take();
        let prev = node.borrow_mut().prev.take();

        if let Some(next_node) = &next {
            next_node.borrow_mut().prev = prev.clone();
        } else {
            *tail = prev.as_ref().and_then(|p| p.upgrade());
        }

        if let Some(prev_weak) = prev {
            if let Some(prev_node) = prev_weak.upgrade() {
                prev_node.borrow_mut().sibling = next;
            }
        } else {
            *head = next;
        }
    }

    fn cut(&mut self, node: Rc<RefCell<FibonacciNode<T>>>, parent: Rc<RefCell<FibonacciNode<T>>>) {
        {
            let mut parent_mut = parent.borrow_mut();
            Self::detach_node(&mut parent_mut.child, &mut None, &node);
            parent_mut.degree = parent_mut.degree.saturating_sub(1);
        }

        node.borrow_mut().parent = None;
        node.borrow_mut().marked = false;
        
        // Add to root list
        if let Some(tail_node) = self.tail.take() {
            node.borrow_mut().prev = Some(Rc::downgrade(&tail_node));
            tail_node.borrow_mut().sibling = Some(node.clone());
            self.tail = Some(node);
        } else {
            self.head = Some(node.clone());
            self.tail = Some(node);
        }
    }

    fn cascading_cut(&mut self, mut node: Rc<RefCell<FibonacciNode<T>>>) {
        loop {
            let parent = node.borrow().parent.as_ref().and_then(|p| p.upgrade());
            let Some(parent_rc) = parent else {
                break;
            };

            if !node.borrow().marked {
                node.borrow_mut().marked = true;
                break;
            }

            self.cut(node.clone(), parent_rc.clone());
            node = parent_rc;
        }
    }

    pub fn merge(&mut self, other: &mut Self) {
        if other.is_empty() {
            return;
        }

        if self.is_empty() {
            self.head = other.head.take();
            self.tail = other.tail.take();
            self.min_node = other.min_node.take();
            self.len = other.len;
            other.len = 0;
            return;
        }

        if let Some(other_min) = &other.min_node {
            self.update_min_node(other_min.clone());
        }

        let self_tail = self.tail.take().expect("self tail");
        let other_head = other.head.take().expect("other head");

        self_tail.borrow_mut().sibling = Some(other_head.clone());
        other_head.borrow_mut().prev = Some(Rc::downgrade(&self_tail));
        self.tail = other.tail.take();

        self.len += other.len;
        other.len = 0;
        other.min_node = None;
    }

    pub fn extract_min(&mut self) -> Option<T> {
        let min_node = self.min_node.take()?;
        Self::detach_node(&mut self.head, &mut self.tail, &min_node);

        let mut child = min_node.borrow_mut().child.take();
        while let Some(c) = child {
            let next = c.borrow_mut().sibling.take();
            c.borrow_mut().prev = None;
            c.borrow_mut().parent = None;
            
            // Add child to root list
            if let Some(tail_node) = self.tail.take() {
                c.borrow_mut().prev = Some(Rc::downgrade(&tail_node));
                tail_node.borrow_mut().sibling = Some(c.clone());
                self.tail = Some(c);
            } else {
                self.head = Some(c.clone());
                self.tail = Some(c);
            }
            child = next;
        }

        self.len = self.len.saturating_sub(1);
        self.consolidate();

        Some(min_node.borrow_mut().value.take().expect("value"))
    }

    pub fn decrease_key(&mut self, handle: FibonacciNodeCursor<T>, new_value: T) {
        if new_value > *handle.value() {
            panic!("decrease_key received a larger replacement value");
        }

        let node = handle.node;
        node.borrow_mut().value = Some(new_value);
        self.update_min_node(node.clone());

        let parent = node.borrow().parent.as_ref().and_then(|p| p.upgrade());
        if let Some(parent_rc) = parent 
            && node.borrow().value < parent_rc.borrow().value 
        {
            self.cut(node.clone(), parent_rc.clone());
            self.cascading_cut(parent_rc);
        }
    }

    pub fn delete(&mut self, handle: FibonacciNodeCursor<T>) -> Option<T> {
        let target = handle.rc();
        let parent = target.borrow().parent.as_ref().and_then(|p| p.upgrade());
        
        if let Some(parent_rc) = parent {
            self.cut(target.clone(), parent_rc.clone());
            self.cascading_cut(parent_rc);
        }

        // Now target is a root.
        // We need to find if it was the min_node.
        let is_min = if let Some(min) = &self.min_node {
            Rc::ptr_eq(min, &target)
        } else {
            false
        };

        if is_min {
            return self.extract_min();
        }

        Self::detach_node(&mut self.head, &mut self.tail, &target);
        
        let mut child = target.borrow_mut().child.take();
        while let Some(c) = child {
            let next = c.borrow_mut().sibling.take();
            c.borrow_mut().prev = None;
            c.borrow_mut().parent = None;
            
            if let Some(tail_node) = self.tail.take() {
                c.borrow_mut().prev = Some(Rc::downgrade(&tail_node));
                tail_node.borrow_mut().sibling = Some(c.clone());
                self.tail = Some(c);
            } else {
                self.head = Some(c.clone());
                self.tail = Some(c);
            }
            child = next;
        }

        self.len = self.len.saturating_sub(1);
        // We don't strictly need to consolidate here in some definitions, 
        // but it's often done or done on next extract_min.
        // Actually, just updating min_node is necessary if we don't consolidate.
        // But since we removed the min_node, we MUST re-scan for it or consolidate.
        self.consolidate();

        Some(target.borrow_mut().value.take().expect("value"))
    }

    pub fn delete_value(&mut self, value: &T) -> Option<T> {
        let cursor = self.search(value)?;
        self.delete(cursor)
    }
}

impl<T: Ord> PriorityQueue<T> for FibonacciHeap<T> {
    type Cursor<'a> = FibonacciNodeCursor<T> where Self: 'a;
    type View<'a> = FibonacciNodeView<T> where Self: 'a;

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
        self.merge(other)
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn len(&self) -> usize {
        self.len
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
        let mut max_degree = 0;
        let mut current = self.head.clone();
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
