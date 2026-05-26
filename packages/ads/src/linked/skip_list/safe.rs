use std::cell::RefCell;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use crate::traits::core::{Map, OrderedMap};

const DEFAULT_MAX_LEVEL: usize = 8;
const DEFAULT_NUMERATOR: u32 = 1;
const DEFAULT_DENOMINATOR: u32 = 2;

type Link<K, V> = Option<Rc<RefCell<Node<K, V>>>>;

#[derive(Debug)]
struct Forward<K, V> {
    next: Link<K, V>,
    span: usize,
}

impl<K, V> Clone for Forward<K, V> {
    fn clone(&self) -> Self {
        Self {
            next: self.next.clone(),
            span: self.span,
        }
    }
}

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
            forwards.push(Forward { next: None, span: 1 });
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
            forwards.push(Forward { next: None, span: 0 });
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

#[derive(Clone)]
pub struct ValueRef<T>(T);

impl<T> Deref for ValueRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct SkipCursor<'a, K, V> {
    index: usize,
    node: Rc<RefCell<Node<K, V>>>,
    marker: PhantomData<&'a SkipList<K, V>>,
}

pub struct SkipView<K, V> {
    node: Rc<RefCell<Node<K, V>>>,
    index: usize,
}

impl<'a, K, V> Clone for SkipCursor<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            node: self.node.clone(),
            marker: PhantomData,
        }
    }
}

impl<K, V> Clone for SkipView<K, V> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            index: self.index,
        }
    }
}

impl<'a, K, V> SkipCursor<'a, K, V> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn key(&self) -> ValueRef<K>
    where
        K: Clone,
    {
        let node = self.node.borrow();
        ValueRef(
            node.key
                .as_ref()
                .expect("cursor should point to a keyed node")
                .clone(),
        )
    }

    pub fn value(&self) -> ValueRef<V>
    where
        V: Clone,
    {
        let node = self.node.borrow();
        ValueRef(
            node.value
                .as_ref()
                .expect("cursor should point to a valued node")
                .clone(),
        )
    }

    pub fn level(&self) -> usize {
        self.node.borrow().level()
    }

    pub fn view(&self) -> SkipView<K, V> {
        SkipView {
            node: self.node.clone(),
            index: self.index,
        }
    }
}

impl<K, V> SkipView<K, V> {
    pub fn key(&self) -> ValueRef<K>
    where
        K: Clone,
    {
        let node = self.node.borrow();
        ValueRef(
            node.key
                .as_ref()
                .expect("view should point to a keyed node")
                .clone(),
        )
    }

    pub fn value(&self) -> ValueRef<V>
    where
        V: Clone,
    {
        let node = self.node.borrow();
        ValueRef(
            node.value
                .as_ref()
                .expect("view should point to a valued node")
                .clone(),
        )
    }

