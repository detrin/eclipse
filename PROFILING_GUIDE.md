# Profiling Guide for Eclipse Minimax Engine

This guide explains how to profile and benchmark the minimax engine to identify performance bottlenecks.

## Quick Start: Basic Benchmarking

Run the included benchmark tool:

```bash
cargo run --release --bin benchmark_minimax
```

This provides:
- Time per move at each difficulty level
- Estimated nodes/second
- Move generation timing
- GameState cloning timing

## Method 1: Flamegraph (Visual Profiling) ⭐ RECOMMENDED

Flamegraphs show exactly where CPU time is spent:

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph (macOS)
cargo flamegraph --bin benchmark_minimax

# On Linux, you may need sudo:
sudo cargo flamegraph --bin benchmark_minimax

# Opens flamegraph.svg in your browser
```

**How to read a flamegraph:**
- Width = time spent (wider = more time)
- Height = call stack depth
- Click on boxes to zoom in
- Look for wide boxes = bottlenecks

## Method 2: Cargo Instruments (macOS Only)

More detailed profiling on macOS:

```bash
# Install cargo-instruments
cargo install cargo-instruments

# Profile with Time Profiler
cargo instruments --release --bin benchmark_minimax -t "Time Profiler"

# Profile with Allocations
cargo instruments --release --bin benchmark_minimax -t "Allocations"
```

## Method 3: Add Performance Counters

For detailed internal metrics, you can add counters to `MinimaxBot`. Here's what to track:

### Key Metrics to Track:

1. **Nodes Searched** - Total positions evaluated
2. **Transposition Table Hit Rate** - % of positions found in cache
3. **Alpha-Beta Cutoff Rate** - % of branches pruned
4. **Average Branching Factor** - Average moves per position
5. **Quiescence Nodes** - Nodes searched in quiescence
6. **Null Move Cutoffs** - How often null move pruning works

### Example Implementation:

Add to `MinimaxBot` struct:
```rust
pub struct MinimaxBot {
    // ... existing fields ...

    // Performance counters (only enabled in debug/profiling builds)
    #[cfg(feature = "profiling")]
    nodes_searched: RefCell<usize>,
    #[cfg(feature = "profiling")]
    tt_hits: RefCell<usize>,
    #[cfg(feature = "profiling")]
    tt_misses: RefCell<usize>,
    #[cfg(feature = "profiling")]
    cutoffs: RefCell<usize>,
}
```

Then add a feature flag in `Cargo.toml`:
```toml
[features]
profiling = []
```

Run with profiling enabled:
```bash
cargo run --release --features profiling --bin benchmark_minimax
```

## Method 4: Criterion Benchmarks (Precise Microbenchmarks)

For precise, statistical benchmarking:

```bash
# Install criterion
cargo install cargo-criterion

# Add to Cargo.toml:
# [dev-dependencies]
# criterion = "0.5"
#
# [[bench]]
# name = "minimax_bench"
# harness = false

# Create benches/minimax_bench.rs with criterion tests

# Run benchmarks
cargo criterion
```

## Common Bottlenecks

Based on typical minimax implementations, check these areas:

### 1. Move Generation (get_legal_moves)
**Symptom**: Move generation takes >1µs per call

**Profile**: Already included in benchmark_minimax
```
Move generation:
  10,000 calls: 0.027s (2.7µs per call)  ← Current performance
```

**If slow, optimize:**
- Cache legal moves per position
- Generate moves lazily
- Pre-compute adjacency maps

### 2. Position Evaluation
**Symptom**: Most time spent in `evaluate_position` on flamegraph

**If slow, optimize:**
- Simplify evaluation function for Easy/Medium
- Cache expensive computations (mobility, protection)
- Use incremental evaluation

### 3. Position Hashing (Transposition Table)
**Symptom**: `hash_position` appears wide on flamegraph

**If slow, optimize:**
- Use Zobrist hashing (incremental)
- Simplify hash computation
- Reduce hash collisions

### 4. Excessive Cloning
**Symptom**: `GameState::clone` appears on flamegraph

**Note**: Already optimized with undo/redo system! Should be minimal now.

### 5. Low Cache Hit Rate
**Symptom**: Transposition table not helping much

**Check:**
- Is table size large enough? (currently 100k entries)
- Are positions hashing correctly?
- Is eviction policy too aggressive?

### 6. High Branching Factor
**Symptom**: Searching too many nodes even with pruning

**Optimize:**
- Improve move ordering (killer moves, history heuristic)
- More aggressive null move pruning
- Late move reductions (LMR)

## Interpreting Results

### Good Performance Targets:

| Metric | Target | Current (estimated) |
|--------|--------|---------------------|
| Nodes/second | >100k | ~186k (Medium) |
| Move generation | <5µs | ~2.7µs ✓ |
| GameState clone | <1µs | ~0.1µs ✓ |
| TT hit rate | >30% | Unknown - add counters |
| Cutoff rate | >50% | Unknown - add counters |

### Performance Comparison:

**From test suite results:**
- Easy (depth 2): ~3ms per move
- Medium (depth 3): ~18ms per move
- Hard (depth 4): ~6ms per move (benefits from iterative deepening + caching)

**Test suite total: ~15s for 113 tests** (reasonable)

## Next Steps

1. **Run flamegraph first** - See where time is actually spent
2. **Add performance counters** - Track TT hits, cutoffs, nodes searched
3. **Profile on real game positions** - Opening vs midgame vs endgame
4. **Compare before/after** - Measure impact of each optimization

## Advanced: Profile-Guided Optimization (PGO)

For maximum performance, use PGO:

```bash
# Step 1: Build with instrumentation
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" cargo build --release

# Step 2: Run to collect profile data
./target/release/benchmark_minimax

# Step 3: Build with profile data
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data/merged.profdata" cargo build --release
```

Can improve performance by 5-15% by optimizing hot paths.

## Questions to Answer with Profiling

1. **Which function takes the most time?**
   - Use flamegraph to identify

2. **How many nodes are we searching?**
   - Add counter in minimax()

3. **Is the transposition table helping?**
   - Track hits vs misses

4. **Are we pruning effectively?**
   - Track cutoff rate vs total branches

5. **Which difficulty is slowest per node?**
   - Compare nodes/second across difficulties

6. **What's the average branching factor?**
   - Track moves per position

## Summary

**Start here:**
1. Run `cargo run --release --bin benchmark_minimax`
2. Run `cargo flamegraph --bin benchmark_minimax`
3. Identify the widest boxes on flamegraph
4. Add counters for that specific area
5. Optimize and measure improvement

**Remember:** Always profile before optimizing! Don't guess where the bottleneck is.
