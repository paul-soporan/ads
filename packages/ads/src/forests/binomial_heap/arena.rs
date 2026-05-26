use crate::traits::{core::PriorityQueue, diagnostics::ForestDiagnostics};

#[derive(Debug, Clone)]
struct BinomialNode<T> {
    value: T,
    degree: usize,
    parent: Option<usize>,
    child: Option<usize>,
    sibling: Option<usize>,
}

impl<T> BinomialNode<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            degree: 0,
            parent: None,
            child: None,
            sibling: None,
        }
    }
}

#[derive(Debug)]
pub struct BinomialNodeView<'a, T> {
    heap: &'a BinomialHeap<T>,
    node_id: usize,
}

impl<'a, T> Clone for BinomialNodeView<'a, T> {
    fn clone(&self) -> Self {
        Self {
            heap: self.heap,
            node_id: self.node_id,
        }
    }
}

impl<'a, T> BinomialNodeView<'a, T> {
    pub(crate) fn identity(&self) -> usize {
        self.node_id
    }

    fn node_ref(&self) -> &BinomialNode<T> {
        self.heap.nodes[self.node_id]
            .as_ref()
            .expect("node id should reference a live node")
    }

    pub fn value(&self) -> &T {
        &self.node_ref().value
    }

    pub fn degree(&self) -> usize {
        self.node_ref().degree
    }

    pub fn child(&self) -> Option<Self> {
        self.node_ref().child.map(|id| Self {
            heap: self.heap,
            node_id: id,
        })
    }

    pub fn sibling(&self) -> Option<Self> {
        self.node_ref().sibling.map(|id| Self {
            heap: self.heap,
            node_id: id,
        })
    }

    pub fn parent(&self) -> Option<Self> {
        self.node_ref().parent.map(|id| Self {
            heap: self.heap,
            node_id: id,
        })
    }
}

#[derive(Debug)]
pub struct BinomialNodeCursor<T> {
    node_id: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Clone for BinomialNodeCursor<T> {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> BinomialNodeCursor<T> {
    pub fn node_view<'a>(&self, heap: &'a BinomialHeap<T>) -> Option<BinomialNodeView<'a, T>> {
        heap.nodes.get(self.node_id).and_then(|n| n.as_ref()).map(|_| BinomialNodeView {
            heap,
            node_id: self.node_id,
        })
    }
}

#[derive(Debug)]
pub struct BinomialHeap<T> {
    nodes: Vec<Option<BinomialNode<T>>>,
    head: Option<usize>,
    len: usize,
}

