pub mod arena;
pub mod raw;
pub mod safe;

#[cfg(test)]
macro_rules! test_skip_list_variant {
    ($module:ident, $list_ty:ident) => {
        mod $module {
            use super::*;
            use std::collections::BTreeMap;
            use rand::{Rng, SeedableRng, rngs::StdRng};
            use std::sync::{Arc, atomic::{AtomicUsize, Ordering as AtomicOrdering}};

            use crate::traits::core::{Map, OrderedMap};

            #[derive(Debug, Clone)]
            struct CountedKey {
                value: i32,
                comparisons: Arc<AtomicUsize>,
            }

            impl CountedKey {
                fn new(value: i32, comparisons: &Arc<AtomicUsize>) -> Self {
                    Self { value, comparisons: comparisons.clone() }
                }
            }

            impl Eq for CountedKey {}
            impl PartialEq for CountedKey {
                fn eq(&self, other: &Self) -> bool { self.value == other.value }
            }

            impl Ord for CountedKey {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    self.comparisons.fetch_add(1, AtomicOrdering::Relaxed);
                    self.value.cmp(&other.value)
                }
            }

            impl PartialOrd for CountedKey {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }

            type List = $list_ty::SkipList<i32, i32>;
            type ComplexityList = $list_ty::SkipList<CountedKey, i32>;

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
            fn stress_random_operations() {
                let mut list = List::new();
                let mut model = BTreeMap::new();
                let mut rng = StdRng::seed_from_u64(42);

                for _ in 0..1000 {
                    match rng.gen_range(0..4) {
                        0 => { // Insert
                            let k = rng.gen_range(0..500);
                            let v = rng.gen_range(0..1000);
                            assert_eq!(list.insert(k, v), model.insert(k, v));
                        }
                        1 if !model.is_empty() => { // Remove
                            let keys: Vec<_> = model.keys().cloned().collect();
                            let k = keys[rng.gen_range(0..keys.len())];
                            assert_eq!(list.remove(&k), model.remove(&k));
                        }
                        2 if !model.is_empty() => { // Search
                            let keys: Vec<_> = model.keys().cloned().collect();
                            let k = keys[rng.gen_range(0..keys.len())];
                            assert!(list.contains_key(&k));
                        }
                        3 => { // Search non-existent
                            let k = rng.gen_range(500..1000);
                            assert!(!list.contains_key(&k));
                        }
                        _ => {}
                    }
                }

                let actual: Vec<_> = list.iter().map(|(k, v)| (*k, *v)).collect();
                let expected: Vec<_> = model.into_iter().collect();
                assert_eq!(actual, expected);
            }

            #[test]
            fn search_comparison_count_scales_logarithmically() {
                fn run_case(size: i32) -> usize {
                    let comparisons = Arc::new(AtomicUsize::new(0));
                    let mut list = ComplexityList::with_config(16, 1, 2);
                    for value in 0..size {
                        list.insert(CountedKey::new(value, &comparisons), value);
                    }

                    comparisons.store(0, AtomicOrdering::Relaxed);
                    let trials = 100;
                    let mut rng = StdRng::seed_from_u64(42);
                    for _ in 0..trials {
                        let value = rng.gen_range(0..size);
                        let key = CountedKey::new(value, &comparisons);
                        assert!(list.contains_key(&key));
                    }
                    comparisons.load(AtomicOrdering::Relaxed) / trials
                }

                let small = run_case(128);
                let large = run_case(1024);

                // For 8x increase in size, log(N) should increase by ~3 comparisons.
                // We'll allow a generous margin due to randomness and trial count.
                assert!(large <= small + 15, "average search cost grew too quickly ({} -> {})", small, large);
            }

            #[test]
            fn indexing_comparison_count_is_minimal() {
                fn run_case(size: i32) -> usize {
                    let comparisons = Arc::new(AtomicUsize::new(0));
                    let mut list = ComplexityList::with_config(16, 1, 2);
                    for value in 0..size {
                        list.insert(CountedKey::new(value, &comparisons), value);
                    }

                    comparisons.store(0, AtomicOrdering::Relaxed);
                    for i in 0..size {
                        assert!(list.cursor_at(i as usize).is_some());
                    }
                    comparisons.load(AtomicOrdering::Relaxed)
                }

                let total = run_case(128);
                // Indexing by position should NOT perform key comparisons in an indexable skip list.
                assert_eq!(total, 0, "indexing by position should perform zero key comparisons");
            }

            #[test]
            fn index_of_key_comparison_count_scales_logarithmically() {
                fn run_case(size: i32) -> usize {
                    let comparisons = Arc::new(AtomicUsize::new(0));
                    let mut list = ComplexityList::with_config(16, 1, 2);
                    for value in 0..size {
                        list.insert(CountedKey::new(value, &comparisons), value);
                    }

                    comparisons.store(0, AtomicOrdering::Relaxed);
                    let trials = 100;
                    let mut rng = StdRng::seed_from_u64(42);
                    for _ in 0..trials {
                        let value = rng.gen_range(0..size);
                        let key = CountedKey::new(value, &comparisons);
                        assert!(list.cursor(&key).is_some());
                    }
                    comparisons.load(AtomicOrdering::Relaxed) / trials
                }

                let small = run_case(128);
                let large = run_case(1024);

                assert!(large <= small + 15, "average index_of_key cost grew too quickly ({} -> {})", small, large);
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
test_skip_list_variant!(safe_variant, safe);
#[cfg(test)]
test_skip_list_variant!(raw_variant, raw);
#[cfg(test)]
test_skip_list_variant!(arena_variant, arena);
