include!("maps.rs");

use criterion::{criterion_group, criterion_main};

criterion_group!(micro_maps_strings_criterion, bench_string_maps);
criterion_main!(micro_maps_strings_criterion);
