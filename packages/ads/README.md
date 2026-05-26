# ADS Core Library

The `ads` package is the heart of the project. It contains highly optimized implementations of various data structures, each provided in three distinct memory management variants to allow for rigorous performance and safety analysis.

## 🏗️ Data Structure Categories

The library is organized by physical memory topology:

### 🌲 Trees (Single-Rooted Hierarchical)
- **Binary Search Tree (BST)**: Simple unbalanced binary tree. [Source](./src/trees/binary_search_tree/)
- **AVL Tree**: Self-balancing binary search tree using height-based rotations. [Source](./src/trees/avl_tree/)
- **Red-Black Tree**: Self-balancing binary search tree using color-based balancing. [Source](./src/trees/red_black_tree/)
- **B-Tree**: Balanced multi-way search tree, optimized for cache efficiency. Supports configurable degrees via const generics. [Source](./src/trees/b_tree/)
- **Splay Tree**: Self-adjusting binary search tree that moves frequently accessed elements to the root. [Source](./src/trees/splay_tree/)

### 🌳 Forests (Multi-Rooted Hierarchical)
- **Binomial Heap**: A forest of binomial trees supporting fast merges. [Source](./src/forests/binomial_heap/)
- **Fibonacci Heap**: A more advanced heap with better amortized performance. [Source](./src/forests/fibonacci_heap/)

### 🔗 Linked (Pointer-Chasing Sequential)
- **Singly Linked List**: Standard one-way pointer-chasing list. [Source](./src/linked/singly_linked_list/)
- **Doubly Linked List**: Two-way pointer-chasing list. [Source](./src/linked/doubly_linked_list/)
- **Skip List**: Probabilistic data structure for fast search within a sorted sequence. [Source](./src/linked/skip_list/)

### 📦 Contiguous (Array-Backed)
- **Binary Heap**: Implicit tree represented as a contiguous array. [Source](./src/contiguous/binary_heap/)
- **Disjoint Set (Union-Find)**: Efficiently manages partitions of a set with path compression and union by rank. [Source](./src/contiguous/disjoint_set/)

## 🛡️ Memory Management Variants

Pointer-based structures are implemented using three strategies:

1.  **Safe (`safe.rs`)**: Uses standard Rust smart pointers (`Box`, `Rc`, `RefCell`, `Weak`). This variant is 100% safe Rust and serves as the baseline for safety and idiomatic Rust design.
2.  **Raw (`raw.rs`)**: Uses raw pointers (`*mut T`) and manual memory management (`std::alloc`). This variant explores the performance gains of bypassing runtime safety checks while maintaining structural integrity.
3.  **Arena (`arena.rs`)**: Uses an arena allocator where nodes are stored in a contiguous `Vec` and referenced by `usize` handles. This strategy significantly improves cache locality, reduces heap fragmentation, and allows for extremely fast mass-deallocation.

## ⚙️ Design Patterns & Trait System

### Purpose Traits ([`traits/core.rs`](./src/traits/core.rs))
A tiered trait system ensures a unified API across all implementations:
- `Map<K, V>`, `OrderedMap<K, V>`, `Sequence<T>`, `PriorityQueue<T>`, `DisjointSet<T>`.

### Cursor & View Pattern
To ensure safe navigation without exposing internal pointers, the library utilizes:
- **Cursor**: A stateful navigator for traversing a structure (e.g., finding a node in a tree). It borrows the owning structure to ensure safety.
- **View**: A read-only, lightweight window into a node's data and its immediate neighbors (children, parent), decoupled from the structure's mutation logic.

### Diagnostic Traits ([`traits/diagnostics.rs`](./src/traits/diagnostics.rs))
Exposes internal metadata for analytical profiling:
- `TreeDiagnostics`, `ForestDiagnostics`, `DisjointSetDiagnostics`.

## 🧪 Testing & Validation

Safety is paramount, especially for `Raw` and `Arena` variants:
- **Miri Validation**: Mathematical proof of the absence of Undefined Behavior, pointer aliasing violations, and memory leaks.

---

*For details on benchmarking, see the [Benchmarks README](./benches/README.md).*