    pub fn level(&self) -> usize {
        self.node.borrow().level()
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

#[derive(Debug)]
pub struct SkipList<K, V> {
    head: Rc<RefCell<Node<K, V>>>,
    len: usize,
    level: usize,
    max_level: usize,
    numerator: u32,
    denominator: u32,
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

        Self {
            head: Rc::new(RefCell::new(Node::head(max_level))),
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

    fn node_at(&self, index: usize) -> Option<Rc<RefCell<Node<K, V>>>> {
        if index >= self.len {
            return None;
        }

        let mut current = self.head.clone();
        let mut pos = 0usize;
        let target = index + 1;

        for level in (0..self.level).rev() {
            loop {
                let (next, span) = {
                    let b = current.borrow();
                    let f = &b.forwards[level];
                    (f.next.clone(), f.span)
                };
                if let Some(n) = next
                    && pos + span <= target
                {
                    pos += span;
                    current = n;
                } else {
                    break;
                }
            }
        }

        if pos == target {
            Some(current)
        } else {
            None
        }
    }

    pub fn cursor_at(&self, index: usize) -> Option<SkipCursor<'_, K, V>> {
        let node = self.node_at(index)?;
        Some(SkipCursor {
            index,
            node,
            marker: PhantomData,
        })
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            next: self.head.borrow().forwards[0].next.clone(),
            marker: PhantomData,
        }
    }

    fn find_update_path(
        &self,
        key: &K,
        update: &mut [Rc<RefCell<Node<K, V>>>],
        rank: &mut [usize],
    ) -> Option<usize> where
        K: Ord,
    {
        let mut current = self.head.clone();
        let mut current_rank = 0usize;
        let mut found_rank = None;

        for level in (0..self.max_level).rev() {
            if level < self.level {
                loop {
                    let (next, span) = {
                        let b = current.borrow();
                        let f = &b.forwards[level];
                        (f.next.clone(), f.span)
                    };
                    let Some(next_node) = next else {
                        break;
                    };

                    let ord = {
                        let borrowed = next_node.borrow();
                        borrowed
                            .key
                            .as_ref()
                            .expect("non-head node should have key")
                            .cmp(key)
                    };

                    if ord == Ordering::Less {
                        current_rank += span;
                        current = next_node;
                        continue;
                    } else if ord == Ordering::Equal {
                        found_rank = Some(current_rank + span);
                    }

                    break;
                }
            }
            update[level] = current.clone();
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
    next: Link<K, V>,
    marker: PhantomData<&'a SkipList<K, V>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Clone,
    V: Clone,
{
    type Item = (ValueRef<K>, ValueRef<V>);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next.take()?;
        let (next, key, value) = {
            let borrowed = current.borrow();
            (
                borrowed.forwards[0].next.clone(),
                borrowed
                    .key
                    .as_ref()
                    .expect("non-head node should have key")
                    .clone(),
                borrowed
                    .value
                    .as_ref()
                    .expect("non-head node should have value")
                    .clone(),
            )
        };

        self.next = next;
        Some((ValueRef(key), ValueRef(value)))
    }
}

impl<K: Ord, V> Map<K, V> for SkipList<K, V> {
    type Cursor<'a>
        = SkipCursor<'a, K, V>
    where
        Self: 'a;

    type View<'a>
        = SkipView<K, V>
    where
        Self: 'a;

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        let mut update = vec![self.head.clone(); self.max_level];
        let mut rank = vec![0usize; self.max_level];
        if self.find_update_path(&key, &mut update, &mut rank).is_some() {
            let candidate = update[0].borrow().forwards[0].next.clone().unwrap();
            let mut candidate_mut = candidate.borrow_mut();
            return Some(
                candidate_mut
                    .value
                    .replace(value)
                    .expect("non-head node should have value"),
            );
        }

        let new_level = self.random_level();
        if new_level > self.level {
            for i in self.level..new_level {
                rank[i] = 0;
                update[i] = self.head.clone();
                update[i].borrow_mut().forwards[i].span = self.len + 1;
            }
            self.level = new_level;
        }

        let new_node = Rc::new(RefCell::new(Node::new(key, value, new_level)));

        for level in 0..new_level {
            let mut updater = update[level].borrow_mut();
            let next = updater.forwards[level].next.take();
            let old_span = updater.forwards[level].span;

            new_node.borrow_mut().forwards[level] = Forward {
                next,
                span: old_span - (rank[0] - rank[level]),
            };
            updater.forwards[level] = Forward {
                next: Some(new_node.clone()),
                span: (rank[0] - rank[level]) + 1,
            };
        }

        for level in new_level..self.level {
            update[level].borrow_mut().forwards[level].span += 1;
        }

        self.len += 1;
        None
    }

    fn cursor<'a>(&'a self, key: &K) -> Option<Self::Cursor<'a>> {
        let mut update = vec![self.head.clone(); self.max_level];
        let mut rank = vec![0usize; self.max_level];
        let found_rank = self.find_update_path(key, &mut update, &mut rank)?;
        let node = update[0].borrow().forwards[0].next.clone()?;
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
        let mut update = vec![self.head.clone(); self.max_level];
        let mut rank = vec![0usize; self.max_level];
        if self.find_update_path(key, &mut update, &mut rank).is_none() {
            return None;
        }

        let target = update[0].borrow().forwards[0].next.clone().unwrap();
        let target_level = target.borrow().level();

        for level in 0..self.level {
            let mut updater = update[level].borrow_mut();
            if level < target_level
                && updater.forwards[level]
                    .next
                    .as_ref()
                    .is_some_and(|candidate| Rc::ptr_eq(candidate, &target))
            {
                let target_borrow = target.borrow();
                updater.forwards[level].span += target_borrow.forwards[level].span - 1;
                updater.forwards[level].next = target_borrow.forwards[level].next.clone();
            } else {
                updater.forwards[level].span -= 1;
            }
        }

        while self.level > 1 && self.head.borrow().forwards[self.level - 1].next.is_none() {
            self.level -= 1;
        }

        self.len -= 1;
        Some(
            target
                .borrow_mut()
                .value
                .take()
                .expect("non-head node should have value"),
        )
    }

    fn contains_key(&self, key: &K) -> bool {
        let mut update = vec![self.head.clone(); self.max_level];
        let mut rank = vec![0usize; self.max_level];
        self.find_update_path(key, &mut update, &mut rank).is_some()
    }

    fn clear(&mut self) {
        self.head = Rc::new(RefCell::new(Node::head(self.max_level)));
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
