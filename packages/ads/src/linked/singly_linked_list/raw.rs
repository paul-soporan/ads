use std::marker::PhantomData;
use std::ptr;

use crate::traits::core::Sequence;

#[derive(Debug)]
struct Node<T> {
    value: T,
    next: *mut Node<T>,
}

#[derive(Debug)]
pub struct SinglyLinkedList<T> {
    head: *mut Node<T>,
    tail: *mut Node<T>,
    len: usize,
}

#[derive(Clone, Copy)]
pub struct SinglyCursor<'a, T> {
    index: usize,
    node: *const Node<T>,
    marker: PhantomData<&'a T>,
}

impl<'a, T> SinglyCursor<'a, T> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn value(&self) -> &'a T {
        // SAFETY: Cursor only exposes immutable access tied to list borrow in cursor_at.
        unsafe { &(*self.node).value }
    }
}

impl<T> Default for SinglyLinkedList<T> {
    fn default() -> Self {
        Self {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            len: 0,
        }
    }
}

impl<T> SinglyLinkedList<T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn node_at_ptr(&self, index: usize) -> *mut Node<T> {
        if index >= self.len {
            return ptr::null_mut();
        }

        let mut current = self.head;
        for _ in 0..index {
            // SAFETY: Traversal only follows links of nodes currently owned by the list.
            unsafe {
                current = (*current).next;
            }
        }
        current
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

        // SAFETY: The iterator is created from an immutable borrow of the list.
        unsafe {
            let node = &*self.next;
            self.next = node.next;
            Some(&node.value)
        }
    }
}

impl<T> Sequence<T> for SinglyLinkedList<T> {
    type Cursor<'a>
        = SinglyCursor<'a, T>
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
            next: self.head,
        }));

        self.head = node;
        if self.tail.is_null() {
            self.tail = node;
        }
        self.len += 1;
    }

    fn push_back(&mut self, value: T) {
        let node = Box::into_raw(Box::new(Node {
            value,
            next: ptr::null_mut(),
        }));

        if self.tail.is_null() {
            self.head = node;
            self.tail = node;
        } else {
            // SAFETY: tail is non-null and points to a node owned by this list.
            unsafe {
                (*self.tail).next = node;
            }
            self.tail = node;
        }
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<T> {
        if self.head.is_null() {
            return None;
        }

        // SAFETY: head points to a node allocated by Box::into_raw in this list.
        unsafe {
            let old_head = self.head;
            self.head = (*old_head).next;
            if self.head.is_null() {
                self.tail = ptr::null_mut();
            }
            self.len -= 1;
            let boxed = Box::from_raw(old_head);
            Some(boxed.value)
        }
    }

    fn pop_back(&mut self) -> Option<T> {
        if self.head.is_null() {
            return None;
        }

        if self.head == self.tail {
            return self.pop_front();
        }

        // SAFETY: head/tail are valid and list has at least two elements.
        unsafe {
            let mut prev = self.head;
            while (*prev).next != self.tail {
                prev = (*prev).next;
            }

            let old_tail = self.tail;
            self.tail = prev;
            (*self.tail).next = ptr::null_mut();
            self.len -= 1;
            let boxed = Box::from_raw(old_tail);
            Some(boxed.value)
        }
    }

    fn cursor_at<'a>(&'a self, index: usize) -> Option<Self::Cursor<'a>> {
        let node = self.node_at_ptr(index);
        if node.is_null() {
            return None;
        }
        Some(SinglyCursor {
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

impl<T> Drop for SinglyLinkedList<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T> std::iter::FromIterator<T> for SinglyLinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        for value in iter {
            list.push_back(value);
        }
        list
    }
}
