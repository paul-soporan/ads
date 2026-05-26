#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;

use ads::traits::core::DisjointSet;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main, black_box};

fn dsu_workload<D>(size: usize) -> usize
where
    D: common::BenchDisjointSet,
{
    let mut ds = D::new();
    for i in 0..size {
        ds.make_set_value(i as u64);
    }

    for i in 0..(size / 2) {
        ds.union_values(&(i as u64), &((i + size / 2) as u64));
    }

    let mut checksum = 0usize;
    for i in 0..size {
        if ds.same_set_values(&(i as u64), &0u64) {
            checksum = checksum.wrapping_add(1);
        }
    }
    black_box(checksum)
}

struct NaiveDisjointSet {
    parent: Vec<usize>,
}

impl NaiveDisjointSet {
    fn new(size: usize) -> Self {
        Self { parent: (0..size).collect() }
    }

    fn find(&self, i: usize) -> usize {
        self.parent[i]
    }

    fn union(&mut self, i: usize, j: usize) {
        let root_i = self.find(i);
        let root_j = self.find(j);
        if root_i != root_j {
            for k in 0..self.parent.len() {
                if self.parent[k] == root_j {
                    self.parent[k] = root_i;
                }
            }
        }
    }
}

fn naive_dsu_workload(size: usize) -> usize {
    let mut ds = NaiveDisjointSet::new(size);
    for i in 0..(size / 2) {
        ds.union(i, i + size / 2);
    }

    let mut checksum = 0usize;
    for i in 0..size {
        if ds.find(i) == ds.find(0) {
            checksum = checksum.wrapping_add(1);
        }
    }
    black_box(checksum)
}

fn dsu_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("micro_dsu_u64");

    for &size in &[1_000usize, 10_000usize] {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_function(BenchmarkId::new("union_find/ads_dsu_safe", size), |b| {
            b.iter(|| dsu_workload::<common::DsuSafe>(size));
        });

        group.bench_function(BenchmarkId::new("union_find/ads_dsu_raw", size), |b| {
            b.iter(|| dsu_workload::<common::DsuRaw>(size));
        });

        group.bench_function(BenchmarkId::new("union_find/ads_dsu_arena", size), |b| {
            b.iter(|| dsu_workload::<common::DsuArena>(size));
        });
    }

    group.finish();
}

fn motivational_dsu_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("motivational_dsu_connectivity_u64");

    for &size in &[100usize, 500usize, 1000usize] {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_function(BenchmarkId::new("union_find/naive_O_N_union", size), |b| {
            b.iter(|| naive_dsu_workload(size));
        });

        group.bench_function(BenchmarkId::new("union_find/ads_dsu_arena_O_alpha_N", size), |b| {
            b.iter(|| dsu_workload::<common::DsuArena>(size));
        });
    }
    group.finish();
}

criterion_group!(micro_dsu, dsu_benches, motivational_dsu_benches);
criterion_main!(micro_dsu);
