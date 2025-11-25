# Performance Optimization Specification

## Purpose

Define coding standards and optimization techniques for maintaining high-performance Rust code in the wayvid video wallpaper engine, ensuring efficient CPU and memory usage across all critical paths.

## Requirements

### PO-1: Inline Annotations for Hot Paths
**Priority**: High  
**Status**: Active

All functions called in rendering loops (60+ fps), memory allocation paths, or FFI boundaries must be marked with `#[inline]` to eliminate function call overhead.

**Scope**:
- Frame timing functions (begin_frame, end_frame, record_skip)
- Memory tracking (track_allocation, track_deallocation)
- FFI property access (get_property_*)
- Layout calculations (calculate_layout)
- Type predicates called frequently (is_streaming, is_image_sequence)

**Rationale**: Rust doesn't inline across crate boundaries by default. Explicit `#[inline]` is required for hot paths to achieve optimal performance.

**Validation**:
- Criterion benchmarks show <5% overhead in hot functions
- Flamegraph analysis confirms inlining occurred

---

### PO-2: Minimize Allocations in Hot Loops
**Priority**: High  
**Status**: Active

Avoid heap allocations, string copies, and collection reallocations in code paths executed >30 times per second.

**Techniques**:
- Pre-allocate buffers with known capacity
- Use `&str` instead of `String` where possible
- Reuse existing allocations (buffer pools)
- Check collection length before removal operations
- Use stack-allocated arrays for small fixed-size data

**Anti-patterns**:
```rust
// BAD: Allocates on every frame
fn render() {
    let temp = Vec::new();  // Allocation!
    for item in items {
        temp.push(process(item));
    }
}

// GOOD: Pre-allocate or reuse
fn render() {
    let mut temp = Vec::with_capacity(items.len());
    // Or reuse existing buffer
}
```

**Validation**:
- Profiling shows <1% time in allocator functions
- Memory usage remains stable during sustained operation

---

### PO-3: Correct Atomic Ordering
**Priority**: Critical  
**Status**: Active

All atomic operations must use appropriate memory ordering to prevent race conditions while minimizing synchronization overhead.

**Guidelines**:
- **Relaxed**: Counter increments with no dependencies (e.g., statistics)
- **Acquire/Release**: Synchronization points (e.g., compare_exchange for shared state)
- **SeqCst**: Only when sequential consistency is required (rare)

**Example**:
```rust
// Statistics tracking (independent)
FRAME_COUNT.fetch_add(1, Ordering::Relaxed);

// Peak usage update (requires synchronization)
PEAK_USAGE.compare_exchange_weak(
    old, new,
    Ordering::Release,  // Success: publish new value
    Ordering::Acquire   // Failure: load current value
);
```

**Rationale**: Incorrect ordering can cause race conditions. Overly strict ordering hurts performance unnecessarily.

**Validation**:
- Code review confirms appropriate ordering choices
- No data races detected under ThreadSanitizer

---

### PO-4: Lock Contention Minimization
**Priority**: High  
**Status**: Active

Minimize time spent holding locks, especially in multi-threaded scenarios (multiple video outputs).

**Techniques**:
- Scope locks tightly (drop before expensive operations)
- Consider `RwLock` for read-heavy workloads
- Use lock-free structures where appropriate
- Batch operations to reduce lock acquisition frequency

**Example**:
```rust
// BAD: Lock held during allocation
let mut pool = self.pool.lock().unwrap();
if pool.is_empty() {
    pool.push(allocate_buffer());  // Lock held!
}

// GOOD: Release lock before allocation
let needs_alloc = {
    let pool = self.pool.lock().unwrap();
    pool.is_empty()
};
if needs_alloc {
    let buf = allocate_buffer();  // No lock
    self.pool.lock().unwrap().push(buf);
}
```

**Validation**:
- Profiling shows <5% time in lock contention
- Multi-monitor setups scale linearly

---

### PO-5: Efficient Iteration Patterns
**Priority**: Medium  
**Status**: Active

Use idiomatic Rust iteration patterns that compile to efficient machine code.

**Patterns**:
- Prefer `iter()` over indexing when order doesn't matter
- Use `enumerate()` instead of manual counters
- Leverage `find()`, `filter()`, `map()` for transformations
- Use `then_some()` for conditional Some() returns

