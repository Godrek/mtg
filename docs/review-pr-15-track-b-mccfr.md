# PR #15 Review: Phase 1B — MCCFR Solver Infrastructure

**Branch**: `claude/implement-track-b-Re3gw`
**Reviewed against**: Track B of `docs/CONSOLIDATED_STRATEGY.md` (Phases 1B.1–1B.5)
**Verdict**: Approve with findings — solid implementation of core MCCFR infrastructure. One significant design gap (missing canonical action integration) and several minor issues to track for Phase 2B.

---

## Summary

PR #15 implements the Phase 1B MCCFR solver infrastructure in ~1,440 lines across 9 files. All 72 tests pass (26 unit + 34 existing integration + 11 new MCCFR integration + 1 deck import). The implementation correctly uses the Phase 0 interfaces (`PlayerView`, `visible_state()`, `legal_actions_abstracted()`, `apply_action()`).

### Files Changed

| File | Lines | Role |
|---|---|---|
| `src/info_set/mod.rs` | +376 | Phase 1B.1 — Information Set module |
| `src/solver/mod.rs` | +255 | Phase 1B.2 — RegretTable, InfoSetData |
| `src/solver/mccfr.rs` | +415 | Phase 1B.3 — External sampling MCCFR traversal |
| `src/strategy/mod.rs` | +75 | Phase 1B.4 — McfrStrategy implementation |
| `src/card/sample.rs` | +44 | Phase 1B.5 — Mini training decks |
| `tests/mccfr_test.rs` | +266 | Integration tests |
| `src/lib.rs` | +2 | Module registration |
| `Cargo.toml` / `Cargo.lock` | +11 | bincode dependency |

---

## Phase 1B.1 — Information Set Module (`src/info_set/mod.rs`)

### What the strategy requires

> Uses `PlayerView` from Phase 0 — does not read raw `GameState` fields.

### Assessment: Compliant

**Strengths:**
- Correctly reads exclusively from `PlayerView`, never from raw `GameState`. The `from_view()` method takes `&PlayerView` and `&CardDatabase` as specified.
- Deterministic hashing — same observable state always produces the same hash. Verified by `test_info_set_same_observable_same_hash` which confirms that different opponent hand contents (hidden information) produce identical hashes.
- Battlefield sorted by `(controller, card_id, tapped, ...)` for canonical ordering — set-like zone gets order-independent representation.
- Hand sorted by CardId for canonical ordering.
- Stack preserves LIFO order (order-dependent, correct).

**Findings:**

1. **Mana representation is lossy** (`info_set/mod.rs:246`): The mana pool is collapsed to a single `u32` total:
   ```rust
   let mana_total = mana.white + mana.blue + mana.black + mana.red + mana.green + mana.colorless;
   ```
   This means `{R, R}` and `{W, W}` map to the same info set hash. For mono-red training (Phase 1B.5) this is fine, but will cause incorrect info set merging in multi-color scenarios. The strategy's Phase 2B.1 information set abstraction should address this — acceptable for now.

2. **`DefaultHasher` is not guaranteed stable across Rust versions** (`info_set/mod.rs:119`). The `std::collections::hash_map::DefaultHasher` may produce different hashes across different Rust compiler versions (it uses SipHash, but the implementation is not contractually stable). This means serialized regret tables trained with one Rust version may not be usable with another. For Phase 1B this is fine; for Phase 2B serialized policy deployment, consider using a deterministic hasher (e.g., `ahash` with fixed seeds, or FxHash).

3. **`PermanentInfo` does not capture keywords or attachments** — a creature with +1/+1 counters vs. one with trample would hash identically if all other stats match. Again, acceptable for the minimal card pool but should be tracked for Phase 2A integration.

4. **Exile zones are not captured** — `PlayerView` exposes `my_exile` and `opp_exile` but `InformationSet` does not include them. Cards in exile can matter for gameplay (flashback, delve, etc.).

---

## Phase 1B.2 — Regret Table (`src/solver/mod.rs`)

### What the strategy requires

> `RegretTable` — `HashMap<u64, InfoSetData>` with cumulative_regret, cumulative_strategy, visit_count. Serializes via serde + bincode.

### Assessment: Fully compliant

**Strengths:**
- Data structure matches the specification exactly.
- `current_strategy()` correctly implements regret matching: positive regret proportional, uniform fallback when all non-positive.
- `average_strategy()` correctly computes the converging Nash strategy.
- bincode serialization roundtrip tested and working.
- `prune()` method for memory management (addresses the "regret pruning" risk mitigation).
- Clean `Default` impl.

**No findings.** This is the cleanest module in the PR.

---

## Phase 1B.3 — External Sampling MCCFR Traversal (`src/solver/mccfr.rs`)

### What the strategy requires

> 1. Calls `legal_actions_abstracted(state)` (bucketed combat)
> 2. Calls `canonicalize()` to map actions to regret table indices
> 3. Calls `state.clone()` + `apply_action()` to explore branches
> 4. Calls `visible_state(player)` to compute information sets
> 5. Skips CFR nodes where only one action is legal

### Assessment: Mostly compliant — one significant gap

