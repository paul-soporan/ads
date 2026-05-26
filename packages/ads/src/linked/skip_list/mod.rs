pub mod arena;
pub mod raw;
pub mod safe;

#[cfg(test)]
macro_rules! test_skip_list_variant {
    ($module:ident, $list_ty:ty) => {
        mod $module {
            use super::*;
            use std::collections::BTreeMap;

            use crate::traits::core::{Map, OrderedMap};

            type List = $list_ty;

            #[test]
            fn insert_contains_remove_are_ordered() {
                let mut list = List::new();
                assert!(list.is_empty());

                assert_eq!(list.insert(20, 200), None);
                assert_eq!(list.insert(5, 50), None);
                assert_eq!(list.insert(15, 150), None);
                assert_eq!(list.insert(40, 400), None);
                assert_eq!(list.insert(15, 151), Some(150));

                assert_eq!(list.len(), 4);
                assert!(list.contains_key(&5));
                assert!(list.contains_key(&15));
                assert!(list.contains_key(&20));
                assert!(list.contains_key(&40));
                assert!(!list.contains_key(&100));

                let values: Vec<_> = list.iter().map(|(k, v)| (*k, *v)).collect();
                assert_eq!(values, vec![(5, 50), (15, 151), (20, 200), (40, 400)]);

                assert_eq!(list.remove(&15), Some(151));
                assert_eq!(list.remove(&15), None);
                assert!(!list.contains_key(&15));
                assert_eq!(
                    list.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
                    vec![5, 20, 40]
                );
            }

            #[test]
            fn cursor_view_and_ordered_boundaries() {
                let mut list = List::new();
                for value in [33, 11, 55, 22, 44] {
                    assert_eq!(list.insert(value, value * 10), None);
                }

                let cursor = list.cursor_at(2).expect("cursor at 2");
                assert_eq!(cursor.index(), 2);
                assert_eq!(*cursor.value(), 330);
                assert!(cursor.level() >= 1);

                let from_key = list.cursor(&33).expect("cursor by key");
                let view = list.view_from_cursor(&from_key);
                assert_eq!(*view.key(), 33);
                assert_eq!(*view.value(), 330);
                assert!(view.level() >= 1);

                let first = list.first_cursor().expect("first cursor");
                let last = list.last_cursor().expect("last cursor");
                assert_eq!(*first.key(), 11);
                assert_eq!(*last.key(), 55);
            }

            #[test]
            fn clear_resets_state() {
                let mut list = List::new();
                for value in 0..7 {
                    assert_eq!(list.insert(value, value), None);
                }
                assert_eq!(list.len(), 7);

                list.clear();

                assert_eq!(list.len(), 0);
                assert!(list.is_empty());
                assert!(list.cursor_at(0).is_none());
                assert!(!list.contains_key(&3));
            }

            #[test]
            fn supports_custom_probability_configuration() {
                let list = List::with_probability(1, 3);
                assert_eq!(list.probability(), (1, 3));

                let tuned = List::with_config(12, 2, 5);
                assert_eq!(tuned.probability(), (2, 5));
                assert_eq!(tuned.max_level(), 12);
            }

            #[test]
            fn mixed_operations_match_btreemap_model() {
                let mut list = List::new();
                let mut model = BTreeMap::new();

                for (k, v) in [(9, 90), (3, 30), (11, 110), (1, 10), (7, 70), (5, 50), (13, 130)]
                {
                    assert_eq!(list.insert(k, v), model.insert(k, v));
                }

                for (k, v) in [(7, 71), (4, 40), (15, 150)] {
                    assert_eq!(list.insert(k, v), model.insert(k, v));
                }

                for key in [1, 4, 9, 100] {
                    assert_eq!(list.contains_key(&key), model.contains_key(&key));
                }

                for key in [3, 11, 42] {
                    assert_eq!(list.remove(&key), model.remove(&key));
                }

                let actual: Vec<_> = list.iter().map(|(k, v)| (*k, *v)).collect();
                let expected: Vec<_> = model.into_iter().collect();
                assert_eq!(actual, expected);
            }

            #[test]
            fn node_levels_respect_configured_max_level() {
                let max_level = 10usize;
                let mut list = List::with_config(max_level, 1, 2);
                for value in 0..512 {
                    let _ = list.insert(value, value);
                }

                let mut max_seen = 0usize;
                for index in 0..list.len() {
                    let level = list.cursor_at(index).expect("cursor").level();
                    max_seen = max_seen.max(level);
                }

                assert!(max_seen <= max_level, "node level exceeds max_level");
            }
        }
    };
}

#[cfg(test)]
test_skip_list_variant!(safe_variant, safe::SkipList<i32, i32>);
#[cfg(test)]
test_skip_list_variant!(raw_variant, raw::SkipList<i32, i32>);
#[cfg(test)]
test_skip_list_variant!(arena_variant, arena::SkipList<i32, i32>);
