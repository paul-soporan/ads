pub mod arena;
pub mod raw;
pub mod safe;

#[cfg(test)]
macro_rules! test_doubly_linked_list_variant {
    ($module:ident, $list_ty:ty) => {
        mod $module {
            use super::*;
            use std::collections::VecDeque;

            use crate::traits::core::{Sequence, SequenceMutGuard};

            type List = $list_ty;

            #[test]
            fn push_and_pop_front_back() {
                let mut list = List::new();
                assert!(list.is_empty());

                list.push_front(2);
                list.push_front(1);
                list.push_back(3);
                list.push_back(4);

                assert_eq!(list.len(), 4);
                assert_eq!(list.pop_front(), Some(1));
                assert_eq!(list.pop_back(), Some(4));
                assert_eq!(list.pop_front(), Some(2));
                assert_eq!(list.pop_back(), Some(3));
                assert_eq!(list.pop_front(), None);
                assert!(list.is_empty());
            }

            #[test]
            fn cursor_and_get_mut_by_index() {
                let mut list = List::new();
                for value in [5, 10, 15, 20, 25] {
                    list.push_back(value);
                }

                let cursor = list.cursor_at(3).expect("cursor at 3");
                assert_eq!(cursor.index(), 3);
                assert_eq!(*cursor.value(), 20);

                list.get_mut(0)
                    .expect("first value")
                    .with_mut(|value| *value = 7);
                list.get_mut(4)
                    .expect("last value")
                    .with_mut(|value| *value = 30);

                assert_eq!(list.cursor_at(0).map(|c| *c.value()), Some(7));
                assert_eq!(list.cursor_at(4).map(|c| *c.value()), Some(30));
            }

            #[test]
            fn clear_resets_state() {
                let mut list = List::new();
                for value in 0..6 {
                    list.push_back(value);
                }
                assert_eq!(list.len(), 6);

                list.clear();

                assert_eq!(list.len(), 0);
                assert!(list.is_empty());
                assert!(list.cursor_at(0).is_none());
                assert_eq!(list.pop_back(), None);
            }

            #[test]
            fn mixed_operations_match_vecdeque_model() {
                let mut list = List::new();
                let mut model = VecDeque::new();

                for value in [10, 20, 30, 40] {
                    list.push_back(value);
                    model.push_back(value);
                }

                for value in [5, 2] {
                    list.push_front(value);
                    model.push_front(value);
                }

                assert_eq!(list.pop_front(), model.pop_front());
                assert_eq!(list.pop_back(), model.pop_back());

                list.push_back(99);
                model.push_back(99);
                list.push_front(1);
                model.push_front(1);

                let mid = model.len() / 2;
                assert_eq!(
                    list.cursor_at(mid).map(|c| *c.value()),
                    model.get(mid).copied()
                );

                list.get_mut(mid)
                    .expect("mutable cursor")
                    .with_mut(|value| *value -= 7);
                *model.get_mut(mid).expect("model mutable") -= 7;

                let actual: Vec<_> = (0..model.len())
                    .map(|i| *list.cursor_at(i).expect("cursor").value())
                    .collect();
                let expected: Vec<_> = model.iter().copied().collect();
                assert_eq!(actual, expected);
            }
        }
    };
}

#[cfg(test)]
test_doubly_linked_list_variant!(safe_variant, safe::DoublyLinkedList<i32>);
#[cfg(test)]
test_doubly_linked_list_variant!(raw_variant, raw::DoublyLinkedList<i32>);
#[cfg(test)]
test_doubly_linked_list_variant!(arena_variant, arena::DoublyLinkedList<i32>);
