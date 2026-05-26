# Rust Data Structures Benchmarking Suite

This project is a comprehensive framework for implementing, testing, and benchmarking a wide variety of data structures in **Rust**. The core objective is to analyze the performance and memory tradeoffs between different memory management strategies: **Safe Rust** (smart pointers), **Unsafe Rust** (raw pointers), and **Arena-allocated** memory.

The project features a high-performance Rust library, a rigorous benchmarking suite utilizing industry-standard tools, and an interactive Next.js dashboard for visualizing and exploring the results.

## ✨ Key Features

- **Multi-Variant Strategy**: Every data structure is implemented in Safe, Raw, and Arena variants for head-to-head comparisons.
- **Trait-Driven Design**: A unified API across all structures ensures consistent usage and benchmarking.
- **Industry-Standard Profiling**: Utilizes `Criterion` for timing, `Callgrind` for instruction counting, and `Dhat` for heap profiling.
- **Deterministic Workloads**: Custom data generators provide reproducible benchmarks with various distributions (Uniform, Zipfian, etc.).
- **Interactive Analytics**: A modern Next.js dashboard with complex visualizations (Scatter Plots, Radar Charts, Pareto Frontier analysis).

## 🚀 Project Overview

The workspace is organized into several key components:

- **[`packages/ads`](./packages/ads)**: The core library containing the implementations of all data structures and their respective variants. [Read more in the library README](./packages/ads/README.md).
- **[`packages/ads/benches`](./packages/ads/benches)**: The benchmarking suite utilizing `Criterion.rs`, `Iai-Callgrind`, and `dhat-rs`. [Read more in the benchmarks README](./packages/ads/benches/README.md).
- **[`frontend`](./frontend)**: An interactive Next.js dashboard to explore the benchmark results.
    - **Tradeoff Matrix**: 2D scatter plot visualizing Memory vs. Latency with logarithmic scaling and outlier detection.
    - **Leaderboard Table**: High-performance sorting and comparison table with percentage deltas against baselines.
    - **Inspector Pane**: Comprehensive drilldown into selected implementations, featuring trend analysis and radar charts.
    - **Profiling Integration**: Deep visibility into deterministic instruction counts (Callgrind) and heap allocations (DHAT).
    - **Export System**: One-click export to CSV or Markdown for all views and comparisons.
    - **State Sharing**: Full URL-based state serialization for sharing specific views and filters.
- **[`xtask`](./xtask)**: A custom automation tool used for running benchmarks, aggregating data, and managing the development lifecycle.

## 🛠️ Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Node.js](https://nodejs.org/) & [Yarn](https://yarnpkg.com/) (for the dashboard)
- [Valgrind](https://valgrind.org/) (required for `callgrind` instruction counting)

### Running Benchmarks

All benchmarking and data aggregation tasks are managed via the `xtask` tool. From the project root, you can run:

```bash
# Run the full CI pipeline: benches all structures, profiles memory, and aggregates data
cargo xtask ci

# Run specific benchmark suites with targeted profiling
cargo xtask bench --suite micro --kind criterion,callgrind

# Advanced Filtering: Run only B-Tree insert benchmarks with u64 payloads
cargo xtask bench --implementation btree --operation insert --payload u64

# Run benchmarks in parallel (using 4 jobs) and aggregate results
cargo xtask bench --parallel --jobs 4 --aggregate

# Pin to a single CPU core for consistent results (Linux only)
cargo xtask ci --pin-core 2
```

The results will be aggregated into [`frontend/public/aggregated_benchmarks.json`](./frontend/public/aggregated_benchmarks.json).

### Launching the Interactive Dashboard

Once the benchmarks have been run and data has been aggregated, you can launch the visualization dashboard:

```bash
cd frontend
yarn install
yarn dev
```

The dashboard will be available at `http://localhost:3000`.

## 📚 Documentation

For more detailed information, please refer to the following sub-READMEs:

1. [**Core Library & Data Structures**](./packages/ads/README.md): Detailed information about the implemented data structures, their memory variants, and the trait system.
2. [**Benchmarking Suite**](./packages/ads/benches/README.md): Information on how to run, configure, and interpret the benchmarks.
