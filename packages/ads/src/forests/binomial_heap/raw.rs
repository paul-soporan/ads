use std::ptr;

use crate::traits::{core::PriorityQueue, diagnostics::ForestDiagnostics};

#[derive(Debug)]
struct BinomialNode<T> {
    value: T,
    degree: usize,
    parent: *mut BinomialNode<T>,
    child: *mut BinomialNode<T>,
    sibling: *mut BinomialNode<T>,
}

impl<T> BinomialNode<T> {
    fn new(value: T) -> *mut Self {
        Box::into_raw(Box::new(Self {
            value,
            degree: 0,
            parent: ptr::null_mut(),
            child: ptr::null_mut(),
            sibling: ptr::null_mut(),
        }))
    }
}

unsafe fn drop_node<T>(node: *mut BinomialNode<T>) {
    if node.is_null() {
        return;
    }
    let mut stack = vec![node];
    while let Some(curr) = stack.pop() {
        unsafe {
            let mut child = (*curr).child;
            while !child.is_null() {
                let next = (*child).sibling;
                stack.push(child);
                child = next;
            }
            drop(Box::from_raw(curr));
        }
    }
}

#[derive(Debug)]
pub struct BinomialNodeView<T> {
    node: *mut BinomialNode<T>,
}

impl<T> Clone for BinomialNodeView<T> {
    fn clone(&self) -> Self {
        Self { node: self.node }
    }
}

impl<T> BinomialNodeView<T> {
    fn node_ref(&self) -> &BinomialNode<T> {
        unsafe { &*self.node }
    }

    pub fn value(&self) -> &T {
        &self.node_ref().value
    }

    pub fn degree(&self) -> usize {
        self.node_ref().degree
    }

    pub fn child(&self) -> Option<Self> {
        let child = self.node_ref().child;
        if child.is_null() {
            None
        } else {
            Some(Self { node: child })
        }
    }

    pub fn sibling(&self) -> Option<Self> {
        let sibling = self.node_ref().sibling;
        if sibling.is_null() {
            None
        } else {
            Some(Self { node: sibling })
        }
    }

    pub fn parent(&self) -> Option<Self> {
        let parent = self.node_ref().parent;
        if parent.is_null() {
            None
        } else {
            Some(Self { node: parent })
        }
    }
}

#[derive(Debug)]
pub struct BinomialNodeCursor<T> {
    node: *mut BinomialNode<T>,
}

impl<T> Clone for BinomialNodeCursor<T> {
    fn clone(&self) -> Self {
        Self { node: self.node }
    }
}

impl<T> BinomialNodeCursor<T> {
    pub fn value(&self) -> &T {
        unsafe { &(*self.node).value }
    }

    pub fn node_view(&self) -> BinomialNodeView<T> {
        BinomialNodeView { node: self.node }
    }
}

#[derive(Debug)]
pub struct BinomialHeap<T> {
    head: *mut BinomialNode<T>,
    len: usize,
}

