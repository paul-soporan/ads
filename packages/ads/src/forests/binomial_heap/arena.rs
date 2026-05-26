use std::sync::Arc;

use crate::traits::{core::PriorityQueue, diagnostics::ForestDiagnostics};

#[derive(Debug)]
struct BinomialNode<T> {
    value: Arc<T>,
    heap_pos: usize,
}

#[derive(Debug)]
pub struct BinomialNodeView<'a, T> {
    heap: &'a BinomialHeap<T>,
    node_id: usize,
}

#[derive(Debug)]
pub struct BinomialNodeCursor<T> {
    node_id: usize,
    value: Arc<T>,
}

impl<'a, T> Clone for BinomialNodeView<'a, T> {
    fn clone(&self) -> Self {
        Self {
            heap: self.heap,
            node_id: self.node_id,
        }
    }
}

impl<T> Clone for BinomialNodeCursor<T> {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id,
            value: Arc::clone(&self.value),
        }
    }
}

impl<'a, T> BinomialNodeView<'a, T> {
    fn node_ref(&self) -> &BinomialNode<T> {
        self.heap.nodes[self.node_id]
            .as_ref()
            .expect("node id should reference a live node")
    }

    pub fn value(&self) -> &T {
        self.node_ref().value.as_ref()
    }

    pub fn degree(&self) -> usize {
        let i = self.node_ref().heap_pos;
        let len = self.heap.heap.len();
        let left = 2 * i + 1;
        let right = 2 * i + 2;
        usize::from(left < len) + usize::from(right < len)
    }

    pub fn child(&self) -> Option<Self> {
        let i = self.node_ref().heap_pos;
        let left = 2 * i + 1;
        self.heap
            .heap
            .get(left)
            .copied()
            .map(|node_id| Self {
                heap: self.heap,
                node_id,
            })
    }

    pub fn sibling(&self) -> Option<Self> {
        let i = self.node_ref().heap_pos;
        if i == 0 {
            return None;
        }

        let sibling_pos = if i % 2 == 1 { i + 1 } else { i.saturating_sub(1) };
        self.heap
            .heap
            .get(sibling_pos)
            .copied()
            .map(|node_id| Self {
                heap: self.heap,
                node_id,
            })
    }

    pub fn parent(&self) -> Option<Self> {
        let i = self.node_ref().heap_pos;
        if i == 0 {
            return None;
        }

        let parent_pos = (i - 1) / 2;
        self.heap
            .heap
            .get(parent_pos)
            .copied()
            .map(|node_id| Self {
                heap: self.heap,
                node_id,
            })
    }
}

impl<T> BinomialNodeCursor<T> {
    pub fn value(&self) -> &T {
        self.value.as_ref()
    }

    pub fn node_view<'a>(&self, heap: &'a BinomialHeap<T>) -> Option<BinomialNodeView<'a, T>> {
        heap.nodes
            .get(self.node_id)
            .and_then(|entry| entry.as_ref())
            .map(|_| BinomialNodeView {
                heap,
                node_id: self.node_id,
            })
    }
}

#[derive(Debug)]
pub struct BinomialHeap<T> {
    nodes: Vec<Option<BinomialNode<T>>>,
    heap: Vec<usize>,
}

