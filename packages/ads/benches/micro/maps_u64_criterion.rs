include!("maps.rs");

use criterion::{criterion_group, criterion_main};

criterion_group!(micro_maps_u64_criterion, bench_u64_maps);
criterion_main!(micro_maps_u64_criterion);
