use std::marker::PhantomData;
use std::ptr;

use crate::traits::core::Sequence;

#[derive(Debug)]
struct Node<T> {
    value: T,
    prev: *mut Node<T>,
    next: *mut Node<T>,
}

#[derive(Debug)]
pub struct DoublyLinkedList<T> {
    head: *mut Node<T>,
    tail: *mut Node<T>,
    len: usize,
}

#[derive(Clone, Copy)]
pub struct DoublyCursor<'a, T> {
    index: usize,
    node: *const Node<T>,
    marker: PhantomData<&'a T>,
}

impl<'a, T> DoublyCursor<'a, T> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn value(&self) -> &'a T {
        // SAFETY: Cursor lifetime is tied to immutable borrow of the list.
        unsafe { &(*self.node).value }
    }
}

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            len: 0,
        }
    }
}

impl<T> DoublyLinkedList<T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn node_at_ptr(&self, index: usize) -> *mut Node<T> {
        if index >= self.len {
            return ptr::null_mut();
        }

        if index <= self.len / 2 {
            let mut current = self.head;
            for _ in 0..index {
                // SAFETY: Traversal remains inside nodes owned by this list.
                unsafe {
                    current = (*current).next;
                }
            }
            current
        } else {
            let mut current = self.tail;
            for _ in 0..(self.len - index - 1) {
                // SAFETY: Traversal remains inside nodes owned by this list.
                unsafe {
                    current = (*current).prev;
                }
            }
            current
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head,
            marker: PhantomData,
        }
    }
}

pub struct Iter<'a, T> {
    next: *const Node<T>,
    marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        }

        // SAFETY: Iterator is constructed from immutable list borrow.
        unsafe {
            let node = &*self.next;
            self.next = node.next;
            Some(&node.value)
        }
    }
}

impl<T> Sequence<T> for DoublyLinkedList<T> {
    type Cursor<'a>
        = DoublyCursor<'a, T>
    where
        Self: 'a;

    type MutView<'a>
        = &'a mut T
    where
        Self: 'a,
        T: 'a;

    fn push_front(&mut self, value: T) {
        let node = Box::into_raw(Box::new(Node {
            value,
            prev: ptr::null_mut(),
            next: self.head,
        }));

        // SAFETY: Existing head belongs to this list.
        unsafe {
            if !self.head.is_null() {
                (*self.head).prev = node;
            } else {
                self.tail = node;
            }
        }

        self.head = node;
        self.len += 1;
    }

    fn push_back(&mut self, value: T) {
        let node = Box::into_raw(Box::new(Node {
            value,
            prev: self.tail,
            next: ptr::null_mut(),
        }));

        // SAFETY: Existing tail belongs to this list.
        unsafe {
            if !self.tail.is_null() {
                (*self.tail).next = node;
            } else {
                self.head = node;
            }
        }

        self.tail = node;
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<T> {
        if self.head.is_null() {
            return None;
        }

        // SAFETY: head points to a node allocated for this list.
        unsafe {
            let old_head = self.head;
            self.head = (*old_head).next;
            if !self.head.is_null() {
                (*self.head).prev = ptr::null_mut();
            } else {
                self.tail = ptr::null_mut();
            }
            self.len -= 1;
            Some(Box::from_raw(old_head).value)
        }
    }

    fn pop_back(&mut self) -> Option<T> {
        if self.tail.is_null() {
            return None;
        }

        // SAFETY: tail points to a node allocated for this list.
        unsafe {
            let old_tail = self.tail;
            self.tail = (*old_tail).prev;
            if !self.tail.is_null() {
                (*self.tail).next = ptr::null_mut();
            } else {
                self.head = ptr::null_mut();
            }
            self.len -= 1;
            Some(Box::from_raw(old_tail).value)
        }
    }

    fn cursor_at<'a>(&'a self, index: usize) -> Option<Self::Cursor<'a>> {
        let node = self.node_at_ptr(index);
        if node.is_null() {
            return None;
        }
        Some(DoublyCursor {
            index,
            node,
            marker: PhantomData,
        })
    }

    fn get_mut<'a>(&'a mut self, index: usize) -> Option<Self::MutView<'a>> {
        let node = self.node_at_ptr(index);
        if node.is_null() {
            return None;
        }
        // SAFETY: Mutable borrow of self guarantees exclusive access.
        unsafe { Some(&mut (*node).value) }
    }

    fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl<T> Drop for DoublyLinkedList<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T> std::iter::FromIterator<T> for DoublyLinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        for value in iter {
            list.push_back(value);
        }
        list
    }
}
