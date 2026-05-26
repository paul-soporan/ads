#![allow(dangerous_implicit_autorefs)]

use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ptr;

use crate::traits::core::{Map, OrderedMap};

const DEFAULT_MAX_LEVEL: usize = 8;
const DEFAULT_NUMERATOR: u32 = 1;
const DEFAULT_DENOMINATOR: u32 = 2;

#[derive(Debug)]
struct Forward<K, V> {
    next: *mut Node<K, V>,
    span: usize,
}

impl<K, V> Clone for Forward<K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> Copy for Forward<K, V> {}

#[derive(Debug)]
struct Node<K, V> {
    key: Option<K>,
    value: Option<V>,
    forwards: Vec<Forward<K, V>>,
}

impl<K, V> Node<K, V> {
    fn head(max_level: usize) -> Self {
        let mut forwards = Vec::with_capacity(max_level);
        for _ in 0..max_level {
            forwards.push(Forward { next: ptr::null_mut(), span: 1 });
        }
        Self {
            key: None,
            value: None,
            forwards,
        }
    }

    fn new(key: K, value: V, level: usize) -> Self {
        let mut forwards = Vec::with_capacity(level);
        for _ in 0..level {
            forwards.push(Forward { next: ptr::null_mut(), span: 0 });
        }
        Self {
            key: Some(key),
            value: Some(value),
            forwards,
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
        unsafe {
            (*self.node)
                .key
                .as_ref()
                .expect("cursor should point to keyed node")
        }
    }

    pub fn value(&self) -> &'a V {
        unsafe {
            (*self.node)
                .value
                .as_ref()
                .expect("cursor should point to valued node")
        }
    }

    pub fn level(&self) -> usize {
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

        let mut current = self.head;
        let mut pos = 0usize;
        let target = index + 1;

        unsafe {
            for level in (0..self.level).rev() {
                while !(*current).forwards[level].next.is_null() {
                    let span = (*current).forwards[level].span;
                    if pos + span <= target {
                        pos += span;
                        current = (*current).forwards[level].next;
                    } else {
                        break;
                    }
                }
            }
        }

        if pos == target { current } else { ptr::null_mut() }
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
        unsafe {
            let mut current = (*self.head).forwards[0].next;
            while !current.is_null() {
                let next = (*current).forwards[0].next;
                drop(Box::from_raw(current));
                current = next;
            }

            for slot in &mut (*self.head).forwards {
                slot.next = ptr::null_mut();
                slot.span = 1;
            }
        }

        self.len = 0;
        self.level = 1;
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        let next = unsafe { (*self.head).forwards[0].next as *const Node<K, V> };
        Iter {
            next,
            marker: PhantomData,
        }
    }

    fn find_update_path(&self, key: &K, update: &mut [*mut Node<K, V>], rank: &mut [usize]) -> Option<usize>
    where
        K: Ord,
    {
        let mut current = self.head;
        let mut current_rank = 0usize;
        let mut found_rank = None;

        unsafe {
            for level in (0..self.max_level).rev() {
                if level < self.level {
                    loop {
                        let next = (*current).forwards[level].next;
                        let span = (*current).forwards[level].span;
                        if next.is_null() {
                            break;
                        }

                        let ord = (*next)
                            .key
                            .as_ref()
                            .expect("non-head node should have key")
                            .cmp(key);

                        if ord == Ordering::Less {
                            current_rank += span;
                            current = next;
                            continue;
                        } else if ord == Ordering::Equal {
                            found_rank = Some(current_rank + span);
                        }

                        break;
                    }
                }
                update[level] = current;
                rank[level] = current_rank;
            }
        }
        found_rank
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

        unsafe {
            let node = &*self.next;
            self.next = node.forwards[0].next;
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
        let mut rank = vec![0usize; self.max_level];
        if self.find_update_path(&key, &mut update, &mut rank).is_some() {
            unsafe {
                let candidate = (*update[0]).forwards[0].next;
                return Some(
                    (*candidate)
                        .value
                        .replace(value)
                        .expect("non-head node should have value"),
                );
            }
        }

        let new_level = self.random_level();
        if new_level > self.level {
            for i in self.level..new_level {
                rank[i] = 0;
                update[i] = self.head;
                unsafe {
                    (*update[i]).forwards[i].span = self.len + 1;
                }
            }
            self.level = new_level;
        }

        let new_node = Box::into_raw(Box::new(Node::new(key, value, new_level)));

        unsafe {
            for level in 0..new_level {
                let next = (*update[level]).forwards[level].next;
                let old_span = (*update[level]).forwards[level].span;

                (*new_node).forwards[level].next = next;
                (*new_node).forwards[level].span = old_span - (rank[0] - rank[level]);
                
                (*update[level]).forwards[level].next = new_node;
                (*update[level]).forwards[level].span = (rank[0] - rank[level]) + 1;
            }

            for level in new_level..self.level {
                (*update[level]).forwards[level].span += 1;
            }
        }

        self.len += 1;
        None
    }

    fn cursor<'a>(&'a self, key: &K) -> Option<Self::Cursor<'a>> {
        let mut update = vec![self.head; self.max_level];
        let mut rank = vec![0usize; self.max_level];
        let found_rank = self.find_update_path(key, &mut update, &mut rank)?;
        let node = unsafe { (*update[0]).forwards[0].next };
        if node.is_null() { return None; }
        
        Some(SkipCursor {
            index: found_rank - 1,
            node,
            marker: PhantomData,
        })
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        cursor.view()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let mut update = vec![self.head; self.max_level];
        let mut rank = vec![0usize; self.max_level];
        if self.find_update_path(key, &mut update, &mut rank).is_none() {
            return None;
        }

        unsafe {
            let target = (*update[0]).forwards[0].next;
            if target.is_null() {
                return None;
            }

            let target_level = (*target).level();

            for level in 0..self.level {
                if level < target_level && (*update[level]).forwards[level].next == target {
                    (*update[level]).forwards[level].span += (*target).forwards[level].span - 1;
                    (*update[level]).forwards[level].next = (*target).forwards[level].next;
                } else {
                    (*update[level]).forwards[level].span -= 1;
                }
            }

            while self.level > 1 && (*self.head).forwards[self.level - 1].next.is_null() {
                self.level -= 1;
            }

            self.len -= 1;
            let boxed = Box::from_raw(target);
            boxed.value
        }
    }

    fn contains_key(&self, key: &K) -> bool {
        let mut update = vec![self.head; self.max_level];
        let mut rank = vec![0usize; self.max_level];
        self.find_update_path(key, &mut update, &mut rank).is_some()
    }

    fn clear(&mut self) {
        unsafe {
            let mut current = (*self.head).forwards[0].next;
            while !current.is_null() {
                let next = (*current).forwards[0].next;
                drop(Box::from_raw(current));
                current = next;
            }

            for slot in &mut (*self.head).forwards {
                slot.next = ptr::null_mut();
                slot.span = 1;
            }
        }

        self.len = 0;
        self.level = 1;
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
