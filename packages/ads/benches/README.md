# ADS Benchmarking Suite

This suite provides a statistically rigorous framework for analyzing the performance characteristics of various data structures and memory management strategies. It is designed to be deterministic, reproducible, and highly informative.

## 🏗️ Benchmark Architecture

The benchmarks are organized by their analytical intent:

### 🔬 Micro Benchmarks ([`micro/`](./micro/))
Focuses on atomic operation throughput:
- **`maps.rs`**: Benchmarks `insert`, `remove`, and `contains` (using Uniform, Zipfian, and Mixed distributions).
    - *Payloads*: `u64`, `String`, and `LargePayload` ([u64; 64]).
    - *Coverage*: All Tree variants (BST, AVL, RBT, B-Tree, Splay), Skip List, and `std` baselines.
- **`sequences.rs`**: Benchmarks `push_back` and `pop_front` for linear structures.
    - *Coverage*: Singly/Doubly Linked Lists (`safe`, `raw`, `arena`), `std::Vec`, `std::VecDeque`, and `std::LinkedList`.
- **`heaps.rs`**: Benchmarks `push` and `pop` throughput for priority queues.
    - *Coverage*: `BinaryHeap`, `BinomialHeap`, `FibonacciHeap` (Safe, Raw, Arena) and `std::BinaryHeap`.
- **`dsu.rs`**: Benchmarks `union` and `find` operations for Disjoint Set Union (Union-Find).
    - *Coverage*: `DisjointSet` (Safe, Raw, Arena).

### 🧪 Macro Benchmarks ([`macro/`](./macro/))
Focuses on complex, interleaved real-world workloads:
- **`read_heavy.rs`**: 95% reads (Zipfian) and 5% updates on prefilled maps.
- **`write_heavy.rs`**: 90% writes (`upsert` + `remove`) and 10% reads on prefilled maps.
- **`thrashing.rs`**: Constant-occupancy churn with interleaved `remove` and `insert` phases.

### 📈 Sweep Benchmarks ([`sweeps/`](./sweeps/))
Focuses on architectural and algorithmic stress testing:
- **`btree_cache.rs`**: Parameterized sweep of B-Tree degrees (T=4, T=16, T=64) to measure cache efficiency.
- **`hash_collisions.rs`**: Forces severe collisions using a `ZeroHasher` to measure performance degradation in HashMaps.

## 🏷️ Benchmark ID Taxonomy

Benchmark IDs are normalized to the format: `<operation>/<implementation>/<payload>/<distribution>/<size>`
*Example: `insert/btree_arena_t8/u64/uniform/10000`*

## 🎲 Deterministic Data Generators

To ensure results are reproducible, we use custom generators ([`generators/`](./generators/)):
- **Uniform**: Randomly shuffled unique keys.
- **Sorted**: Monotonic keys (worst-case for unbalanced trees).
- **Zipfian (80/20)**: Skewed "hot-spot" access pattern.
- **Temporal Locality**: Probes recently accessed or inserted keys.

## 🛠️ Profiling Tools Deep Dive

1.  **[Criterion.rs](https://github.com/bheisler/criterion.rs)**: Wall-clock timing (Mean/Median) with 95% confidence intervals.
2.  **[Iai-Callgrind](https://github.com/iai-callgrind/iai-callgrind)**:
    - `Ir`: Instruction count (deterministic work).
    - `D1mr` / `DLmr`: Data cache read misses.
    - `D1mw` / `DLmw`: Data cache write misses.
3.  **[dhat-rs](https://github.com/rust-itertools/dhat-rs)**:
    - Utilizes specialized targets in [`memory/`](./memory/) to capture precise heap metrics.
    - `max_bytes`: Peak heap usage.
    - `total_bytes` / `total_blocks`: Total allocation volume and count.

## 🚀 Running Benchmarks

Benchmarks are managed via `cargo xtask`.

```bash
# Full CI pipeline
cargo xtask ci --pin-core 2

# Targeted Suites
cargo xtask bench --suite micro --kind criterion,callgrind

# Advanced Filtering
cargo xtask bench --implementation avl --operation insert --payload String

# Parallel Execution
cargo xtask bench --parallel --jobs 8 --aggregate

# Incremental Runs (Only runs changed implementations)
cargo xtask bench --suite macro --incremental
```

### Advanced Filtering Options
- `--implementation`: Filter by implementation name (e.g., `btree`, `avl`, `skip_list`).
- `--operation`: Filter by operation (e.g., `insert`, `remove`, `contains`, `union`).
- `--payload`: Filter by payload type (`u64`, `String`, `LargePayload`).
- `--distribution`: Filter by distribution (`uniform`, `zipfian`, `sorted`).
- `--variant`: Filter by memory variant (`safe`, `raw`, `arena`).

---

*The raw data is aggregated into [`aggregated_benchmarks.json`](../../../frontend/public/aggregated_benchmarks.json) and can be explored in the project's [interactive dashboard](../../frontend).*
