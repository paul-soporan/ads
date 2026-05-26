use std::ptr;

use crate::traits::core::PriorityQueue;

#[derive(Debug)]
struct Node<T> {
    value: Option<T>,
    left: *mut Node<T>,
    right: *mut Node<T>,
    parent: *mut Node<T>,
}

impl<T> Node<T> {
    fn new(value: T) -> *mut Self {
        Box::into_raw(Box::new(Self {
            value: Some(value),
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            parent: ptr::null_mut(),
        }))
    }
}

pub struct BinaryHeapCursor<T> {
    node: *mut Node<T>,
    generation: u64,
}

impl<T> Clone for BinaryHeapCursor<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node,
            generation: self.generation,
        }
    }
}

pub struct BinaryHeapView<'a, T> {
    node: *const Node<T>,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Clone for BinaryHeapView<'a, T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> BinaryHeapView<'a, T> {
    pub fn value(&self) -> &T {
        unsafe {
            (*self.node)
                .value
                .as_ref()
                .expect("node value should be present")
        }
    }
}

pub struct BinaryHeap<T> {
    root: *mut Node<T>,
    len: usize,
    generation: u64,
}

impl<T> BinaryHeap<T> {
    pub fn new() -> Self {
        Self {
            root: ptr::null_mut(),
            len: 0,
            generation: 0,
        }
    }

    fn bump_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    fn get_path(n: usize) -> Vec<bool> {
        if n <= 1 {
            return Vec::new();
        }
        let mut path = Vec::new();
        let mut curr = n;
        while curr > 1 {
            path.push(curr % 2 == 1);
            curr /= 2;
        }
        path.reverse();
        path
    }

    unsafe fn free_all(node: *mut Node<T>) {
        if node.is_null() { return; }
        unsafe {
            Self::free_all((*node).left);
            Self::free_all((*node).right);
            drop(Box::from_raw(node));
        }
    }
}

impl<T: Ord> BinaryHeap<T> {
    fn sift_up(&mut self, mut node: *mut Node<T>) {
        unsafe {
            while !(*node).parent.is_null() {
                let parent = (*node).parent;
                if (*node).value.as_ref().unwrap() < (*parent).value.as_ref().unwrap() {
                    std::mem::swap(&mut (*node).value, &mut (*parent).value);
                    node = parent;
                } else {
                    break;
                }
            }
        }
    }

    fn sift_down(&mut self, mut node: *mut Node<T>) {
        unsafe {
            loop {
                let left = (*node).left;
                let right = (*node).right;
                let mut smallest = node;

                if !left.is_null() && (*left).value.as_ref().unwrap() < (*smallest).value.as_ref().unwrap() {
                    smallest = left;
                }
                if !right.is_null() && (*right).value.as_ref().unwrap() < (*smallest).value.as_ref().unwrap() {
                    smallest = right;
                }

                if smallest == node {
                    break;
                }

                std::mem::swap(&mut (*node).value, &mut (*smallest).value);
                node = smallest;
            }
        }
    }
}

impl<T: Ord> PriorityQueue<T> for BinaryHeap<T> {
    type Cursor<'a> = BinaryHeapCursor<T> where Self: 'a;
    type View<'a> = BinaryHeapView<'a, T> where Self: 'a;

    fn push(&mut self, value: T) {
        self.bump_generation();
        let new_len = self.len + 1;
        let path = Self::get_path(new_len);
        let new_node = Node::new(value);

        if path.is_empty() {
            self.root = new_node;
        } else {
            unsafe {
                let mut curr = self.root;
                for (i, &is_right) in path.iter().enumerate() {
                    if i == path.len() - 1 {
                        (*new_node).parent = curr;
                        if is_right {
                            (*curr).right = new_node;
                        } else {
                            (*curr).left = new_node;
                        }
                    } else {
                        curr = if is_right { (*curr).right } else { (*curr).left };
                    }
                }
            }
        }

        self.len = new_len;
        self.sift_up(new_node);
    }

    fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.bump_generation();

