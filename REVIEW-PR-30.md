# PR #30 Review (Round 2): Parallelize MCCFR training with rayon sharding

**Branch:** `claude/parallelize-mccfr-training-5BjNT`
**Commits:** `1a2ed84` + `228a71d` (address review feedback)
**Files changed:** `src/main.rs`, `src/solver/mccfr.rs`, `src/bin/commander_goldfish.rs`

## Summary

Adds parallel goldfish MCCFR training with rayon sharding, progress reporting,
and checkpointing. Also adds a `train_parallel_basic` wrapper for 2-player
MCCFR. Updates both `main.rs` and `commander_goldfish` to use parallel training.

## Verdict: Approve

All five review items from Round 1 have been addressed. The code is clean,
tests pass (67/67), and the API layering is well-designed. Two minor nits below
that are non-blocking.

## Review of addressed items

### 1. commander_goldfish binary updated — Good

Now uses `train_goldfish_parallel_with_progress` with `AtomicU32`-based progress
reporting and 2-second throttling. The old `Cell<Instant>` approach was correctly
replaced with `AtomicU32` seconds tracking that works across threads.

Note: The old progress callback reported `info_sets` and `exploit` mid-training.
The new parallel callback can't access shard-private tables, so those are
omitted. This is the right trade-off — the doc comment on
`train_goldfish_parallel_with_progress` (line 712-713) explicitly calls this out:
"Because tables are shard-private, the callback cannot inspect regret tables
mid-training — use checkpointing for intermediate snapshots."

### 2. train_parallel_basic deduplication — Good

Reduced from ~40 lines of duplicated shard logic to a 5-line wrapper:
```rust
let train_config = TrainConfig {
    mccfr: config.clone(),
    ..TrainConfig::default()
};
train_parallel(initial_state, num_iterations, num_shards, &train_config)
```
Clean delegation. `McfrConfig` derives `Clone` so `config.clone()` is fine.

### 3. Progress reporting in parallel goldfish — Good

`train_goldfish_parallel_with_progress` uses a shared `AtomicU32` with
`Ordering::Relaxed` (line 777). `Relaxed` is appropriate here — the counter is
advisory for progress display, not used for synchronization. The callback
signature `Fn(u32, u32, &AtomicU32) + Send + Sync` is correct.

### 4. Encapsulated rayon dependency — Good

`default_num_shards()` (line 84-89) encapsulates `rayon::current_num_threads()`
in the solver module. Both callers use `mccfr::default_num_shards()` now.

### 5. Checkpoint support in parallel goldfish — Good

Per-shard checkpointing via `checkpoint_interval: Option<u32>` and
`checkpoint_dir: Option<&str>`. Consistent with the existing `train_parallel`
approach. Silently ignores checkpoint errors (`let _ = save_checkpoint(...)`) —
matches existing convention.

## API layering

The function hierarchy is clean:

```
train_goldfish_parallel                           (simple entry point)
  -> train_goldfish_parallel_with_progress        (full-featured, 9 params)

train_goldfish_parallel_with_abstraction          (mid-level convenience)
  -> train_goldfish_parallel_with_progress

train_parallel_basic                              (simple 2-player)
  -> train_parallel                               (existing, full-featured)
```

## Minor nits (non-blocking)

### Nit 1: Progress callback race in commander_goldfish — cosmetic only

In `commander_goldfish.rs:91-113`, multiple threads may concurrently evaluate
`should_print` and both pass the throttle check before either stores the
updated `last_print_secs`. This means occasionally 2-3 threads may print
within the same second window. This is purely cosmetic (interleaved `eprint!`
output) and not worth fixing with a `compare_exchange` — the output is
human-readable status, not correctness-critical.

### Nit 2: `train_goldfish_parallel_with_progress` has 9 parameters

The function takes 9 positional parameters. A builder or config struct could
improve ergonomics, but this matches the existing codebase style (e.g.,
`traverse_goldfish` takes 8 params) and the convenience wrappers
(`train_goldfish_parallel`, `train_goldfish_parallel_with_abstraction`) mitigate
the usability concern. Fine as-is for an internal solver API.

## Tests

All 67 unit tests pass with the PR applied. The three new tests cover:
- `test_train_goldfish_parallel` — happy path, validates info sets and visit counts
- `test_train_parallel_basic` — 2-player parallel with default config
- `test_train_goldfish_parallel_more_shards_than_iterations` — edge case

Build succeeds with no warnings.
