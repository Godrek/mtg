# PR #21 Review: Implement Phase 3 — Integration & Quality

**Branch**: `claude/implement-phase-3-strategy-nOBgz`
**Commit**: `926dfb5` — "Implement Phase 3: Integration & Quality"
**Diff**: +1,918 / -50 lines across 11 files

## Build & Test Status

- **Compiles**: Yes (`cargo check` passes clean)
- **All tests pass**: 176 tests (87 integration + 25 MCCFR + 59 unit + 4 benchmark + 1 deck import)
- **Benchmarks**: Engine ~2,377 games/sec, MCCFR ~2.5 iters/sec

## Summary

Single large commit implementing all three Phase 3 sub-tracks:

- **3A (Rules Engine Polish)**: Token creation, mana generation (`Effect::AddMana`), ETB/death watcher triggers, full 704.5 SBA suite (counter cancellation, legendary rule, planeswalker uniqueness), combat requirements/restrictions (`MustAttack`/`CantBlock`), extra turns, skip phases, zone-change counters, `GameStateSnapshot`, replacement ordering, and `DynamicValue` enum.
- **3B (MCCFR Polish)**: Warm-starting from `GreedyStrategy`, Bayesian `OpponentModel`, `PolicySnapshot` visualization, and `MultiPhaseAbstraction`.
- **3C (Benchmarks)**: 4 benchmark tests measuring engine throughput, solver throughput, MCCFR training, and warm-start comparison.

## Detailed Findings

### Strengths

1. **Comprehensive coverage**: All Phase 3 checklist items from `CONSOLIDATED_STRATEGY.md` are addressed in a single coherent commit.

2. **Token creation (CR 111)**: Well-implemented. Uses `Arc::make_mut` to register token `CardDef`s in the shared database — avoids unnecessary clones when only one reference exists. Token removal on zone-leave (CR 111.7) is correctly placed in `move_object()`.

3. **SBA additions**: Counter cancellation (704.5d), legendary rule (704.5j), and planeswalker uniqueness (704.5i) are implemented correctly and placed in the right position within the existing SBA loop — before the lethal-damage check, which is the correct CR 704 ordering.

4. **`GameStateSnapshot`**: Clean save/restore pattern. Deliberately excludes `card_db` (shared Arc) and `characteristics_cache` (derived). The `restore()` method correctly invalidates the cache and clears pending events.

5. **Watcher triggers**: The `ACreatureEnters` / `ACreatureDies` trigger conditions are cleanly separated from self-ETB/self-dies, which avoids the false-firing bug noted in the original code comments.

6. **MustAttack/CantBlock**: Enforced at the action-enumeration level (`legal_actions_with`), which is the correct place for the MCCFR solver to see legal action constraints.

7. **Test quality**: 17 new integration tests cover all major Phase 3A/3B features with clear setup and assertions. Benchmark tests have reasonable sanity-check thresholds.

### Issues

#### Medium Severity

1. **`DefaultHasher` not stable across Rust versions** (`src/rules/mod.rs:1440-1446`): `token_card_id()` uses `std::collections::hash_map::DefaultHasher` to generate token card IDs. `DefaultHasher` is explicitly documented as not guaranteed to produce the same hash across Rust versions. If regret tables or checkpoints are serialized and loaded across compiler versions, token card IDs could change, causing lookup misses. Consider using a stable hash function (e.g., FNV or a simple custom hash) or deriving a deterministic ID from the token properties.

2. **MustAttack filtering allows "no attack at all"** (`src/action/mod.rs:213-218`): The must-attack enforcement skips checking when `subset.is_empty()`, meaning a player can choose to declare zero attackers even when they control a creature with `MustAttack`. Per CR 508.1d, a creature that must attack must attack if able — passing the declare-attackers step entirely should only be legal if the creature is unable to attack (e.g., tapped, has defender). The current logic is overly permissive.

3. **Warm-start produces worse exploitability than cold-start** (benchmark results): The warm-start benchmark shows exploitability of 0.3347 vs cold-start 0.0037. While the warm-start is intended to accelerate convergence, these results suggest the greedy-seeded regrets may actually bias the solver away from equilibrium. The warm-start weight or strategy needs investigation — it may need a decay mechanism or the initial regret seeding may be too aggressive.

4. **Legendary/planeswalker SBA code duplication** (`src/rules/mod.rs:1031-1103`): The legendary rule and planeswalker uniqueness rule implementations are nearly identical (same pattern: build name_map, find duplicates, keep newest). This should be extracted into a shared helper function parameterized by the filter predicate (legendary vs planeswalker).

#### Low Severity

5. **`skip_phases` cleared too early** (`src/rules/mod.rs:1373`): `skip_phases.clear()` is called at the start of `next_turn()`, which means skip effects always last exactly one turn. This is correct for "skip your next draw step" effects, but effects like Stasis ("players skip their untap step") are ongoing and shouldn't be cleared per-turn. The current approach works for the simple case but may need rethinking when persistent skip effects are added.

6. **`extra_turns.remove(0)` is O(n)** (`src/rules/mod.rs:1379`): Using `Vec::remove(0)` to dequeue extra turns is O(n). This is fine in practice since extra turn queues are tiny, but a `VecDeque` would be more idiomatic.

7. **`dynamic_power`/`dynamic_toughness` not used anywhere**: The `DynamicValue` enum and `CardDef` fields are defined but never evaluated in the layer engine or elsewhere. `compute_characteristics()` in `src/layers/mod.rs` doesn't check for dynamic values. This is effectively dead code until wired in.

8. **Boilerplate in `sample.rs`**: Adding `dynamic_power: None, dynamic_toughness: None` to every CardDef construction (+40 lines) is noisy. Consider using `..Default::default()` or a builder pattern if Rust's struct update syntax is applicable.

9. **`GameStateSnapshot` fields are all `pub`** (`src/game/mod.rs:402-423`): The snapshot struct exposes all fields publicly, but it should be an opaque type — consumers should only use `snapshot()` and `restore()`. Making fields private would prevent misuse.

10. **`OpponentModel` likelihood values are hardcoded** (`src/solver/mccfr.rs:794-800`): The Bayesian update uses fixed likelihood values (0.8 for signature cards, 0.2 for non-signature). These should probably be configurable or derived from actual deck composition data.

11. **`MultiPhaseAbstraction` uses magic numbers for phase IDs** (`src/solver/mccfr.rs:937-940`): Phases 3-10 are hardcoded as "strategic" phases. These should reference `Phase` enum variants or constants to avoid breakage if phase ordering changes.

### Missing Tests

- No test for the legendary rule SBA (CR 704.5j)
- No test for the planeswalker uniqueness SBA (CR 704.5i)
- No test for `MustAttack` enforcement in action enumeration
- No test for `Effect::ExtraTurn` through spell resolution (only tested by directly pushing to `extra_turns`)
- No test for `DynamicValue` evaluation (it's dead code)

## Verdict

**Approve with reservations.** The PR delivers on all Phase 3 checklist items, compiles clean, and all 176 tests pass. The architecture is sound — new features are placed in the right layers (action enumeration for combat keywords, SBA loop for state-based rules, `move_object` for token cleanup). The `GameStateSnapshot` and token creation patterns are well-designed.

The medium-severity issues (MustAttack allowing zero attackers, warm-start regression, DefaultHasher instability) should be tracked for follow-up. The code duplication between legendary/planeswalker SBAs is worth a quick refactor. Missing tests for the legendary rule, planeswalker uniqueness, and MustAttack enforcement should be added.
