pub mod arena;
pub mod raw;
pub mod safe;

use crate::traits::{core::DisjointSet as DisjointSetTrait, diagnostics::DisjointSetDiagnostics};

pub trait DisjointSetVariant<T>:
    DisjointSetTrait<T>
    + DisjointSetDiagnostics<Value = T, SetId = <Self as DisjointSetTrait<T>>::SetId>
{
}

impl<T: Clone + std::hash::Hash + Eq> DisjointSetVariant<T> for safe::DisjointSet<T> {}
impl<T: Clone + std::hash::Hash + Eq> DisjointSetVariant<T> for raw::DisjointSet<T> {}
impl<T: Clone + std::hash::Hash + Eq> DisjointSetVariant<T> for arena::DisjointSet<T> {}

#[cfg(test)]
macro_rules! test_disjoint_set_variant {
    ($module:ident, $ds_ty:ident) => {
        mod $module {
            use super::*;
            use crate::traits::diagnostics::DisjointSetDiagnostics;

            type Ds = $ds_ty::DisjointSet<i32>;

            #[test]
            fn make_set_find_and_same_set() {
                let mut ds = Ds::new();
                ds.make_set(10);
                ds.make_set(20);

                let root_h1 = ds.find(&10).expect("root");
                let root_h2 = ds.find(&20).expect("root");

                assert_eq!(ds.root_value(root_h1), Some(10));
                assert_eq!(ds.root_value(root_h2), Some(20));
                assert_eq!(*ds.view(&10).expect("view").value(), 10);
                assert_eq!(*ds.view(&20).expect("view").value(), 20);
                assert!(!ds.same_set(&10, &20));
                assert_eq!(ds.element_count(), 2);
                assert_eq!(ds.component_count(), 2);
            }

            #[test]
            fn union_by_rank_and_path_compression() {
                let mut ds = Ds::new();
                for i in 0..10 {
                    ds.make_set(i);
                }

                for i in 0..9 {
                    ds.union(&i, &(i + 1));
                }

                let root = ds.find(&9).expect("root");
                let node_view = ds.view(&9).expect("view");
                assert_eq!(node_view.parent_id(), Some(root));
                assert!(ds.same_set(&0, &9));
                assert!(ds.max_rank() > 0);

                let root_again = ds.find(&9).expect("root again");
                assert_eq!(root_again, root);
                assert_eq!(ds.view(&9).expect("view again").parent_id(), Some(root));
            }

            #[test]
            fn components() {
                let mut ds = Ds::new();
                for i in 0..12 {
                    ds.make_set(i);
                }

                for group in 0..3 {
                    let base = group * 4;
                    ds.union(&base, &(base + 1));
                    ds.union(&(base + 1), &(base + 2));
                    ds.union(&(base + 2), &(base + 3));
                }

                let mut sizes: Vec<_> = ds.components().into_iter().map(|(_, m)| m.len()).collect();
                sizes.sort_unstable();
                assert_eq!(sizes, vec![4, 4, 4]);
                assert_eq!(ds.component_count(), 3);
            }

            #[test]
            fn duplicate_make_set_reuses_existing_component() {
                let mut ds = Ds::new();
                let root_a = ds.make_set(42);
                let root_b = ds.make_set(42);

                assert_eq!(root_a, root_b);
                assert_eq!(ds.element_count(), 1);
                assert_eq!(ds.component_count(), 1);
                assert_eq!(ds.root_value(root_a), Some(42));
            }

            #[test]
            fn union_is_idempotent_and_component_count_stable() {
                let mut ds = Ds::new();
                for value in 0..6 {
                    ds.make_set(value);
                }

                assert!(ds.union(&0, &1));
                assert!(!ds.union(&0, &1));
                assert!(!ds.union(&1, &0));
                assert_eq!(ds.component_count(), 5);

                assert!(ds.union(&2, &3));
                assert!(ds.union(&3, &4));
                assert!(ds.same_set(&2, &4));
                assert_eq!(ds.component_count(), 3);
            }

            #[test]
            fn stress_random_operations() {
                use rand::{Rng, SeedableRng, rngs::StdRng};

                let mut ds = Ds::new();
                let mut rng = StdRng::seed_from_u64(42);
                let mut values = Vec::new();

                for _ in 0..1000 {
                    match rng.gen_range(0..3) {
                        0 => { // make_set
                            let val = rng.gen_range(0..2000);
                            ds.make_set(val);
                            values.push(val);
                        }
                        1 if !values.is_empty() => { // union
                            let v1 = values[rng.gen_range(0..values.len())];
                            let v2 = values[rng.gen_range(0..values.len())];
                            ds.union(&v1, &v2);
                        }
                        2 if !values.is_empty() => { // find/same_set
                            let v1 = values[rng.gen_range(0..values.len())];
                            let v2 = values[rng.gen_range(0..values.len())];
                            let _ = ds.same_set(&v1, &v2);
                        }
                        _ => {}
                    }
                }

                assert!(ds.element_count() > 0);
                assert!(ds.component_count() > 0);
                assert!(ds.component_count() <= ds.element_count());
            }
        }
    };
}

#[cfg(test)]
test_disjoint_set_variant!(safe_variant, safe);
#[cfg(test)]
test_disjoint_set_variant!(raw_variant, raw);
#[cfg(test)]
test_disjoint_set_variant!(arena_variant, arena);
