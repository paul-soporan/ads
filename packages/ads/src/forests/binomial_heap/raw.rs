use crate::traits::{core::PriorityQueue, diagnostics::ForestDiagnostics};

#[derive(Debug)]
struct BinomialNode<T> {
    value: T,
    index: usize,
}

impl<T> BinomialNode<T> {
    fn new(value: T, index: usize) -> Self {
        Self { value, index }
    }
}

#[derive(Debug)]
pub struct BinomialNodeView<T> {
    heap: *const BinomialHeap<T>,
    node: *mut BinomialNode<T>,
}

#[derive(Debug)]
pub struct BinomialNodeCursor<T> {
    heap: *const BinomialHeap<T>,
    node: *mut BinomialNode<T>,
}

impl<T> Clone for BinomialNodeView<T> {
    fn clone(&self) -> Self {
        Self {
            heap: self.heap,
            node: self.node,
        }
    }
}

impl<T> Clone for BinomialNodeCursor<T> {
    fn clone(&self) -> Self {
        Self {
            heap: self.heap,
            node: self.node,
        }
    }
}

impl<T> BinomialNodeView<T> {
    fn heap_ref(&self) -> &BinomialHeap<T> {
        // SAFETY: heap pointer is set by BinomialHeap methods and remains valid for view usage.
        unsafe { &*self.heap }
    }

    fn index(&self) -> usize {
        // SAFETY: node pointer is valid while the node remains in the heap.
        unsafe { (*self.node).index }
    }

    pub fn value(&self) -> &T {
        // SAFETY: node pointer is valid while the node remains in the heap.
        unsafe { &(*self.node).value }
    }

    pub fn degree(&self) -> usize {
        let len = self.heap_ref().nodes.len();
        if len == 0 {
            return 0;
        }

        let i = self.index();
        let left = 2 * i + 1;
        let right = 2 * i + 2;

        usize::from(left < len) + usize::from(right < len)
    }

    pub fn child(&self) -> Option<Self> {
        let heap = self.heap_ref();
        let left = 2 * self.index() + 1;
        heap.nodes.get(left).copied().map(|node| Self {
            heap: self.heap,
            node,
        })
    }

    pub fn sibling(&self) -> Option<Self> {
        let heap = self.heap_ref();
        let i = self.index();
        if i == 0 {
            return None;
        }

        let sibling_index = if i % 2 == 1 { i + 1 } else { i.saturating_sub(1) };
        heap.nodes.get(sibling_index).copied().map(|node| Self {
            heap: self.heap,
            node,
        })
    }

    pub fn parent(&self) -> Option<Self> {
        let i = self.index();
        if i == 0 {
            return None;
        }

        let parent_index = (i - 1) / 2;
        self.heap_ref()
            .nodes
            .get(parent_index)
            .copied()
            .map(|node| Self {
                heap: self.heap,
                node,
            })
    }
}

impl<T> BinomialNodeCursor<T> {
    pub fn value(&self) -> &T {
        // SAFETY: node pointer is valid while the node remains in the heap.
        unsafe { &(*self.node).value }
    }

    pub fn node_view(&self) -> BinomialNodeView<T> {
        BinomialNodeView {
            heap: self.heap,
            node: self.node,
        }
    }

    fn index_hint(&self) -> usize {
        // SAFETY: node pointer is valid while the node remains in the heap.
        unsafe { (*self.node).index }
    }
}

#[derive(Debug)]
pub struct BinomialHeap<T> {
    nodes: Vec<*mut BinomialNode<T>>,
}

impl<T: Ord> BinomialHeap<T> {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    fn view_for_index(&self, index: usize) -> Option<BinomialNodeView<T>> {
        self.nodes.get(index).copied().map(|node| BinomialNodeView {
            heap: self as *const Self,
            node,
        })
    }

    fn cursor_for_index(&self, index: usize) -> Option<BinomialNodeCursor<T>> {
        self.nodes.get(index).copied().map(|node| BinomialNodeCursor {
            heap: self as *const Self,
            node,
        })
    }

    fn update_index(ptr: *mut BinomialNode<T>, index: usize) {
        // SAFETY: ptr comes from Box::into_raw and points to a valid node.
        unsafe {
            (*ptr).index = index;
        }
    }

    fn swap_nodes(&mut self, i: usize, j: usize) {
        self.nodes.swap(i, j);
        Self::update_index(self.nodes[i], i);
        Self::update_index(self.nodes[j], j);
    }

    fn sift_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent = (index - 1) / 2;

            let should_swap = {
                // SAFETY: indices are in-bounds and node pointers are valid.
                unsafe { (*self.nodes[index]).value < (*self.nodes[parent]).value }
            };

            if !should_swap {
                break;
            }

