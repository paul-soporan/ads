pub trait TreeDiagnostics {
    type NodeCursor<'a>: Clone
    where
        Self: 'a;

    fn height(&self) -> usize;
    fn node_count(&self) -> usize;
    fn node_height<'a>(&'a self, cursor: &Self::NodeCursor<'a>) -> usize;
}

pub trait HashTableDiagnostics {
    fn load_factor(&self) -> f64;
    fn collision_count(&self) -> usize;
    fn bucket_count(&self) -> usize;
}

pub trait ForestDiagnostics {
    fn root_count(&self) -> usize;
    fn node_count(&self) -> usize;
    fn max_root_degree(&self) -> usize;
}

pub trait DisjointSetDiagnostics {
    type SetId: Copy + Eq;
    type Value: Clone;

    fn element_count(&self) -> usize;
    fn component_count(&self) -> usize;
    fn max_rank(&self) -> usize;
    fn root_value(&self, set_id: Self::SetId) -> Option<Self::Value>;
}
