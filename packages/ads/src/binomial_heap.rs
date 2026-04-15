use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

#[derive(Debug)]
struct BinomialNode<T> {
    value: T,
    degree: usize,

    parent: Option<Weak<RefCell<BinomialNode<T>>>>,
    child: Option<Rc<RefCell<BinomialNode<T>>>>,
    sibling: Option<Rc<RefCell<BinomialNode<T>>>>,
}

impl<T> BinomialNode<T> {
    pub fn new(value: T) -> Self {
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
pub struct BinomialNodeView<T> {
    node: Rc<RefCell<BinomialNode<T>>>,
}

impl<T> From<Rc<RefCell<BinomialNode<T>>>> for BinomialNodeView<T> {
    fn from(node: Rc<RefCell<BinomialNode<T>>>) -> Self {
        BinomialNodeView { node }
    }
}

impl<T> BinomialNodeView<T> {
    pub fn value(&self) -> Ref<T> {
        Ref::map(self.node.borrow(), |node| &node.value)
    }

    pub fn degree(&self) -> usize {
        self.node.borrow().degree
    }

    pub fn child(&self) -> Option<BinomialNodeView<T>> {
        self.node
            .borrow()
            .child
            .as_ref()
            .map(|child_rc| BinomialNodeView {
                node: child_rc.clone(),
            })
    }

    pub fn sibling(&self) -> Option<BinomialNodeView<T>> {
        self.node
            .borrow()
            .sibling
            .as_ref()
            .map(|sibling_rc| BinomialNodeView {
                node: sibling_rc.clone(),
            })
    }

    pub fn parent(&self) -> Option<BinomialNodeView<T>> {
        self.node
            .borrow()
            .parent
            .as_ref()
            .and_then(|weak_parent| weak_parent.upgrade())
            .map(|parent_rc| BinomialNodeView { node: parent_rc })
    }
}

#[derive(Debug)]
pub struct BinomialHeap<T> {
    head: Option<Rc<RefCell<BinomialNode<T>>>>,
}

impl<T: Ord> BinomialHeap<T> {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn head_view(&self) -> Option<BinomialNodeView<T>> {
        self.head
            .as_ref()
            .map(|h| BinomialNodeView { node: h.clone() })
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn search(&self, value: &T) -> Option<BinomialNodeView<T>> {
        let mut stack = Vec::new();
        let mut current = self.head.clone();

        while let Some(node) = current {
            stack.push(node.clone());
            current = node.borrow().sibling.clone();
        }

        while let Some(node) = stack.pop() {
            if &node.borrow().value == value {
                return Some(BinomialNodeView::from(node));
            }

            if &node.borrow().value < value {
                let mut child = node.borrow().child.clone();
                while let Some(c) = child {
                    stack.push(c.clone());
                    child = c.borrow().sibling.clone();
                }
            }
        }
        None
    }

    pub fn min(&self) -> Option<BinomialNodeView<T>> {
        let mut min_node: Option<Rc<RefCell<BinomialNode<T>>>> = None;
        let mut current = self.head.clone();

        while let Some(node) = current {
            let is_smaller = match &min_node {
                None => true,
                Some(m) => node.borrow().value < m.borrow().value,
            };

            if is_smaller {
                min_node = Some(node.clone());
            }

            let next = node.borrow().sibling.clone();
            current = next;
        }

        min_node.map(BinomialNodeView::from)
    }

    pub fn insert(&mut self, value: T) {
        let mut new_heap = BinomialHeap {
            head: Some(Rc::new(RefCell::new(BinomialNode::new(value)))),
        };
        self.merge(&mut new_heap);
    }

    fn link(y: Rc<RefCell<BinomialNode<T>>>, z: Rc<RefCell<BinomialNode<T>>>) {
        y.borrow_mut().parent = Some(Rc::downgrade(&z));
        let z_child = z.borrow_mut().child.take();
        y.borrow_mut().sibling = z_child;
        z.borrow_mut().child = Some(y);
        z.borrow_mut().degree += 1;
    }

    fn merge_root_lists(
        mut h1: Option<Rc<RefCell<BinomialNode<T>>>>,
        mut h2: Option<Rc<RefCell<BinomialNode<T>>>>,
    ) -> Option<Rc<RefCell<BinomialNode<T>>>> {
        let mut head = None;
        let mut tail_ref: Option<Rc<RefCell<BinomialNode<T>>>> = None;

        while h1.is_some() && h2.is_some() {
            let n1 = h1.as_ref().unwrap().clone();
            let n2 = h2.as_ref().unwrap().clone();

            let take_n1 = n1.borrow().degree <= n2.borrow().degree;

            let next_node = if take_n1 {
                h1 = n1.borrow_mut().sibling.take();
                n1
            } else {
                h2 = n2.borrow_mut().sibling.take();
                n2
            };

            if let Some(t) = &tail_ref {
                t.borrow_mut().sibling = Some(next_node.clone());
            } else {
                head = Some(next_node.clone());
            }
            tail_ref = Some(next_node);
        }

        let rem = if h1.is_some() { h1 } else { h2 };

        if let Some(r) = rem {
            if let Some(t) = &tail_ref {
                t.borrow_mut().sibling = Some(r);
            } else {
                head = Some(r);
            }
        }

        head
    }

    pub fn merge(&mut self, other: &mut Self) {
        let h1 = self.head.take();
        let h2 = other.head.take();

        if h1.is_none() {
            self.head = h2;
            return;
        }
        if h2.is_none() {
            self.head = h1;
            return;
        }

        let mut real_head = Self::merge_root_lists(h1, h2);
        if real_head.is_none() {
            return;
        }

        let mut prev: Option<Rc<RefCell<BinomialNode<T>>>> = None;
        let mut x = real_head.clone().unwrap();
        let mut next = x.borrow().sibling.clone();

        while let Some(n) = next {
            let sibling_of_next = n.borrow().sibling.clone();
            let x_degree = x.borrow().degree;
            let next_degree = n.borrow().degree;
            let next_next_degree = sibling_of_next.as_ref().map(|s| s.borrow().degree);

            if x_degree != next_degree || next_next_degree == Some(x_degree) {
                prev = Some(x.clone());
                x = n.clone();
            } else {
                let x_val_le_next_val = x.borrow().value <= n.borrow().value;
                if x_val_le_next_val {
                    x.borrow_mut().sibling = sibling_of_next.clone();
                    Self::link(n.clone(), x.clone());
                } else {
                    if let Some(p) = &prev {
                        p.borrow_mut().sibling = Some(n.clone());
                    } else {
                        real_head = Some(n.clone());
                    }
                    Self::link(x.clone(), n.clone());
                    x = n.clone();
                }
            }
            next = x.borrow().sibling.clone();
        }

        self.head = real_head;
    }

    pub fn extract_min(&mut self) -> Option<T> {
        if self.head.is_none() {
            return None;
        }

        let mut min_node = self.head.clone().unwrap();
        let mut min_prev = None;

        {
            let mut current = min_node.borrow().sibling.clone();
            let mut prev = Some(min_node.clone());

            while let Some(node) = current {
                if node.borrow().value < min_node.borrow().value {
                    min_node = node.clone();
                    min_prev = prev.clone();
                }
                prev = Some(node.clone());
                let next = node.borrow().sibling.clone();
                current = next;
            }
        }

        if let Some(p) = min_prev {
            p.borrow_mut().sibling = min_node.borrow_mut().sibling.take();
        } else {
            self.head = min_node.borrow_mut().sibling.take();
        }

        let mut child = min_node.borrow_mut().child.take();
        let mut new_head = None;

        while let Some(c) = child {
            let next = c.borrow_mut().sibling.take();
            c.borrow_mut().parent = None;
            c.borrow_mut().sibling = new_head;
            new_head = Some(c.clone());
            child = next;
        }

        let mut new_heap = BinomialHeap { head: new_head };
        self.merge(&mut new_heap);

        Some(
            Rc::try_unwrap(min_node)
                .unwrap_or_else(|_| {
                    unreachable!("Deleted node should only have one strong reference")
                })
                .into_inner()
                .value,
        )
    }

    pub fn decrease_key(&mut self, handle: BinomialNodeView<T>, new_value: T) {
        if new_value > *handle.value() {
            panic!("decrease_key called with a value greater than the current node value.");
        }

        handle.node.borrow_mut().value = new_value;
        let mut current = handle.node.clone();

        loop {
            let parent_rc = {
                let b = current.borrow();
                b.parent.as_ref().and_then(|w| w.upgrade())
            };

            if let Some(p) = parent_rc {
                let needs_swap = current.borrow().value < p.borrow().value;
                if needs_swap {
                    let mut p_mut = p.borrow_mut();
                    let mut c_mut = current.borrow_mut();
                    std::mem::swap(&mut p_mut.value, &mut c_mut.value);

                    drop(p_mut);
                    drop(c_mut);
                    current = p;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub fn delete(&mut self, handle: BinomialNodeView<T>) -> Option<T> {
        let mut current = handle.node.clone();

        loop {
            let parent_rc = {
                let b = current.borrow();
                b.parent.as_ref().and_then(|w| w.upgrade())
            };

            if let Some(p) = parent_rc {
                let mut p_mut = p.borrow_mut();
                let mut c_mut = current.borrow_mut();
                std::mem::swap(&mut p_mut.value, &mut c_mut.value);

                drop(p_mut);
                drop(c_mut);
                current = p;
            } else {
                break;
            }
        }

        let target = current;

        drop(handle);

        {
            let mut prev: Option<Rc<RefCell<BinomialNode<T>>>> = None;
            let mut curr = self.head.clone();

            while let Some(n) = curr {
                if Rc::ptr_eq(&n, &target) {
                    if let Some(p) = prev {
                        p.borrow_mut().sibling = n.borrow_mut().sibling.take();
                    } else {
                        self.head = n.borrow_mut().sibling.take();
                    }
                    break;
                }
                prev = Some(n.clone());
                let next = n.borrow().sibling.clone();
                curr = next;
            }
        }

        let mut child = target.borrow_mut().child.take();
        let mut new_head = None;

        while let Some(c) = child {
            let next = c.borrow_mut().sibling.take();
            c.borrow_mut().parent = None;
            c.borrow_mut().sibling = new_head;
            new_head = Some(c.clone());
            child = next;
        }

        let mut new_heap = BinomialHeap { head: new_head };
        self.merge(&mut new_heap);

        Some(
            Rc::try_unwrap(target)
                .unwrap_or_else(|_| unreachable!("Deleted node should have one strong reference"))
                .into_inner()
                .value,
        )
    }

    pub fn delete_value(&mut self, value: &T) -> Option<T> {
        let handle = self.search(value)?;
        self.delete(handle)
    }
}

impl<T: Ord> Default for BinomialHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_heap() {
        let mut heap = BinomialHeap::<i32>::new();
        assert!(heap.is_empty());
        assert!(heap.min().is_none());
        assert_eq!(heap.extract_min(), None);
    }

    #[test]
    fn test_insert_and_min() {
        let mut heap = BinomialHeap::new();
        heap.insert(5);
        heap.insert(3);
        heap.insert(7);

        assert!(!heap.is_empty());
        assert_eq!(*heap.min().unwrap().value(), 3);
    }

    #[test]
    fn test_extract_min() {
        let mut heap = BinomialHeap::new();
        let values = [5, 3, 7, 2, 4, 6, 8, 1];
        for &v in &values {
            heap.insert(v);
        }

        let mut extracted = Vec::new();
        while let Some(min) = heap.extract_min() {
            extracted.push(min);
        }

        assert_eq!(extracted, vec![1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(heap.is_empty());
    }

    #[test]
    fn test_merge() {
        let mut heap1 = BinomialHeap::new();
        heap1.insert(5);
        heap1.insert(1);
        heap1.insert(8);

        let mut heap2 = BinomialHeap::new();
        heap2.insert(3);
        heap2.insert(7);
        heap2.insert(2);

        heap1.merge(&mut heap2);
        assert!(heap2.is_empty());

        let mut extracted = Vec::new();
        while let Some(min) = heap1.extract_min() {
            extracted.push(min);
        }

        assert_eq!(extracted, vec![1, 2, 3, 5, 7, 8]);
    }

    #[test]
    fn test_duplicates() {
        let mut heap = BinomialHeap::new();
        heap.insert(5);
        heap.insert(3);
        heap.insert(5);
        heap.insert(1);
        heap.insert(3);

        let mut extracted = Vec::new();
        while let Some(min) = heap.extract_min() {
            extracted.push(min);
        }

        assert_eq!(extracted, vec![1, 3, 3, 5, 5]);
    }

    #[test]
    fn test_search() {
        let mut heap = BinomialHeap::new();
        heap.insert(10);
        heap.insert(20);
        heap.insert(30);

        let view = heap.search(&20);
        assert!(view.is_some());
        assert_eq!(*view.unwrap().value(), 20);

        assert!(heap.search(&40).is_none());
    }

    #[test]
    fn test_decrease_key() {
        let mut heap = BinomialHeap::new();
        heap.insert(50);
        heap.insert(40);
        heap.insert(30);

        let handle = heap.search(&50).unwrap();
        heap.decrease_key(handle, 10);

        assert_eq!(*heap.min().unwrap().value(), 10);
        assert_eq!(heap.extract_min(), Some(10));
    }

    #[test]
    fn test_delete_value() {
        let mut heap = BinomialHeap::new();
        let values = [10, 20, 30, 40, 50, 60, 70];
        for &v in &values {
            heap.insert(v);
        }

        let deleted = heap.delete_value(&40);
        assert_eq!(deleted, Some(40));
        assert!(heap.search(&40).is_none());

        let mut extracted = Vec::new();
        while let Some(min) = heap.extract_min() {
            extracted.push(min);
        }

        assert_eq!(extracted, vec![10, 20, 30, 50, 60, 70]);
    }

    #[test]
    fn test_complex_operations() {
        let mut heap = BinomialHeap::new();

        // Insert a large number of elements in reverse order
        for i in (1..=100).rev() {
            heap.insert(i);
        }

        // Search and delete an arbitrary element
        assert_eq!(heap.delete_value(&75), Some(75));

        // Extract half of the elements
        for i in 1..=50 {
            assert_eq!(heap.extract_min(), Some(i));
        }

        // Insert new elements intermixed with what's left
        for i in 101..=150 {
            heap.insert(i);
        }

        // Empty the remaining components of the heap
        let mut remaining = Vec::new();
        while let Some(val) = heap.extract_min() {
            remaining.push(val);
        }

        let mut expected = (51..=150).collect::<Vec<_>>();
        expected.retain(|&x| x != 75); // Ensure 75 is missing

        assert_eq!(remaining, expected);
        assert!(heap.is_empty());
    }
}