impl<T: Ord> BinomialHeap<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            head: None,
            len: 0,
        }
    }

    fn node_ref(&self, id: usize) -> &BinomialNode<T> {
        self.nodes[id].as_ref().unwrap()
    }

    fn node_mut(&mut self, id: usize) -> &mut BinomialNode<T> {
        self.nodes[id].as_mut().unwrap()
    }

    fn link(&mut self, child: usize, parent: usize) {
        self.node_mut(child).parent = Some(parent);
        self.node_mut(child).sibling = self.node_ref(parent).child;
        self.node_mut(parent).child = Some(child);
        self.node_mut(parent).degree += 1;
    }

    fn merge_root_lists(&mut self, mut h1: Option<usize>, mut h2: Option<usize>) -> Option<usize> {
        let mut head = None;
        let mut tail_id = None;

        while let (Some(id1), Some(id2)) = (h1, h2) {
            let take_first = self.node_ref(id1).degree <= self.node_ref(id2).degree;
            let next_id = if take_first {
                let next = self.node_ref(id1).sibling;
                h1 = next;
                id1
            } else {
                let next = self.node_ref(id2).sibling;
                h2 = next;
                id2
            };

            self.node_mut(next_id).sibling = None;
            if let Some(t) = tail_id {
                self.node_mut(t).sibling = Some(next_id);
            } else {
                head = Some(next_id);
            }
            tail_id = Some(next_id);
        }

        let remaining = h1.or(h2);
        if let Some(rem) = remaining {
            if let Some(t) = tail_id {
                self.node_mut(t).sibling = Some(rem);
            } else {
                head = Some(rem);
            }
        }

        head
    }

    fn consolidate(&mut self, head: Option<usize>) -> Option<usize> {
        let mut head = head?;
        let mut prev: Option<usize> = None;
        let mut x = head;
        let mut next = self.node_ref(x).sibling;

        while let Some(n) = next {
            let n_next = self.node_ref(n).sibling;
            if self.node_ref(x).degree != self.node_ref(n).degree 
                || (n_next.is_some() && self.node_ref(n_next.unwrap()).degree == self.node_ref(x).degree) 
            {
                prev = Some(x);
                x = n;
            } else if self.node_ref(x).value <= self.node_ref(n).value {
                self.node_mut(x).sibling = n_next;
                self.link(n, x);
            } else {
                if let Some(p) = prev {
                    self.node_mut(p).sibling = Some(n);
                } else {
                    head = n;
                }
                self.link(x, n);
                x = n;
            }
            next = self.node_ref(x).sibling;
        }
        Some(head)
    }

    pub fn merge(&mut self, other: &mut Self) {
        if other.head.is_none() { return; }

        let offset = self.nodes.len();
        let other_nodes = std::mem::take(&mut other.nodes);
        self.nodes.extend(other_nodes.into_iter().map(|slot| {
            slot.map(|mut node| {
                node.parent = node.parent.map(|id| id + offset);
                node.child = node.child.map(|id| id + offset);
                node.sibling = node.sibling.map(|id| id + offset);
                node
            })
        }));

        let h1 = self.head;
        let h2 = other.head.map(|id| id + offset);
        
        let merged_head = self.merge_root_lists(h1, h2);
        self.head = self.consolidate(merged_head);
        self.len += other.len;
        other.len = 0;
        other.head = None;
    }

    pub fn insert(&mut self, value: T) {
        let node_id = self.nodes.len();
        self.nodes.push(Some(BinomialNode::new(value)));
        
        let h1 = self.head;
        let h2 = Some(node_id);
        let merged_head = self.merge_root_lists(h1, h2);
        self.head = self.consolidate(merged_head);
        self.len += 1;
    }

    pub fn extract_min(&mut self) -> Option<T> {
        let min_id = {
            let mut min_node_id = self.head?;
            let mut curr = self.node_ref(min_node_id).sibling;
            while let Some(id) = curr {
                if self.node_ref(id).value < self.node_ref(min_node_id).value {
                    min_node_id = id;
                }
                curr = self.node_ref(id).sibling;
            }
            min_node_id
        };

        if self.head == Some(min_id) {
            self.head = self.node_ref(min_id).sibling;
        } else {
            let mut prev = self.head.unwrap();
            while self.node_ref(prev).sibling != Some(min_id) {
                prev = self.node_ref(prev).sibling.unwrap();
            }
            self.node_mut(prev).sibling = self.node_ref(min_id).sibling;
        }

        let mut child_id = self.node_ref(min_id).child;
        let mut new_head = None;
        while let Some(id) = child_id {
            let next = self.node_ref(id).sibling;
            self.node_mut(id).parent = None;
            self.node_mut(id).sibling = new_head;
            new_head = Some(id);
            child_id = next;
        }

        let h1 = self.head;
        let merged_head = self.merge_root_lists(h1, new_head);
        self.head = self.consolidate(merged_head);
        self.len = self.len.saturating_sub(1);

        Some(self.nodes[min_id].take().unwrap().value)
    }

    pub fn min(&self) -> Option<BinomialNodeCursor<T>> {
        let mut min_id = self.head?;
        let mut curr = self.node_ref(min_id).sibling;
        while let Some(id) = curr {
            if self.node_ref(id).value < self.node_ref(min_id).value {
                min_id = id;
            }
            curr = self.node_ref(id).sibling;
        }
        Some(BinomialNodeCursor { node_id: min_id, _marker: std::marker::PhantomData })
    }

    pub fn decrease_key(&mut self, handle: BinomialNodeCursor<T>, new_value: T) {
        let node_id = handle.node_id;
        if self.node_ref(node_id).value < new_value {
            panic!("decrease_key received a larger replacement value");
        }
        self.node_mut(node_id).value = new_value;
        
        let mut curr = node_id;
        while let Some(parent) = self.node_ref(curr).parent {
            if self.node_ref(curr).value < self.node_ref(parent).value {
                let (v1, v2) = if curr < parent {
                    let (s1, s2) = self.nodes.split_at_mut(parent);
                    (&mut s1[curr].as_mut().unwrap().value, &mut s2[0].as_mut().unwrap().value)
                } else {
                    let (s1, s2) = self.nodes.split_at_mut(curr);
                    (&mut s2[0].as_mut().unwrap().value, &mut s1[parent].as_mut().unwrap().value)
                };
                std::mem::swap(v1, v2);
                curr = parent;
            } else {
                break;
            }
        }
    }

    pub fn delete(&mut self, handle: BinomialNodeCursor<T>) -> Option<T> {
        let mut curr = handle.node_id;
        while let Some(parent) = self.node_ref(curr).parent {
            let (v1, v2) = if curr < parent {
                let (s1, s2) = self.nodes.split_at_mut(parent);
                (&mut s1[curr].as_mut().unwrap().value, &mut s2[0].as_mut().unwrap().value)
            } else {
                let (s1, s2) = self.nodes.split_at_mut(curr);
                (&mut s2[0].as_mut().unwrap().value, &mut s1[parent].as_mut().unwrap().value)
            };
            std::mem::swap(v1, v2);
            curr = parent;
        }

        if self.head == Some(curr) {
            self.head = self.node_ref(curr).sibling;
        } else {
            let mut prev = self.head.unwrap();
            while self.node_ref(prev).sibling != Some(curr) {
                prev = self.node_ref(prev).sibling.unwrap();
            }
            self.node_mut(prev).sibling = self.node_ref(curr).sibling;
        }

        let mut child_id = self.node_ref(curr).child;
        let mut new_head = None;
        while let Some(id) = child_id {
            let next = self.node_ref(id).sibling;
            self.node_mut(id).parent = None;
            self.node_mut(id).sibling = new_head;
            new_head = Some(id);
            child_id = next;
        }

        let h1 = self.head;
        let merged_head = self.merge_root_lists(h1, new_head);
        self.head = self.consolidate(merged_head);
        self.len = self.len.saturating_sub(1);

        Some(self.nodes[curr].take().unwrap().value)
    }

    pub fn search(&self, value: &T) -> Option<BinomialNodeCursor<T>> {
        let mut stack = Vec::new();
        if let Some(h) = self.head { stack.push(h); }

        while let Some(root_list_start) = stack.pop() {
            let mut curr = Some(root_list_start);
            while let Some(id) = curr {
                let node = self.node_ref(id);
                if &node.value == value {
                    return Some(BinomialNodeCursor { node_id: id, _marker: std::marker::PhantomData });
                }
                if &node.value < value {
                    if let Some(child) = node.child {
                        stack.push(child);
                    }
                }
                curr = node.sibling;
            }
        }
        None
    }

    pub fn head_view(&self) -> Option<BinomialNodeView<'_, T>> {
        self.head.map(|id| BinomialNodeView { heap: self, node_id: id })
    }

    pub fn roots(&self) -> Vec<BinomialNodeView<'_, T>> {
        let mut roots = Vec::new();
        let mut curr = self.head;
        while let Some(id) = curr {
            roots.push(BinomialNodeView { heap: self, node_id: id });
            curr = self.node_ref(id).sibling;
        }
        roots
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.head = None;
        self.len = 0;
    }
}

