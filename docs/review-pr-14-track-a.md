# PR #14 Review: Track A — Phase 1A Rules Engine Foundations

**Branch**: `claude/implement-track-a-SQGvj`
**Reviewed against**: `docs/CONSOLIDATED_STRATEGY.md`, Phase 1A (sections 1A.1, 1A.2, 1A.3)

---

## Review History

| Round | Commit | Result |
|---|---|---|
| R1 | `a206205` | Approve with minor comments — 2 P1 items, 4 P2 items |
| R2 | `a2630a7` | **Approve** — all P1 items resolved, 3 P2 items remain |

---

## R2 Changes Summary

Commit `a2630a7` — "Address PR review: fix cascading SBA bug, add EachCreature target, GreedyStrategy"

All **72 tests pass** (up from 68), 0 failures, 0 warnings.

### P1 Items Resolved

**1. Cascading SBA test (RESOLVED)**: Two new cascading tests added:
- `test_cascading_sba_dies_trigger_kills_another_creature`: Pyroclasm Elemental dies → trigger deals 2 to each creature → Grizzly Bears dies from SBA cascade.
- `test_cascading_sba_chain_of_three`: 3-deep cascade — Pyro A dies → trigger kills Pyro B + Bears → Pyro B's trigger fires → resolves with empty battlefield.

These directly satisfy the acceptance criterion: "creature with 'when ~ dies, deal 2 damage to each creature' killing another creature at 2 toughness."

**2. GreedyStrategy handles ChooseReplacementOrder (RESOLVED)**: `src/strategy/mod.rs` updated — the Priority 0 match arm now uses `matches!(action, Action::OrderTriggers { .. } | Action::ChooseReplacementOrder { .. })`. Integration test `test_greedy_strategy_handles_replacement_order` verifies no panic.

### Additional Fixes (Beyond Review Items)

**3. Dies trigger disambiguation (RESOLVED)**: The R1 review flagged that `check_triggers(state, TriggerCondition::Dies, None)` would erroneously fire surviving creatures' "when ~ dies" triggers. The fix correctly removes the `None` variant call from both `check_state_based_actions` and `resolve_effect` (Destroy handler). Only `check_triggers(state, TriggerCondition::Dies, Some(obj_id))` is now called — scoped to the dying creature itself. A clear comment documents that future "when any creature dies" watchers (e.g., Blood Artist) will need a separate `ACreatureDies` trigger condition.

This is a correctness fix, not just a style change. It prevents phantom trigger firings on boards with multiple "when ~ dies" creatures where only some die.

**4. New `TargetSpec::EachCreature` variant**: Added to `src/card/mod.rs` to support "deal N damage to each creature" effects. `resolve_effect` auto-targets all battlefield creatures when this spec is used. `enumerate_targets_for_spell` correctly skips target generation (untargeted effect). New test card `Pyroclasm Elemental` (3/1, id 201) uses this.

---

## Updated Acceptance Criteria Checklist

| Criterion | R1 | R2 | Notes |
|---|---|---|---|
| SBA loop correctly handles recursive triggers | PARTIAL | **YES** | Cascading tests exercise 2-deep and 3-deep SBA→trigger→SBA chains. |
| Events fire for all zone changes | YES | YES | No change. |
| Events fire for damage | YES | YES | No change. |
| Events fire for life changes | YES | YES | No change. |
| Event handlers do not affect `GameState::clone()` cost | YES | YES | No change. |
| All existing tests pass | YES | YES | 72 tests pass (53 integration + 18 unit + 1 deck import). |

All Phase 1A acceptance criteria are now fully met.

---

## Remaining P2 Items (Non-Blocking)

1. **Unify `Zone` and `ZoneType`**: The duplicate `events::Zone` enum mirrors `card::ZoneType`. Both are in the same crate. Low maintenance risk but creates a spot where new zones must be added in two places.

2. **Route all zone transitions through `move_object`**: `CastSpell` manually emits `ZoneChange` and removes from hand via `retain()` instead of using `move_object`. Dual path could cause double-emit if refactored later.

3. **Fill in `source: 0` placeholders in `DamageDealt` events**: Combat damage and effect damage emit `source: 0`. Must be resolved before Phase 2A cards that care about damage sources.

---

## Verdict

**Approve.** All Phase 1A deliverables are implemented correctly per the consolidated strategy. The R2 commit resolves both P1 review items and proactively fixes the Dies trigger disambiguation bug that R1 identified as a pre-existing risk. The cascading SBA tests (2-deep and 3-deep chains) fully validate the CR 704.3 recurrence loop. The 3 remaining P2 items are cosmetic or deferred-to-Phase-2A by design.
