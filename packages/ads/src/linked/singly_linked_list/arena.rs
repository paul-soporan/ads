use crate::traits::core::Sequence;

#[derive(Debug)]
struct Node<T> {
    value: T,
    next: Option<usize>,
}

#[derive(Debug)]
struct Slot<T> {
    node: Option<Node<T>>,
    generation: u64,
}

#[derive(Debug, Clone, Copy)]
struct Handle {
    index: usize,
    generation: u64,
}

#[derive(Debug)]
pub struct SinglyLinkedList<T> {
    head: Option<Handle>,
    tail: Option<Handle>,
    len: usize,
    slots: Vec<Slot<T>>,
    free: Vec<usize>,
}

impl<T> Default for SinglyLinkedList<T> {
    fn default() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            slots: Vec::new(),
            free: Vec::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SinglyCursor<'a, T> {
    index: usize,
    handle: Handle,
    list: &'a SinglyLinkedList<T>,
}

impl<'a, T> SinglyCursor<'a, T> {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn value(&self) -> &'a T {
        let slot = &self.list.slots[self.handle.index];
        debug_assert_eq!(slot.generation, self.handle.generation);
        &slot
            .node
            .as_ref()
            .expect("cursor should point to a live node")
            .value
    }
}

impl<T> SinglyLinkedList<T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn alloc_node(&mut self, node: Node<T>) -> Handle {
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

    fn get_node(&self, handle: Handle) -> Option<&Node<T>> {
        let slot = self.slots.get(handle.index)?;
        if slot.generation != handle.generation {
            return None;
        }
        slot.node.as_ref()
    }

    fn get_node_mut(&mut self, handle: Handle) -> Option<&mut Node<T>> {
        let slot = self.slots.get_mut(handle.index)?;
        if slot.generation != handle.generation {
            return None;
        }
        slot.node.as_mut()
    }

    fn free_node(&mut self, handle: Handle) -> Option<Node<T>> {
        let slot = self.slots.get_mut(handle.index)?;
        if slot.generation != handle.generation {
            return None;
        }
        let node = slot.node.take()?;
        slot.generation = slot.generation.wrapping_add(1);
        self.free.push(handle.index);
        Some(node)
    }

    fn handle_at(&self, index: usize) -> Option<Handle> {
        if index >= self.len {
            return None;
        }

        let mut current = self.head?;
        for _ in 0..index {
            let next = self.get_node(current)?.next?;
            current = Handle {
                index: next,
                generation: self.slots[next].generation,
            };
        }
        Some(current)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            list: self,
            next: self.head,
        }
    }
}

pub struct Iter<'a, T> {
    list: &'a SinglyLinkedList<T>,
    next: Option<Handle>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let handle = self.next?;
        let node = self.list.get_node(handle)?;
        self.next = node.next.map(|idx| Handle {
            index: idx,
            generation: self.list.slots[idx].generation,
        });
        Some(&node.value)
    }
}

impl<T> Sequence<T> for SinglyLinkedList<T> {
    type Cursor<'a>
        = SinglyCursor<'a, T>
    where
        Self: 'a;

    type MutView<'a>
        = &'a mut T
    where
        Self: 'a,
        T: 'a;

    fn push_front(&mut self, value: T) {
        let next = self.head.map(|h| h.index);
        let handle = self.alloc_node(Node { value, next });
        self.head = Some(handle);
        if self.tail.is_none() {
            self.tail = Some(handle);
        }
        self.len += 1;
    }

    fn push_back(&mut self, value: T) {
        let handle = self.alloc_node(Node { value, next: None });
        match self.tail {
            Some(tail) => {
                self.get_node_mut(tail).expect("live tail").next = Some(handle.index);
                self.tail = Some(handle);
            }
            None => {
                self.head = Some(handle);
                self.tail = Some(handle);
            }
        }
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<T> {
        let head = self.head?;
        let next_index = self.get_node(head)?.next;
        let node = self.free_node(head)?;
        self.head = next_index.map(|idx| Handle {
            index: idx,
            generation: self.slots[idx].generation,
        });
        if self.head.is_none() {
            self.tail = None;
        }
        self.len -= 1;
        Some(node.value)
    }

    fn pop_back(&mut self) -> Option<T> {
        match self.len {
            0 => None,
            1 => self.pop_front(),
            _ => {
                let mut current = self.head.expect("non-empty");
                loop {
                    let next = self.get_node(current).and_then(|n| n.next).expect("next");
                    let next_handle = Handle {
                        index: next,
                        generation: self.slots[next].generation,
                    };
                    if Some(next_handle.index) == self.tail.map(|h| h.index) {
                        self.get_node_mut(current).expect("live node").next = None;
                        self.tail = Some(current);
                        let old_tail = next_handle;
                        let node = self.free_node(old_tail).expect("tail should exist");
                        self.len -= 1;
                        return Some(node.value);
                    }
                    current = next_handle;
                }
            }
        }
    }

    fn cursor_at<'a>(&'a self, index: usize) -> Option<Self::Cursor<'a>> {
        let handle = self.handle_at(index)?;
        Some(SinglyCursor {
            index,
            handle,
            list: self,
        })
    }

    fn get_mut<'a>(&'a mut self, index: usize) -> Option<Self::MutView<'a>> {
        let handle = self.handle_at(index)?;
        Some(&mut self.get_node_mut(handle)?.value)
    }

    fn clear(&mut self) {
        while self.pop_front().is_some() {}
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
