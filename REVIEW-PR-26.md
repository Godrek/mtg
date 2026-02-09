# PR #26 Review: Add MCCFR goldfish training for solitaire strategy optimization

## Summary

This PR adds goldfish (solitaire) MCCFR training — a single-agent optimization where the
solver trains player 0's policy against a fixed passive opponent (`GoldfishStrategy`). The
opponent's action space collapses to deterministic choices, which should converge faster
than standard 2-player MCCFR.

**Files changed:** 3 files, +501 / -5 lines
- `src/solver/mccfr.rs` — +200: `train_goldfish`, `train_goldfish_with_abstraction`, `traverse_goldfish`
- `src/main.rs` — +65/-5: MCCFR goldfish demo section with red/green comparisons
- `tests/mccfr_test.rs` — +236: 5 new tests

**All 235 tests pass with zero regressions.**

---

## What works well

1. **Clean factoring**: The goldfish traversal is a well-motivated simplification of the
   standard `traverse`. Collapsing the opponent to `GoldfishStrategy` eliminates the
   opponent-node sampling branch and the second regret table, making the code simpler and
   the algorithm converge faster.

2. **Consistent patterns**: The new code faithfully follows the existing conventions —
   depth/action limits, canonicalization, info set hashing, `[RegretTable; 2]` return type
   for API compatibility.

3. **Good documentation**: Doc comments clearly explain the design rationale and differences
   from standard 2-player MCCFR.

4. **Thorough tests**: 5 tests covering basic training, legal game play, strategy comparison
   (MCCFR vs Greedy vs Random), abstraction support, and a second deck (Green Stompy).

---

## Issues

### 1. Unused `rng` parameter in `traverse_goldfish` (Bug / Dead code)

**`src/solver/mccfr.rs:1011`**

`traverse_goldfish` accepts `rng: &mut impl Rng` but never reads from it. In the standard
`traverse`, `rng` is consumed by `sample_from_distribution` at opponent nodes. The goldfish
version has no opponent sampling (opponent is deterministic) and the traverser explores ALL
actions (no sampling), so `rng` is threaded through every recursive call but never used.

This should either be removed (it's dead code) or `#[allow(unused)]` should be added with a
comment explaining it's reserved for future use (e.g., chance-node sampling for draw steps).

### 2. No rollout support at depth limit (Design gap)

**`src/solver/mccfr.rs:1078-1080`**

At the depth limit, goldfish training falls back to `heuristic_utility` directly:
```rust
if config.max_depth > 0 && depth >= config.max_depth {
    return heuristic_utility(&state, 0);
}
```

The standard `traverse` uses `evaluate_at_depth_limit`, which supports both `Heuristic` and
`Strategy { max_rollout_actions }` modes. This means rollout-based depth evaluation is
unavailable for goldfish training. If this is intentional (goldfish games are shorter, so
rollouts matter less), a brief comment explaining the choice would help.

### 3. Test comment/code mismatch

**`tests/mccfr_test.rs:826`**

```rust
eprintln!(
    "Goldfish MCCFR (20 iters): P0 info sets = {}, P1 info sets = {}",
```

The `eprintln!` says "20 iters" but the training uses `10` iterations (line 822:
`mccfr::train_goldfish(&state, 10, &config)`). Should be "10 iters".

### 4. Hardcoded player indices reduce flexibility

**`src/solver/mccfr.rs:1014-1015, 1026, 1089`**

The traversal hardcodes `0` as the pilot and checks `player == 1` for the goldfish. The
standard `traverse` is parameterized by `traverser`, making it work for either player.
While the goldfish API is specifically designed for player 0, accepting a `pilot: PlayerIndex`
parameter would make the code more general with minimal additional complexity.

---

## Nits

### 5. `main.rs` duplication

The MCCFR demo block has nearly identical setup/train/evaluate/compare sequences for the red
and green decks. Consider extracting a helper like:

```rust
fn train_and_compare_goldfish(db: &CardDatabase, deck: &[&str], name: &str, config: &McfrConfig, baseline: &GoldfishResults) { ... }
```

This is a demo binary so duplication is tolerable, but it would cut ~25 lines.

---

## Verdict

**Approve with minor changes.** The core algorithm is correct, well-tested, and follows
established patterns. The unused `rng` and the test comment mismatch should be fixed before
merge. The design gaps (no rollout support, hardcoded player indices) are acceptable
simplifications for a first iteration but worth tracking as follow-ups.
