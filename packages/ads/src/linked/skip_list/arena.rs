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

#[derive(Debug, Clone, Copy)]
struct Forward {
    next: Option<Handle>,
    span: usize,
}

#[derive(Debug)]
struct Node<K, V> {
    key: Option<K>,
    value: Option<V>,
    forwards: Vec<Forward>,
}

impl<K, V> Node<K, V> {
    fn head(max_level: usize) -> Self {
        Self {
            key: None,
            value: None,
            forwards: vec![Forward { next: None, span: 1 }; max_level],
        }
    }

    fn new(key: K, value: V, level: usize) -> Self {
        Self {
            key: Some(key),
            value: Some(value),
            forwards: vec![Forward { next: None, span: 0 }; level],
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
            .and_then(|node| node.forwards.get(level).and_then(|f| f.next))
    }

    fn span(&self, handle: Handle, level: usize) -> usize {
        self.get_node(handle)
            .and_then(|node| node.forwards.get(level).map(|f| f.span))
            .unwrap_or(0)
    }

    fn set_forward(&mut self, handle: Handle, level: usize, target: Option<Handle>, span: usize) {
        if let Some(node) = self.get_node_mut(handle) {
            if let Some(f) = node.forwards.get_mut(level) {
                f.next = target;
                f.span = span;
            }
        }
    }

    fn node_at_handle(&self, index: usize) -> Option<Handle> {
        if index >= self.len {
            return None;
        }

        let mut current = self.head;
        let mut pos = 0usize;
        let target = index + 1;

        for level in (0..self.level).rev() {
            while let Some(next) = self.forward(current, level) {
                let span = self.span(current, level);
                if pos + span <= target {
                    pos += span;
                    current = next;
                } else {
                    break;
                }
            }
        }

        if pos == target { Some(current) } else { None }
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
            for f in &mut head.forwards {
                f.next = None;
                f.span = 1;
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

    fn find_update_path(&self, key: &K, update: &mut [Handle], rank: &mut [usize]) -> Option<usize>
    where
        K: Ord,
    {
        let mut current = self.head;
        let mut current_rank = 0usize;
        let mut found_rank = None;

        for level in (0..self.max_level).rev() {
            if level < self.level {
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
                        current_rank += self.span(current, level);
                        current = next;
                        continue;
                    } else if ord == Ordering::Equal {
                        found_rank = Some(current_rank + self.span(current, level));
                    }

                    break;
                }
            }
            update[level] = current;
            rank[level] = current_rank;
        }
        found_rank
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
        self.next = node.forwards[0].next;
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
        let mut rank = vec![0usize; self.max_level];
        if self.find_update_path(&key, &mut update, &mut rank).is_some() {
            let candidate = self.forward(update[0], 0).unwrap();
            let node = self
                .get_node_mut(candidate)
                .expect("candidate should remain live during insertion");
            return Some(
                node.value
                    .replace(value)
                    .expect("non-head node should have value"),
            );
        }

        let new_level = self.random_level();
        if new_level > self.level {
            for i in self.level..new_level {
                rank[i] = 0;
                update[i] = self.head;
                let old_len = self.len;
                self.set_forward(update[i], i, None, old_len + 1);
            }
            self.level = new_level;
        }

        let new_handle = self.alloc_node(Node::new(key, value, new_level));

        for level in 0..new_level {
            let next = self.forward(update[level], level);
            let old_span = self.span(update[level], level);

            self.set_forward(new_handle, level, next, old_span - (rank[0] - rank[level]));
            self.set_forward(update[level], level, Some(new_handle), (rank[0] - rank[level]) + 1);
        }

        for level in new_level..self.level {
            let span = self.span(update[level], level);
            self.set_forward(update[level], level, self.forward(update[level], level), span + 1);
        }

        self.len += 1;
        None
    }

    fn cursor<'a>(&'a self, key: &K) -> Option<Self::Cursor<'a>> {
        let mut update = vec![self.head; self.max_level];
        let mut rank = vec![0usize; self.max_level];
        let found_rank = self.find_update_path(key, &mut update, &mut rank)?;
        let handle = self.forward(update[0], 0)?;
        Some(SkipCursor {
            index: found_rank - 1,
            handle,
            list: self,
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

        let target = self.forward(update[0], 0).unwrap();
        let target_level = self
            .get_node(target)
            .expect("target should remain live during removal")
            .level();

        for level in 0..self.level {
            let predecessor = update[level];
            if level < target_level && self.forward(predecessor, level).is_some_and(|h| h == target) {
                let target_span = self.span(target, level);
                let pred_span = self.span(predecessor, level);
                let next = self.forward(target, level);
                self.set_forward(predecessor, level, next, pred_span + target_span - 1);
            } else {
                let span = self.span(predecessor, level);
                self.set_forward(predecessor, level, self.forward(predecessor, level), span - 1);
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
        let mut update = vec![self.head; self.max_level];
        let mut rank = vec![0usize; self.max_level];
        self.find_update_path(key, &mut update, &mut rank).is_some()
    }

    fn clear(&mut self) {
        self.clear()
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