**Example**:
```rust
// Efficient pattern matching
(condition).then_some(expensive_value())  // Only evaluates if true

// Efficient iteration
items.iter()
    .filter(|x| x.is_valid())
    .map(|x| x.process())
    .collect()
```

**Validation**:
- Generated assembly shows optimized loops (SIMD where applicable)
- Benchmarks match hand-optimized versions

---

### PO-6: Caching Frequently Accessed Data
**Priority**: Medium  
**Status**: Active

Cache expensive computations or property queries when results are stable.

**Examples**:
- Video dimensions (query once, cache until source changes)
- Layout transforms (recompute only on resize)
- Configuration-derived constants

**Implementation**:
```rust
pub struct Player {
    cached_dimensions: Option<(i32, i32)>,
}

impl Player {
    #[inline]
    pub fn get_dimensions(&mut self) -> Option<(i32, i32)> {
        if let Some(dims) = self.cached_dimensions {
            return Some(dims);  // Fast path
        }
        // Query and cache
        let dims = self.query_mpv_dimensions()?;
        self.cached_dimensions = Some(dims);
        Some(dims)
    }
}
```

**Validation**:
- Property access shows cache hit rate >95%
- Profiling confirms reduced FFI overhead

---

### PO-7: Profiling and Benchmarking Requirements
**Priority**: High  
**Status**: Planned

All performance-critical changes must be validated with profiling data and benchmarks.

**Tools**:
- **Flamegraph**: Identify hot paths and call stacks
- **Criterion**: Microbenchmarks for specific functions
- **Perf**: System-level profiling (cache misses, branch predictions)

**Process**:
1. Establish baseline measurements
2. Apply optimization
3. Re-run benchmarks and compare
4. Document results in PR/change proposal

**Thresholds**:
- Optimizations must show ≥5% improvement to justify code complexity
- No regressions allowed in critical paths
- Memory usage must not increase by >10%

---

## Scenarios

### SC-1: Optimizing Video Rendering Loop
**Priority**: Critical

**Given**: Video playing at 60 fps on a 4K display  
**When**: Frame timing and render operations execute  
**Then**:
- Frame timing overhead <1ms per frame
- Memory allocations <10 per second (steady state)
- CPU usage <15% on modern hardware (single output)

**Implementation**:
- Inline frame timing functions
- Pre-allocate frame history buffer
- Cache video dimensions
- Minimize lock contention in shared decode path

---

### SC-2: Multi-Monitor Performance Scaling
**Priority**: High

**Given**: Three 4K displays with independent video sources  
**When**: All outputs render simultaneously  
**Then**:
- CPU usage scales linearly (~3x single output)
- No lock contention between render threads
- Frame timing remains stable (<2ms jitter)

**Implementation**:
- Per-output buffer pools (no shared locks)
- Lock-free memory tracking
- Efficient layout calculation caching

---

### SC-3: Memory-Constrained Environments
**Priority**: Medium

**Given**: System with 4GB RAM, integrated GPU  
**When**: Playing 4K video with HDR tone mapping  
**Then**:
- Total memory usage <500MB
- No memory leaks over 24h operation
- Buffer pool size self-limits based on available memory

**Implementation**:
- Bounded buffer pools with LRU eviction
- Precise allocation tracking
- Periodic memory usage reporting

---

## Non-Functional Requirements

### NFR-1: Performance Regression Prevention
All CI pipelines must detect performance regressions before merge:
- Benchmark suite runs on performance-critical PRs
- Automated flamegraph comparison
- Alert on >10% performance degradation

### NFR-2: Profiling Documentation
Performance optimization techniques must be documented:
- Inline comments explain non-obvious optimizations
- Change proposals include profiling data
- PERFORMANCE_SUMMARY.md tracks cumulative improvements

### NFR-3: Optimization Maintainability
Performance optimizations must not sacrifice code clarity:
- Use descriptive variable names even in hot paths
- Document why specific orderings/patterns are used
- Avoid premature optimization (profile first)

---

## References

- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- Rust std docs: Atomic ordering, inline attributes
- MPV render API performance considerations
- Wayland protocol efficiency guidelines

## Change Log

- **2025-11-25**: Initial specification created
  - 7 requirements covering inlining, allocations, atomics, locks, iteration, caching, profiling
  - 3 critical scenarios for rendering, multi-monitor, memory-constrained environments
  - 3 non-functional requirements for regression prevention and maintainability
