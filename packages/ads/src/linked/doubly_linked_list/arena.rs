use crate::traits::core::Sequence;

#[derive(Debug)]
struct Node<T> {
    value: T,
    prev: Option<usize>,
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
pub struct DoublyLinkedList<T> {
    head: Option<Handle>,
    tail: Option<Handle>,
    len: usize,
    slots: Vec<Slot<T>>,
    free: Vec<usize>,
}

impl<T> Default for DoublyLinkedList<T> {
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
pub struct DoublyCursor<'a, T> {
    index: usize,
    handle: Handle,
    list: &'a DoublyLinkedList<T>,
}

impl<'a, T> DoublyCursor<'a, T> {
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

impl<T> DoublyLinkedList<T> {
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

    fn make_handle(&self, index: usize) -> Handle {
        Handle {
            index,
            generation: self.slots[index].generation,
        }
    }

    fn handle_at(&self, index: usize) -> Option<Handle> {
        if index >= self.len {
            return None;
        }

        if index <= self.len / 2 {
            let mut current = self.head?;
            for _ in 0..index {
                let next = self.get_node(current)?.next?;
                current = self.make_handle(next);
            }
            Some(current)
        } else {
            let mut current = self.tail?;
            for _ in 0..(self.len - index - 1) {
                let prev = self.get_node(current)?.prev?;
                current = self.make_handle(prev);
            }
            Some(current)
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            list: self,
            next: self.head,
        }
    }
}

pub struct Iter<'a, T> {
    list: &'a DoublyLinkedList<T>,
    next: Option<Handle>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let handle = self.next?;
        let node = self.list.get_node(handle)?;
        self.next = node.next.map(|idx| self.list.make_handle(idx));
        Some(&node.value)
    }
}

impl<T> Sequence<T> for DoublyLinkedList<T> {
    type Cursor<'a>
        = DoublyCursor<'a, T>
    where
        Self: 'a;

    type MutView<'a>
        = &'a mut T
    where
        Self: 'a,
        T: 'a;

    fn push_front(&mut self, value: T) {
        let old_head = self.head;
        let handle = self.alloc_node(Node {
            value,
            prev: None,
            next: old_head.map(|h| h.index),
        });

        if let Some(head) = old_head {
            self.get_node_mut(head).expect("live head").prev = Some(handle.index);
        } else {
            self.tail = Some(handle);
        }

        self.head = Some(handle);
        self.len += 1;
    }

    fn push_back(&mut self, value: T) {
        let old_tail = self.tail;
        let handle = self.alloc_node(Node {
            value,
            prev: old_tail.map(|h| h.index),
            next: None,
        });

        if let Some(tail) = old_tail {
            self.get_node_mut(tail).expect("live tail").next = Some(handle.index);
        } else {
            self.head = Some(handle);
        }

        self.tail = Some(handle);
        self.len += 1;
    }

    fn pop_front(&mut self) -> Option<T> {
        let head = self.head?;
        let next = self.get_node(head)?.next;
        let node = self.free_node(head)?;

        self.head = next.map(|idx| self.make_handle(idx));
        if let Some(new_head) = self.head {
            self.get_node_mut(new_head).expect("live head").prev = None;
        } else {
            self.tail = None;
        }

        self.len -= 1;
        Some(node.value)
    }

    fn pop_back(&mut self) -> Option<T> {
        let tail = self.tail?;
        let prev = self.get_node(tail)?.prev;
        let node = self.free_node(tail)?;

        self.tail = prev.map(|idx| self.make_handle(idx));
        if let Some(new_tail) = self.tail {
            self.get_node_mut(new_tail).expect("live tail").next = None;
        } else {
            self.head = None;
        }

        self.len -= 1;
        Some(node.value)
    }

    fn cursor_at<'a>(&'a self, index: usize) -> Option<Self::Cursor<'a>> {
        let handle = self.handle_at(index)?;
        Some(DoublyCursor {
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

impl<T> std::iter::FromIterator<T> for DoublyLinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        for value in iter {
            list.push_back(value);
        }
        list
    }
}
