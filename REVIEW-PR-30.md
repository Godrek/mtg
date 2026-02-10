# PR #30 Review: Parallelize MCCFR training with rayon sharding

**Branch:** `claude/parallelize-mccfr-training-5BjNT`
**Commit:** `1a2ed84` — "Parallelize MCCFR training with rayon sharding"
**Files changed:** `src/main.rs` (+6/-2), `src/solver/mccfr.rs` (+207)

## Summary

Adds two new parallel training entry points:
- `train_goldfish_parallel` / `train_goldfish_parallel_with_abstraction` — parallel goldfish (solitaire) MCCFR
- `train_parallel_basic` — parallel 2-player MCCFR with default settings

Each distributes iterations across rayon thread-pool shards with independent `RegretTable` instances, then merges by summing cumulative regret/strategy values. Updates `main.rs` to use parallel goldfish training with auto-detected thread count.

Includes three unit tests covering the happy path, 2-player variant, and the edge case where `num_shards > num_iterations`.

## Verdict: Request Changes

The implementation is structurally sound and follows the established `train_parallel` pattern correctly. The math for merging independent CFR shards is valid. However, there are several issues ranging from code duplication to a missing update in the `commander_goldfish` binary.

## Issues

### 1. `commander_goldfish` binary not updated (Medium)

`src/bin/commander_goldfish.rs:81` still uses `train_goldfish_with_progress` (sequential). If the goal is to parallelize goldfish training, this is the most computationally expensive user-facing entry point and should benefit from parallelism too. At minimum, document why it was intentionally left sequential (e.g., progress callback incompatibility), or add a parallel variant with progress support.

### 2. `train_parallel_basic` largely duplicates `train_parallel` (Medium)

`train_parallel_basic` (new, lines 735-776 in the PR) duplicates the shard-distribution and merge pattern from the existing `train_parallel` (lines 506-564). The only difference is that `train_parallel_basic` calls `run_iteration` with default settings while `train_parallel` takes a `TrainConfig`.

This could be a one-liner wrapper:

```rust
pub fn train_parallel_basic(
    initial_state: &GameState,
    num_iterations: u32,
    num_shards: u32,
    config: &McfrConfig,
) -> [RegretTable; 2] {
    train_parallel(initial_state, num_iterations, num_shards, &TrainConfig::from_mccfr(config))
}
```

If `TrainConfig` doesn't have a convenient constructor from `McfrConfig`, adding one would be cleaner than duplicating ~40 lines of parallel iteration logic.

### 3. No progress reporting in parallel goldfish (Low-Medium)

The sequential `train_goldfish_with_progress` supports a progress callback that reports iteration count, elapsed time, and exploitability. The parallel variant has no equivalent. For long training runs (which is the primary use case for parallelism), the user gets no feedback until completion. Consider adding an `AtomicU32` iteration counter with periodic progress reporting, similar to how other parallel MCCFR frameworks handle this.

### 4. `main.rs` change leaks rayon dependency into caller (Low)

In `src/main.rs:21`, the `goldfish_mccfr_report` function now calls `rayon::current_num_threads()` directly:

```rust
let num_shards = rayon::current_num_threads() as u32;
```

This couples the caller to rayon. A cleaner approach would be to either:
- Have `train_goldfish_parallel` accept `num_shards: Option<u32>` and default to `rayon::current_num_threads()` internally, or
- Add a `train_goldfish_parallel_auto` that picks shard count automatically

This keeps rayon as an implementation detail of the solver module.

### 5. No checkpoint support in parallel goldfish (Low)

The existing `train_parallel` function supports per-shard checkpointing via `TrainConfig`. The new `train_goldfish_parallel_with_abstraction` does not. For consistency, consider whether goldfish parallel training should also support checkpointing, especially given that goldfish training at scale is the motivating use case.

## Positives

- **Correct parallelism model**: Independent shards with post-hoc merge is the right approach for MCCFR — cumulative regret/strategy sums are order-independent, so merging is mathematically sound.
- **Proper iteration distribution**: The `iterations_per_shard + remainder` logic correctly handles uneven division and the edge case where shards exceed iterations.
- **Good test coverage**: Tests the happy path, 2-player variant, and the `num_shards > num_iterations` edge case.
- **Clean integration**: Reuses `traverse_goldfish`, `run_iteration`, and `merge_regret_tables` without modification.
- **Build and tests pass**: All 67 unit tests pass with the PR applied. No warnings.

## Recommendation

Address issue #1 (commander_goldfish binary) and #2 (code duplication) before merging. The other issues are lower priority and could be follow-ups.