**Strengths:**
- Correctly uses `legal_actions_abstracted()` (point 1). ✅
- Correctly uses `state.clone()` + `apply_action()` (point 3). ✅
- Correctly uses `visible_state(player)` → `InformationSet::from_view()` (point 4). ✅
- Single-action nodes skipped without creating CFR entries (point 5). ✅
- Depth-limited traversal with configurable `max_depth` and `max_actions`.
- Heuristic evaluation (`heuristic_utility`) uses life total + board power — reasonable for burn vs. creatures.
- External sampling algorithm is correct: traverser explores all actions, opponent samples one.

**Findings:**

5. **`canonicalize()` is NOT used (point 2)** — This is the most significant gap. The strategy explicitly requires:
   > Calls `canonicalize()` to map actions to regret table indices

   Instead, regret tables are keyed by info set hash, and actions are indexed by **positional index** in the `legal_actions_abstracted()` return vector. This means:
   - If the same info set produces actions in a different order (e.g., because `HashMap` iteration order differs), the regret/strategy entries will be misaligned.
   - Action index `i` in one visit to an info set may correspond to a different action than index `i` in another visit.

   **Current mitigation**: The `legal_actions_abstracted()` function appears to produce deterministic ordering for a given game state (lands first, then spells sorted by cost, then pass priority). Since the info set hash captures the observable state, and `legal_actions_abstracted()` is deterministic given the state, the positional indexing is likely correct *in practice* for the minimal scenario. But this is fragile.

   **Recommendation**: Before Phase 2B, actions should be mapped through `canonicalize()` and the regret table should key on `(info_set_hash, CanonicalAction)` rather than `(info_set_hash, action_index)`. This is the whole reason Phase 0.2 exists.

6. **`rng` is created per-iteration, not passed in** (`mccfr.rs:86`):
   ```rust
   let mut rng = rand::thread_rng();
   ```
   This makes training non-reproducible. For debugging and regression testing, the `train()` function should accept an optional seed/rng. Minor for Phase 1B.

7. **No parallel training** — `train()` runs iterations sequentially. The strategy's Phase 2B.2 explicitly plans for rayon parallelism with sharded tables. Acceptable for Phase 1B, but the current `&mut [RegretTable; 2]` API will need to change (can't share mutable references across threads).

8. **`approximate_exploitability()` is a rough proxy** — it computes average positive regret across info sets, which correlates with but does not measure true exploitability (which requires best-response computation). The doc comments acknowledge this. The `test_mccfr_exploitability_decreases` test does not actually assert the decrease (it only checks finiteness), which is correct given MCCFR's stochastic nature.

9. **Depth counting may be imprecise** — both single-action skip and empty-action pass increment `depth` and `actions_taken`. This means the effective depth budget is consumed by forced passes, which reduces exploration of actual decision nodes. Consider only incrementing depth at multi-action nodes.

---

## Phase 1B.4 — McfrStrategy Implementation (`src/strategy/mod.rs`)

### What the strategy requires

> ```rust
> impl Strategy for McfrStrategy {
>     fn choose_action(&self, state: &GameState, player: PlayerIndex) -> Action {
>         let view = state.visible_state(player);
>         let info_set = InformationSet::from_view(&view, state.card_db());
>         let actions = legal_actions_abstracted(state);
>         let canonical: Vec<_> = actions.iter().map(|a| canonicalize(a, state)).collect();
>         // look up distribution, sample, resolve back to concrete Action
>     }
> }
> ```

### Assessment: Functionally correct, same canonicalize gap

**Strengths:**
- Correctly implements `Strategy` trait.
- Uses `legal_actions_abstracted()` (not `legal_actions()`).
- Falls back to uniform random for unseen info sets — correct behavior.
- Uses `average_strategy()` (not `current_strategy()`) for play — this is crucial and correct. The current strategy oscillates during training; the average strategy converges to Nash.
- Single-action fast path avoids unnecessary lookups.

**Findings:**

10. **Same canonicalize gap as traversal** — Uses positional action indexing instead of canonical action mapping. Same risk as finding #5.

11. **Duplicate utility function** — `sample_action_index()` in `strategy/mod.rs:203` is identical to `sample_from_distribution()` in `solver/mccfr.rs:287`. Should be deduplicated into a shared utility.

12. **`McfrStrategy` does not include `InfoSetAbstraction`** — The strategy spec shows:
    ```rust
    pub struct McfrStrategy {
        policy: RegretTable,
        abstraction: InfoSetAbstraction,
    }
    ```
    The PR omits the `abstraction` field. This is fine for Phase 1B (no abstraction needed for the minimal scenario), but the type should be extended for Phase 2B.1.

---

## Phase 1B.5 — Minimal Training Scenario (`src/card/sample.rs`, `tests/mccfr_test.rs`)

### What the strategy requires

> - **Decks**: 15-card decks. Mountains + Lightning Bolts vs. Mountains + Gray Ogres.
> - **Target**: Games end in 3-5 turns. Reachable info sets fit in memory (<100K entries).
> - **Validation**: Exploitability decreases, McfrStrategy beats GreedyStrategy, mirror match ~50%.

