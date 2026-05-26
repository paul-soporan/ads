use std::collections::HashMap;

use crate::traits::{core as core_traits, diagnostics::DisjointSetDiagnostics};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetId(pub usize);

impl SetId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub struct DisjointSetView<'a, T> {
    tree: &'a DisjointSet<T>,
    idx: usize,
}

impl<'a, T> Clone for DisjointSetView<'a, T> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree,
            idx: self.idx,
        }
    }
}

impl<'a, T> DisjointSetView<'a, T> {
    pub fn set_id(&self) -> SetId {
        SetId(self.tree.find_root_without_compression(self.idx))
    }

    pub fn value(&self) -> &T {
        &self.tree.values[self.idx]
    }

    pub fn parent_id(&self) -> Option<SetId> {
        let parent = self.tree.parents[self.idx];
        if parent == self.idx {
            None
        } else {
            Some(SetId(self.tree.find_root_without_compression(parent)))
        }
    }

    pub fn rank(&self) -> usize {
        self.tree.ranks[self.idx]
    }

    pub fn is_root(&self) -> bool {
        self.tree.parents[self.idx] == self.idx
    }
}

#[derive(Debug)]
pub struct DisjointSet<T> {
    parents: Vec<usize>,
    ranks: Vec<usize>,
    values: Vec<T>,
    value_map: HashMap<T, usize>,
}

impl<T> DisjointSet<T> {
    pub fn new() -> Self {
        Self {
            parents: Vec::new(),
            ranks: Vec::new(),
            values: Vec::new(),
            value_map: HashMap::new(),
        }
    }

    fn find_root_with_compression(&mut self, mut idx: usize) -> usize {
        let mut path = Vec::new();
        while self.parents[idx] != idx {
            path.push(idx);
            idx = self.parents[idx];
        }
        for p_idx in path {
            self.parents[p_idx] = idx;
        }
        idx
    }

    fn find_root_without_compression(&self, mut idx: usize) -> usize {
        while self.parents[idx] != idx {
            idx = self.parents[idx];
        }
        idx
    }

    pub fn make_set(&mut self, value: T) -> SetId
    where
        T: std::hash::Hash + Eq + Clone,
    {
        if let Some(&idx) = self.value_map.get(&value) {
            return SetId(self.find_root_with_compression(idx));
        }

        let idx = self.values.len();
        self.parents.push(idx);
        self.ranks.push(0);
        self.values.push(value.clone());
        self.value_map.insert(value, idx);
        SetId(idx)
    }

    pub fn find(&mut self, value: &T) -> Option<SetId>
    where
        T: std::hash::Hash + Eq,
    {
        let &idx = self.value_map.get(value)?;
        Some(SetId(self.find_root_with_compression(idx)))
    }

    pub fn union(&mut self, left: &T, right: &T) -> bool
    where
        T: std::hash::Hash + Eq,
    {
        let l_idx = self.value_map.get(left).copied();
        let r_idx = self.value_map.get(right).copied();

        match (l_idx, r_idx) {
            (Some(l), Some(r)) => {
                let root_l = self.find_root_with_compression(l);
                let root_r = self.find_root_with_compression(r);

                if root_l == root_r {
                    return false;
                }

                if self.ranks[root_l] < self.ranks[root_r] {
                    self.parents[root_l] = root_r;
                } else if self.ranks[root_l] > self.ranks[root_r] {
                    self.parents[root_r] = root_l;
                } else {
                    self.parents[root_r] = root_l;
                    self.ranks[root_l] += 1;
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
        let l_root = self.find(left);
        let r_root = self.find(right);
        match (l_root, r_root) {
            (Some(l), Some(r)) => l == r,
            _ => false,
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn clear(&mut self) {
        self.parents.clear();
        self.ranks.clear();
        self.values.clear();
        self.value_map.clear();
    }

    pub fn components(&self) -> Vec<(SetId, Vec<T>)>
    where
        T: Clone,
    {
        let mut groups: HashMap<usize, Vec<T>> = HashMap::new();
        for i in 0..self.values.len() {
            let root = self.find_root_without_compression(i);
            groups.entry(root).or_default().push(self.values[i].clone());
        }
        groups
            .into_iter()
            .map(|(root, members)| (SetId(root), members))
            .collect()
    }

    pub fn component_views(&self) -> Vec<(DisjointSetView<'_, T>, Vec<DisjointSetView<'_, T>>)>
    where
        T: Clone + std::hash::Hash + Eq,
    {
        let mut groups: HashMap<usize, Vec<DisjointSetView<'_, T>>> = HashMap::new();
        for i in 0..self.values.len() {
            let root = self.find_root_without_compression(i);
            groups
                .entry(root)
                .or_default()
                .push(DisjointSetView { tree: self, idx: i });
        }
        groups
            .into_iter()
            .map(|(root, members)| (DisjointSetView { tree: self, idx: root }, members))
            .collect()
    }

    pub fn component_count(&self) -> usize {
        let mut roots = std::collections::HashSet::new();
        for i in 0..self.values.len() {
            roots.insert(self.find_root_without_compression(i));
        }
        roots.len()
    }

    pub fn max_rank(&self) -> usize {
        self.ranks.iter().copied().max().unwrap_or(0)
    }

    pub fn root_value(&self, set_id: SetId) -> Option<T>
    where
        T: Clone,
    {
        let idx = set_id.index();
        if idx < self.values.len() {
            let root_idx = self.find_root_without_compression(idx);
            Some(self.values[root_idx].clone())
        } else {
            None
        }
    }
}

impl<T: std::hash::Hash + Eq + Clone> core_traits::DisjointSet<T> for DisjointSet<T> {
    type SetId = SetId;
    type View<'a> = DisjointSetView<'a, T> where Self: 'a;

    fn make_set(&mut self, value: T) -> Self::SetId {
        self.make_set(value)
    }

    fn find(&mut self, value: &T) -> Option<Self::SetId> {
        self.find(value)
    }

    fn union(&mut self, left: &T, right: &T) -> bool {
        self.union(left, right)
    }

    fn same_set(&mut self, left: &T, right: &T) -> bool {
        self.same_set(left, right)
    }

    fn view<'a>(&'a self, value: &T) -> Option<Self::View<'a>> {
        self.value_map.get(value).map(|&idx| DisjointSetView { tree: self, idx })
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn len(&self) -> usize {
        self.len()
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
        self.root_value(set_id)
    }
}

impl<T> Default for DisjointSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
