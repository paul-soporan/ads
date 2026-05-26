use crate::traits::{core::PriorityQueue, diagnostics::ForestDiagnostics};

#[derive(Debug)]
struct FibonacciNode<T> {
    value: Option<T>,
    degree: usize,
    marked: bool,
    parent: Option<usize>,
    child: Option<usize>,
    left: usize,
    right: usize,
}

impl<T> FibonacciNode<T> {
    fn new(value: T, id: usize) -> Self {
        Self {
            value: Some(value),
            degree: 0,
            marked: false,
            parent: None,
            child: None,
            left: id,
            right: id,
        }
    }
}

#[derive(Debug)]
pub struct FibonacciNodeView<'a, T> {
    heap: &'a FibonacciHeap<T>,
    node_id: usize,
}

impl<'a, T> Clone for FibonacciNodeView<'a, T> {
    fn clone(&self) -> Self {
        Self {
            heap: self.heap,
            node_id: self.node_id,
        }
    }
}

impl<'a, T> FibonacciNodeView<'a, T> {
    pub(crate) fn identity(&self) -> usize {
        self.node_id
    }

    fn node_ref(&self) -> &FibonacciNode<T> {
        self.heap.nodes[self.node_id]
            .as_ref()
            .expect("node id should reference a live node")
    }

    pub fn value(&self) -> &T {
        self.node_ref()
            .value
            .as_ref()
            .expect("node value should be present")
    }

    pub fn degree(&self) -> usize {
        self.node_ref().degree
    }

    pub fn child(&self) -> Option<Self> {
        self.node_ref().child.map(|node_id| Self {
            heap: self.heap,
            node_id,
        })
    }

    pub fn sibling(&self) -> Option<Self> {
        Some(Self {
            heap: self.heap,
            node_id: self.node_ref().right,
        })
    }

    pub fn parent(&self) -> Option<Self> {
        self.node_ref().parent.map(|node_id| Self {
            heap: self.heap,
            node_id,
        })
    }
}

#[derive(Debug)]
pub struct FibonacciNodeCursor<T> {
    heap: *const FibonacciHeap<T>,
    node_id: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Clone for FibonacciNodeCursor<T> {
    fn clone(&self) -> Self {
        Self {
            heap: self.heap,
            node_id: self.node_id,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> FibonacciNodeCursor<T> {
    pub fn value(&self) -> &T {
        assert!(!self.heap.is_null(), "cursor heap pointer should be live");
        unsafe {
            (&*self.heap).nodes[self.node_id]
                .as_ref()
                .expect("node id should reference a live node")
                .value
                .as_ref()
                .expect("node value should be present")
        }
    }

    pub fn node_view<'a>(&self, heap: &'a FibonacciHeap<T>) -> Option<FibonacciNodeView<'a, T>> {
        heap.nodes
            .get(self.node_id)
            .and_then(|entry| entry.as_ref())
            .map(|_| FibonacciNodeView {
                heap,
                node_id: self.node_id,
            })
    }
}

#[derive(Debug)]
pub struct FibonacciHeap<T> {
    nodes: Vec<Option<FibonacciNode<T>>>,
    min_node: Option<usize>,
    len: usize,
}

impl<T> FibonacciHeap<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            min_node: None,
            len: 0,
        }
    }

    fn node_ref(&self, node_id: usize) -> &FibonacciNode<T> {
        self.nodes[node_id]
            .as_ref()
            .expect("node id should reference a live node")
    }

    fn node_mut(&mut self, node_id: usize) -> &mut FibonacciNode<T> {
        self.nodes[node_id]
            .as_mut()
            .expect("node id should reference a live node")
    }

    fn list_append(&mut self, list1: Option<usize>, list2: Option<usize>) -> Option<usize> {
        match (list1, list2) {
            (None, None) => None,
            (Some(id), None) | (None, Some(id)) => Some(id),
            (Some(id1), Some(id2)) => {
                let id1_right = self.node_ref(id1).right;
                let id2_left = self.node_ref(id2).left;

                self.node_mut(id1).right = id2;
                self.node_mut(id2).left = id1;
                self.node_mut(id1_right).left = id2_left;
                self.node_mut(id2_left).right = id1_right;

                Some(id1)
            }
        }
    }

    fn list_remove(&mut self, node_id: usize) -> Option<usize> {
        let left = self.node_ref(node_id).left;
        let right = self.node_ref(node_id).right;

        if left == node_id {
            self.node_mut(node_id).left = node_id;
            self.node_mut(node_id).right = node_id;
            return None;
        }

        self.node_mut(left).right = right;
        self.node_mut(right).left = left;

        self.node_mut(node_id).left = node_id;
        self.node_mut(node_id).right = node_id;

        Some(right)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.min_node = None;
        self.len = 0;
    }
}

