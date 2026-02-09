# PR #21 Review: Implement Phase 3 — Integration & Quality

**Branch**: `claude/implement-phase-3-strategy-nOBgz`
**Commits**: `926dfb5` + `85009b9` + `bb96206`
**Total diff**: +2,714 / -443 across 11 files (net +2,271)

## Review Rounds

### Round 1 — Initial Review

Identified 4 medium-severity and 7 low-severity issues. All 176 tests passed.

### Round 2 — Fix Validation

All medium and low-severity issues from Round 1 addressed. 4 new tests added. 180 tests pass. 4 non-blocking observations remained.

### Round 3 — Final Validation

All 4 remaining observations addressed. 2 new tests added. 182 tests pass.

## Build & Test Status

- **Compiles**: Yes (`cargo check` clean, zero warnings)
- **All tests pass**: 182 tests (93 integration + 25 MCCFR + 59 unit + 4 benchmark + 1 deck import)
- **No regressions**: All pre-existing tests continue to pass

## Round 3 Fix Validation

### 1. Warm-start exploitability — **Fully resolved**

Redesigned `warm_start_from_greedy()` to only pre-populate info set entries (registering which info sets and actions exist) without seeding any regret values. The `warmup_weight` parameter is retained as `_warmup_weight` for API compatibility but is no longer used.

**Benchmark results across all rounds:**

| Round | Cold-start | Warm-start | Ratio |
|-------|-----------|------------|-------|
| 1 | 0.0037 | 0.3347 | 90x worse |
| 2 | 0.0043 | 0.0440 | 10x worse |
| 3 | 0.0022 | 0.0026 | **~1.2x — essentially equivalent** |

The warm-start is now bias-free. The benefit is pre-discovering reachable game-tree nodes so early MCCFR iterations don't start from a completely empty table. This is a sound approach — it gives the solver a "map" without putting a thumb on the scale.

### 2. DynamicValue::CardsInHand / CardTypesInGraveyards — **Fully resolved**

Added `DynamicContext` struct (`src/card/mod.rs`) carrying `hand_size: usize` and `graveyard_card_types: Vec<Vec<CardType>>`. The propagation chain:

1. `GameState::get_characteristics()` builds `DynamicContext` from `players[controller].hand.len()` and all players' graveyards
2. Calls `compute_characteristics_with_ctx()` (new wrapper in `src/layers/mod.rs`)
3. Passes context through to `DynamicValue::evaluate()`
4. `CardsInHand` returns `ctx.hand_size`; `CardTypesInGraveyards` counts distinct `CardType` values across all graveyard cards

The original `compute_characteristics()` is preserved as a convenience wrapper passing `None` for the context, maintaining backward compatibility for callers that don't need dynamic values.

New test `test_dynamic_value_cards_in_hand` validates a Maro-like creature (power/toughness = cards in hand) through the full pipeline: 0 cards → P/T 0/0, 3 cards → 3/3, 5 cards → 5/5.

### 3. skip_phases cleared per-turn — **Acknowledged, documented**

Correctly identified as a non-issue for the current card pool. No persistent skip effects (Stasis, etc.) exist in the sample card database. This is appropriate to revisit if/when such cards are added.

### 4. Effect::ExtraTurn through spell resolution — **Fully resolved**

New test `test_extra_turn_through_spell_resolution` creates a "Time Walk Test" sorcery card (id 9400, cost {1}{U}, effect `Effect::ExtraTurn`), casts it from hand, verifies it goes on the stack, resolves it via double pass-priority, confirms the extra turn is queued, then plays through the turn to verify player 0 gets the extra turn. This covers the full `CastSpell` → stack → `resolve_effect(ExtraTurn)` → `extra_turns.push_back()` → `next_turn()` path.

## Minor Observation (Non-blocking)

**`_warmup_weight` dead parameter**: The `warmup_weight` parameter on `warm_start_from_greedy()` is now prefixed with `_` and ignored. All callers still pass `1.0`. Consider removing the parameter entirely in a future cleanup, or documenting that it's reserved for potential future use.

## Verdict

**Approve — no remaining issues.** All issues identified across three review rounds have been resolved. The warm-start exploitability gap (the most significant concern) is now fully eliminated with an elegant redesign. The `DynamicValue` system properly threads game state context through the layer engine. 182 tests pass with no regressions. The code is ready to merge.
