use std::cmp::Ordering;

use crate::traits::core::{Map, OrderedMap};

const DEFAULT_MAX_LEVEL: usize = 8;
const DEFAULT_NUMERATOR: u32 = 1;
const DEFAULT_DENOMINATOR: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Handle {
    index: usize,
    generation: u64,
}

#[derive(Debug)]
struct Node<K, V> {
    key: Option<K>,
    value: Option<V>,
    forwards: Vec<Option<Handle>>,
}

impl<K, V> Node<K, V> {
    fn head(max_level: usize) -> Self {
        Self {
            key: None,
            value: None,
            forwards: vec![None; max_level],
        }
    }

    fn new(key: K, value: V, level: usize) -> Self {
        Self {
            key: Some(key),
            value: Some(value),
            forwards: vec![None; level],
        }
    }

    fn level(&self) -> usize {
        self.forwards.len()
    }
}

#[derive(Debug)]
struct Slot<K, V> {
    node: Option<Node<K, V>>,
    generation: u64,
}

#[derive(Debug)]
pub struct SkipList<K, V> {
    head: Handle,
    len: usize,
    level: usize,
    max_level: usize,
    slots: Vec<Slot<K, V>>,
    free: Vec<usize>,
    numerator: u32,
    denominator: u32,
}

pub struct SkipCursor<'a, K, V> {
    index: usize,
    handle: Handle,
    list: &'a SkipList<K, V>,
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
        self.list
            .get_node(self.handle)
            .and_then(|node| node.key.as_ref())
            .expect("cursor should point to keyed node")
    }

    pub fn value(&self) -> &'a V {
        self.list
            .get_node(self.handle)
            .and_then(|node| node.value.as_ref())
            .expect("cursor should point to valued node")
    }

    pub fn level(&self) -> usize {
        self.list
            .get_node(self.handle)
            .expect("cursor should point to live node")
            .level()
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

        let slots = vec![Slot {
            node: Some(Node::head(max_level)),
            generation: 0,
        }];

        Self {
            head: Handle {
                index: 0,
                generation: 0,
            },
            len: 0,
            level: 1,
            max_level,
            slots,
            free: Vec::new(),
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

    fn alloc_node(&mut self, node: Node<K, V>) -> Handle {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index];
            slot.node = Some(node);
            return Handle {
                index,
                generation: slot.generation,
            };
        }

        let index = self.slots.len();
        self.slots.push(Slot {
            node: Some(node),
            generation: 0,
        });
        Handle {
            index,
            generation: 0,
        }
    }

    fn get_node(&self, handle: Handle) -> Option<&Node<K, V>> {
        let slot = self.slots.get(handle.index)?;
        if slot.generation != handle.generation {
            return None;
        }
        slot.node.as_ref()
    }

    fn get_node_mut(&mut self, handle: Handle) -> Option<&mut Node<K, V>> {
        let slot = self.slots.get_mut(handle.index)?;
        if slot.generation != handle.generation {
            return None;
        }
        slot.node.as_mut()
    }

    fn free_node(&mut self, handle: Handle) -> Option<Node<K, V>> {
        let slot = self.slots.get_mut(handle.index)?;
        if slot.generation != handle.generation {
            return None;
        }

        let node = slot.node.take()?;
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(handle.index);
        Some(node)
    }

    fn forward(&self, handle: Handle, level: usize) -> Option<Handle> {
        self.get_node(handle)
            .and_then(|node| node.forwards.get(level).copied().flatten())
    }

    fn set_forward(&mut self, handle: Handle, level: usize, target: Option<Handle>) {
        if let Some(node) = self.get_node_mut(handle) {
            if let Some(slot) = node.forwards.get_mut(level) {
                *slot = target;
            }
        }
    }

    fn node_at_handle(&self, index: usize) -> Option<Handle> {
        if index >= self.len {
            return None;
        }

        let mut current = self.forward(self.head, 0);
        for _ in 0..index {
            current = self.forward(current?, 0);
        }
        current
    }

    pub fn cursor_at(&self, index: usize) -> Option<SkipCursor<'_, K, V>> {
        let handle = self.node_at_handle(index)?;
        Some(SkipCursor {
            index,
            handle,
            list: self,
        })
    }

    pub fn clear(&mut self) {
        let mut current = self.forward(self.head, 0);
        while let Some(handle) = current {
            current = self.forward(handle, 0);
            let _ = self.free_node(handle);
        }

        if let Some(head) = self.get_node_mut(self.head) {
            for pointer in &mut head.forwards {
                *pointer = None;
            }
        }

        self.len = 0;
        self.level = 1;
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            list: self,
            next: self.forward(self.head, 0),
        }
    }

    fn find_update_path(&self, key: &K, update: &mut [Handle])
    where
        K: Ord,
    {
        let mut current = self.head;

        for level in (0..self.level).rev() {
            loop {
                let Some(next) = self.forward(current, level) else {
                    break;
                };

                let ord = self
                    .get_node(next)
                    .and_then(|node| node.key.as_ref())
                    .expect("non-head node should have key")
                    .cmp(key);

                if ord == Ordering::Less {
                    current = next;
                    continue;
                }

                break;
            }

            update[level] = current;
        }
    }

    fn search_node(&self, key: &K) -> Option<Handle>
    where
        K: Ord,
    {
        let mut current = self.head;

        for level in (0..self.level).rev() {
            loop {
                let Some(next) = self.forward(current, level) else {
                    break;
                };

                let ord = self
                    .get_node(next)
                    .and_then(|node| node.key.as_ref())
                    .expect("non-head node should have key")
                    .cmp(key);

                match ord {
                    Ordering::Less => current = next,
                    Ordering::Equal => return Some(next),
                    Ordering::Greater => break,
                }
            }
        }

        let candidate = self.forward(current, 0)?;
        let is_match = self
            .get_node(candidate)
            .and_then(|node| node.key.as_ref())
            .is_some_and(|candidate_key| candidate_key == key);

        is_match.then_some(candidate)
    }

    fn index_of_key(&self, key: &K) -> Option<usize>
    where
        K: Ord,
    {
        let mut current = self.forward(self.head, 0);
        let mut index = 0usize;

        while let Some(handle) = current {
            let ord = self
                .get_node(handle)
                .and_then(|node| node.key.as_ref())
                .expect("non-head node should have key")
                .cmp(key);

            match ord {
                Ordering::Less => {
                    index += 1;
                    current = self.forward(handle, 0);
                }
                Ordering::Equal => return Some(index),
                Ordering::Greater => return None,
            }
        }

        None
    }
}

