#![allow(dangerous_implicit_autorefs)]

use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ptr;

use crate::traits::core::{Map, OrderedMap};

const DEFAULT_MAX_LEVEL: usize = 8;
const DEFAULT_NUMERATOR: u32 = 1;
const DEFAULT_DENOMINATOR: u32 = 2;

#[derive(Debug)]
struct Node<K, V> {
    key: Option<K>,
    value: Option<V>,
    forwards: Vec<*mut Node<K, V>>,
}

impl<K, V> Node<K, V> {
    fn head(max_level: usize) -> Self {
        Self {
            key: None,
            value: None,
            forwards: vec![ptr::null_mut(); max_level],
        }
    }

    fn new(key: K, value: V, level: usize) -> Self {
        Self {
            key: Some(key),
            value: Some(value),
            forwards: vec![ptr::null_mut(); level],
        }
    }

    fn level(&self) -> usize {
        self.forwards.len()
    }
}

#[derive(Debug)]
pub struct SkipList<K, V> {
    head: *mut Node<K, V>,
    len: usize,
    level: usize,
    max_level: usize,
    numerator: u32,
    denominator: u32,
}

pub struct SkipCursor<'a, K, V> {
    index: usize,
    node: *const Node<K, V>,
    marker: PhantomData<&'a SkipList<K, V>>,
}

pub struct SkipView<'a, K, V> {
    key: &'a K,
    value: &'a V,
    level: usize,
    index: usize,
}

impl<'a, K, V> Clone for SkipCursor<'a, K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, K, V> Copy for SkipCursor<'a, K, V> {}

impl<'a, K, V> Clone for SkipView<'a, K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, K, V> Copy for SkipView<'a, K, V> {}

impl<'a, K, V> SkipCursor<'a, K, V> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn key(&self) -> &'a K {
        // SAFETY: cursor lifetime is tied to immutable list borrow and only points to keyed nodes.
        unsafe {
            (*self.node)
                .key
                .as_ref()
                .expect("cursor should point to keyed node")
        }
    }

    pub fn value(&self) -> &'a V {
        // SAFETY: cursor lifetime is tied to immutable list borrow and only points to valued nodes.
        unsafe {
            (*self.node)
                .value
                .as_ref()
                .expect("cursor should point to valued node")
        }
    }

    pub fn level(&self) -> usize {
        // SAFETY: cursor points to a live node owned by this list.
        unsafe { (*self.node).level() }
    }

    pub fn view(&self) -> SkipView<'a, K, V> {
        SkipView {
            key: self.key(),
            value: self.value(),
            level: self.level(),
            index: self.index,
        }
    }
}

impl<'a, K, V> SkipView<'a, K, V> {
    pub fn key(&self) -> &'a K {
        self.key
    }

    pub fn value(&self) -> &'a V {
        self.value
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

