# Review: PR #46 — Add multithreaded MCTS via root parallelization

**Branch:** `claude/add-mcts-multithreading-LNzKB`
**Commit:** `650ea28` — "Add multithreaded MCTS via root parallelization"
**Files changed:** 3 (+230, -1)

## Summary

This PR adds optional multithreading to MCTS decisions using root parallelization.
Each thread builds an independent search tree with a share of the total iterations,
then per-action visit counts and rewards are summed to select the best action.
A new `num_threads` field on `MctsConfig` (default 1) controls the behavior, and the
`mcts_goldfish` binary exposes it via a `THREADS` env var.

## Verdict: Approve with suggestions

The design is clean and well-suited for MCTS. Root parallelization is the right first
approach — it avoids lock contention entirely since each tree is thread-local, and the
merge-by-visit-count strategy is standard in the MCTS literature. The code is
well-structured, backward-compatible, and the iteration distribution handles remainders
correctly.

## Detailed Findings

### P1 — No issues found

No correctness bugs identified. The parallel/single-threaded dispatch is correct,
the merge logic correctly sums visits and rewards, and thread safety is sound
(`GameState` and `MctsConfig` are implicitly `Send + Sync`; `MctsNode` is only
used thread-locally).

### P2 — Suggestions

#### 1. No integration test exercises the parallel path

All six existing integration tests in `tests/integration_test.rs` set `num_threads: 1`.
There is no test that verifies the parallel path produces valid results end-to-end.

**Suggestion:** Add at least one integration test with `num_threads: 2` (or 4) and a
small iteration count to confirm the parallel path doesn't panic or produce degenerate
results. Something like:

```rust
#[test]
fn test_mcts_parallel_goldfish_completes() {
    let db = sample::build_sample_db();
    let red = sample::red_deck();
    let config = MctsConfig {
        iterations_per_move: 100,
        exploration_constant: 1.0,
        max_tree_depth: 0,
        max_rollout_actions: 2_000,
        num_threads: 2,
    };
    let result = simulation::run_mcts_goldfish_game(&db, &red, &config, false);
    assert!(result.kill_turn.is_some() || result.actions_taken > 0);
}
```

#### 2. Nested parallelism with the outer game loop

`simulate_mcts_goldfish` (simulation/mod.rs:679) already runs games in parallel via
`(0..num_games).into_par_iter()`. If `num_threads > 1` in the config, each game's MCTS
decisions also spawn parallel work on the same rayon thread pool. Rayon handles nested
parallelism via work-stealing without creating extra threads, but the inner par_iter
tasks compete with outer tasks for the same pool, which can reduce effective parallelism
of the outer loop.

**Suggestion:** Document this interaction. Consider either:
- Automatically setting `num_threads = 1` when called from `simulate_mcts_goldfish`
  (which already has game-level parallelism), or
- Adding a note in the `MctsConfig` docs that `num_threads > 1` is most beneficial for
  single-game scenarios (interactive mode, debugging) rather than batch simulations.

#### 3. Root node tree data discarded in parallel path

In single-threaded mode, if a caller passes a `root` with an existing sub-tree,
`mcts_search_single` continues building on it. In `mcts_search_parallel`, each thread
creates a fresh `MctsNode::new()`, so any existing sub-tree in `root` is overwritten
with merged stats from the fresh trees.

This doesn't affect current usage since `MctsStrategy::choose_action` creates a new
root per decision. But it's a behavioral difference worth documenting in the function's
doc comment, in case future callers attempt tree reuse.

### P3 — Nits

#### 4. Unit tests verify merge logic in isolation, not via the actual function

`test_parallel_merge_sums_visits` manually reproduces the merge loop rather than calling
`mcts_search_parallel`. If the merge implementation changes, the test won't catch
regressions. This is understandable given the difficulty of constructing a full
`GameState` in a unit test, but worth noting.

#### 5. Consider `usize` for `num_threads`

The rest of the config uses `u32` for iteration counts, so `u32` is consistent. However,
thread counts are idiomatically `usize` in Rust (matching `rayon::current_num_threads()`
and `std::thread` APIs). Minor and not blocking.

## What's done well

- **Clean separation:** The dispatch into `mcts_search_single` / `mcts_search_parallel`
  keeps the original path untouched and easy to reason about.
- **No locks:** Thread-local trees avoid all synchronization complexity.
- **Remainder handling:** `iters_per_thread + if thread_idx < remainder { 1 } else { 0 }`
  ensures total iterations are preserved (slightly more due to rounding up, never fewer).
- **Nested parallelism prevention:** Setting `num_threads: 1` in the per-thread config
  avoids recursive parallel dispatch.
- **Backward compatibility:** Default `num_threads: 1` means no behavior change for
  existing callers.
- **Good test coverage** for the merge math and iteration distribution logic.