impl<T> BinomialHeap<T> {
    pub fn new() -> Self {
        Self {
            head: ptr::null_mut(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        unsafe {
            let mut curr = self.head;
            while !curr.is_null() {
                let next = (*curr).sibling;
                drop_node(curr);
                curr = next;
            }
        }
        self.head = ptr::null_mut();
        self.len = 0;
    }
}

impl<T: Ord> BinomialHeap<T> {
    fn link(child: *mut BinomialNode<T>, parent: *mut BinomialNode<T>) {
        unsafe {
            (*child).parent = parent;
            (*child).sibling = (*parent).child;
            (*parent).child = child;
            (*parent).degree += 1;
        }
    }

    fn merge_root_lists(mut h1: *mut BinomialNode<T>, mut h2: *mut BinomialNode<T>) -> *mut BinomialNode<T> {
        let mut head = ptr::null_mut();
        let mut tail = &mut head;

        unsafe {
            while !h1.is_null() && !h2.is_null() {
                if (*h1).degree <= (*h2).degree {
                    let next = (*h1).sibling;
                    *tail = h1;
                    h1 = next;
                } else {
                    let next = (*h2).sibling;
                    *tail = h2;
                    h2 = next;
                }
                (*(*tail)).sibling = ptr::null_mut();
                tail = &mut (*(*tail)).sibling;
            }

            if !h1.is_null() {
                *tail = h1;
            } else {
                *tail = h2;
            }
        }
        head
    }

    pub fn merge(&mut self, other: &mut Self) {
        if other.head.is_null() {
            return;
        }
        if self.head.is_null() {
            self.head = other.head;
            self.len = other.len;
            other.head = ptr::null_mut();
            other.len = 0;
            return;
        }

        let total_len = self.len + other.len;
        let mut head = Self::merge_root_lists(self.head, other.head);
        self.head = ptr::null_mut();
        other.head = ptr::null_mut();

        if head.is_null() {
            self.len = 0;
            other.len = 0;
            return;
        }

        unsafe {
            let mut prev = ptr::null_mut();
            let mut x = head;
            let mut next = (*x).sibling;

            while !next.is_null() {
                let n_next = (*next).sibling;
                if (*x).degree != (*next).degree || (!n_next.is_null() && (*n_next).degree == (*x).degree) {
                    prev = x;
                    x = next;
                } else if (*x).value <= (*next).value {
                    (*x).sibling = n_next;
                    Self::link(next, x);
                } else {
                    if prev.is_null() {
                        head = next;
                    } else {
                        (*prev).sibling = next;
                    }
                    Self::link(x, next);
                    x = next;
                }
                next = (*x).sibling;
            }
        }

        self.head = head;
        self.len = total_len;
        other.len = 0;
    }

    pub fn insert(&mut self, value: T) {
        let mut singleton = Self {
            head: BinomialNode::new(value),
            len: 1,
        };
        self.merge(&mut singleton);
    }

    pub fn extract_min(&mut self) -> Option<T> {
        if self.head.is_null() {
            return None;
        }

        let mut min_node = self.head;
        let mut min_prev = ptr::null_mut();
        
        unsafe {
            let mut curr = (*self.head).sibling;
            let mut prev = self.head;

            while !curr.is_null() {
                if (*curr).value < (*min_node).value {
                    min_node = curr;
                    min_prev = prev;
                }
                prev = curr;
                curr = (*curr).sibling;
            }

            if min_prev.is_null() {
                self.head = (*min_node).sibling;
            } else {
                (*min_prev).sibling = (*min_node).sibling;
            }

            let mut child = (*min_node).child;
            let mut new_head = ptr::null_mut();
            while !child.is_null() {
                let next = (*child).sibling;
                (*child).parent = ptr::null_mut();
                (*child).sibling = new_head;
                new_head = child;
                child = next;
            }

            let mut child_heap = Self {
                head: new_head,
                len: 0,
            };
            let old_len = self.len;
            self.merge(&mut child_heap);
            self.len = old_len.saturating_sub(1);

            let node_box = Box::from_raw(min_node);
            Some(node_box.value)
        }
    }

    pub fn min(&self) -> Option<BinomialNodeCursor<T>> {
        if self.head.is_null() {
            return None;
        }
        let mut min_node = self.head;
        unsafe {
            let mut curr = (*self.head).sibling;
            while !curr.is_null() {
                if (*curr).value < (*min_node).value {
                    min_node = curr;
                }
                curr = (*curr).sibling;
            }
        }
        Some(BinomialNodeCursor { node: min_node })
    }

    pub fn decrease_key(&mut self, handle: BinomialNodeCursor<T>, new_value: T) {
        let node = handle.node;
        unsafe {
            if new_value > (*node).value {
                panic!("decrease_key received a larger replacement value");
            }
            (*node).value = new_value;

            let mut curr = node;
            let mut parent = (*curr).parent;
            while !parent.is_null() && (*curr).value < (*parent).value {
                std::mem::swap(&mut (*curr).value, &mut (*parent).value);
                curr = parent;
                parent = (*curr).parent;
            }
        }
    }

    pub fn delete(&mut self, handle: BinomialNodeCursor<T>) -> Option<T> {
        let node = handle.node;
        unsafe {
            let mut curr = node;
            let mut parent = (*curr).parent;
            while !parent.is_null() {
                std::mem::swap(&mut (*curr).value, &mut (*parent).value);
                curr = parent;
                parent = (*curr).parent;
            }

            let target = curr;
            let mut prev = ptr::null_mut();
            let mut r = self.head;
            while !r.is_null() && r != target {
                prev = r;
                r = (*r).sibling;
            }

            if r.is_null() {
                return None;
            }

            if prev.is_null() {
                self.head = (*r).sibling;
            } else {
                (*prev).sibling = (*r).sibling;
            }

            let mut child = (*r).child;
            let mut new_head = ptr::null_mut();
            while !child.is_null() {
                let next = (*child).sibling;
                (*child).parent = ptr::null_mut();
                (*child).sibling = new_head;
                new_head = child;
                child = next;
            }

            let mut child_heap = Self {
                head: new_head,
                len: 0,
            };
            let old_len = self.len;
            self.merge(&mut child_heap);
            self.len = old_len.saturating_sub(1);

            let node_box = Box::from_raw(r);
            Some(node_box.value)
        }
    }

    pub fn search(&self, value: &T) -> Option<BinomialNodeCursor<T>> {
        if self.head.is_null() {
            return None;
        }
        let mut stack = vec![self.head];
        unsafe {
            while let Some(root_list_start) = stack.pop() {
                let mut curr = root_list_start;
                while !curr.is_null() {
                    if (*curr).value == *value {
                        return Some(BinomialNodeCursor { node: curr });
                    }
                    if (*curr).value < *value {
                        let child = (*curr).child;
                        if !child.is_null() {
                            stack.push(child);
                        }
                    }
                    curr = (*curr).sibling;
                }
            }
        }
        None
    }

    pub fn head_view(&self) -> Option<BinomialNodeView<T>> {
        if self.head.is_null() {
            None
        } else {
            Some(BinomialNodeView { node: self.head })
        }
    }

    pub fn roots(&self) -> Vec<BinomialNodeView<T>> {
        let mut roots = Vec::new();
        let mut curr = self.head;
        while !curr.is_null() {
            roots.push(BinomialNodeView { node: curr });
            unsafe {
                curr = (*curr).sibling;
            }
        }
        roots
    }
}

impl<T> Drop for BinomialHeap<T> {
    fn drop(&mut self) {
        self.clear();
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

    fn remove_cursor<'a>(&mut self, cursor: Self::Cursor<'a>) -> Option<T> where T: 'a { self.delete(cursor) }
    fn merge(&mut self, other: &mut Self) { self.merge(other) }
    fn clear(&mut self) { self.clear() }


    fn len(&self) -> usize {
        self.len
    }
}

impl<T: Ord> ForestDiagnostics for BinomialHeap<T> {
    fn root_count(&self) -> usize {
        let mut count = 0;
        let mut curr = self.head;
        while !curr.is_null() {
            count += 1;
            unsafe {
                curr = (*curr).sibling;
            }
        }
        count
    }

    fn node_count(&self) -> usize {
        self.len
    }

    fn max_root_degree(&self) -> usize {
        let mut max_degree = 0;
        let mut curr = self.head;
        while !curr.is_null() {
            unsafe {
                max_degree = max_degree.max((*curr).degree);
                curr = (*curr).sibling;
            }
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