impl<K, V> SkipList<K, V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_probability(numerator: u32, denominator: u32) -> Self {
        Self::with_config(DEFAULT_MAX_LEVEL, numerator, denominator)
    }

    pub fn with_config(max_level: usize, numerator: u32, denominator: u32) -> Self {
        assert!(max_level > 0, "max_level must be > 0");
        assert!(denominator > 0, "denominator must be > 0");
        assert!(numerator <= denominator, "numerator must be <= denominator");

        let head = Box::into_raw(Box::new(Node::head(max_level)));
        Self {
            head,
            len: 0,
            level: 1,
            max_level,
            numerator,
            denominator,
        }
    }

    pub fn probability(&self) -> (u32, u32) {
        (self.numerator, self.denominator)
    }

    pub fn max_level(&self) -> usize {
        self.max_level
    }

    fn random_level(&self) -> usize {
        if self.numerator == 1 && self.denominator == 2 {
            return (fastrand::u32(..).trailing_zeros() as usize + 1).min(self.max_level);
        }

        let mut level = 1usize;
        while level < self.max_level && fastrand::u32(0..self.denominator) < self.numerator {
            level += 1;
        }
        level
    }

    fn node_at_ptr(&self, index: usize) -> *mut Node<K, V> {
        if index >= self.len {
            return ptr::null_mut();
        }

        // SAFETY: head is always allocated during list lifetime.
        let mut current = unsafe { (*self.head).forwards[0] };
        for _ in 0..index {
            if current.is_null() {
                return ptr::null_mut();
            }
            // SAFETY: current points to a live node in level-0 chain.
            current = unsafe { (*current).forwards[0] };
        }

        current
    }

    pub fn cursor_at(&self, index: usize) -> Option<SkipCursor<'_, K, V>> {
        let node = self.node_at_ptr(index);
        if node.is_null() {
            return None;
        }

        Some(SkipCursor {
            index,
            node,
            marker: PhantomData,
        })
    }

    pub fn clear(&mut self) {
        // SAFETY: head exists for the lifetime of the list.
        let mut current = unsafe { (*self.head).forwards[0] };
        while !current.is_null() {
            // SAFETY: current points to live node, read next before deallocation.
            let next = unsafe { (*current).forwards[0] };
            // SAFETY: each node is dropped exactly once while walking level-0 chain.
            unsafe {
                drop(Box::from_raw(current));
            }
            current = next;
        }

        // SAFETY: head is valid and can be reset in-place.
        unsafe {
            for slot in &mut (*self.head).forwards {
                *slot = ptr::null_mut();
            }
        }

        self.len = 0;
        self.level = 1;
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        // SAFETY: head is valid and level-0 forward pointer may be null.
        let next = unsafe { (*self.head).forwards[0] as *const Node<K, V> };
        Iter {
            next,
            marker: PhantomData,
        }
    }

    fn search_node(&self, key: &K) -> *mut Node<K, V>
    where
        K: Ord,
    {
        let mut current = self.head;

        for level in (0..self.level).rev() {
            loop {
                // SAFETY: current is always a live node in the structure.
                let next = unsafe { (*current).forwards[level] };
                if next.is_null() {
                    break;
                }

                // SAFETY: next is live and non-head, so key is present.
                let ord = unsafe {
                    (*next)
                        .key
                        .as_ref()
                        .expect("non-head node should have key")
                        .cmp(key)
                };

                match ord {
                    Ordering::Less => current = next,
                    Ordering::Equal => return next,
                    Ordering::Greater => break,
                }
            }
        }

        // SAFETY: current is valid; candidate may be null.
        let candidate = unsafe { (*current).forwards[0] };
        if candidate.is_null() {
            return ptr::null_mut();
        }

        // SAFETY: candidate is non-null and in structure.
        let is_match = unsafe {
            (*candidate)
                .key
                .as_ref()
                .expect("non-head node should have key")
                == key
        };

        if is_match { candidate } else { ptr::null_mut() }
    }

    fn index_of_key(&self, key: &K) -> Option<usize>
    where
        K: Ord,
    {
        // SAFETY: head is valid.
        let mut current = unsafe { (*self.head).forwards[0] };
        let mut index = 0usize;

        while !current.is_null() {
            // SAFETY: current is live and non-head.
            let ord = unsafe {
                (*current)
                    .key
                    .as_ref()
                    .expect("non-head node should have key")
                    .cmp(key)
            };

            match ord {
                Ordering::Less => {
                    index += 1;
                    // SAFETY: current is live.
                    current = unsafe { (*current).forwards[0] };
                }
                Ordering::Equal => return Some(index),
                Ordering::Greater => return None,
            }
        }

        None
    }

    fn find_update_path(&self, key: &K, update: &mut [*mut Node<K, V>])
    where
        K: Ord,
    {
        let mut current = self.head;

        for level in (0..self.level).rev() {
            loop {
                // SAFETY: current is always live.
                let next = unsafe { (*current).forwards[level] };
                if next.is_null() {
                    break;
                }

                // SAFETY: next is live and non-head.
                let ord = unsafe {
                    (*next)
                        .key
                        .as_ref()
                        .expect("non-head node should have key")
                        .cmp(key)
                };

                if ord == Ordering::Less {
                    current = next;
                    continue;
                }

                break;
            }

            update[level] = current;
        }
    }
}

impl<K, V> Default for SkipList<K, V> {
    fn default() -> Self {
        Self::with_config(DEFAULT_MAX_LEVEL, DEFAULT_NUMERATOR, DEFAULT_DENOMINATOR)
    }
}

impl<K, V> Drop for SkipList<K, V> {
    fn drop(&mut self) {
        self.clear();
        // SAFETY: head is uniquely owned and dropped exactly once here.
        unsafe {
            drop(Box::from_raw(self.head));
        }
    }
}

