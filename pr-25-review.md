# PR #25 Review (Pass 2): Add Kinnan, Bonder Prodigy Commander Deck

## Summary

Second review pass after the fix commit (`1ce141e`). The PR now has 2 commits:
1. `da320aa` — Original: 95 card definitions, Kinnan deck, deck import, 4 new effects
2. `1ce141e` — Fix: correctness bugs + 9 new tests

**Verdict: Approve with minor issues**

All 224 tests pass. The goldfish removal has been properly unbundled. The fix commit addresses the major correctness bugs from review pass 1. Two new issues remain — one is a real bug in `auto_tap_lands`, the other is dead trigger code.

---

## What Changed (8 files, +2086 / -181 lines)

| File | Change |
|------|--------|
| `decks/kinnan_bonder_prodigy.txt` | New 100-card Commander decklist |
| `src/action/mod.rs` | `TapForColorlessAmount` handling in `can_potentially_pay` |
| `src/card/mod.rs` | +4 trigger conditions, +4 effects, `TapForColorlessAmount(u32)`, `commanders` on `Decklist` |
| `src/card/sample.rs` | 95 new card definitions + `kinnan_commander_deck()` builder |
| `src/deck_import.rs` | Section-based parsing, `parse_card_line()` / `insert_entry()` helpers |
| `src/rules/mod.rs` | Effect resolution for 4 new effects, `TapForColorlessAmount` in mana activation + `auto_tap_lands` |
| `tests/deck_import_test.rs` | +2 tests: commander section parsing, Kinnan deck file import |
| `tests/integration_test.rs` | +7 tests: deck builder, mana rocks, Rhystic Study, fetchlands, mana resolution, Kinnan card, new cards |

---

## Issues Resolved from Pass 1

All major correctness bugs from the first review have been fixed:

- **Basalt Monolith** `enters_tapped: true` removed, changed to `TapForColorlessAmount(3)`
- **Grim Monolith & Mana Vault** changed to `TapForColorlessAmount(3)`
- **Sol Ring** changed from empty `mana_abilities: vec![]` to `TapForColorlessAmount(2)`
- **Ancient Tomb** changed to `TapForColorlessAmount(2)`
- **Rhystic Study** trigger changed to `OpponentCastsSpell`
- **Fetchlands** moved from `spell_effect` to `activated_abilities`
- **Goldfish removal unbundled** from this PR
- **9 new tests** added covering all fixed items

---

## New Issues Found in Pass 2

### Bug: `auto_tap_lands` ignores `TapForColorlessAmount` — always adds 1 mana

In `src/rules/mod.rs:1948-1951`, `TapForColorlessAmount(_)` is grouped with `TapForColorless` and `TapForAny` into `TapDecision::Colorless(land_id)`. However, the `TapDecision` enum only has:

```rust
enum TapDecision {
    Color(ObjectId, Color),
    Colorless(ObjectId),  // <-- no amount field
}
```

When Phase 2 applies these decisions (line 1977-1978):
```rust
TapDecision::Colorless(land_id) => {
    state.players[player].mana_pool.colorless += 1;  // Always 1!
}
```

This means **auto-tapping a Sol Ring (2 colorless), Basalt Monolith (3 colorless), or Mana Vault (3 colorless) only adds 1 colorless mana** when the engine auto-taps to pay for a spell. The explicit `ActivateManaAbility` action correctly adds the full amount (line 136), but the `auto_tap_lands` shortcut used during `CastSpell` does not.

**Fix:** Add an amount to the `Colorless` variant: `Colorless(ObjectId, u32)`, and use it in Phase 2. Also needs to handle the `still_need` counter properly — tapping one Sol Ring should reduce `still_need` by 2, not 1.

Additionally, in the generic-cost Phase 1 loop (line 1962), `still_need -= 1` is wrong for multi-mana producers. A Sol Ring tap should decrement `still_need` by the amount produced (2), not by 1.

### Dead Code: New trigger conditions are never fired by the rules engine