impl<T: Ord> FibonacciHeap<T> {
    fn link(&mut self, child_root: usize, parent_root: usize) {
        self.node_mut(child_root).parent = Some(parent_root);
        self.node_mut(child_root).marked = false;
        let parent_child = self.node_ref(parent_root).child;
        let new_child_list = self.list_append(parent_child, Some(child_root));
        self.node_mut(parent_root).child = new_child_list;
        self.node_mut(parent_root).degree += 1;
    }

    fn consolidate(&mut self) {
        let min_id = if let Some(id) = self.min_node { id } else { return; };

        let mut buckets: Vec<Option<usize>> = Vec::new();
        let mut current = min_id;
        let mut roots = Vec::new();
        loop {
            let next = self.node_ref(current).right;
            self.node_mut(current).left = current;
            self.node_mut(current).right = current;
            self.node_mut(current).parent = None;
            roots.push(current);
            if next == min_id { break; }
            current = next;
        }

        for mut root in roots {
            loop {
                let degree = self.node_ref(root).degree;
                if buckets.len() <= degree {
                    buckets.resize_with(degree + 1, || None);
                }

                if let Some(other) = buckets[degree].take() {
                    let (parent, child) = {
                        let v_root = self.node_ref(root).value.as_ref().unwrap();
                        let v_other = self.node_ref(other).value.as_ref().unwrap();
                        if v_root <= v_other { (root, other) } else { (other, root) }
                    };
                    self.link(child, parent);
                    root = parent;
                } else {
                    buckets[degree] = Some(root);
                    break;
                }
            }
        }

        self.min_node = None;
        for root_opt in buckets {
            if let Some(root) = root_opt {
                if self.min_node.is_none() {
                    self.min_node = Some(root);
                } else {
                    self.min_node = self.list_append(self.min_node, Some(root));
                    let v_root = self.node_ref(root).value.as_ref().unwrap();
                    let v_min = self.node_ref(self.min_node.unwrap()).value.as_ref().unwrap();
                    if v_root < v_min {
                        self.min_node = Some(root);
                    }
                }
            }
        }
    }

