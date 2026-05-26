pub trait Map<K, V> {
    type Cursor<'a>: Clone
    where
        Self: 'a;

    type View<'a>: Clone
    where
        Self: 'a;

    fn insert(&mut self, key: K, value: V) -> Option<V>;
    fn cursor<'a>(&'a self, key: &K) -> Option<Self::Cursor<'a>>;
    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a>;
    fn remove(&mut self, key: &K) -> Option<V>;
    fn contains_key(&self, key: &K) -> bool;
    fn clear(&mut self);
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait OrderedMap<K, V>: Map<K, V> {
    fn first_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>>;
    fn last_cursor<'a>(&'a self) -> Option<Self::Cursor<'a>>;
}

pub trait Set<K>: Map<K, ()> {
    fn insert_key(&mut self, key: K) -> bool {
        self.insert(key, ()).is_none()
    }

    fn contains(&self, key: &K) -> bool {
        self.contains_key(key)
    }

    fn remove_key(&mut self, key: &K) -> bool {
        self.remove(key).is_some()
    }
}

pub trait OrderedSet<K>: Set<K> + OrderedMap<K, ()> {}

impl<K, T> OrderedSet<K> for T where T: Set<K> + OrderedMap<K, ()> {}

pub trait CollectionSetOps<K>: Set<K> {
    fn union_keys(&self, other: &Self) -> Vec<K>
    where
        K: Clone + Ord,
        Self: Sized;

    fn intersection_keys(&self, other: &Self) -> Vec<K>
    where
        K: Clone + Ord,
        Self: Sized;
}

pub trait SequenceMutGuard<T> {
    fn with_mut<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R;
}

impl<T> SequenceMutGuard<T> for &mut T {
    fn with_mut<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(self)
    }
}

pub trait Sequence<T> {
    type Cursor<'a>
    where
        Self: 'a;

    type MutView<'a>: SequenceMutGuard<T>
    where
        Self: 'a,
        T: 'a;

    fn push_front(&mut self, value: T);
    fn push_back(&mut self, value: T);
    fn pop_front(&mut self) -> Option<T>;
    fn pop_back(&mut self) -> Option<T>;
    fn cursor_at<'a>(&'a self, index: usize) -> Option<Self::Cursor<'a>>;
    fn get_mut<'a>(&'a mut self, index: usize) -> Option<Self::MutView<'a>>;
    fn clear(&mut self);
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait PriorityQueue<T> {
    type Cursor<'a>: Clone
    where
        Self: 'a;

    type View<'a>: Clone
    where
        Self: 'a;

    fn push(&mut self, value: T);
    fn pop(&mut self) -> Option<T>;
    fn peek<'a>(&'a self) -> Option<Self::Cursor<'a>>;
    fn cursor<'a>(&'a self, value: &T) -> Option<Self::Cursor<'a>>;
    fn view_from_cursor<'a>(&'a self, cursor: &Self::Cursor<'a>) -> Self::View<'a>;
    fn remove_cursor<'a>(&mut self, cursor: Self::Cursor<'a>) -> Option<T>
    where
        T: 'a;
    fn clear(&mut self);
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait DisjointSet<T> {
    type SetId: Copy + Eq;

    type View<'a>: Clone
    where
        Self: 'a;

    fn make_set(&mut self, value: T) -> Self::SetId;
    fn find(&mut self, value: &T) -> Option<Self::SetId>;
    fn union(&mut self, left: &T, right: &T) -> bool;
    fn same_set(&mut self, left: &T, right: &T) -> bool;
    fn view<'a>(&'a self, value: &T) -> Option<Self::View<'a>>;
    fn clear(&mut self);
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait SpatialIndex<P, V> {
    fn insert(&mut self, point: P, value: V) -> Option<V>;
    fn nearest_neighbor(&self, point: &P) -> Option<(&P, &V)>;
    fn range_search<'a>(&'a self, min: &P, max: &P) -> Vec<(&'a P, &'a V)>;
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait ProbabilisticSet<T> {
    fn insert(&mut self, value: T);
    fn might_contain(&self, value: &T) -> bool;
    fn false_positive_rate(&self) -> f64;
    fn clear(&mut self);
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
