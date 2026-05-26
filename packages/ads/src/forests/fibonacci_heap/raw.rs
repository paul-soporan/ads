use std::ptr;

use crate::traits::{core::PriorityQueue, diagnostics::ForestDiagnostics};

#[derive(Debug)]
struct FibonacciNode<T> {
    value: Option<T>,
    degree: usize,
    marked: bool,
    parent: *mut FibonacciNode<T>,
    child: *mut FibonacciNode<T>,
    left: *mut FibonacciNode<T>,
    right: *mut FibonacciNode<T>,
}

impl<T> FibonacciNode<T> {
    fn new(value: T) -> Self {
        Self {
            value: Some(value),
            degree: 0,
            marked: false,
            parent: ptr::null_mut(),
            child: ptr::null_mut(),
            left: ptr::null_mut(),
            right: ptr::null_mut(),
        }
    }
}

unsafe fn drop_forest<T>(min_node: *mut FibonacciNode<T>) {
    if min_node.is_null() {
        return;
    }

    let mut stack = Vec::new();
    stack.push(min_node);

    while let Some(root_list_start) = stack.pop() {
        let mut current = root_list_start;
        loop {
            unsafe {
                let next = (*current).right;
                let child = (*current).child;
                if !child.is_null() {
                    stack.push(child);
                }
                
                drop(Box::from_raw(current));
                
                if next == root_list_start {
                    break;
                }
                current = next;
            }
        }
    }
}

#[derive(Debug)]
pub struct FibonacciNodeView<T> {
    node: *mut FibonacciNode<T>,
}

impl<T> Clone for FibonacciNodeView<T> {
    fn clone(&self) -> Self {
        Self { node: self.node }
    }
}

impl<T> FibonacciNodeView<T> {
    pub(crate) fn identity(&self) -> usize {
        self.node as usize
    }

    fn node_ref(&self) -> &FibonacciNode<T> {
        assert!(!self.node.is_null(), "node pointer should be live");
        unsafe { &*self.node }
    }

