include!("maps.rs");

use criterion::{criterion_group, criterion_main};

criterion_group!(
    micro_maps_large_payload_criterion,
    bench_large_payload_maps
);
criterion_main!(micro_maps_large_payload_criterion);
