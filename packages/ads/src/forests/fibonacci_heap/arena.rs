use std::cell::Ref;

use crate::traits::{core::PriorityQueue, diagnostics::ForestDiagnostics};

use super::safe;

#[derive(Debug)]
pub struct FibonacciNodeView<'a, T> {
    inner: safe::FibonacciNodeView<T>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, T> Clone for FibonacciNodeView<'a, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> FibonacciNodeView<'a, T> {
    pub fn value(&self) -> Ref<'_, T> {
        self.inner.value()
    }

    pub fn degree(&self) -> usize {
        self.inner.degree()
    }

    pub fn child(&self) -> Option<Self> {
        self.inner.child().map(|inner| Self {
            inner,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn sibling(&self) -> Option<Self> {
        self.inner.sibling().map(|inner| Self {
            inner,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn parent(&self) -> Option<Self> {
        self.inner.parent().map(|inner| Self {
            inner,
            _marker: std::marker::PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct FibonacciNodeCursor<T> {
    inner: safe::FibonacciNodeCursor<T>,
}

impl<T> Clone for FibonacciNodeCursor<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> FibonacciNodeCursor<T> {
    pub fn value(&self) -> Ref<'_, T> {
        self.inner.value()
    }

    pub fn node_view<'a>(&self, _heap: &'a FibonacciHeap<T>) -> Option<FibonacciNodeView<'a, T>> {
        Some(FibonacciNodeView {
            inner: self.inner.node_view(),
            _marker: std::marker::PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct FibonacciHeap<T> {
    inner: safe::FibonacciHeap<T>,
}

impl<T: Ord> FibonacciHeap<T> {
    pub fn new() -> Self {
        Self {
            inner: safe::FibonacciHeap::new(),
        }
    }

    pub fn head_view<'a>(&'a self) -> Option<FibonacciNodeView<'a, T>> {
        self.inner.head_view().map(|inner| FibonacciNodeView {
            inner,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn roots<'a>(&'a self) -> Vec<FibonacciNodeView<'a, T>> {
        self.inner
            .roots()
            .into_iter()
            .map(|inner| FibonacciNodeView {
                inner,
                _marker: std::marker::PhantomData,
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn search(&self, value: &T) -> Option<FibonacciNodeCursor<T>> {
        self.inner
            .search(value)
            .map(|inner| FibonacciNodeCursor { inner })
    }

    pub fn min(&self) -> Option<FibonacciNodeCursor<T>> {
        self.inner.min().map(|inner| FibonacciNodeCursor { inner })
    }

    pub fn insert(&mut self, value: T) {
        self.inner.insert(value);
    }

    pub fn merge(&mut self, other: &mut Self) {
        self.inner.merge(&mut other.inner);
    }

    pub fn extract_min(&mut self) -> Option<T> {
        self.inner.extract_min()
    }

    pub fn decrease_key(&mut self, handle: FibonacciNodeCursor<T>, new_value: T) {
        self.inner.decrease_key(handle.inner, new_value);
    }

    pub fn delete(&mut self, handle: FibonacciNodeCursor<T>) -> Option<T> {
        self.inner.delete(handle.inner)
    }

    pub fn delete_value(&mut self, value: &T) -> Option<T> {
        self.inner.delete_value(value)
    }
}

impl<T: Ord> PriorityQueue<T> for FibonacciHeap<T> {
    type Cursor<'a>
        = FibonacciNodeCursor<T>
    where
        Self: 'a;

    type View<'a>
        = FibonacciNodeView<'a, T>
    where
        Self: 'a;

    fn push(&mut self, value: T) {
        FibonacciHeap::insert(self, value)
    }

    fn pop(&mut self) -> Option<T> {
        FibonacciHeap::extract_min(self)
    }

    fn peek<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        FibonacciHeap::min(self)
    }

    fn cursor<'a>(&'a self, value: &T) -> Option<Self::Cursor<'a>> {
        FibonacciHeap::search(self, value)
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        cursor
            .node_view(self)
            .expect("cursor must reference a live node")
    }

    fn remove_cursor<'a>(&mut self, cursor: Self::Cursor<'a>) -> Option<T>
    where
        T: 'a,
    {
        FibonacciHeap::delete(self, cursor)
    }

    fn clear(&mut self) {
        FibonacciHeap::clear(self)
    }

    fn len(&self) -> usize {
        FibonacciHeap::len(self)
    }
}

impl<T: Ord> ForestDiagnostics for FibonacciHeap<T> {
    fn root_count(&self) -> usize {
        self.inner.root_count()
    }

    fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    fn max_root_degree(&self) -> usize {
        self.inner.max_root_degree()
    }
}

impl<T: Ord> FromIterator<T> for FibonacciHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut heap = Self::new();
        for value in iter {
            heap.insert(value);
        }
        heap
    }
}

impl<T: Ord> Default for FibonacciHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}