    pub fn value(&self) -> &T {
        self.node_ref()
            .value
            .as_ref()
            .expect("node value should be present")
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
        let sibling = self.node_ref().right;
        Some(Self { node: sibling })
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
pub struct FibonacciNodeCursor<T> {
    node: *mut FibonacciNode<T>,
}

impl<T> Clone for FibonacciNodeCursor<T> {
    fn clone(&self) -> Self {
        Self { node: self.node }
    }
}

impl<T> FibonacciNodeCursor<T> {
    fn node_ref(&self) -> &FibonacciNode<T> {
        assert!(!self.node.is_null(), "node pointer should be live");
        unsafe { &*self.node }
    }

    pub fn value(&self) -> &T {
        self.node_ref()
            .value
            .as_ref()
            .expect("node value should be present")
    }

    pub fn node_view(&self) -> FibonacciNodeView<T> {
        FibonacciNodeView { node: self.node }
    }
}

#[derive(Debug)]
pub struct FibonacciHeap<T> {
    min_node: *mut FibonacciNode<T>,
    len: usize,
}

impl<T> FibonacciHeap<T> {
    pub fn new() -> Self {
        Self {
            min_node: ptr::null_mut(),
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
            drop_forest(self.min_node);
        }
        self.min_node = ptr::null_mut();
        self.len = 0;
    }

    unsafe fn list_append(list1: *mut FibonacciNode<T>, list2: *mut FibonacciNode<T>) -> *mut FibonacciNode<T> {
        if list1.is_null() { return list2; }
        if list2.is_null() { return list1; }

        unsafe {
            let l1_right = (*list1).right;
            let l2_left = (*list2).left;

            (*list1).right = list2;
            (*list2).left = list1;
            (*l1_right).left = l2_left;
            (*l2_left).right = l1_right;
        }

        list1
    }

    unsafe fn list_remove(node: *mut FibonacciNode<T>) -> *mut FibonacciNode<T> {
        unsafe {
            let left = (*node).left;
            let right = (*node).right;

            if left == node {
                return ptr::null_mut();
            }

            (*left).right = right;
            (*right).left = left;

            (*node).left = node;
            (*node).right = node;

            right
        }
    }
}

impl<T: Ord> FibonacciHeap<T> {
    fn link(&mut self, child_root: *mut FibonacciNode<T>, parent_root: *mut FibonacciNode<T>) {
        unsafe {
            (*child_root).parent = parent_root;
            (*child_root).marked = false;
            (*parent_root).child = Self::list_append((*parent_root).child, child_root);
            (*parent_root).degree += 1;
        }
    }

    fn consolidate(&mut self) {
        if self.min_node.is_null() { return; }

        let mut buckets: Vec<Option<*mut FibonacciNode<T>>> = Vec::new();
        
        let mut current = self.min_node;
        let mut roots = Vec::new();
        loop {
            let next = unsafe { (*current).right };
            unsafe {
                (*current).left = current;
                (*current).right = current;
                (*current).parent = ptr::null_mut();
            }
            roots.push(current);
            if next == self.min_node { break; }
            current = next;
        }

        for mut root in roots {
            loop {
                let degree = unsafe { (*root).degree };
                if buckets.len() <= degree {
                    buckets.resize_with(degree + 1, || None);
                }

                if let Some(other) = buckets[degree].take() {
                    let (parent, child) = unsafe {
                        if (*root).value <= (*other).value { (root, other) } else { (other, root) }
                    };
                    self.link(child, parent);
                    root = parent;
                } else {
                    buckets[degree] = Some(root);
                    break;
                }
            }
        }

        self.min_node = ptr::null_mut();
        for root_opt in buckets {
            if let Some(root) = root_opt {
                if self.min_node.is_null() {
                    self.min_node = root;
                } else {
                    unsafe {
                        self.min_node = Self::list_append(self.min_node, root);
                        if (*root).value < (*self.min_node).value {
                            self.min_node = root;
                        }
                    }
                }
            }
        }
    }

    pub fn head_view(&self) -> Option<FibonacciNodeView<T>> {
        if self.min_node.is_null() {
            None
        } else {
            Some(FibonacciNodeView { node: self.min_node })
        }
    }

    pub fn roots(&self) -> Vec<FibonacciNodeView<T>> {
        let mut roots = Vec::new();
        if self.min_node.is_null() { return roots; }

        let mut current = self.min_node;
        loop {
            roots.push(FibonacciNodeView { node: current });
            current = unsafe { (*current).right };
            if current == self.min_node { break; }
        }

        roots
    }

    pub fn search(&self, value: &T) -> Option<FibonacciNodeCursor<T>> {
        if self.min_node.is_null() { return None; }
        
        let mut stack = Vec::new();
        stack.push(self.min_node);

        unsafe {
            while let Some(root_list_start) = stack.pop() {
                let mut current = root_list_start;
                loop {
                    let node_value = (*current).value.as_ref().expect("value");
                    if node_value == value {
                        return Some(FibonacciNodeCursor { node: current });
                    }

                    if node_value < value {
                        let child = (*current).child;
                        if !child.is_null() {
                            stack.push(child);
                        }
                    }

                    let next = (*current).right;
                    if next == root_list_start { break; }
                    current = next;
                }
            }
        }

        None
    }

    pub fn min(&self) -> Option<FibonacciNodeCursor<T>> {
        if self.min_node.is_null() {
            None
        } else {
            Some(FibonacciNodeCursor { node: self.min_node })
        }
    }

    pub fn insert(&mut self, value: T) {
        let node = Box::into_raw(Box::new(FibonacciNode::new(value)));
        unsafe {
            (*node).left = node;
            (*node).right = node;
        }
        
        if self.min_node.is_null() {
            self.min_node = node;
        } else {
            unsafe {
                self.min_node = Self::list_append(self.min_node, node);
                if (*node).value < (*self.min_node).value {
                    self.min_node = node;
                }
            }
        }
        self.len += 1;
    }

    pub fn merge(&mut self, other: &mut Self) {
        if other.min_node.is_null() { return; }

        if self.min_node.is_null() {
            self.min_node = other.min_node;
            self.len = other.len;
        } else {
            unsafe {
                self.min_node = Self::list_append(self.min_node, other.min_node);
                if (*other.min_node).value < (*self.min_node).value {
                    self.min_node = other.min_node;
                }
            }
            self.len += other.len;
        }

        other.min_node = ptr::null_mut();
        other.len = 0;
    }

    pub fn extract_min(&mut self) -> Option<T> {
        if self.min_node.is_null() { return None; }

        let min_node = self.min_node;
        unsafe {
            let child = (*min_node).child;
            if !child.is_null() {
                let mut c = child;
                loop {
                    (*c).parent = ptr::null_mut();
                    c = (*c).right;
                    if c == child { break; }
                }
                self.min_node = Self::list_append(self.min_node, child);
            }

            let next = Self::list_remove(min_node);
            if next.is_null() {
                self.min_node = ptr::null_mut();
            } else {
                self.min_node = next;
                self.consolidate();
            }

            self.len = self.len.saturating_sub(1);
            let value = (*min_node).value.take().expect("value");
            drop(Box::from_raw(min_node));
            Some(value)
        }
    }

    pub fn decrease_key(&mut self, handle: FibonacciNodeCursor<T>, new_value: T) {
        let node = handle.node;
        unsafe {
            if new_value > *(*node).value.as_ref().unwrap() {
                panic!("decrease_key received a larger value");
            }
            (*node).value = Some(new_value);
            
            let parent = (*node).parent;
            if !parent.is_null() && (*node).value < (*parent).value {
                self.cut(node, parent);
                self.cascading_cut(parent);
            }

            if (*node).value < (*self.min_node).value {
                self.min_node = node;
            }
        }
    }

    unsafe fn cut(&mut self, node: *mut FibonacciNode<T>, parent: *mut FibonacciNode<T>) {
        unsafe {
            (*parent).child = Self::list_remove(node);
            (*parent).degree -= 1;
            (*node).parent = ptr::null_mut();
            (*node).marked = false;
            self.min_node = Self::list_append(self.min_node, node);
        }
    }

    unsafe fn cascading_cut(&mut self, node: *mut FibonacciNode<T>) {
        unsafe {
            let parent = (*node).parent;
            if !parent.is_null() {
                if !(*node).marked {
                    (*node).marked = true;
                } else {
                    self.cut(node, parent);
                    self.cascading_cut(parent);
                }
            }
        }
    }

    pub fn delete(&mut self, handle: FibonacciNodeCursor<T>) -> Option<T> {
        let node = handle.node;
        unsafe {
            let parent = (*node).parent;
            if !parent.is_null() {
                self.cut(node, parent);
                self.cascading_cut(parent);
            }
            self.min_node = node; // Make it the minimum to extract it
            self.extract_min()
        }
    }

    pub fn delete_value(&mut self, value: &T) -> Option<T> {
        let cursor = self.search(value)?;
        self.delete(cursor)
    }
}

impl<T: Ord> PriorityQueue<T> for FibonacciHeap<T> {
    type Cursor<'a> = FibonacciNodeCursor<T> where Self: 'a;
    type View<'a> = FibonacciNodeView<T> where Self: 'a;

    fn push(&mut self, value: T) { self.insert(value) }
    fn pop(&mut self) -> Option<T> { self.extract_min() }
    fn peek<'a>(&'a self) -> Option<Self::Cursor<'a>> { self.min() }
    fn cursor<'a>(&'a self, value: &T) -> Option<Self::Cursor<'a>> { self.search(value) }
    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> { cursor.node_view() }
    fn remove_cursor<'a>(&mut self, cursor: Self::Cursor<'a>) -> Option<T> where T: 'a { self.delete(cursor) }
    fn merge(&mut self, other: &mut Self) { self.merge(other) }
    fn clear(&mut self) { self.clear() }
    fn len(&self) -> usize { self.len }
}

impl<T: Ord> ForestDiagnostics for FibonacciHeap<T> {
    fn root_count(&self) -> usize {
        if self.min_node.is_null() { return 0; }
        let mut count = 0;
        let mut current = self.min_node;
        loop {
            count += 1;
            current = unsafe { (*current).right };
            if current == self.min_node { break; }
        }
        count
    }

    fn node_count(&self) -> usize { self.len }

    fn max_root_degree(&self) -> usize {
        if self.min_node.is_null() { return 0; }
        let mut max_degree = 0;
        let mut current = self.min_node;
        loop {
            max_degree = max_degree.max(unsafe { (*current).degree });
            current = unsafe { (*current).right };
            if current == self.min_node { break; }
        }
        max_degree
    }
}

impl<T: Ord> Default for FibonacciHeap<T> {
    fn default() -> Self { Self::new() }
}

impl<T: Ord> FromIterator<T> for FibonacciHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut heap = Self::new();
        for value in iter { heap.insert(value); }
        heap
    }
}

impl<T> Drop for FibonacciHeap<T> {
    fn drop(&mut self) { self.clear(); }
}