impl<T: Ord> PriorityQueue<T> for BinomialHeap<T> {
    type Cursor<'a> = BinomialNodeCursor<T> where Self: 'a;
    type View<'a> = BinomialNodeView<'a, T> where Self: 'a;

    fn push(&mut self, value: T) { self.insert(value) }
    fn pop(&mut self) -> Option<T> { self.extract_min() }
    fn peek<'a>(&'a self) -> Option<Self::Cursor<'a>> { self.min() }
    fn cursor<'a>(&'a self, value: &T) -> Option<Self::Cursor<'a>> { self.search(value) }
    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a> {
        cursor.node_view(self).expect("live node")
    }
    fn remove_cursor<'a>(&mut self, cursor: Self::Cursor<'a>) -> Option<T> where T: 'a { self.delete(cursor) }
    fn merge(&mut self, other: &mut Self) { self.merge(other) }
    fn clear(&mut self) { self.clear() }

    fn len(&self) -> usize { self.len }
}

impl<T: Ord> ForestDiagnostics for BinomialHeap<T> {
    fn root_count(&self) -> usize {
        let mut count = 0;
        let mut curr = self.head;
        while let Some(id) = curr {
            count += 1;
            curr = self.node_ref(id).sibling;
        }
        count
    }

    fn node_count(&self) -> usize { self.len }

    fn max_root_degree(&self) -> usize {
        let mut max_degree = 0;
        let mut curr = self.head;
        while let Some(id) = curr {
            max_degree = max_degree.max(self.node_ref(id).degree);
            curr = self.node_ref(id).sibling;
        }
        max_degree
    }
}

impl<T: Ord> Default for BinomialHeap<T> {
    fn default() -> Self { Self::new() }
}

impl<T: Ord> FromIterator<T> for BinomialHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut heap = Self::new();
        for value in iter { heap.insert(value); }
        heap
    }
}
