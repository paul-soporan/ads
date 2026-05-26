use std::{collections::HashMap, ptr};

use crate::traits::{core as core_traits, diagnostics::DisjointSetDiagnostics};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetId(pub usize);

impl SetId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug)]
struct DisjointSetNode<T> {
    id: SetId,
    value: T,
    parent: *mut DisjointSetNode<T>,
    rank: usize,
}

impl<T> DisjointSetNode<T> {
    fn new(id: SetId, value: T) -> Self {
        Self {
            id,
            value,
            parent: ptr::null_mut(),
            rank: 0,
        }
    }
}

#[derive(Debug)]
pub struct DisjointSetView<T> {
    node: *mut DisjointSetNode<T>,
}

impl<T> Clone for DisjointSetView<T> {
    fn clone(&self) -> Self {
        Self { node: self.node }
    }
}

impl<T> DisjointSetView<T> {
    fn find_root(mut node: *mut DisjointSetNode<T>) -> *mut DisjointSetNode<T> {
        // SAFETY: Caller guarantees node points at a valid node allocated by this DSU.
        unsafe {
            while !(*node).parent.is_null() {
                node = (*node).parent;
            }
        }
        node
    }

    pub fn set_id(&self) -> SetId {
        // SAFETY: view.node always comes from a DSU-owned node.
        unsafe { (*Self::find_root(self.node)).id }
    }

    pub fn value(&self) -> &T {
        // SAFETY: view.node always comes from a DSU-owned node.
        unsafe { &(*self.node).value }
    }

    pub fn parent_id(&self) -> Option<SetId> {
        // SAFETY: view.node always comes from a DSU-owned node.
        unsafe {
            if (*self.node).parent.is_null() {
                None
            } else {
                Some((*Self::find_root((*self.node).parent)).id)
            }
        }
    }

    pub fn rank(&self) -> usize {
        // SAFETY: view.node always comes from a DSU-owned node.
        unsafe { (*self.node).rank }
    }

    pub fn is_root(&self) -> bool {
        unsafe { (*self.node).parent.is_null() }
    }
}

#[derive(Debug, Default)]
pub struct DisjointSet<T> {
    nodes: Vec<*mut DisjointSetNode<T>>,
    value_map: HashMap<T, usize>,
}

