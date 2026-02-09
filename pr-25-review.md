# PR #25 Review: Add Kinnan, Bonder Prodigy Commander Deck with 95 New Card Definitions

## Summary

This PR adds a complete Kinnan, Bonder Prodigy cEDH (competitive Commander) deck to the simulator, along with 95 new card definitions, Commander-format deck parsing support, four new effect types, three new trigger conditions, and the removal of the Goldfish mode feature.

**Verdict: Approve with suggestions**

The PR compiles cleanly and all 206 tests pass. The card definitions are thorough and the deck import refactoring is well-structured. There are several modeling simplifications worth noting and one bundling concern.

---

## What Changed (9 files, +1816 / -756 lines)

| File | Change |
|------|--------|
| `decks/kinnan_bonder_prodigy.txt` | New 100-card Commander decklist with `~~Commanders~~` / `~~Mainboard~~` section headers |
| `src/card/mod.rs` | +3 trigger conditions (`YouCastSpell`, `OpponentCastsNoncreatureSpell`, `OpponentDrawsCard`), +4 effects (`SearchLibrary`, `BounceAllNonlandOpponents`, `ReturnToTopOfLibrary`, `UntapTarget`), `commanders` field on `Decklist` |
| `src/card/sample.rs` | 95 new card definitions (lands, artifacts, creatures, instants, sorceries, enchantments, planeswalker, DFCs) + `kinnan_commander_deck()` builder |
| `src/deck_import.rs` | Section-based parsing (`~~Commanders~~`, `~~Mainboard~~`), refactored into `parse_card_line()` and `insert_entry()` helpers |
| `src/main.rs` | Removed goldfish mode invocations |
| `src/rules/mod.rs` | Effect resolution for the 4 new effect types |
| `src/simulation/mod.rs` | Removed goldfish simulation (250+ lines), fixed `is_multiple_of` → `% 10 == 0` |
| `src/strategy/mod.rs` | Removed `GoldfishStrategy` (73 lines) |
| `tests/integration_test.rs` | Removed all goldfish tests (230 lines) |

---

## Positive Aspects

1. **Clean deck import refactoring.** The `deck_import.rs` changes extract `parse_card_line()` and `insert_entry()` helpers, reducing duplication. Section-based parsing (`~~Commanders~~`) is a sensible format choice that stays backward-compatible.

2. **Good effect modeling coverage.** The four new effects (`SearchLibrary`, `BounceAllNonlandOpponents`, `ReturnToTopOfLibrary`, `UntapTarget`) cover the most important mechanical patterns in the Kinnan deck — tutors, Cyclonic Rift overload, graveyard recursion, and untap combo pieces.

3. **All tests pass.** 206 tests across 7 test suites, including 93 integration tests, all pass. The build is clean with no new warnings.

4. **Solid card coverage.** The 95 new cards represent a realistic cEDH Kinnan list with mana dorks, fast mana, counterspells, tutors, and combo finishers (Tidespout Tyrant, Hullbreaker Horror).

---

## Issues & Suggestions

### Major: Goldfish removal is unrelated and should be a separate PR

The removal of `GoldfishStrategy`, `simulate_goldfish()`, and all associated tests (~550 lines deleted) is logically independent of adding the Kinnan deck. Bundling feature removal with feature addition makes it harder to bisect regressions and review each change on its merits. Consider splitting this into two PRs.

### Modeling Fidelity Concerns

These are worth tracking even if acceptable for the current simulation fidelity:

1. **`SearchLibrary` always takes the top card.** (`src/rules/mod.rs:826-831`) The simplified tutor just moves `library[0]`, which means tutors don't actually search. For a Kinnan deck where tutoring for the right combo piece is the core strategic decision, this significantly reduces the deck's simulated win rate and strategic depth. Consider at minimum allowing the strategy to choose the card.

2. **Basalt Monolith marked `enters_tapped: true`.** (`src/card/sample.rs`) Basalt Monolith doesn't enter tapped — it just doesn't untap during the untap step. The `enters_tapped` flag likely prevents it from being tapped for mana the turn it resolves, which is incorrect since it can produce {C}{C}{C} immediately.

3. **Gaea's Cradle produces only {G}.** The card should produce {G} for each creature you control (a dynamic mana ability). Modeling it as `TapForColor(Color::Green)` — producing exactly one {G} — significantly undervalues the card. A `DynamicMana` variant or similar would be more accurate.

4. **Chrome Mox / Mox Diamond / Mox Opal** are modeled as `TapForAny` with no imprint/discard/metalcraft cost. These should at minimum not produce mana without their conditions met, or they'll be strictly better than they actually are in simulation.

5. **`UntapTarget { target: TargetSpec::Controller }`** for Basalt/Grim Monolith's self-untap abilities is semantically odd — `TargetSpec::Controller` likely targets the controlling player, not "self." Consider a `TargetSpec::Self` variant or similar.

6. **Fetchlands use `spell_effect` for their activated ability.** Flooded Strand, Misty Rainforest, and Windswept Heath have `spell_effect: Some(Effect::SearchLibrary { ... })`. Since they're lands (not spells), this field won't trigger on entering the battlefield. These should likely be `activated_abilities` with a tap + sacrifice cost.

7. **Rhystic Study trigger is modeled as `OpponentCastsNoncreatureSpell`.** Rhystic Study triggers on *any* spell an opponent casts, not just noncreature spells. The `TriggerCondition::OpponentCastsNoncreatureSpell` is correct for Mystic Remora but not for Rhystic Study.

8. **Faerie Mastermind trigger simplification.** The real card triggers only on the opponent's *second* draw each turn, not on every draw. Modeling it as `OpponentDrawsCard → DrawCards { count: 1 }` makes it far too powerful.

9. **The One Ring's draw ability** is modeled as drawing 1 card, but the real card draws cards equal to the number of burden counters (cumulative). The simulation will significantly undercount its card advantage.

10. **Mana Vault produces `TapForColorless` (1 colorless)** but actually produces {C}{C}{C}. Same issue with Basalt Monolith and Grim Monolith.

### Minor / Style

- **`ReturnToTopOfLibrary` uses `_ => {}`** match arm (`src/rules/mod.rs:865`). Prefer `Target::Player(_) => {}` to be explicit about what's being ignored, which also future-proofs against new `Target` variants.
- **`UntapTarget` also uses `_ => {}`** (`src/rules/mod.rs:875`). Same suggestion.
- **`is_multiple_of(10)` → `% 10 == 0`** in `simulation/mod.rs:201` is a good fix — `is_multiple_of` is a nightly-only method.

---

## Test Coverage

- All existing tests pass (206 total)
- The goldfish tests were properly removed alongside the goldfish feature
- No new tests were added for the Kinnan deck or the new effects — consider adding at least:
  - A test that `kinnan_commander_deck()` returns exactly 100 cards
  - A test that the deck import correctly parses `~~Commanders~~` sections
  - Tests for `SearchLibrary`, `BounceAllNonlandOpponents`, `ReturnToTopOfLibrary`, `UntapTarget` effect resolution

---

## Recommendation

**Approve with suggestions.** The core changes (card definitions, deck import, new effects) are solid and the code compiles + passes all tests. The main actionable items are:

1. Split goldfish removal into a separate PR (or document why it's bundled)
2. Fix Basalt Monolith `enters_tapped: true` — this is a correctness bug
3. Fix Rhystic Study trigger condition (should trigger on all spells, not just noncreature)
4. Add basic tests for the new effects and deck builder
5. Track the modeling simplifications (tutor always picking top card, mana amounts, etc.) as future work items