impl<T: Ord> BinomialHeap<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            heap: Vec::new(),
        }
    }

    fn node_ref(&self, node_id: usize) -> &BinomialNode<T> {
        self.nodes[node_id]
            .as_ref()
            .expect("node id should reference a live node")
    }

    fn node_mut(&mut self, node_id: usize) -> &mut BinomialNode<T> {
        self.nodes[node_id]
            .as_mut()
            .expect("node id should reference a live node")
    }

    fn compare_pos(&self, left_pos: usize, right_pos: usize) -> std::cmp::Ordering {
        let left_id = self.heap[left_pos];
        let right_id = self.heap[right_pos];
        self.node_ref(left_id).value.cmp(&self.node_ref(right_id).value)
    }

    fn swap_heap_pos(&mut self, i: usize, j: usize) {
        self.heap.swap(i, j);
        let left_id = self.heap[i];
        let right_id = self.heap[j];
        self.node_mut(left_id).heap_pos = i;
        self.node_mut(right_id).heap_pos = j;
    }

    fn sift_up(&mut self, mut pos: usize) {
        while pos > 0 {
            let parent = (pos - 1) / 2;
            if self.compare_pos(pos, parent).is_lt() {
                self.swap_heap_pos(pos, parent);
                pos = parent;
            } else {
                break;
            }
        }
    }

    fn sift_down(&mut self, mut pos: usize) {
        let len = self.heap.len();
        loop {
            let left = 2 * pos + 1;
            let right = 2 * pos + 2;
            let mut smallest = pos;

            if left < len && self.compare_pos(left, smallest).is_lt() {
                smallest = left;
            }

            if right < len && self.compare_pos(right, smallest).is_lt() {
                smallest = right;
            }

            if smallest == pos {
                break;
            }

            self.swap_heap_pos(pos, smallest);
            pos = smallest;
        }
    }

    fn heapify(&mut self) {
        if self.heap.len() <= 1 {
            return;
        }

        for idx in (0..=(self.heap.len() / 2)).rev() {
            self.sift_down(idx);
        }
    }

    fn remove_pos(&mut self, pos: usize) -> Option<T> {
        if pos >= self.heap.len() {
            return None;
        }

        let last = self.heap.len() - 1;
        self.swap_heap_pos(pos, last);
        let removed_id = self.heap.pop()?;

        if pos < self.heap.len() {
            if pos > 0 {
                let parent = (pos - 1) / 2;
                if self.compare_pos(pos, parent).is_lt() {
                    self.sift_up(pos);
                } else {
                    self.sift_down(pos);
                }
            } else {
                self.sift_down(pos);
            }
        }

        let removed = self.nodes[removed_id]
            .take()
            .expect("removed id should reference a live node");
        let value = Arc::try_unwrap(removed.value)
            .ok()
            .expect("removed node value should be uniquely owned");
        Some(value)
    }

    fn live_cursor(&self, node_id: usize) -> Option<BinomialNodeCursor<T>> {
        self.nodes
            .get(node_id)
            .and_then(|entry| entry.as_ref())
            .map(|node| BinomialNodeCursor {
                node_id,
                value: Arc::clone(&node.value),
            })
    }

    pub fn head_view(&self) -> Option<BinomialNodeView<'_, T>> {
        self.heap.first().copied().map(|node_id| BinomialNodeView {
            heap: self,
            node_id,
        })
    }

    pub fn roots(&self) -> Vec<BinomialNodeView<'_, T>> {
        self.head_view().into_iter().collect()
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.heap.clear();
    }

    pub fn search(&self, value: &T) -> Option<BinomialNodeCursor<T>> {
        self.heap
            .iter()
            .copied()
            .find(|node_id| self.node_ref(*node_id).value.as_ref() == value)
            .and_then(|node_id| self.live_cursor(node_id))
    }

    pub fn min(&self) -> Option<BinomialNodeCursor<T>> {
        self.heap.first().copied().and_then(|node_id| self.live_cursor(node_id))
    }

    pub fn insert(&mut self, value: T) {
        let node_id = self.nodes.len();
        let pos = self.heap.len();
        self.nodes.push(Some(BinomialNode {
            value: Arc::new(value),
            heap_pos: pos,
        }));
        self.heap.push(node_id);
        self.sift_up(pos);
    }

    pub fn merge(&mut self, other: &mut Self) {
        for node_id in other.heap.drain(..) {
            let node = other.nodes[node_id]
                .take()
                .expect("drained heap id should reference a live node");
            let new_id = self.nodes.len();
            let pos = self.heap.len();
            self.nodes.push(Some(BinomialNode {
                value: node.value,
                heap_pos: pos,
            }));
            self.heap.push(new_id);
        }
        self.heapify();
    }

    pub fn extract_min(&mut self) -> Option<T> {
        self.remove_pos(0)
    }

    pub fn decrease_key(&mut self, handle: BinomialNodeCursor<T>, new_value: T) {
        let node_id = handle.node_id;
        drop(handle);

        if self
            .nodes
            .get(node_id)
            .and_then(|entry| entry.as_ref())
            .is_none()
        {
            return;
        }

        if new_value > *self.node_ref(node_id).value.as_ref() {
            panic!("decrease_key received a larger replacement value");
        }

        let pos = self.node_ref(node_id).heap_pos;
        self.node_mut(node_id).value = Arc::new(new_value);
        self.sift_up(pos);
    }

    pub fn delete(&mut self, handle: BinomialNodeCursor<T>) -> Option<T> {
        let node_id = handle.node_id;
        drop(handle);

        self.nodes
            .get(node_id)
            .and_then(|entry| entry.as_ref())?;
        let pos = self.node_ref(node_id).heap_pos;
        self.remove_pos(pos)
    }

    pub fn delete_value(&mut self, value: &T) -> Option<T> {
        let node_id = self
            .heap
            .iter()
            .copied()
            .find(|id| self.node_ref(*id).value.as_ref() == value)?;
        let pos = self.node_ref(node_id).heap_pos;
        self.remove_pos(pos)
    }
}

impl<T: Ord> PriorityQueue<T> for BinomialHeap<T> {
    type Cursor<'a>
        = BinomialNodeCursor<T>
    where
        Self: 'a;

    type View<'a>
        = BinomialNodeView<'a, T>
    where
        Self: 'a;

    fn push(&mut self, value: T) {
        BinomialHeap::insert(self, value)
    }

    fn pop(&mut self) -> Option<T> {
        BinomialHeap::extract_min(self)
    }

    fn peek<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        BinomialHeap::min(self)
    }

    fn cursor<'a>(&'a self, value: &T) -> Option<Self::Cursor<'a>> {
        BinomialHeap::search(self, value)
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
        BinomialHeap::delete(self, cursor)
    }

    fn clear(&mut self) {
        BinomialHeap::clear(self)
    }

    fn len(&self) -> usize {
        BinomialHeap::len(self)
    }
}

impl<T: Ord> ForestDiagnostics for BinomialHeap<T> {
    fn root_count(&self) -> usize {
        usize::from(!self.is_empty())
    }

    fn node_count(&self) -> usize {
        self.len()
    }

    fn max_root_degree(&self) -> usize {
        self.head_view().map_or(0, |root| root.degree())
    }
}

impl<T: Ord> Default for BinomialHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> FromIterator<T> for BinomialHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut heap = Self::new();
        for value in iter {
            heap.insert(value);
        }
        heap
    }
}