        unsafe {
            let old_root = self.root;
            if self.len == 1 {
                self.root = ptr::null_mut();
                self.len = 0;
                let boxed = Box::from_raw(old_root);
                return boxed.value;
            }

            let last_idx = self.len;
            let path = Self::get_path(last_idx);
            let mut curr = self.root;
            let mut last_node = ptr::null_mut();
            
            for (i, &is_right) in path.iter().enumerate() {
                if i == path.len() - 1 {
                    last_node = if is_right {
                        let n = (*curr).right;
                        (*curr).right = ptr::null_mut();
                        n
                    } else {
                        let n = (*curr).left;
                        (*curr).left = ptr::null_mut();
                        n
                    };
                } else {
                    curr = if is_right { (*curr).right } else { (*curr).left };
                }
            }

            std::mem::swap(&mut (*old_root).value, &mut (*last_node).value);
            let result = (*last_node).value.take();
            drop(Box::from_raw(last_node));

            self.len -= 1;
            self.sift_down(old_root);
            result
        }
    }

    fn peek<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        (!self.root.is_null()).then(|| BinaryHeapCursor {
            node: self.root,
            generation: self.generation,
        })
    }

    fn cursor<'a>(&'a self, value: &T) -> Option<Self::Cursor<'a>> {
        let mut stack = Vec::new();
        if !self.root.is_null() { stack.push(self.root); }
        while let Some(node) = stack.pop() {
            unsafe {
                if (*node).value.as_ref().unwrap() == value {
                    return Some(BinaryHeapCursor { node, generation: self.generation });
                }
                if !(*node).right.is_null() { stack.push((*node).right); }
                if !(*node).left.is_null() { stack.push((*node).left); }
            }
        }
        None
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        BinaryHeapView {
            node: cursor.node,
            _marker: std::marker::PhantomData,
        }
    }

    fn remove_cursor<'a>(&mut self, cursor: Self::Cursor<'a>) -> Option<T> where T: 'a {
        if cursor.generation != self.generation { return None; }
        
        let target = cursor.node;
        unsafe {
            if self.len == 1 {
                if self.root == target {
                    self.len = 0;
                    let boxed = Box::from_raw(self.root);
                    self.root = ptr::null_mut();
                    return boxed.value;
                }
                return None;
            }

            let last_idx = self.len;
            let path = Self::get_path(last_idx);
            let mut curr = self.root;
            let mut last_node = ptr::null_mut();
            
            for (i, &is_right) in path.iter().enumerate() {
                if i == path.len() - 1 {
                    last_node = if is_right {
                        let n = (*curr).right;
                        (*curr).right = ptr::null_mut();
                        n
                    } else {
                        let n = (*curr).left;
                        (*curr).left = ptr::null_mut();
                        n
                    };
                } else {
                    curr = if is_right { (*curr).right } else { (*curr).left };
                }
            }

            if target == last_node {
                self.len -= 1;
                let boxed = Box::from_raw(last_node);
                return boxed.value;
            }

            std::mem::swap(&mut (*target).value, &mut (*last_node).value);
            let result = (*last_node).value.take();
            drop(Box::from_raw(last_node));

            self.len -= 1;
            let parent = (*target).parent;
            if !parent.is_null() && (*target).value.as_ref().unwrap() < (*parent).value.as_ref().unwrap() {
                self.sift_up(target);
            } else {
                self.sift_down(target);
            }

            result
        }
    }

    fn merge(&mut self, other: &mut Self) {
        while let Some(val) = other.pop() {
            self.push(val);
        }
    }

    fn clear(&mut self) {
        unsafe {
            Self::free_all(self.root);
        }
        self.root = ptr::null_mut();
        self.len = 0;
        self.bump_generation();
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl<T: Ord> Default for BinaryHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> FromIterator<T> for BinaryHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut heap = Self::new();
        for value in iter {
            heap.push(value);
        }
        heap
    }
}

impl<T> Drop for BinaryHeap<T> {
    fn drop(&mut self) {
        unsafe {
            Self::free_all(self.root);
        }
    }
}