impl<T> DisjointSet<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            value_map: HashMap::new(),
        }
    }

    fn free_all_nodes(&mut self) {
        for node_ptr in self.nodes.drain(..) {
            // SAFETY: node_ptr values come from Box::into_raw and are unique in self.nodes.
            unsafe {
                drop(Box::from_raw(node_ptr));
            }
        }
        self.value_map.clear();
    }

    fn node_at_slot(&self, slot: usize) -> Option<*mut DisjointSetNode<T>> {
        self.nodes.get(slot).copied()
    }

    fn representative_node_for_set_id(&self, set_id: SetId) -> Option<*mut DisjointSetNode<T>> {
        let node = self.node_at_slot(set_id.index())?;
        Some(self.find_root_without_compression(node))
    }

    fn node_by_value(&self, value: &T) -> Option<*mut DisjointSetNode<T>>
    where
        T: std::hash::Hash + Eq,
    {
        let &idx = self.value_map.get(value)?;
        self.nodes.get(idx).copied()
    }

    fn find_root_with_compression(
        &mut self,
        mut node: *mut DisjointSetNode<T>,
    ) -> *mut DisjointSetNode<T> {
        let mut path = Vec::new();

        // SAFETY: node and all parent pointers in the chain are valid DSU nodes.
        unsafe {
            while !(*node).parent.is_null() {
                path.push(node);
                node = (*node).parent;
            }

            for path_node in path {
                (*path_node).parent = node;
            }
        }

        node
    }

    fn find_root_without_compression(
        &self,
        mut node: *mut DisjointSetNode<T>,
    ) -> *mut DisjointSetNode<T> {
        // SAFETY: node and all parent pointers in the chain are valid DSU nodes.
        unsafe {
            while !(*node).parent.is_null() {
                node = (*node).parent;
            }
        }

        node
    }

    pub fn make_set(&mut self, value: T) -> SetId
    where
        T: std::hash::Hash + Eq + Clone,
    {
        if let Some(existing) = self.node_by_value(&value) {
            // SAFETY: existing is a valid node pointer returned by node_by_value.
            return unsafe { (*self.find_root_with_compression(existing)).id };
        }

        let idx = self.nodes.len();
        let id = SetId(idx);
        let node_ptr = Box::into_raw(Box::new(DisjointSetNode::new(id, value.clone())));
        self.nodes.push(node_ptr);
        self.value_map.insert(value, idx);
        id
    }

    pub fn find(&mut self, value: &T) -> Option<SetId>
    where
        T: std::hash::Hash + Eq,
    {
        let node = self.node_by_value(value)?;
        // SAFETY: node is valid and compression keeps pointers within this DSU.
        Some(unsafe { (*self.find_root_with_compression(node)).id })
    }

    pub fn view(&self, value: &T) -> Option<DisjointSetView<T>>
    where
        T: std::hash::Hash + Eq,
    {
        self.node_by_value(value)
            .map(|node| DisjointSetView { node })
    }

    pub fn view_by_set_id(&self, set_id: SetId) -> Option<DisjointSetView<T>> {
        self.representative_node_for_set_id(set_id)
            .map(|node| DisjointSetView { node })
    }

    pub fn union(&mut self, left: &T, right: &T) -> bool
    where
        T: std::hash::Hash + Eq,
    {
        let left_node = self.node_by_value(left);
        let right_node = self.node_by_value(right);

        match (left_node, right_node) {
            (Some(l), Some(r)) => {
                let root_left = self.find_root_with_compression(l);
                let root_right = self.find_root_with_compression(r);

                if root_left == root_right {
                    return false;
                }

                // SAFETY: both roots are valid and distinct.
                unsafe {
                    if (*root_left).rank < (*root_right).rank {
                        (*root_left).parent = root_right;
                    } else if (*root_left).rank > (*root_right).rank {
                        (*root_right).parent = root_left;
                    } else {
                        (*root_right).parent = root_left;
                        (*root_left).rank += 1;
                    }
                }

                true
            }
            _ => false,
        }
    }

    pub fn same_set(&mut self, left: &T, right: &T) -> bool
    where
        T: std::hash::Hash + Eq,
    {
        let root_left = self.find(left);
        let root_right = self.find(right);
        match (root_left, root_right) {
            (Some(l), Some(r)) => l == r,
            _ => false,
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn clear(&mut self) {
        self.free_all_nodes();
    }

    pub fn components(&self) -> Vec<(SetId, Vec<T>)>
    where
        T: Clone,
    {
        let mut groups: HashMap<SetId, Vec<T>> = HashMap::new();

        for node in self.nodes.iter().copied() {
            let root = self.find_root_without_compression(node);
            // SAFETY: both pointers are valid DSU nodes.
            unsafe {
                groups
                    .entry((*root).id)
                    .or_default()
                    .push((*node).value.clone());
            }
        }

        groups.into_iter().collect()
    }

    pub fn component_views(&self) -> Vec<(DisjointSetView<T>, Vec<DisjointSetView<T>>)>
    where
        T: Clone + std::hash::Hash + Eq,
    {
        let mut groups: HashMap<SetId, Vec<DisjointSetView<T>>> = HashMap::new();

        for node in self.nodes.iter().copied() {
            let root = self.find_root_without_compression(node);
            unsafe {
                groups
                    .entry((*root).id)
                    .or_default()
                    .push(DisjointSetView { node });
            }
        }

        groups
            .into_iter()
            .map(|(root_id, members)| {
                let root_view = self.view_by_set_id(root_id).unwrap();
                (root_view, members)
            })
            .collect()
    }

    pub fn component_count(&self) -> usize
    where
        T: Clone,
    {
        let mut roots = std::collections::HashSet::new();
        for node in self.nodes.iter().copied() {
            unsafe {
                roots.insert((*self.find_root_without_compression(node)).id);
            }
        }
        roots.len()
    }

    pub fn max_rank(&self) -> usize {
        self.nodes
            .iter()
            .copied()
            .map(|node| {
                // SAFETY: pointers in self.nodes are valid.
                unsafe { (*node).rank }
            })
            .max()
            .unwrap_or(0)
    }

    pub fn root_value(&self, set_id: SetId) -> Option<T>
    where
        T: Clone,
    {
        let root = self.representative_node_for_set_id(set_id)?;
        // SAFETY: root is a valid DSU node pointer.
        Some(unsafe { (*root).value.clone() })
    }
}

impl<T> Drop for DisjointSet<T> {
    fn drop(&mut self) {
        self.free_all_nodes();
    }
}

impl<T: std::hash::Hash + Eq + Clone> core_traits::DisjointSet<T> for DisjointSet<T> {
    type SetId = SetId;

    type View<'a>
        = DisjointSetView<T>
    where
        Self: 'a;

    fn make_set(&mut self, value: T) -> Self::SetId {
        DisjointSet::make_set(self, value)
    }

    fn find(&mut self, value: &T) -> Option<Self::SetId> {
        DisjointSet::find(self, value)
    }

    fn union(&mut self, left: &T, right: &T) -> bool {
        DisjointSet::union(self, left, right)
    }

    fn same_set(&mut self, left: &T, right: &T) -> bool {
        DisjointSet::same_set(self, left, right)
    }

    fn view<'a>(&'a self, value: &T) -> Option<Self::View<'a>> {
        DisjointSet::view(self, value)
    }

    fn clear(&mut self) {
        DisjointSet::clear(self)
    }

    fn len(&self) -> usize {
        DisjointSet::len(self)
    }
}

impl<T: Clone + std::hash::Hash + Eq> DisjointSetDiagnostics for DisjointSet<T> {
    type SetId = SetId;
    type Value = T;

    fn element_count(&self) -> usize {
        self.len()
    }

    fn component_count(&self) -> usize {
        DisjointSet::component_count(self)
    }

    fn max_rank(&self) -> usize {
        DisjointSet::max_rank(self)
    }

    fn root_value(&self, set_id: Self::SetId) -> Option<Self::Value> {
        DisjointSet::root_value(self, set_id)
    }
}