    pub fn head_view<'a>(&'a self) -> Option<FibonacciNodeView<'a, T>> {
        self.min_node.map(|node_id| FibonacciNodeView {
            heap: self,
            node_id,
        })
    }

    pub fn roots<'a>(&'a self) -> Vec<FibonacciNodeView<'a, T>> {
        let mut roots = Vec::new();
        let start = if let Some(id) = self.min_node { id } else { return roots; };

        let mut current = start;
        loop {
            roots.push(FibonacciNodeView {
                heap: self,
                node_id: current,
            });
            current = self.node_ref(current).right;
            if current == start { break; }
        }

        roots
    }

    pub fn search(&self, value: &T) -> Option<FibonacciNodeCursor<T>> {
        let start = self.min_node?;
        let mut stack = Vec::new();
        stack.push(start);

        while let Some(root_list_start) = stack.pop() {
            let mut current = root_list_start;
            loop {
                let node = self.node_ref(current);
                let node_value = node.value.as_ref().expect("value");
                if node_value == value {
                    return Some(FibonacciNodeCursor {
                        heap: self as *const Self,
                        node_id: current,
                        _marker: std::marker::PhantomData,
                    });
                }

                if node_value < value {
                    if let Some(child) = node.child {
                        stack.push(child);
                    }
                }

                current = node.right;
                if current == root_list_start { break; }
            }
        }

        None
    }

    pub fn min(&self) -> Option<FibonacciNodeCursor<T>> {
        self.min_node.map(|node_id| FibonacciNodeCursor {
            heap: self as *const Self,
            node_id,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn insert(&mut self, value: T) {
        let node_id = self.nodes.len();
        self.nodes.push(Some(FibonacciNode::new(value, node_id)));
        
        if self.min_node.is_none() {
            self.min_node = Some(node_id);
        } else {
            self.min_node = self.list_append(self.min_node, Some(node_id));
            let v_node = self.node_ref(node_id).value.as_ref().unwrap();
            let v_min = self.node_ref(self.min_node.unwrap()).value.as_ref().unwrap();
            if v_node < v_min {
                self.min_node = Some(node_id);
            }
        }
        self.len += 1;
    }

    pub fn merge(&mut self, other: &mut Self) {
        if other.len == 0 { return; }

        let offset = self.nodes.len();
        let other_nodes = std::mem::take(&mut other.nodes);
        self.nodes.extend(other_nodes.into_iter().map(|slot| {
            slot.map(|mut node| {
                node.parent = node.parent.map(|node_id| node_id + offset);
                node.child = node.child.map(|node_id| node_id + offset);
                node.left += offset;
                node.right += offset;
                node
            })
        }));

        let other_min = other.min_node.map(|id| id + offset);
        if self.min_node.is_none() {
            self.min_node = other_min;
        } else if let Some(other_id) = other_min {
            self.min_node = self.list_append(self.min_node, Some(other_id));
            let v_other = self.node_ref(other_id).value.as_ref().unwrap();
            let v_min = self.node_ref(self.min_node.unwrap()).value.as_ref().unwrap();
            if v_other < v_min {
                self.min_node = Some(other_id);
            }
        }

        self.len += other.len;
        other.len = 0;
        other.min_node = None;
    }

    pub fn extract_min(&mut self) -> Option<T> {
        let min_id = self.min_node?;
        
        if let Some(child_id) = self.node_ref(min_id).child {
            let mut current = child_id;
            loop {
                self.node_mut(current).parent = None;
                current = self.node_ref(current).right;
                if current == child_id { break; }
            }
            self.min_node = self.list_append(self.min_node, Some(child_id));
        }

        let next = self.list_remove(min_id);
        self.min_node = next;
        if self.min_node.is_some() {
            self.consolidate();
        }

        self.len = self.len.saturating_sub(1);
        let removed = self.nodes[min_id].take().unwrap();
        Some(removed.value.unwrap())
    }

    pub fn decrease_key(&mut self, handle: FibonacciNodeCursor<T>, new_value: T) {
        let node_id = handle.node_id;
        let old_value = self.node_ref(node_id).value.as_ref().unwrap();
        if new_value > *old_value {
            panic!("decrease_key received a larger value");
        }
        self.node_mut(node_id).value = Some(new_value);
        
        if let Some(parent_id) = self.node_ref(node_id).parent {
            let v_node = self.node_ref(node_id).value.as_ref().unwrap();
            let v_parent = self.node_ref(parent_id).value.as_ref().unwrap();
            if v_node < v_parent {
                self.cut(node_id, parent_id);
                self.cascading_cut(parent_id);
            }
        }

        let v_node = self.node_ref(node_id).value.as_ref().unwrap();
        let v_min = self.node_ref(self.min_node.unwrap()).value.as_ref().unwrap();
        if v_node < v_min {
            self.min_node = Some(node_id);
        }
    }

    fn cut(&mut self, node_id: usize, parent_id: usize) {
        let new_child_list = self.list_remove(node_id);
        self.node_mut(parent_id).child = new_child_list;
        self.node_mut(parent_id).degree -= 1;
        
        self.node_mut(node_id).parent = None;
        self.node_mut(node_id).marked = false;
        self.min_node = self.list_append(self.min_node, Some(node_id));
    }

    fn cascading_cut(&mut self, node_id: usize) {
        if let Some(parent_id) = self.node_ref(node_id).parent {
            if !self.node_ref(node_id).marked {
                self.node_mut(node_id).marked = true;
            } else {
                self.cut(node_id, parent_id);
                self.cascading_cut(parent_id);
            }
        }
    }

    pub fn delete(&mut self, handle: FibonacciNodeCursor<T>) -> Option<T> {
        let node_id = handle.node_id;
        if let Some(parent_id) = self.node_ref(node_id).parent {
            self.cut(node_id, parent_id);
            self.cascading_cut(parent_id);
        }
        self.min_node = Some(node_id);
        self.extract_min()
    }

    pub fn delete_value(&mut self, value: &T) -> Option<T> {
        let cursor = self.search(value)?;
        self.delete(cursor)
    }
}

impl<T: Ord> PriorityQueue<T> for FibonacciHeap<T> {
    type Cursor<'a> = FibonacciNodeCursor<T> where Self: 'a;
    type View<'a> = FibonacciNodeView<'a, T> where Self: 'a;

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

impl<T: Ord> ForestDiagnostics for FibonacciHeap<T> {
    fn root_count(&self) -> usize {
        let start = if let Some(id) = self.min_node { id } else { return 0; };
        let mut count = 0;
        let mut current = start;
        loop {
            count += 1;
            current = self.node_ref(current).right;
            if current == start { break; }
        }
        count
    }

    fn node_count(&self) -> usize { self.len }

    fn max_root_degree(&self) -> usize {
        let start = if let Some(id) = self.min_node { id } else { return 0; };
        let mut max_degree = 0;
        let mut current = start;
        loop {
            max_degree = max_degree.max(self.node_ref(current).degree);
            current = self.node_ref(current).right;
            if current == start { break; }
        }
        max_degree
    }
}

impl<T: Ord> Default for FibonacciHeap<T> {
    fn default() -> Self { Self::new() }
}

impl<T: Ord> FromIterator<T> for FibonacciHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut heap = Self::new();
        for value in iter { heap.insert(value); }
        heap
    }
}
