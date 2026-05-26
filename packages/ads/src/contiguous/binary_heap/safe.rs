use std::marker::PhantomData;

use crate::traits::core::PriorityQueue;

#[derive(Debug, PartialEq, Eq)]
pub struct BinaryHeapCursor<T> {
    index: usize,
    generation: u64,
    _marker: PhantomData<T>,
}

impl<T> Clone for BinaryHeapCursor<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            generation: self.generation,
            _marker: PhantomData,
        }
    }
}

impl<T> BinaryHeapCursor<T> {
    pub fn index(&self) -> usize {
        self.index
    }
}

#[derive(Debug)]
pub struct BinaryHeapView<'a, T> {
    heap: &'a BinaryHeap<T>,
    index: usize,
}

impl<'a, T> Clone for BinaryHeapView<'a, T> {
    fn clone(&self) -> Self {
        Self {
            heap: self.heap,
            index: self.index,
        }
    }
}

impl<'a, T> BinaryHeapView<'a, T> {
    pub fn value(&self) -> &T {
        &self.heap.data[self.index]
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn parent_index(&self) -> Option<usize> {
        if self.index == 0 {
            None
        } else {
            Some((self.index - 1) / 2)
        }
    }

    pub fn left_child_index(&self) -> Option<usize> {
        let left = 2 * self.index + 1;
        (left < self.heap.data.len()).then_some(left)
    }

    pub fn right_child_index(&self) -> Option<usize> {
        let right = 2 * self.index + 2;
        (right < self.heap.data.len()).then_some(right)
    }
}

#[derive(Debug)]
pub struct BinaryHeap<T> {
    data: Vec<T>,
    generation: u64,
}

impl<T: Ord> BinaryHeap<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            generation: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            generation: 0,
        }
    }

    pub fn data_at(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    fn bump_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    fn sift_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent = (index - 1) / 2;
            if self.data[index] >= self.data[parent] {
                break;
            }
            self.data.swap(index, parent);
            index = parent;
        }
    }

    fn sift_down(&mut self, mut index: usize) {
        let len = self.data.len();

        loop {
            let left = 2 * index + 1;
            let right = 2 * index + 2;
            let mut smallest = index;

            if left < len && self.data[left] < self.data[smallest] {
                smallest = left;
            }

            if right < len && self.data[right] < self.data[smallest] {
                smallest = right;
            }

            if smallest == index {
                break;
            }

            self.data.swap(index, smallest);
            index = smallest;
        }
    }

    fn remove_at(&mut self, index: usize) -> Option<T> {
        if index >= self.data.len() {
            return None;
        }

        self.bump_generation();
        let removed = self.data.swap_remove(index);

        if index < self.data.len() {
            let parent = index.checked_sub(1).map(|i| i / 2);
            let should_sift_up = parent.is_some_and(|p| self.data[index] < self.data[p]);
            if should_sift_up {
                self.sift_up(index);
            } else {
                self.sift_down(index);
            }
        }

        Some(removed)
    }
}

impl<T: Ord> PriorityQueue<T> for BinaryHeap<T> {
    type Cursor<'a>
        = BinaryHeapCursor<T>
    where
        Self: 'a;

    type View<'a>
        = BinaryHeapView<'a, T>
    where
        Self: 'a;

    fn push(&mut self, value: T) {
        self.bump_generation();
        self.data.push(value);
        let last = self.data.len() - 1;
        self.sift_up(last);
    }

    fn pop(&mut self) -> Option<T> {
        self.remove_at(0)
    }

    fn peek<'a>(&'a self) -> Option<Self::Cursor<'a>> {
        (!self.data.is_empty()).then_some(BinaryHeapCursor {
            index: 0,
            generation: self.generation,
            _marker: PhantomData,
        })
    }

    fn cursor<'a>(&'a self, value: &T) -> Option<Self::Cursor<'a>> {
        let index = self.data.iter().position(|item| item == value)?;
        Some(BinaryHeapCursor {
            index,
            generation: self.generation,
            _marker: PhantomData,
        })
    }

    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        BinaryHeapView {
            heap: self,
            index: cursor.index,
        }
    }

    fn remove_cursor<'a>(&mut self, cursor: Self::Cursor<'a>) -> Option<T>
    where
        T: 'a,
    {
        if cursor.generation != self.generation {
            return None;
        }
        self.remove_at(cursor.index)
    }

    fn clear(&mut self) {
        if self.data.is_empty() {
            return;
        }

        self.bump_generation();
        self.data.clear();
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<T: Ord> Default for BinaryHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> FromIterator<T> for BinaryHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut heap = Self::new();
        for value in iter {
            heap.push(value);
        }
        heap
    }
}