            self.swap_nodes(index, parent);
            index = parent;
        }
    }

    fn sift_down(&mut self, mut index: usize) {
        let len = self.nodes.len();
        loop {
            let left = 2 * index + 1;
            let right = 2 * index + 2;
            let mut smallest = index;

            if left < len {
                let left_smaller = {
                    // SAFETY: indices are in-bounds and node pointers are valid.
                    unsafe { (*self.nodes[left]).value < (*self.nodes[smallest]).value }
                };
                if left_smaller {
                    smallest = left;
                }
            }

            if right < len {
                let right_smaller = {
                    // SAFETY: indices are in-bounds and node pointers are valid.
                    unsafe { (*self.nodes[right]).value < (*self.nodes[smallest]).value }
                };
                if right_smaller {
                    smallest = right;
                }
            }

            if smallest == index {
                break;
            }

            self.swap_nodes(index, smallest);
            index = smallest;
        }
    }

    fn heapify(&mut self) {
        if self.nodes.len() <= 1 {
            return;
        }

        for idx in (0..=(self.nodes.len() / 2)).rev() {
            self.sift_down(idx);
        }
    }

    fn remove_index(&mut self, index: usize) -> Option<T> {
        if index >= self.nodes.len() {
            return None;
        }

        let last = self.nodes.len() - 1;
        self.swap_nodes(index, last);

        let removed = self.nodes.pop()?;

        if index < self.nodes.len() {
            if index > 0 {
                let parent = (index - 1) / 2;
                let should_sift_up = {
                    // SAFETY: indices are valid and pointers are valid.
                    unsafe { (*self.nodes[index]).value < (*self.nodes[parent]).value }
                };

                if should_sift_up {
                    self.sift_up(index);
                } else {
                    self.sift_down(index);
                }
            } else {
                self.sift_down(index);
            }
        }

        // SAFETY: removed was allocated via Box::into_raw and is no longer referenced by self.nodes.
        let removed_box = unsafe { Box::from_raw(removed) };
        Some(removed_box.value)
    }

    fn index_of_cursor(&self, cursor: &BinomialNodeCursor<T>) -> Option<usize> {
        let hinted = cursor.index_hint();
        if let Some(node) = self.nodes.get(hinted).copied() {
            if node == cursor.node {
                return Some(hinted);
            }
        }

        self.nodes.iter().position(|node| *node == cursor.node)
    }

    pub fn head_view(&self) -> Option<BinomialNodeView<T>> {
        self.view_for_index(0)
    }

    pub fn roots(&self) -> Vec<BinomialNodeView<T>> {
        self.head_view().into_iter().collect()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn clear(&mut self) {
        while let Some(node) = self.nodes.pop() {
            // SAFETY: every pointer in self.nodes comes from Box::into_raw and is unique.
            unsafe {
                drop(Box::from_raw(node));
            }
        }
    }

    pub fn search(&self, value: &T) -> Option<BinomialNodeCursor<T>> {
        self.nodes
            .iter()
            .position(|node| {
                // SAFETY: pointers in self.nodes are valid.
                unsafe { (**node).value == *value }
            })
            .and_then(|index| self.cursor_for_index(index))
    }

    pub fn min(&self) -> Option<BinomialNodeCursor<T>> {
        self.cursor_for_index(0)
    }

    pub fn insert(&mut self, value: T) {
        let index = self.nodes.len();
        let node = Box::into_raw(Box::new(BinomialNode::new(value, index)));
        self.nodes.push(node);
        self.sift_up(index);
    }

    pub fn merge(&mut self, other: &mut Self) {
        for node in other.nodes.drain(..) {
            let index = self.nodes.len();
            Self::update_index(node, index);
            self.nodes.push(node);
        }
        self.heapify();
    }

    pub fn extract_min(&mut self) -> Option<T> {
        self.remove_index(0)
    }

    pub fn decrease_key(&mut self, handle: BinomialNodeCursor<T>, new_value: T) {
        let Some(index) = self.index_of_cursor(&handle) else {
            return;
        };

        let should_panic = {
            // SAFETY: index points to a valid node in self.nodes.
            unsafe { new_value > (*self.nodes[index]).value }
        };

        if should_panic {
            panic!("decrease_key received a larger replacement value");
        }

        // SAFETY: index points to a valid node in self.nodes.
        unsafe {
            (*self.nodes[index]).value = new_value;
        }
        self.sift_up(index);
    }

    pub fn delete(&mut self, handle: BinomialNodeCursor<T>) -> Option<T> {
        let index = self.index_of_cursor(&handle)?;
        self.remove_index(index)
    }

    pub fn delete_value(&mut self, value: &T) -> Option<T> {
        let cursor = self.search(value)?;
        self.delete(cursor)
    }
}

impl<T> Drop for BinomialHeap<T> {
    fn drop(&mut self) {
        while let Some(node) = self.nodes.pop() {
            // SAFETY: every pointer in self.nodes comes from Box::into_raw and is unique.
            unsafe {
                drop(Box::from_raw(node));
            }
        }
    }
}

impl<T: Ord> PriorityQueue<T> for BinomialHeap<T> {
    type Cursor<'a>
        = BinomialNodeCursor<T>
    where
        Self: 'a;

    type View<'a>
        = BinomialNodeView<T>
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
        cursor.node_view()
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
