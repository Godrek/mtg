# PR #32 Re-Review: Reshuffle Opening Hands + London Mulligan

## Summary

Four commits total. Commits 1-2 were reviewed previously; commits 3-4 address the feedback. All 146 tests pass. One new low-severity issue introduced by the fix-up; otherwise the feedback has been addressed well.

---

## Previous Issues — Resolution Status

### [Bug / High] PassPriority fallback in Mulligan → **FIXED** (commit `e1587d4`)

Replaced `return vec![Action::PassPriority]` with a descriptive `unreachable!()`. Good — this both prevents the silent corruption and serves as documentation.

### [Medium] `MulliganMulligan` doesn't check `MAX_MULLIGANS` → **FIXED** (commit `cf82923`)

Added `debug_assert!` in `apply_action` guarding against exceeding `MAX_MULLIGANS`. `MAX_MULLIGANS` was also made `pub` so the assert can reference it. Clean fix.

### [Medium] `GoldfishStrategy` had no bottom-card handling → **FIXED** (commit `cf82923`)

`GoldfishStrategy` now checks for `MulliganBottomCard` actions before falling back to `MulliganKeep`. The comment correctly explains the defensive reasoning. Good.

### [Low] `MulliganBottomCard` canonical action included `hand_index` → **Changed** (commit `cf82923`)

`hand_index` was removed from `CanonicalAction::MulliganBottomCard`, leaving only `card_id`. The `resolve()` function now always resolves to the first matching instance via `find_in_hand_by_index(state, player, *card_id, 0)`.

This works correctly in practice because:
- **MCCFR traversal** (`traverse`, `traverse_goldfish`) applies concrete `Action`s directly — not via `resolve()`. Canonical actions are only keys in the regret table.
- **`AbstractedMcfrStrategy::choose_action`** samples an index into the concrete action list, not via resolve.
- **Commander decks are singleton** (1 copy of each non-basic), so duplicates only arise for basic lands.

However, this does introduce a **subtle MCCFR strategy skew** when duplicate cards exist (see new issue below).

### [Medium] `reshuffle_opening_hand` safety / doc-comment → **Not addressed**

Still only safe when called on a freshly-cloned initial state. The doc comment still reads "Resets turn/phase/mana state so the game starts cleanly from Turn 1" without the caveat. Low-risk since usage is correct, but would be nice to document.

### [Low] Hardcoded player count / Greedy heuristic / TURN_ORDER comment → **Not addressed**

These were low-priority suggestions and are fine to skip.

### [Weak test coverage] → **FIXED** (commit `cf82923`)

Excellent test coverage added — 7 new tests in `tests/commander_test.rs`:

| Test | What it verifies |
|------|-----------------|
| `test_commander_starts_in_mulligan_phase` | Setup puts game in Mulligan phase with 7-card hands |
| `test_mulligan_keep_advances_to_next_player` | P0 keep → priority moves to P1 |
| `test_mulligan_both_keep_transitions_to_untap` | Both keep → exits Mulligan to Untap |
| `test_mulligan_once_then_keep_requires_bottom_one` | Mulligan 1 → keep → bottom 1 → hand=6 |
| `test_mulligan_twice_bottoms_two` | Mulligan 2 → keep → bottom 2 → hand=5 |
| `test_mulligan_full_game_with_greedy` | Full game completes with mulligan phase active |

These cover the state machine transitions, hand size math, and end-to-end integration. Solid.

---

## New Issue from Fix-Up Commits

### [Low] Duplicate canonical actions skew MCCFR strategy probabilities

Removing `hand_index` from `MulliganBottomCard` means two copies of the same card (e.g., 2 Forests) produce identical canonical actions. In `current_strategy()`, both entries look up the same regret value, so the combined probability mass for "bottom a Forest" is inflated relative to unique cards.

**Example:** Hand = [Forest, Forest, Island], must bottom 1.
- Canonical actions: `[Bottom(Forest), Bottom(Forest), Bottom(Island)]`
- `current_strategy` with uniform regrets: `[1/3, 1/3, 1/3]`
- Effective probability: Forest gets 2/3, Island gets 1/3 — should be 1/2 each

This doesn't affect gameplay correctness (whichever Forest is bottomed, the outcome is identical), and Commander singleton decks rarely have duplicates beyond basic lands. The prior approach with `hand_index` was consistent with `PlayLand`/`CastSpell`/`Discard` canonicalization but inflated the info set space. This is a reasonable trade-off.

**Suggestion (non-blocking):** If this becomes a concern, deduplicating canonical actions before passing to `current_strategy` would be the clean fix — but it's not worth the complexity for the current use case.

---

## Overall Assessment (Re-Review)

| Aspect | Rating | Change |
|--------|--------|--------|
| Correctness | Strong | Improved — bug fixed, debug_assert added |
| Architecture | Strong | Unchanged |
| MCCFR integration | Strong | Minor strategy skew on duplicates (acceptable) |
| Strategy support | Strong | Improved — GoldfishStrategy now defensive |
| Test coverage | Strong | Improved — 7 targeted mulligan tests |
| Code clarity | Good | Unchanged |

**Verdict: Approve.** The high-severity bug is fixed, test coverage is solid, and the remaining items are non-blocking nits. The duplicate canonical action skew is theoretical and acceptable for singleton Commander decks.
