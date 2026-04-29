use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

#[derive(Debug)]
struct DisjointSetNode<T> {
    value: T,
    /// Points to the parent node. If `None`, this node is the root of its set.
    parent: Option<Rc<RefCell<DisjointSetNode<T>>>>,
    /// Upper bound on the depth of the tree rooted at this node.
    rank: usize,
}

impl<T> DisjointSetNode<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            parent: None,
            rank: 0,
        }
    }
}

/// A handle to a specific element within the Disjoint Set.
#[derive(Debug)]
pub struct DisjointSetHandle<T> {
    node: Rc<RefCell<DisjointSetNode<T>>>,
}

impl<T> Clone for DisjointSetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

impl<T> DisjointSetHandle<T> {
    /// Returns a reference to the underlying value.
    pub fn value(&self) -> Ref<'_, T> {
        Ref::map(self.node.borrow(), |node| &node.value)
    }

    /// Returns the direct parent of this handle.
    /// Returns `None` if this handle is the root of its set.
    pub fn parent(&self) -> Option<Self> {
        self.node
            .borrow()
            .parent
            .as_ref()
            .map(|p| Self { node: p.clone() })
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

    /// Creates a new disjoint set containing the given value and returns a handle to it.
    pub fn make_set(&mut self, value: T) -> DisjointSetHandle<T> {
        let node = Rc::new(RefCell::new(DisjointSetNode::new(value)));
        self.nodes.push(node.clone());
        DisjointSetHandle { node }
    }

    /// Finds the root representative of the set containing the element, applying path compression.
    pub fn find(&self, handle: &DisjointSetHandle<T>) -> DisjointSetHandle<T> {
        let mut path = Vec::new();
        let mut current = handle.node.clone();

        loop {
            let parent = current.borrow().parent.clone();
            if let Some(p) = parent {
                path.push(current.clone());
                current = p;
            } else {
                break;
            }
        }

        let root = current;

        for node in path {
            node.borrow_mut().parent = Some(root.clone());
        }

        DisjointSetHandle { node: root }
    }

    /// Merges the two sets containing the elements `x` and `y` using union by rank.
    pub fn union(&mut self, x: &DisjointSetHandle<T>, y: &DisjointSetHandle<T>) {
        let root_x = self.find(x).node;
        let root_y = self.find(y).node;

        if Rc::ptr_eq(&root_x, &root_y) {
            return;
        }

        let mut rx = root_x.borrow_mut();
        let mut ry = root_y.borrow_mut();

        if rx.rank < ry.rank {
            rx.parent = Some(root_y.clone());
        } else if rx.rank > ry.rank {
            ry.parent = Some(root_x.clone());
        } else {
            ry.parent = Some(root_x.clone());
            rx.rank += 1;
        }
    }

    /// Checks if the two elements belong to the same set.
    pub fn same_set(&self, x: &DisjointSetHandle<T>, y: &DisjointSetHandle<T>) -> bool {
        let root_x = self.find(x).node;
        let root_y = self.find(y).node;
        Rc::ptr_eq(&root_x, &root_y)
    }

    /// Returns the total number of elements managed by this Disjoint Set.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns whether the Disjoint Set is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Walks up to the root of `handle`'s set WITHOUT applying path compression.
    fn find_root_handle(&self, handle: &DisjointSetHandle<T>) -> DisjointSetHandle<T> {
        let mut current = handle.node.clone();
        loop {
            let parent = current.borrow().parent.clone();
            match parent {
                Some(p) => current = p,
                None => break,
            }
        }
        DisjointSetHandle { node: current }
    }

    /// Groups all handles by their root representative without modifying tree structure.
    /// Returns a `Vec` of `(root_handle, member_handles)` pairs in insertion order of the root.
    pub fn components(&self) -> Vec<(DisjointSetHandle<T>, Vec<DisjointSetHandle<T>>)> {
        let mut groups: Vec<(DisjointSetHandle<T>, Vec<DisjointSetHandle<T>>)> = Vec::new();
        for node_rc in &self.nodes {
            let handle = DisjointSetHandle {
                node: node_rc.clone(),
            };
            let root = self.find_root_handle(&handle);
            if let Some(entry) = groups
                .iter_mut()
                .find(|(r, _)| Rc::ptr_eq(&r.node, &root.node))
            {
                entry.1.push(handle);
            } else {
                groups.push((root, vec![handle]));
            }
        }
        groups
    }
}

impl<T> Default for DisjointSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_set_and_find() {
        let mut ds = DisjointSet::new();
        let handle1 = ds.make_set(10);
        let handle2 = ds.make_set(20);

        assert_eq!(*ds.find(&handle1).value(), 10);
        assert_eq!(*ds.find(&handle2).value(), 20);
        assert!(!ds.same_set(&handle1, &handle2));
    }

    #[test]
    fn test_simple_union() {
        let mut ds = DisjointSet::new();
        let h1 = ds.make_set(1);
        let h2 = ds.make_set(2);

        ds.union(&h1, &h2);

        assert!(ds.same_set(&h1, &h2));

        let root_val = *ds.find(&h1).value();
        assert!(root_val == 1 || root_val == 2);
    }

    #[test]
    fn test_union_by_rank() {
        let mut ds = DisjointSet::new();
        let h1 = ds.make_set(1);
        let h2 = ds.make_set(2);
        let h3 = ds.make_set(3);
        let h4 = ds.make_set(4);

        ds.union(&h1, &h2);
        ds.union(&h3, &h4);

        ds.union(&h1, &h3);

        assert!(ds.same_set(&h1, &h4));
        assert!(ds.same_set(&h2, &h3));

        let root_val = *ds.find(&h4).value();
        assert!(root_val == 1 || root_val == 3);
    }

    #[test]
    fn test_path_compression() {
        let mut ds = DisjointSet::new();
        let mut handles = Vec::new();

        for i in 0..10 {
            handles.push(ds.make_set(i));
        }

        for i in 0..9 {
            ds.union(&handles[i], &handles[i + 1]);
        }

        let root = ds.find(&handles[9]);

        let parent_ptr = handles[9].node.borrow().parent.clone().unwrap();
        assert!(Rc::ptr_eq(&parent_ptr, &root.node));
    }

    #[test]
    fn test_complex_connected_components() {
        let mut ds = DisjointSet::new();

        let nodes: Vec<_> = (0..100).map(|i| ds.make_set(i)).collect();

        for group in 0..10 {
            for i in 1..10 {
                let u = group * 10;
                let v = group * 10 + i;
                ds.union(&nodes[u], &nodes[v]);
            }
        }

        for group in 0..10 {
            for i in 1..10 {
                assert!(ds.same_set(&nodes[group * 10], &nodes[group * 10 + i]));
            }
        }

        assert!(!ds.same_set(&nodes[0], &nodes[10]));
        assert!(!ds.same_set(&nodes[45], &nodes[55]));

        ds.union(&nodes[0], &nodes[15]);

        assert!(ds.same_set(&nodes[5], &nodes[19]));

        assert!(!ds.same_set(&nodes[0], &nodes[25]));
    }
}
