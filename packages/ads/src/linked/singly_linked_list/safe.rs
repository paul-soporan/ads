use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use crate::traits::core::{Sequence, SequenceMutGuard};

type Link<T> = Option<Rc<RefCell<Node<T>>>>;
type WeakLink<T> = Option<Weak<RefCell<Node<T>>>>;

#[derive(Debug)]
struct Node<T> {
    value: T,
    next: Link<T>,
}

#[derive(Debug)]
pub struct SinglyLinkedList<T> {
    head: Link<T>,
    tail: WeakLink<T>,
    len: usize,
}

impl<T> Default for SinglyLinkedList<T> {
    fn default() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SinglyCursor<'a, T> {
    index: usize,
    list: &'a SinglyLinkedList<T>,
}

pub struct CursorValue<T>(T);

pub struct SinglyMutView<T> {
    node: Rc<RefCell<Node<T>>>,
}

impl<T> Deref for CursorValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> SequenceMutGuard<T> for SinglyMutView<T> {
    fn with_mut<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut node = self.node.borrow_mut();
        f(&mut node.value)
    }
}

impl<'a, T> SinglyCursor<'a, T> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn value(&self) -> CursorValue<T>
    where
        T: Clone,
    {
        let node = self
            .list
            .node_at(self.index)
            .expect("cursor should point to a live node");
        CursorValue(node.borrow().value.clone())
    }
}

impl<T> SinglyLinkedList<T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn tail_node(&self) -> Link<T> {
        self.tail.as_ref().and_then(Weak::upgrade)
    }

    fn node_at(&self, index: usize) -> Link<T> {
        if index >= self.len {
            return None;
        }

        let mut current = self.head.as_ref()?.clone();
        for _ in 0..index {
            let next = current.borrow().next.as_ref()?.clone();
            current = next;
        }
        Some(current)
    }

    pub fn iter(&self) -> Iter<'_, T>
    where
        T: Clone,
    {
        Iter {
            next: self.head.clone(),
            _marker: PhantomData,
        }
    }
}

pub struct Iter<'a, T> {
    next: Link<T>,
    _marker: PhantomData<&'a SinglyLinkedList<T>>,
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next.take()?;
        let (next, value) = {
            let node = current.borrow();
            (node.next.clone(), node.value.clone())
        };
        self.next = next;
        Some(value)
    }
}

impl<T> Sequence<T> for SinglyLinkedList<T> {
    type Cursor<'a>
        = SinglyCursor<'a, T>
    where
        Self: 'a;

    type MutView<'a>
        = SinglyMutView<T>
    where
        Self: 'a,
        T: 'a;

    fn push_front(&mut self, value: T) {
        let new_head = Rc::new(RefCell::new(Node {
            value,
            next: self.head.take(),
        }));

        if self.tail.is_none() {
            self.tail = Some(Rc::downgrade(&new_head));
        }

        self.head = Some(new_head);
        self.len += 1;
    }

    fn push_back(&mut self, value: T) {
        let new_tail = Rc::new(RefCell::new(Node { value, next: None }));

        if let Some(old_tail) = self.tail_node() {
            old_tail.borrow_mut().next = Some(new_tail.clone());
        } else {
            self.head = Some(new_tail.clone());
        }

        self.tail = Some(Rc::downgrade(&new_tail));
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<T> {
        let old_head = self.head.take()?;
        let next = old_head.borrow_mut().next.take();
        self.head = next;

        if self.head.is_none() {
            self.tail = None;
        }

        self.len -= 1;
        Some(
            Rc::try_unwrap(old_head)
                .unwrap_or_else(|_| panic!("singly node unexpectedly shared during front removal"))
                .into_inner()
                .value,
        )
    }

    fn pop_back(&mut self) -> Option<T> {
        match self.len {
            0 => None,
            1 => self.pop_front(),
            _ => {
                let tail_ptr = self.tail.as_ref()?.as_ptr();
                let mut current = self.head.as_ref()?.clone();

                loop {
                    let next = current
                        .borrow()
                        .next
                        .as_ref()
                        .cloned()
                        .expect("tail should be reachable");

                    if Rc::as_ptr(&next) == tail_ptr {
                        current.borrow_mut().next = None;
                        self.tail = Some(Rc::downgrade(&current));
                        self.len -= 1;
                        return Some(
                            Rc::try_unwrap(next)
                                .unwrap_or_else(|_| {
                                    panic!("singly node unexpectedly shared during back removal")
                                })
                                .into_inner()
                                .value,
                        );
                    }

                    current = next;
                }
            }
        }
    }

    fn cursor_at<'a>(&'a self, index: usize) -> Option<Self::Cursor<'a>> {
        if index >= self.len {
            return None;
        }

        Some(SinglyCursor { index, list: self })
    }

    fn get_mut<'a>(&'a mut self, index: usize) -> Option<Self::MutView<'a>> {
        let node = self.node_at(index)?;
        Some(SinglyMutView { node })
    }

    fn clear(&mut self) {
        self.head = None;
        self.tail = None;
        self.len = 0;
    }

    fn len(&self) -> usize {
        self.len
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