The four new `TriggerCondition` variants are defined and assigned to cards but **never invoked** in the rules engine:

- `YouCastSpell` — never called via `check_triggers(state, TriggerCondition::YouCastSpell, ...)` in the `CastSpell` action handler (line 65-102)
- `OpponentCastsSpell` — same (affects Rhystic Study)
- `OpponentCastsNoncreatureSpell` — same (affects Mystic Remora, Nezahal)
- `OpponentDrawsCard` — never called in the card draw logic (line ~1527) (affects Consecrated Sphinx, Faerie Mastermind)

The `CastSpell` handler emits a `GameEvent::SpellCast` event (line 91) and the draw function emits `GameEvent::CardDrawn` (line 1527), but neither calls `check_triggers()`. This means **Rhystic Study, Mystic Remora, Consecrated Sphinx, Tidespout Tyrant, Hullbreaker Horror, and other trigger-based cards in the Kinnan deck will never fire their triggered abilities during simulation.**

The test `test_rhystic_study_triggers_on_any_spell` only verifies the trigger condition value on the `CardDef`, not that it actually fires during gameplay.

**Fix:** Add `check_triggers` calls in the appropriate locations:
- After spell cast (line ~101): fire `YouCastSpell` for the caster's permanents, `OpponentCastsSpell` for opponents' permanents, and `OpponentCastsNoncreatureSpell` for noncreature spells
- After card draw (line ~1530): fire `OpponentDrawsCard` for the drawing player's opponents' permanents

### Minor: Wildcard match arms in new effect handlers

`ReturnToTopOfLibrary` (line ~863) and `UntapTarget` (line ~873) use `_ => {}` to silently ignore `Target::Player(_)`. Should use explicit patterns:

```rust
Target::Player(_) => {} // Players can't be returned/untapped
```

This prevents silently ignoring future `Target` variants.

---

## Remaining Modeling Simplifications (Tracked, Not Blocking)

These were noted in pass 1 and remain. They're acceptable for the current simulation fidelity but worth tracking:

1. **`SearchLibrary` always takes `library[0]`** — tutors can't actually choose a card
2. **Gaea's Cradle** produces 1 {G} instead of {G} per creature
3. **Chrome Mox / Mox Diamond / Mox Opal** have no imprint/discard/metalcraft cost
4. **Faerie Mastermind** triggers on every opponent draw instead of only the second
5. **The One Ring** draws 1 card instead of cards equal to burden counters
6. **`UntapTarget { target: TargetSpec::Controller }`** for self-untap is semantically wrong (should be `TargetSpec::Self`)

---

## Test Coverage Assessment

Good improvement from pass 1. The 9 new tests cover:
- Kinnan deck builder (100 cards, singleton, all in DB)
- Mana rock amounts (Sol Ring=2, Basalt/Grim/Vault=3, Tomb=2)
- Rhystic Study trigger condition
- Fetchland ability type
- `TapForColorlessAmount` mana resolution via explicit `ActivateManaAbility`
- Kinnan card definition
- New card definitions in DB

**Still missing:**
- End-to-end test that Rhystic Study actually triggers during a game (will fail — see dead trigger code issue)
- Test for `auto_tap_lands` with `TapForColorlessAmount` (will expose the += 1 bug)
- Tests for `BounceAllNonlandOpponents`, `ReturnToTopOfLibrary`, `UntapTarget` effect resolution

---

## Recommendation

**Approve with minor issues.** The fix commit resolves all major correctness bugs from pass 1. The remaining issues are:

1. **Bug (should fix):** `auto_tap_lands` adds only 1 colorless for `TapForColorlessAmount(N)` sources — needs `TapDecision::Colorless(ObjectId, u32)` and updated `still_need` decrement
2. **Dead code (should fix or document):** The 4 new trigger conditions are never fired by the rules engine, making several key Kinnan deck cards inert during simulation
3. **Minor:** Use explicit match arms instead of `_ => {}`
