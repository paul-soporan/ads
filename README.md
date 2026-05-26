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
    - **Tradeoff Matrix**: 2D scatter plot visualizing Memory vs. Latency.
    - **Variant Drilldown**: Radar charts comparing Safe/Raw/Arena performance.
    - **Versus Mode**: Direct head-to-head comparison of up to 4 implementations.
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

# Run specific benchmark suites
cargo xtask bench --suite micro
cargo xtask bench --suite macro

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