pub struct Iter<'a, K, V> {
    next: *const Node<K, V>,
    marker: PhantomData<&'a SkipList<K, V>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            return None;
        }

        // SAFETY: iterator is tied to immutable borrow of list and follows level-0 links.
        unsafe {
            let node = &*self.next;
            self.next = node.forwards[0];
            Some((
                node.key.as_ref().expect("iterator node should have key"),
                node.value
                    .as_ref()
                    .expect("iterator node should have value"),
            ))
        }
    }
}

impl<K: Ord, V> Map<K, V> for SkipList<K, V> {
    type Cursor<'a>
        = SkipCursor<'a, K, V>
    where
        Self: 'a;

    type View<'a>
        = SkipView<'a, K, V>
    where
        Self: 'a;

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        let mut update = vec![self.head; self.max_level];
        self.find_update_path(&key, &mut update);

        // SAFETY: update[0] always points to a live predecessor.
        let candidate = unsafe { (*update[0]).forwards[0] };
        if !candidate.is_null() {
            // SAFETY: candidate is live and non-head.
            let is_equal = unsafe {
                (*candidate)
                    .key
                    .as_ref()
                    .expect("non-head node should have key")
                    == &key
            };

            if is_equal {
                // SAFETY: candidate is live and has value.
                return unsafe {
                    Some(
                        (*candidate)
                            .value
                            .replace(value)
                            .expect("non-head node should have value"),
                    )
                };
            }
        }

        let new_level = self.random_level();
        if new_level > self.level {
            for slot in update.iter_mut().take(new_level).skip(self.level) {
                *slot = self.head;
            }
            self.level = new_level;
        }

        let new_node = Box::into_raw(Box::new(Node::new(key, value, new_level)));

        for (level, predecessor) in update.iter().enumerate().take(new_level) {
            // SAFETY: predecessor is live and has this level available.
            let next = unsafe { (**predecessor).forwards[level] };
            // SAFETY: new_node is newly allocated and writable.
            unsafe {
                (*new_node).forwards[level] = next;
            }
            // SAFETY: predecessor is live and writable.
            unsafe {
                (**predecessor).forwards[level] = new_node;
            }
        }

        self.len += 1;
        None
    }

    fn cursor<'a>(&'a self, key: &K) -> Option<Self::Cursor<'a>> {
        let node = self.search_node(key);
        if node.is_null() {
            return None;
        }

        let index = self.index_of_key(key)?;
        Some(SkipCursor {
            index,
            node,
            marker: PhantomData,
        })
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        cursor.view()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let mut update = vec![self.head; self.max_level];
        self.find_update_path(key, &mut update);

        // SAFETY: update[0] always points to a live predecessor.
        let target = unsafe { (*update[0]).forwards[0] };
        if target.is_null() {
            return None;
        }

        // SAFETY: target is live and non-head.
        let is_match = unsafe {
            (*target)
                .key
                .as_ref()
                .expect("non-head node should have key")
                == key
        };
        if !is_match {
            return None;
        }

        // SAFETY: target is live.
        let target_level = unsafe { (*target).level() };

        for (level, predecessor) in update.iter().enumerate().take(target_level) {
            // SAFETY: predecessor is live.
            let points_to_target = unsafe { (**predecessor).forwards[level] == target };
            if points_to_target {
                // SAFETY: target is live.
                let successor = unsafe { (*target).forwards[level] };
                // SAFETY: predecessor is live and writable.
                unsafe {
                    (**predecessor).forwards[level] = successor;
                }
            }
        }

        while self.level > 1 {
            // SAFETY: head is live.
            let top = unsafe { (*self.head).forwards[self.level - 1] };
            if !top.is_null() {
                break;
            }
            self.level -= 1;
        }

        self.len -= 1;

        // SAFETY: target has been detached from all levels where it appears.
        let boxed = unsafe { Box::from_raw(target) };
        boxed.value
    }

    fn contains_key(&self, key: &K) -> bool {
        !self.search_node(key).is_null()
    }

    fn clear(&mut self) {
        Self::clear(self)
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl<K: Ord, V> OrderedMap<K, V> for SkipList<K, V> {
    fn first_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        self.cursor_at(0)
    }

    fn last_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        if self.len == 0 {
            None
        } else {
            self.cursor_at(self.len - 1)
        }
    }
}

impl<K: Ord, V> std::iter::FromIterator<(K, V)> for SkipList<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut list = Self::new();
        for (key, value) in iter {
            let _ = list.insert(key, value);
        }
        list
    }
}
