pub mod safe;

#[cfg(test)]
mod tests {
    use super::safe::BinaryHeap;
    use crate::traits::core::PriorityQueue;

    fn assert_min_heap_invariant(heap: &BinaryHeap<i32>) {
        let data = heap.as_slice();
        for index in 0..data.len() {
            let left = 2 * index + 1;
            let right = 2 * index + 2;

            if left < data.len() {
                assert!(data[index] <= data[left], "left child violates min-heap order");
            }
            if right < data.len() {
                assert!(
                    data[index] <= data[right],
                    "right child violates min-heap order"
                );
            }
        }
    }

    #[test]
    fn empty_heap_behaves_as_expected() {
        let mut heap = BinaryHeap::<i32>::new();

        assert!(heap.is_empty());
        assert_eq!(heap.len(), 0);
        assert!(heap.peek().is_none());
        assert_eq!(heap.pop(), None);

        heap.push(10);
        assert!(!heap.is_empty());
        heap.clear();
        assert!(heap.is_empty());
    }

    #[test]
    fn push_peek_and_pop_are_min_heap_ordered() {
        let mut heap = BinaryHeap::new();
        for value in [5, 3, 9, 1, 8, 2, 4] {
            heap.push(value);
            assert_min_heap_invariant(&heap);
        }

        let cursor = heap.peek().expect("peek cursor");
        let view = heap.view_from_cursor(&cursor);
        assert_eq!(*view.value(), 1);

        let mut popped = Vec::new();
        while let Some(value) = heap.pop() {
            popped.push(value);
            assert_min_heap_invariant(&heap);
        }

        assert_eq!(popped, vec![1, 2, 3, 4, 5, 8, 9]);
        assert!(heap.is_empty());
    }

    #[test]
    fn cursor_and_view_expose_heap_position() {
        let mut heap = BinaryHeap::new();
        for value in [20, 7, 12, 30, 18, 22, 25] {
            heap.push(value);
        }

        let cursor = heap.cursor(&18).expect("cursor for 18");
        let view = heap.view_from_cursor(&cursor);

        assert_eq!(*view.value(), 18);
        assert_eq!(view.index(), cursor.index());

        if let Some(parent_index) = view.parent_index() {
            let parent_value = heap
                .data_at(parent_index)
                .expect("parent value should exist");
            assert!(*parent_value <= *view.value());
        }
    }

    #[test]
    fn remove_cursor_removes_target_value() {
        let mut heap = BinaryHeap::new();
        for value in [11, 6, 17, 3, 9, 14, 20] {
            heap.push(value);
        }

        let cursor = heap.cursor(&14).expect("cursor for 14");
        assert_eq!(heap.remove_cursor(cursor), Some(14));
        assert!(heap.cursor(&14).is_none());
        assert_min_heap_invariant(&heap);

        let mut popped = Vec::new();
        while let Some(value) = heap.pop() {
            popped.push(value);
            assert_min_heap_invariant(&heap);
        }
        assert_eq!(popped, vec![3, 6, 9, 11, 17, 20]);
    }

    #[test]
    fn stale_cursor_is_rejected_after_mutation() {
        let mut heap = BinaryHeap::new();
        for value in [4, 10, 7] {
            heap.push(value);
        }

        let cursor = heap.cursor(&10).expect("cursor for 10");
        heap.push(2);

        assert_eq!(heap.remove_cursor(cursor), None);
    }

    #[test]
    fn cursor_is_stale_after_pop_and_clear() {
        let mut heap = BinaryHeap::new();
        for value in [9, 1, 5, 3] {
            heap.push(value);
        }

        let cursor = heap.cursor(&5).expect("cursor for 5");
        let _ = heap.pop();
        assert_eq!(heap.remove_cursor(cursor.clone()), None);

        let fresh = heap.cursor(&9).expect("cursor for 9");
        heap.clear();
        assert_eq!(heap.remove_cursor(fresh), None);
        assert!(heap.is_empty());
    }

    #[test]
    fn mixed_operations_match_sorted_reference() {
        let mut heap = BinaryHeap::new();
        let mut expected = Vec::new();

        for value in [12, 4, 18, 7, 2, 10, 15, 1] {
            heap.push(value);
            expected.push(value);
            assert_min_heap_invariant(&heap);
        }

        let removed = heap.cursor(&10).expect("cursor 10");
        assert_eq!(heap.remove_cursor(removed), Some(10));
        expected.retain(|v| *v != 10);
        assert_min_heap_invariant(&heap);

        expected.sort_unstable();
        let mut actual = Vec::new();
        while let Some(value) = heap.pop() {
            actual.push(value);
            assert_min_heap_invariant(&heap);
        }

        assert_eq!(actual, expected);
    }

    #[test]
    fn from_iterator_builds_valid_heap() {
        let mut heap: BinaryHeap<i32> = [9, 2, 7, 2, 5, 1].into_iter().collect();
        assert_eq!(heap.len(), 6);

        let mut popped = Vec::new();
        while let Some(value) = heap.pop() {
            popped.push(value);
        }

        assert_eq!(popped, vec![1, 2, 2, 5, 7, 9]);
    }
}
