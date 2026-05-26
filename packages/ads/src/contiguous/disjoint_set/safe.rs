use std::{
    cell::{Ref, RefCell},
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
        self.parent_id().is_none()
    }
}

#[derive(Debug)]
pub struct DisjointSet<T> {
    nodes: Vec<Rc<RefCell<DisjointSetNode<T>>>>,
}

impl<T> DisjointSet<T> {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
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
        T: PartialEq,
    {
        self.nodes
            .iter()
            .find(|node| node.borrow().value == *value)
            .cloned()
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
        T: PartialEq,
    {
        if let Some(existing) = self.node_by_value(&value) {
            return self.find_root_with_compression(existing).borrow().id;
        }

        let id = SetId(self.nodes.len());
        let node = Rc::new(RefCell::new(DisjointSetNode::new(id, value)));
        self.nodes.push(node);
        id
    }

    pub fn find(&mut self, value: &T) -> Option<SetId>
    where
        T: PartialEq,
    {
        let node = self.node_by_value(value)?;
        Some(self.find_root_with_compression(node).borrow().id)
    }

    pub fn view(&self, value: &T) -> Option<DisjointSetView<T>>
    where
        T: PartialEq,
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
        T: PartialEq,
    {
        let Some(left_node) = self.node_by_value(left) else {
            return false;
        };
        let Some(right_node) = self.node_by_value(right) else {
            return false;
        };

        let root_left = self.find_root_with_compression(left_node);
        let root_right = self.find_root_with_compression(right_node);

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

    pub fn same_set(&mut self, left: &T, right: &T) -> bool
    where
        T: PartialEq,
    {
        let Some(root_left) = self.find(left) else {
            return false;
        };
        let Some(root_right) = self.find(right) else {
            return false;
        };
        root_left == root_right
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    pub fn components(&self) -> Vec<(SetId, Vec<T>)>
    where
        T: Clone,
    {
        let mut groups: Vec<(SetId, Vec<T>)> = Vec::new();

        for node in &self.nodes {
            let (root_id, value) = {
                let b = node.borrow();
                let root_id = self.find_root_without_compression(node.clone()).borrow().id;
                (root_id, b.value.clone())
            };

            if let Some(existing) = groups
                .iter_mut()
                .find(|(group_root, _)| *group_root == root_id)
            {
                existing.1.push(value);
            } else {
                groups.push((root_id, vec![value]));
            }
        }

        groups
    }

    pub fn component_views(&self) -> Vec<(DisjointSetView<T>, Vec<DisjointSetView<T>>)>
    where
        T: Clone + PartialEq,
    {
        self.components()
            .into_iter()
            .filter_map(|(root, members)| {
                let root_view = self.view_by_set_id(root)?;
                let member_views: Vec<_> = members
                    .into_iter()
                    .filter_map(|member_value| self.view(&member_value))
                    .collect();
                Some((root_view, member_views))
            })
            .collect()
    }

    pub fn component_count(&self) -> usize
    where
        T: Clone,
    {
        self.components().len()
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

impl<T: PartialEq> core_traits::DisjointSet<T> for DisjointSet<T> {
    type SetId = crate::contiguous::disjoint_set::safe::SetId;

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

impl<T: Clone> DisjointSetDiagnostics for DisjointSet<T> {
    type SetId = crate::contiguous::disjoint_set::safe::SetId;
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
