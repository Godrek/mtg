# PR #24 Review: Fix mana verification, commander setup, ManaCost parsing, and magic number

**Branch:** `claude/fix-mana-commander-setup-l22Rb`
**Commit:** `00acb51`
**Author:** Claude (automated)

## Summary

PR #24 bundles five changes:
1. Guard `CastSpell` against insufficient mana (check `pay()` return)
2. Add `command_zone` and `commander_damage` fields to `PlayerState`; add `setup_commander_game()`
3. Extend `ManaCost` with `x_count`, `phyrexian`, and `hybrid` fields + parser updates
4. Replace magic number `10_000` with named constant `DEFAULT_MAX_ACTIONS`
5. Add `test_commander_snapshot_restore` integration test; fix `test_sba_recurrence_in_full_game_context` land types

## Verdict: Request Changes

The PR contains several valuable fixes, but parts conflict with or duplicate work already merged into mainline. It needs to be rebased and partially reworked.

---

## Detailed Findings

### 1. CastSpell Mana Guard -- VALID, STILL NEEDED

**File:** `src/rules/mod.rs:73-78`

The `pay()` return value is silently ignored in the current `CastSpell` handler, allowing spells to resolve without actually paying their mana cost. The PR correctly fixes this:

```rust
// Before (bug):
state.players[player].mana_pool.pay(cost);

// After (fix):
if !state.players[player].mana_pool.pay(cost) {
    return;
}
```

This is a real correctness bug. **However**, the same issue exists in two other places that the PR does not fix:
- `Action::CastCommander` (`src/rules/mod.rs:298`) -- ignores `pay()` return
- `Action::ActivateAbility` (`src/rules/mod.rs:164`) -- ignores `pay()` return

**Recommendation:** Apply the fix to all three call sites for consistency.

### 2. Commander Fields in PlayerState -- CONFLICTS WITH MAINLINE

**File:** `src/game/mod.rs`

The PR adds:
- `command_zone: Vec<ObjectId>`
- `commander_damage: HashMap<ObjectId, u32>`

But mainline (merged via PR #22) already has a more comprehensive commander model:
- `command_zone: Vec<ObjectId>` (already exists)
- `commander_card_id: Option<CardId>`
- `commander_object_id: Option<ObjectId>`
- `commander_tax: u32`
- `commander_damage_received: Vec<i32>` (indexed by player, not ObjectId)

**Issue:** The PR's `commander_damage: HashMap<ObjectId, u32>` representation conflicts with mainline's `commander_damage_received: Vec<i32>`. Mainline's approach (indexed by player) is more aligned with CR 903.10a (which tracks damage per opposing commander's owner, not per object). The PR's HashMap approach would break if a commander changes ObjectId (e.g., returns to command zone and is recast).

**Recommendation:** Drop the commander field changes entirely; mainline already has a more complete and correct implementation.

### 3. `setup_commander_game()` -- SUPERSEDED BY MAINLINE

**File:** `src/rules/mod.rs`

The PR's version is a thin wrapper around `setup_game()` that sets 40 life and creates commander objects. Mainline already has a full implementation that:
- Extracts the commander from the deck list before shuffling
- Records `commander_card_id` and `commander_object_id`
- Handles command zone placement correctly
- Includes `validate_commander_deck()` for deck legality checks

**Recommendation:** Drop this change; mainline's version is strictly better.

### 4. ManaCost X/Hybrid/Phyrexian Parsing -- VALID, NEW FEATURE

**File:** `src/mana/mod.rs`

This is a genuine addition not present in mainline. The parser is extended to handle:
- `{X}` symbols (tracked in `x_count`)
- `{W/U}`-style hybrid mana (tracked in `hybrid: Vec<(Color, Color)>`)
- `{R/P}`-style phyrexian mana (tracked in `phyrexian: Vec<Color>`)

The CMC calculation is correct per CR 202.3d/e (hybrid = 1 per symbol, phyrexian = 1 per symbol, X = 0).

**Issues found:**
- **`colors()` doc lie:** The doc comment is updated to say "including those in hybrid and phyrexian symbols" but the method body is NOT updated. Colors from hybrid/phyrexian symbols are not included in the return value.
- **`can_pay()` / `pay()` not updated:** The mana pool payment functions don't know how to handle hybrid or phyrexian costs. A spell with `{W/U}` in its cost can be parsed but never actually paid. This is acceptable as a parsing-only change if documented, but the PR doesn't note this limitation.
- **`Display` impl ordering:** X symbols are written before generic, which matches MTG convention. Hybrid and phyrexian are appended after colored symbols, which is a reasonable choice.

**Recommendation:** Accept with fixes:
1. Either update `colors()` to include hybrid/phyrexian colors, or revert the doc comment
2. Add a doc note that `can_pay()`/`pay()` don't yet support hybrid/phyrexian

### 5. `DEFAULT_MAX_ACTIONS` Constant -- VALID, MINOR IMPROVEMENT

**File:** `src/solver/mccfr.rs`

Replaces `10_000` with a named constant. Simple and correct.

**Recommendation:** Accept as-is.

### 6. Test Fixes -- PARTIALLY VALID

**File:** `tests/integration_test.rs`

- **`test_sba_recurrence_in_full_game_context` land fix:** Changes `m1` from `MOUNTAIN` to `PLAINS` because Fiery Conclusion Elemental costs `{2}{W}` (needs white mana). This is correct and still needed in mainline. The test currently passes only because the mana guard bug (finding #1) allows the spell to resolve without proper payment.

- **`test_commander_snapshot_restore`:** Tests snapshot/restore of command zone and commander damage. This test uses the PR's `setup_commander_game()` and `commander_damage` HashMap, both of which conflict with mainline. Would need rewriting to use mainline's commander model.

**Recommendation:** Keep the Plains fix. Rewrite the commander test to use mainline's API.

---

## Required Actions Before Merge

1. **Rebase onto mainline** (`45d0b0a`) -- the PR is based on `bb96206` which is on an unmerged feature branch
2. **Drop commander field changes** -- already in mainline with a better design
3. **Drop `setup_commander_game()`** -- already in mainline
4. **Fix `colors()` to include hybrid/phyrexian** or revert its doc comment
5. **Extend the mana guard fix** to `CastCommander` and `ActivateAbility`
6. **Rewrite the commander snapshot test** to use mainline's commander API

## What Should Be Kept

- CastSpell `pay()` return check (and extended to other handlers)
- ManaCost X/hybrid/phyrexian parsing (with `colors()` fix)
- `DEFAULT_MAX_ACTIONS` constant
- Plains test fix for `test_sba_recurrence_in_full_game_context`
