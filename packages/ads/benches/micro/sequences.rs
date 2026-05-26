use std::collections::{LinkedList, VecDeque};

use ads::traits::core::Sequence;
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

type SinglySafe = ads::linked::singly_linked_list::safe::SinglyLinkedList<u64>;
type SinglyRaw = ads::linked::singly_linked_list::raw::SinglyLinkedList<u64>;
type SinglyArena = ads::linked::singly_linked_list::arena::SinglyLinkedList<u64>;
type DoublySafe = ads::linked::doubly_linked_list::safe::DoublyLinkedList<u64>;
type DoublyRaw = ads::linked::doubly_linked_list::raw::DoublyLinkedList<u64>;
type DoublyArena = ads::linked::doubly_linked_list::arena::DoublyLinkedList<u64>;

fn vec_queue_like_workload(size: usize) -> usize {
    let mut values = Vec::with_capacity(size);
    for i in 0..size {
        values.push(i as u64);
    }

    let mut checksum = 0usize;
    for value in values {
        checksum = checksum.wrapping_add(value as usize);
    }
    black_box(checksum)
}

fn vec_deque_workload(size: usize) -> usize {
    let mut values = VecDeque::with_capacity(size);
    for i in 0..size {
        values.push_back(i as u64);
    }

    let mut checksum = 0usize;
    while let Some(value) = values.pop_front() {
        checksum = checksum.wrapping_add(value as usize);
    }
    black_box(checksum)
}

fn linked_list_workload(size: usize) -> usize {
    let mut values = LinkedList::new();
    for i in 0..size {
        values.push_back(i as u64);
    }

    let mut checksum = 0usize;
    while let Some(value) = values.pop_front() {
        checksum = checksum.wrapping_add(value as usize);
    }
    black_box(checksum)
}

fn ads_sequence_workload<S>(size: usize) -> usize
where
    S: Sequence<u64> + Default,
{
    let mut values = S::default();
    for i in 0..size {
        values.push_back(i as u64);
    }

    let mut checksum = 0usize;
    while let Some(value) = values.pop_front() {
        checksum = checksum.wrapping_add(value as usize);
    }
    black_box(checksum)
}

fn sequence_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("micro_sequences");

    for &size in &[1_000usize, 10_000usize, 100_000usize] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(BenchmarkId::new("push_pop/std_vec", size), |b| {
            b.iter(|| vec_queue_like_workload(size));
        });
        group.bench_function(BenchmarkId::new("push_pop/std_vecdeque", size), |b| {
            b.iter(|| vec_deque_workload(size));
        });
        group.bench_function(BenchmarkId::new("push_pop/std_linked_list", size), |b| {
            b.iter(|| linked_list_workload(size));
        });
        group.bench_function(BenchmarkId::new("push_pop/singly_safe", size), |b| {
            b.iter(|| ads_sequence_workload::<SinglySafe>(size));
        });
        group.bench_function(BenchmarkId::new("push_pop/singly_raw", size), |b| {
            b.iter(|| ads_sequence_workload::<SinglyRaw>(size));
        });
        group.bench_function(BenchmarkId::new("push_pop/singly_arena", size), |b| {
            b.iter(|| ads_sequence_workload::<SinglyArena>(size));
        });
        group.bench_function(BenchmarkId::new("push_pop/doubly_safe", size), |b| {
            b.iter(|| ads_sequence_workload::<DoublySafe>(size));
        });
        group.bench_function(BenchmarkId::new("push_pop/doubly_raw", size), |b| {
            b.iter(|| ads_sequence_workload::<DoublyRaw>(size));
        });
        group.bench_function(BenchmarkId::new("push_pop/doubly_arena", size), |b| {
            b.iter(|| ads_sequence_workload::<DoublyArena>(size));
        });
    }

    group.finish();
}

criterion_group!(micro_sequences, sequence_benches);
criterion_main!(micro_sequences);