impl<K, V> Default for SkipList<K, V> {
    fn default() -> Self {
        Self::with_config(DEFAULT_MAX_LEVEL, DEFAULT_NUMERATOR, DEFAULT_DENOMINATOR)
    }
}

pub struct Iter<'a, K, V> {
    list: &'a SkipList<K, V>,
    next: Option<Handle>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let handle = self.next?;
        let node = self.list.get_node(handle)?;
        self.next = node.forwards[0];
        Some((
            node.key.as_ref().expect("iterator node should have key"),
            node.value
                .as_ref()
                .expect("iterator node should have value"),
        ))
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

        if let Some(candidate) = self.forward(update[0], 0) {
            let is_equal = self
                .get_node(candidate)
                .and_then(|node| node.key.as_ref())
                .is_some_and(|candidate_key| candidate_key == &key);

            if is_equal {
                let node = self
                    .get_node_mut(candidate)
                    .expect("candidate should remain live during insertion");
                return Some(
                    node.value
                        .replace(value)
                        .expect("non-head node should have value"),
                );
            }
        }

        let new_level = self.random_level();
        if new_level > self.level {
            for slot in update.iter_mut().take(new_level).skip(self.level) {
                *slot = self.head;
            }
            self.level = new_level;
        }

        let new_handle = self.alloc_node(Node::new(key, value, new_level));

        for (level, predecessor) in update.iter().enumerate().take(new_level) {
            let next = self.forward(*predecessor, level);
            self.set_forward(new_handle, level, next);
            self.set_forward(*predecessor, level, Some(new_handle));
        }

        self.len += 1;
        None
    }

    fn cursor<'a>(&'a self, key: &K) -> Option<Self::Cursor<'a>> {
        let handle = self.search_node(key)?;
        let index = self.index_of_key(key)?;
        Some(SkipCursor {
            index,
            handle,
            list: self,
        })
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        cursor.view()
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let mut update = vec![self.head; self.max_level];
        self.find_update_path(key, &mut update);

        let target = self.forward(update[0], 0)?;
        let is_match = self
            .get_node(target)
            .and_then(|node| node.key.as_ref())
            .is_some_and(|candidate_key| candidate_key == key);

        if !is_match {
            return None;
        }

        let target_level = self
            .get_node(target)
            .expect("target should remain live during removal")
            .level();

        for (level, predecessor) in update.iter().enumerate().take(target_level) {
            let points_to_target = self
                .forward(*predecessor, level)
                .is_some_and(|h| h == target);
            if points_to_target {
                let successor = self.forward(target, level);
                self.set_forward(*predecessor, level, successor);
            }
        }

        while self.level > 1 && self.forward(self.head, self.level - 1).is_none() {
            self.level -= 1;
        }

        self.len -= 1;
        let removed = self
            .free_node(target)
            .expect("target should still be live while removing");
        removed.value
    }

    fn contains_key(&self, key: &K) -> bool {
        self.search_node(key).is_some()
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
