use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::traits::core::{Sequence, SequenceMutGuard};

#[cfg(any(test, feature = "bench"))]
use crate::traits::diagnostics::SequenceDiagnostics;

#[cfg(any(test, feature = "bench"))]
use crate::traits::diagnostics::SequenceDiagnostics as SequenceDiagnosticsTrait;

#[derive(Debug, Clone)]
struct Node<T> {
    value: T,
    prev: Option<usize>,
    next: Option<usize>,
}

#[derive(Debug)]
pub struct DoublyLinkedList<T> {
    nodes: Vec<Option<Node<T>>>,
    head: Option<usize>,
    tail: Option<usize>,
    free: Vec<usize>,
    len: usize,
    #[cfg(any(test, feature = "bench"))]
    walk_steps: AtomicUsize,
}

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            head: None,
            tail: None,
            free: Vec::new(),
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
    node_idx: usize,
    list: &'a mut DoublyLinkedList<T>,
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
        let node = self.list.nodes[self.node_idx].as_mut().unwrap();
        f(&mut node.value)
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
        let node_idx = self.list.node_idx_at(self.index).unwrap();
        CursorValue(self.list.nodes[node_idx].as_ref().unwrap().value.clone())
    }
}

impl<T> DoublyLinkedList<T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn alloc_node(&mut self, node: Node<T>) -> usize {
        if let Some(idx) = self.free.pop() {
            self.nodes[idx] = Some(node);
            idx
        } else {
            let idx = self.nodes.len();
            self.nodes.push(Some(node));
            idx
        }
    }

    fn free_node(&mut self, idx: usize) -> T {
        let node = self.nodes[idx].take().unwrap();
        self.free.push(idx);
        node.value
    }

    fn node_idx_at(&self, index: usize) -> Option<usize> {
        if index >= self.len {
            return None;
        }

        if index <= self.len / 2 {
            let mut current = self.head?;
            for _ in 0..index {
                #[cfg(any(test, feature = "bench"))]
                self.walk_steps.fetch_add(1, Ordering::Relaxed);
                current = self.nodes[current].as_ref().unwrap().next?;
            }
            Some(current)
        } else {
            let mut current = self.tail?;
            for _ in 0..(self.len - 1 - index) {
                #[cfg(any(test, feature = "bench"))]
                self.walk_steps.fetch_add(1, Ordering::Relaxed);
                current = self.nodes[current].as_ref().unwrap().prev?;
            }
            Some(current)
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head,
            list: self,
        }
    }
}

pub struct Iter<'a, T> {
    next: Option<usize>,
    list: &'a DoublyLinkedList<T>,
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.next?;
        let node = self.list.nodes[idx].as_ref().unwrap();
        let value = node.value.clone();
        self.next = node.next;
        Some(value)
    }
}

impl<T> Sequence<T> for DoublyLinkedList<T> {
    type Cursor<'a> = DoublyCursor<'a, T> where Self: 'a;
    type MutView<'a> = DoublyMutView<'a, T> where Self: 'a, T: 'a;

    fn push_front(&mut self, value: T) {
        let new_idx = self.alloc_node(Node {
            value,
            prev: None,
            next: self.head,
        });

        if let Some(old_head_idx) = self.head {
            self.nodes[old_head_idx].as_mut().unwrap().prev = Some(new_idx);
        } else {
            self.tail = Some(new_idx);
        }

        self.head = Some(new_idx);
        self.len += 1;
    }

    fn push_back(&mut self, value: T) {
        let new_idx = self.alloc_node(Node {
            value,
            prev: self.tail,
            next: None,
        });

        if let Some(old_tail_idx) = self.tail {
            self.nodes[old_tail_idx].as_mut().unwrap().next = Some(new_idx);
        } else {
            self.head = Some(new_idx);
        }

        self.tail = Some(new_idx);
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<T> {
        let old_head_idx = self.head?;
        let next = self.nodes[old_head_idx].as_ref().unwrap().next;

        if let Some(new_head_idx) = next {
            self.nodes[new_head_idx].as_mut().unwrap().prev = None;
            self.head = Some(new_head_idx);
        } else {
            self.head = None;
            self.tail = None;
        }

        self.len -= 1;
        Some(self.free_node(old_head_idx))
    }

    fn pop_back(&mut self) -> Option<T> {
        let old_tail_idx = self.tail?;
        let prev = self.nodes[old_tail_idx].as_ref().unwrap().prev;

        if let Some(new_tail_idx) = prev {
            self.nodes[new_tail_idx].as_mut().unwrap().next = None;
            self.tail = Some(new_tail_idx);
        } else {
            self.head = None;
            self.tail = None;
        }

        self.len -= 1;
        Some(self.free_node(old_tail_idx))
    }

    fn cursor_at<'a>(&'a self, index: usize) -> Option<Self::Cursor<'a>> {
        if index >= self.len {
            return None;
        }
        Some(DoublyCursor { index, list: self })
    }

    fn get_mut<'a>(&'a mut self, index: usize) -> Option<Self::MutView<'a>> {
        let node_idx = self.node_idx_at(index)?;
        Some(DoublyMutView {
            node_idx,
            list: self,
        })
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.head = None;
        self.tail = None;
        self.free.clear();
        self.len = 0;
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

impl<T> std::iter::FromIterator<T> for DoublyLinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        for value in iter {
            list.push_back(value);
        }
        list
    }
}
