use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

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
    parent: Option<Rc<RefCell<DisjointSetNode<T>>>>,
    rank: usize,
}

impl<T> DisjointSetNode<T> {
    fn new(id: SetId, value: T) -> Self {
        Self {
            id,
            value,
            parent: None,
            rank: 0,
        }
    }
}

#[derive(Debug)]
pub struct DisjointSetView<T> {
    node: Rc<RefCell<DisjointSetNode<T>>>,
}

impl<T> Clone for DisjointSetView<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl<T> DisjointSetView<T> {
    pub fn set_id(&self) -> SetId {
        let mut current = self.node.clone();
        loop {
            let parent = current.borrow().parent.clone();
            match parent {
                Some(parent) => current = parent,
                None => break,
            }
        }
        current.borrow().id
    }

    pub fn value(&self) -> Ref<'_, T> {
        Ref::map(self.node.borrow(), |node| &node.value)
    }

    pub fn parent_id(&self) -> Option<SetId> {
        let mut current = self.node.borrow().parent.clone()?;
        loop {
            let parent = current.borrow().parent.clone();
            match parent {
                Some(parent) => current = parent,
                None => break,
            }
        }
        Some(current.borrow().id)
    }

    pub fn rank(&self) -> usize {
        self.node.borrow().rank
    }

    pub fn is_root(&self) -> bool {
        self.node.borrow().parent.is_none()
    }
}

#[derive(Debug)]
pub struct DisjointSet<T> {
    nodes: Vec<Rc<RefCell<DisjointSetNode<T>>>>,
    value_map: HashMap<T, usize>,
}

impl<T> DisjointSet<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            value_map: HashMap::new(),
        }
    }

    fn node_at_slot(&self, slot: usize) -> Option<Rc<RefCell<DisjointSetNode<T>>>> {
        self.nodes.get(slot).cloned()
    }

    fn representative_node_for_set_id(
        &self,
        set_id: SetId,
    ) -> Option<Rc<RefCell<DisjointSetNode<T>>>> {
        let node = self.node_at_slot(set_id.index())?;
        Some(self.find_root_without_compression(node))
    }

    fn node_by_value(&self, value: &T) -> Option<Rc<RefCell<DisjointSetNode<T>>>>
    where
        T: std::hash::Hash + Eq,
    {
        let &idx = self.value_map.get(value)?;
        self.nodes.get(idx).cloned()
    }

    fn find_root_with_compression(
        &mut self,
        node: Rc<RefCell<DisjointSetNode<T>>>,
    ) -> Rc<RefCell<DisjointSetNode<T>>> {
        let mut path = Vec::new();
        let mut current = node;

        loop {
            let parent = current.borrow().parent.clone();
            if let Some(parent) = parent {
                path.push(current.clone());
                current = parent;
            } else {
                break;
            }
        }

        let root = current;
        for path_node in path {
            path_node.borrow_mut().parent = Some(root.clone());
        }

        root
    }

    fn find_root_without_compression(
        &self,
        node: Rc<RefCell<DisjointSetNode<T>>>,
    ) -> Rc<RefCell<DisjointSetNode<T>>> {
        let mut current = node;
        loop {
            let parent = current.borrow().parent.clone();
            match parent {
                Some(parent) => current = parent,
                None => break,
            }
        }
        current
    }

    pub fn make_set(&mut self, value: T) -> SetId
    where
        T: std::hash::Hash + Eq + Clone,
    {
        if let Some(existing) = self.node_by_value(&value) {
            return self.find_root_with_compression(existing).borrow().id;
        }

        let idx = self.nodes.len();
        let id = SetId(idx);
        let node = Rc::new(RefCell::new(DisjointSetNode::new(id, value.clone())));
        self.nodes.push(node);
        self.value_map.insert(value, idx);
        id
    }

    pub fn find(&mut self, value: &T) -> Option<SetId>
    where
        T: std::hash::Hash + Eq,
    {
        let node = self.node_by_value(value)?;
        Some(self.find_root_with_compression(node).borrow().id)
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

                if Rc::ptr_eq(&root_left, &root_right) {
                    return false;
                }

                let mut left_mut = root_left.borrow_mut();
                let mut right_mut = root_right.borrow_mut();

                if left_mut.rank < right_mut.rank {
                    left_mut.parent = Some(root_right.clone());
                } else if left_mut.rank > right_mut.rank {
                    right_mut.parent = Some(root_left.clone());
                } else {
                    right_mut.parent = Some(root_left.clone());
                    left_mut.rank += 1;
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
        self.nodes.clear();
        self.value_map.clear();
    }

    pub fn components(&self) -> Vec<(SetId, Vec<T>)>
    where
        T: Clone,
    {
        let mut groups: HashMap<SetId, Vec<T>> = HashMap::new();

        for node in &self.nodes {
            let root_id = self.find_root_without_compression(node.clone()).borrow().id;
            groups
                .entry(root_id)
                .or_default()
                .push(node.borrow().value.clone());
        }

        groups.into_iter().collect()
    }

    pub fn component_views(&self) -> Vec<(DisjointSetView<T>, Vec<DisjointSetView<T>>)>
    where
        T: Clone + std::hash::Hash + Eq,
    {
        let mut groups: HashMap<SetId, Vec<DisjointSetView<T>>> = HashMap::new();

        for node in &self.nodes {
            let root_node = self.find_root_without_compression(node.clone());
            let root_id = root_node.borrow().id;
            groups
                .entry(root_id)
                .or_default()
                .push(DisjointSetView { node: node.clone() });
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
        for node in &self.nodes {
            roots.insert(self.find_root_without_compression(node.clone()).borrow().id);
        }
        roots.len()
    }

    pub fn max_rank(&self) -> usize {
        self.nodes
            .iter()
            .map(|node| node.borrow().rank)
            .max()
            .unwrap_or(0)
    }

    pub fn root_value(&self, set_id: SetId) -> Option<T>
    where
        T: Clone,
    {
        let root = self.representative_node_for_set_id(set_id)?;
        Some(root.borrow().value.clone())
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
        self.component_count()
    }

    fn max_rank(&self) -> usize {
        self.max_rank()
    }

    fn root_value(&self, set_id: Self::SetId) -> Option<Self::Value> {
        DisjointSet::root_value(self, set_id)
    }
}

impl<T> Default for DisjointSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