### Assessment: Partially compliant

**Strengths:**
- 15-card decks implemented: `mini_red_burn()` (8 Mountain, 4 Lightning Bolt, 3 Shock) and `mini_red_creatures()` (8 Mountain, 4 Grey Ogre, 3 Goblin Guide).
- Deck sizes verified by test assertions.
- Training loop tested at 5, 20, and 50 iterations.
- `test_mcfr_strategy_plays_legal_games` verifies complete games terminate.
- `test_mcfr_strategy_vs_random` runs 50 games without crashes.

**Findings:**

13. **Deck composition deviates from spec** — The strategy says "Mountains + Lightning Bolts vs. Mountains + Gray Ogres." The PR uses:
    - Burn deck: 8 Mountain + 4 Lightning Bolt + **3 Shock** (Shock not in spec)
    - Creature deck: 8 Mountain + 4 Grey Ogre + **3 Goblin Guide** (Goblin Guide not in spec)

    This is arguably an improvement (more interesting gameplay), but deviates from the spec without explanation.

14. **McfrStrategy vs. GreedyStrategy test missing** — The strategy requires "McfrStrategy beats GreedyStrategy after sufficient training." The test suite only tests McfrStrategy vs. RandomStrategy. There is no test comparing against GreedyStrategy.

15. **Mirror match convergence test missing** — The strategy requires "McfrStrategy mirror match converges to ~50% win rate." No mirror match test exists.

16. **Training iterations are low** — Tests use 5-50 iterations with `max_depth=30` and `max_actions=200`. For the 15-card scenario, this is likely insufficient for convergence. The tests pass because they don't assert convergence — only that the machinery runs without panics. This is acceptable for CI, but a manual training run with more iterations should be documented.

---

## Acceptance Criteria Assessment

The strategy defines four acceptance criteria for Phase 1B:

| Criterion | Status | Notes |
|---|---|---|
| Training loop runs and converges on minimal scenario | **Partial** | Loop runs ✅. Convergence not validated — no assertion that exploitability decreases or that trained policy beats baseline |
| Regret table serializes/deserializes via serde + bincode | **Pass** | `test_regret_table_roundtrip` and `test_regret_table_serialization_roundtrip` both verify round-trip |
| McfrStrategy implements `Strategy` and plays legal games | **Pass** | `test_mcfr_strategy_plays_legal_games` runs 10 complete games. `test_mcfr_strategy_name` verifies trait |
| `cargo test` passes for all new modules | **Pass** | All 72 tests pass (26 unit + 35 integration + 11 MCCFR) |

---

## Consolidated Findings

### Must Fix (before merge or in immediate follow-up)

None — the PR is functional and all tests pass. The findings below should be tracked as follow-ups.

### Should Fix (before Phase 2B)

| # | Finding | Location | Impact |
|---|---|---|---|
| 5 | `canonicalize()` not used for regret table indexing | `solver/mccfr.rs` traversal | Action misalignment risk if `legal_actions_abstracted()` ordering changes |
| 10 | Same canonicalize gap in `McfrStrategy` | `strategy/mod.rs` | Same risk |
| 14 | No McfrStrategy vs. GreedyStrategy test | `tests/mccfr_test.rs` | Acceptance criterion not validated |
| 15 | No mirror match convergence test | `tests/mccfr_test.rs` | Acceptance criterion not validated |

### Minor / Track for Later

| # | Finding | Location | Impact |
|---|---|---|---|
| 1 | Mana pool collapsed to single u32 | `info_set/mod.rs:246` | Multi-color info set merging |
| 2 | `DefaultHasher` not stable across Rust versions | `info_set/mod.rs:119` | Serialized policy portability |
| 3 | PermanentInfo omits keywords | `info_set/mod.rs:171` | Keyword-dependent info set precision |
| 4 | Exile zones not captured | `info_set/mod.rs` | Missing game information |
| 6 | Non-reproducible RNG | `solver/mccfr.rs:86` | Debug difficulty |
| 7 | Sequential-only training | `solver/mccfr.rs` | Phase 2B.2 blocker |
| 8 | Exploitability metric is approximate | `solver/mccfr.rs:320` | Convergence measurement |
| 9 | Depth consumed by forced passes | `solver/mccfr.rs` | Reduced exploration |
| 11 | Duplicate sampling function | `strategy/mod.rs` + `solver/mccfr.rs` | Code duplication |
| 12 | No `InfoSetAbstraction` field on McfrStrategy | `strategy/mod.rs` | Phase 2B.1 blocker |
| 13 | Deck composition deviates from spec | `card/sample.rs` | Minor spec divergence |
| 16 | Low training iterations in tests | `tests/mccfr_test.rs` | Not validating convergence |

---

## Recommendation

**Approve.** The PR delivers a functional, well-tested MCCFR implementation that correctly integrates with the Phase 0 interfaces. The core algorithm (external sampling MCCFR with regret matching) is correctly implemented. The most significant gap — not using `canonicalize()` for regret table action indexing — is a real concern but does not cause incorrect behavior in the current deterministic-ordering scenario. It should be addressed before Phase 2B when action spaces grow and ordering assumptions may break.
