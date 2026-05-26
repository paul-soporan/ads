use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

use crate::traits::core::PriorityQueue;

#[derive(Debug)]
struct Node<T> {
    value: Option<T>,
    left: Option<Rc<RefCell<Node<T>>>>,
    right: Option<Rc<RefCell<Node<T>>>>,
    parent: Option<Weak<RefCell<Node<T>>>>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self {
            value: Some(value),
            left: None,
            right: None,
            parent: None,
        }
    }
}

pub struct BinaryHeapCursor<T> {
    node: Rc<RefCell<Node<T>>>,
    generation: u64,
}

impl<T> Clone for BinaryHeapCursor<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            generation: self.generation,
        }
    }
}

pub struct BinaryHeapView<'a, T> {
    node: Rc<RefCell<Node<T>>>,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Clone for BinaryHeapView<'a, T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> BinaryHeapView<'a, T> {
    pub fn value(&self) -> Ref<'_, T> {
        Ref::map(self.node.borrow(), |node| {
            node.value.as_ref().expect("node value should be present")
        })
    }
}

pub struct BinaryHeap<T> {
    root: Option<Rc<RefCell<Node<T>>>>,
    len: usize,
    generation: u64,
}

impl<T: Ord> BinaryHeap<T> {
    pub fn new() -> Self {
        Self {
            root: None,
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

    fn sift_up(&mut self, mut node: Rc<RefCell<Node<T>>>) {
        loop {
            let parent = {
                let b = node.borrow();
                b.parent.as_ref().and_then(|p| p.upgrade())
            };
            let Some(p) = parent else { break };

            let should_swap = node.borrow().value.as_ref().unwrap() < p.borrow().value.as_ref().unwrap();
            if should_swap {
                let mut b_node = node.borrow_mut();
                let mut b_parent = p.borrow_mut();
                std::mem::swap(&mut b_node.value, &mut b_parent.value);
                drop(b_node);
                drop(b_parent);
                node = p;
            } else {
                break;
            }
        }
    }

    fn sift_down(&mut self, mut node: Rc<RefCell<Node<T>>>) {
        loop {
            let (left, right) = {
                let b = node.borrow();
                (b.left.clone(), b.right.clone())
            };

            let mut smallest = node.clone();

            if let Some(l) = left {
                if l.borrow().value.as_ref().unwrap() < smallest.borrow().value.as_ref().unwrap() {
                    smallest = l;
                }
            }

            if let Some(r) = right {
                if r.borrow().value.as_ref().unwrap() < smallest.borrow().value.as_ref().unwrap() {
                    smallest = r;
                }
            }

            if Rc::ptr_eq(&smallest, &node) {
                break;
            }

            let mut b_node = node.borrow_mut();
            let mut b_smallest = smallest.borrow_mut();
            std::mem::swap(&mut b_node.value, &mut b_smallest.value);
            drop(b_node);
            drop(b_smallest);
            node = smallest;
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
        let new_node = Rc::new(RefCell::new(Node::new(value)));

        if path.is_empty() {
            self.root = Some(new_node.clone());
        } else {
            let mut curr = self.root.as_ref().unwrap().clone();
            for (i, &is_right) in path.iter().enumerate() {
                if i == path.len() - 1 {
                    new_node.borrow_mut().parent = Some(Rc::downgrade(&curr));
                    if is_right {
                        curr.borrow_mut().right = Some(new_node.clone());
                    } else {
                        curr.borrow_mut().left = Some(new_node.clone());
                    }
                } else {
                    let next = if is_right {
                        curr.borrow().right.as_ref().unwrap().clone()
                    } else {
                        curr.borrow().left.as_ref().unwrap().clone()
                    };
                    curr = next;
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

        let old_root = self.root.as_ref().unwrap().clone();
        if self.len == 1 {
            self.root = None;
            self.len = 0;
            return old_root.borrow_mut().value.take();
        }

        let last_idx = self.len;
        let path = Self::get_path(last_idx);
        let mut curr = self.root.as_ref().unwrap().clone();
        
        let mut last_node = None;
        for (i, &is_right) in path.iter().enumerate() {
            if i == path.len() - 1 {
                last_node = if is_right {
                    curr.borrow_mut().right.take()
                } else {
                    curr.borrow_mut().left.take()
                };
            } else {
                let next = if is_right {
                    curr.borrow().right.as_ref().unwrap().clone()
                } else {
                    curr.borrow().left.as_ref().unwrap().clone()
                };
                curr = next;
            }
        }

        let last_node = last_node.unwrap();
        let mut b_root = old_root.borrow_mut();
        let mut b_last = last_node.borrow_mut();
        std::mem::swap(&mut b_root.value, &mut b_last.value);
        let result = b_last.value.take();
        drop(b_root);
        drop(b_last);

        self.len -= 1;
        self.sift_down(old_root);
        result
    }

    fn peek<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.root.as_ref().map(|node| BinaryHeapCursor {
            node: node.clone(),
            generation: self.generation,
        })
    }

    fn cursor<'a>(&'a self, value: &T) -> Option<Self::Cursor<'a>> {
        let mut stack = Vec::new();
        if let Some(r) = &self.root { stack.push(r.clone()); }
        while let Some(node) = stack.pop() {
            if node.borrow().value.as_ref().unwrap() == value {
                return Some(BinaryHeapCursor { node, generation: self.generation });
            }
            if let Some(r) = &node.borrow().right { stack.push(r.clone()); }
            if let Some(l) = &node.borrow().left { stack.push(l.clone()); }
        }
        None
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        BinaryHeapView {
            node: cursor.node.clone(),
            _marker: std::marker::PhantomData,
        }
    }

    fn remove_cursor<'a>(&mut self, cursor: Self::Cursor<'a>) -> Option<T> where T: 'a {
        if cursor.generation != self.generation { return None; }
        
        let target = cursor.node;
        if self.len == 0 { return None; }
        
        let last_idx = self.len;
        let last_path = Self::get_path(last_idx);
        let mut curr = self.root.as_ref().unwrap().clone();
        let mut last_node = None;
        
        if last_path.is_empty() {
            last_node = self.root.take();
            self.len = 0;
            return last_node.unwrap().borrow_mut().value.take();
        }

        for (i, &is_right) in last_path.iter().enumerate() {
            if i == last_path.len() - 1 {
                last_node = if is_right {
                    curr.borrow_mut().right.take()
                } else {
                    curr.borrow_mut().left.take()
                };
            } else {
                let next = if is_right {
                    curr.borrow().right.as_ref().unwrap().clone()
                } else {
                    curr.borrow().left.as_ref().unwrap().clone()
                };
                curr = next;
            }
        }

        let last_node = last_node.unwrap();
        if Rc::ptr_eq(&target, &last_node) {
            self.len -= 1;
            return last_node.borrow_mut().value.take();
        }

        let mut b_target = target.borrow_mut();
        let mut b_last = last_node.borrow_mut();
        std::mem::swap(&mut b_target.value, &mut b_last.value);
        let result = b_last.value.take();
        drop(b_target);
        drop(b_last);

        self.len -= 1;
        // Check if we should sift up or down
        let parent = target.borrow().parent.as_ref().and_then(|p| p.upgrade());
        if let Some(p) = parent {
            if target.borrow().value.as_ref().unwrap() < p.borrow().value.as_ref().unwrap() {
                self.sift_up(target);
            } else {
                self.sift_down(target);
            }
        } else {
            self.sift_down(target);
        }

        result
    }

    fn merge(&mut self, other: &mut Self) {
        while let Some(val) = other.pop() {
            self.push(val);
        }
    }

    fn clear(&mut self) {
        self.root = None;
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
