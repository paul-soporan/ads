use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::traits::core::{Sequence, SequenceMutGuard};

#[cfg(any(test, feature = "bench"))]
use crate::traits::diagnostics::SequenceDiagnostics;

#[cfg(any(test, feature = "bench"))]
use crate::traits::diagnostics::SequenceDiagnostics as SequenceDiagnosticsTrait;

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
    #[cfg(any(test, feature = "bench"))]
    walk_steps: AtomicUsize,
}

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            len: 0,
            #[cfg(any(test, feature = "bench"))]
            walk_steps: AtomicUsize::new(0),
        }
    }
}

pub struct DoublyCursor<'a, T> {
    index: usize,
    list: &'a DoublyLinkedList<T>,
}

pub struct CursorValue<T>(T);

pub struct DoublyMutView<'a, T> {
    node: *mut Node<T>,
    _marker: PhantomData<&'a mut DoublyLinkedList<T>>,
}

impl<T> Deref for CursorValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> SequenceMutGuard<T> for DoublyMutView<'a, T> {
    fn with_mut<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        unsafe { f(&mut (*self.node).value) }
    }
}

impl<'a, T> Clone for DoublyCursor<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for DoublyCursor<'a, T> {}

impl<'a, T> crate::traits::core::SequenceCursor for DoublyCursor<'a, T> {
    fn index(&self) -> usize {
        self.index
    }
}

impl<'a, T> DoublyCursor<'a, T> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn value(&self) -> CursorValue<T>
    where
        T: Clone,
    {
        let node = self.list.node_at(self.index);
        assert!(!node.is_null(), "cursor node should be live");
        unsafe { CursorValue((*node).value.clone()) }
    }
}

impl<T> DoublyLinkedList<T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn node_at(&self, index: usize) -> *mut Node<T> {
        if index >= self.len {
            return ptr::null_mut();
        }

        if index <= self.len / 2 {
            let mut current = self.head;
            for _ in 0..index {
                #[cfg(any(test, feature = "bench"))]
                self.walk_steps.fetch_add(1, Ordering::Relaxed);
                unsafe {
                    current = (*current).next;
                }
            }
            current
        } else {
            let mut current = self.tail;
            for _ in 0..(self.len - 1 - index) {
                #[cfg(any(test, feature = "bench"))]
                self.walk_steps.fetch_add(1, Ordering::Relaxed);
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
            _marker: PhantomData,
        }
    }
}

pub struct Iter<'a, T> {
    next: *const Node<T>,
    _marker: PhantomData<&'a DoublyLinkedList<T>>,
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        }

        unsafe {
            let current = &*self.next;
            let value = current.value.clone();
            self.next = current.next;
            Some(value)
        }
    }
}

impl<T> Sequence<T> for DoublyLinkedList<T> {
    type Cursor<'a> = DoublyCursor<'a, T> where Self: 'a;
    type MutView<'a> = DoublyMutView<'a, T> where Self: 'a, T: 'a;

    fn push_front(&mut self, value: T) {
        let new_node = Box::into_raw(Box::new(Node {
            value,
            prev: ptr::null_mut(),
            next: self.head,
        }));

        if !self.head.is_null() {
            unsafe {
                (*self.head).prev = new_node;
            }
        } else {
            self.tail = new_node;
        }

        self.head = new_node;
        self.len += 1;
    }

    fn push_back(&mut self, value: T) {
        let new_node = Box::into_raw(Box::new(Node {
            value,
            prev: self.tail,
            next: ptr::null_mut(),
        }));

        if !self.tail.is_null() {
            unsafe {
                (*self.tail).next = new_node;
            }
        } else {
            self.head = new_node;
        }

        self.tail = new_node;
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<T> {
        if self.head.is_null() {
            return None;
        }

        let old_head = self.head;
        unsafe {
            self.head = (*old_head).next;
            if !self.head.is_null() {
                (*self.head).prev = ptr::null_mut();
            } else {
                self.tail = ptr::null_mut();
            }

            self.len -= 1;
            let boxed = Box::from_raw(old_head);
            Some(boxed.value)
        }
    }

    fn pop_back(&mut self) -> Option<T> {
        if self.tail.is_null() {
            return None;
        }

        let old_tail = self.tail;
        unsafe {
            self.tail = (*old_tail).prev;
            if !self.tail.is_null() {
                (*self.tail).next = ptr::null_mut();
            } else {
                self.head = ptr::null_mut();
            }

            self.len -= 1;
            let boxed = Box::from_raw(old_tail);
            Some(boxed.value)
        }
    }

    fn cursor_at<'a>(&'a self, index: usize) -> Option<Self::Cursor<'a>> {
        if index >= self.len {
            return None;
        }
        Some(DoublyCursor { index, list: self })
    }

    fn get_mut<'a>(&'a mut self, index: usize) -> Option<Self::MutView<'a>> {
        let node = self.node_at(index);
        if node.is_null() {
            return None;
        }
        Some(DoublyMutView {
            node,
            _marker: PhantomData,
        })
    }

    fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    fn len(&self) -> usize {
        self.len
    }
}

#[cfg(any(test, feature = "bench"))]
impl<T> SequenceDiagnostics for DoublyLinkedList<T> {
    fn walk_steps(&self) -> usize {
        self.walk_steps.load(Ordering::Relaxed)
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
