# Code Review: PR #17 — Phase 2B: MCCFR Scaling to Realistic Decks

**Branch**: `claude/phase-b-implementation-DiXzR` → `mainline`
**Files changed**: 5 (info_set/mod.rs, solver/mccfr.rs, strategy/mod.rs, tests/mccfr_test.rs, CONSOLIDATED_STRATEGY.md)
**+1,444 / -48 lines**

---

## Summary

This PR implements Phase 2B of the MTG GTO project: scaling MCCFR training
to 60-card decks. It adds four major capabilities:

1. **Information set abstraction** (`InfoSetAbstraction` trait,
   `BucketedAbstraction`, `CardAwareBucketedAbstraction`)
2. **Parallel MCCFR training** (`train_parallel()` with sharded regret tables)
3. **Depth-limited rollouts** (`RolloutMode::Strategy`, `rollout_utility()`)
4. **Checkpointing** (`save_checkpoint()` / `load_checkpoint()`)

All 78 tests pass (53 existing + 25 MCCFR). Clean build with zero warnings.

---

## Strengths

- **Clean abstraction design**: The `InfoSetAbstraction` trait is well-designed
  with a simple `abstract_info_set() -> u64` interface. The three
  implementations (Identity, Bucketed, CardAwareBucketed) form a natural
  hierarchy of increasing sophistication.
- **Backward compatibility**: The existing `train()` and `run_iteration()`
  functions are preserved unchanged, delegating to the new
  `_with_abstraction` variants. Phase 1B tests continue to pass without
  modification.
- **Sharded parallelism**: `train_parallel()` avoids lock contention by giving
  each rayon task its own regret tables, then merging via
  `merge_regret_tables()`. The merge correctly sums cumulative regret and
  strategy values, which is valid due to CFR's linearity property.
- **Comprehensive testing**: 11 new integration tests cover all four
  subsystems, including scale validation on 60-card decks, checkpoint
  round-trip, and parallel-vs-sequential consistency.
- **Documentation**: Strategy doc updated to reflect completion, with all
  checklist items checked off and acceptance criteria met.

---

## Issues & Suggestions

### 1. `BucketedAbstraction::board_stats()` inconsistency with `abstract_info_set()` [Medium]

`board_stats()` accepts an `Option<&CardDatabase>` parameter and does proper
creature detection when `card_db` is `Some`. However, the
`BucketedAbstraction::abstract_info_set()` method never uses `board_stats()`
— it counts all permanents per controller directly from
`info_set.battlefield`. This means:

- `BucketedAbstraction` counts lands, enchantments, artifacts, etc. as
  "creatures" in its board hash
- `CardAwareBucketedAbstraction` correctly uses `board_stats()` with the
  card_db
- The `board_stats()` helper on `BucketedAbstraction` appears to only be used
  by `CardAwareBucketedAbstraction`

This isn't a *bug* per se (both sides count identically, so the abstraction
is still consistent), but the doc comment on `BucketedAbstraction` claims it
tracks "total power, total toughness, creature count" when it actually just
tracks permanent count per controller. Consider either:

- Updating the doc comment to reflect the actual behavior ("permanent count
  per controller, no power/toughness")
- Or moving `board_stats()` to `CardAwareBucketedAbstraction` where it's
  actually used

### 2. `hand_categories()` only used by `CardAwareBucketedAbstraction` [Low]

Similar to above: `BucketedAbstraction::hand_categories()` is defined as a
method on `BucketedAbstraction` but only called by
`CardAwareBucketedAbstraction`. The `BucketedAbstraction::abstract_info_set()`
method just hashes hand_size directly. This is fine functionally but the
methods could be better organized.

### 3. `rollout_utility()` doesn't canonicalize actions [Low]

In the rollout loop, `legal_actions()` is used (full, not abstracted), and
the chosen action is applied directly without canonicalization. This is
correct for gameplay (the strategy returns a real `Action`), but it means
rollouts use the full action space rather than the bucketed one. This is
likely intentional (rollouts should be fast, not training), but worth noting
for anyone expecting rollouts to match training-time abstraction.

### 4. Potential wasted work in `train_parallel()` when `num_shards > num_iterations` [Low]

In `train_parallel()`, when `num_shards > num_iterations`,
`iterations_per_shard` will be 0 and only `remainder` shards will do 1
iteration each. This works correctly, but the remaining shards will spin up
rayon tasks that do zero iterations (wasted work). A minor guard could skip
zero-work shards:

```rust
if iters == 0 { return [RegretTable::new(), RegretTable::new()]; }
```

### 5. `RolloutMode` should derive `Debug` [Low]

`McfrConfig` derives `Debug, Clone` but `RolloutMode` derives neither. Since
`TrainConfig` contains `RolloutMode`, the whole config struct can't be
debug-printed, which hampers diagnostics.

### 6. Per-shard checkpointing writes to unbounded subdirectories [Low]

In `train_parallel()`, per-shard checkpoints write to
`{dir}/shard_{idx}/`. These are never cleaned up or merged. For long training
runs with many shards and frequent checkpoints, this could accumulate
significant disk usage. Consider documenting this behavior or only
checkpointing the final merged result.

### 7. Memory estimation in `training_stats()` is rough [Nit]

The 48-bytes-per-action estimate (`action:~32 + entry:16`) is a reasonable
approximation but doesn't account for HashMap overhead (load factor, bucket
metadata). The actual memory usage could be 1.5-2x higher. Fine for
diagnostics but might surprise users if they compare against actual RSS.
Consider adding a note that this is an estimate.

---

## Verdict

**Approve with minor suggestions.** The implementation is solid, well-tested,
and correctly integrates with the existing codebase. The abstraction layer is
clean, parallelism is correctly implemented, and all acceptance criteria are
met. The suggestions above are mostly about documentation accuracy and minor
code organization — none are blockers.

---

## Follow-up Review: Commit `0dd15b7` — All 7 Items Addressed

The follow-up commit cleanly addresses every review item:

| # | Issue | Resolution | Status |
|---|-------|------------|--------|
| 1 | `BucketedAbstraction` doc overstated board tracking | Doc rewritten: now says "permanent count per controller (no power/toughness)" and points to `CardAwareBucketedAbstraction` for richer stats | Resolved |
| 2 | `board_stats()` / `hand_categories()` misplaced on `BucketedAbstraction` | Both methods moved to `CardAwareBucketedAbstraction` impl block. Signatures simplified — `Option<&CardDatabase>` → `&CardDatabase` (no longer optional since the type always has one) | Resolved |
| 3 | `rollout_utility()` action space not documented | Doc comment added explaining rollouts intentionally use full action space for speed | Resolved |
| 4 | Zero-work shards in `train_parallel()` | Early return guard added: `if iters == 0 { return [RegretTable::new(), ...]; }` | Resolved |
| 5 | `RolloutMode` missing `Debug` derive | `#[derive(Debug)]` added to `RolloutMode` | Resolved |
| 6 | Per-shard checkpoint cleanup undocumented | Inline comment added warning that per-shard checkpoints are not auto-cleaned and suggesting `train_extended()` for merged-only checkpointing | Resolved |
| 7 | Memory estimate accuracy undocumented | Comment updated: "Rough memory estimate (lower bound). Does not account for HashMap overhead... actual RSS may be 1.5-2x higher" | Resolved |

All 78 tests pass. Clean build, zero warnings. No behavioral changes — purely
doc accuracy, code organization, and minor guards.

**Final verdict: Approve.** Ready to merge.
