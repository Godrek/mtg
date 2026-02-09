# PR #22 Review: Commander Format + Scryfall API Integration

**Commits:** 2 (`880a4c6` Add 1v1 Commander format support, `fb537bc` Add Scryfall API integration)
**Files changed:** 14 | **+3,149 / -9**
**Build:** Compiles cleanly | **Tests:** 166 pass, 0 fail (4 network-gated ignored)

---

## Summary

This PR adds two major features:

1. **1v1 Commander format** — command zone, commander tax, commander damage (21 lethal), death/exile redirect to command zone, 100-card singleton deck validation, full game setup and simulation.
2. **Scryfall API integration** — fetch any Magic card by name, convert to internal `CardDef`, disk-cache results, import full decklists.

---

## What works well

- **Clean separation of concerns.** Commander logic lives in the right places: format enum in `game/mod.rs`, rules in `rules/mod.rs`, actions in `action/`, simulation in `simulation/mod.rs`. The Scryfall module is fully self-contained.
- **Comprehensive test coverage.** 24 commander tests cover every mechanic (command zone, tax, redirect, damage tracking, deck validation, full game sim). 17 offline Scryfall parser tests plus 4 gated live tests.
- **Correct commander redirect.** `move_object` intercepts graveyard/exile destinations for commanders (CR 903.9a). The implementation correctly updates the emitted `ZoneType` events to reflect the actual destination.
- **Rate limiting and caching.** Scryfall fetcher respects the 100ms rate limit and caches to disk, which is good API citizenship.
- **Backward compatibility.** Existing `GameState::new()` defaults to `Standard` format. All pre-existing tests continue to pass.
- **Combat damage source tracking fixed.** The `DamageEvent.source_id` field replaces the hardcoded `source: 0` placeholder — a real bug fix that benefits more than just commander.

---

## Issues

### Bugs / Correctness

1. **Commander redirect is not optional (CR 903.9a).** The current implementation *always* redirects the commander to the command zone on death/exile. Per the 2024 rules update, the commander's owner must *choose* whether to send it to the command zone. This is a forced redirect in the code (`move_object` in `src/game/mod.rs:836-843`). The code comment says "the GTO-optimal choice in almost all cases," which is a reasonable simplification for a GTO solver, but it does deviate from the actual rules. This should be documented more prominently or gated behind a flag.

2. **Commander tax is applied before mana payment verification in `apply_action`.** In `src/rules/mod.rs:285-310`, `CastCommander` calls `auto_tap_lands` then `mana_pool.pay` but does not check whether payment actually succeeded. If `can_potentially_pay` in `legal_actions` has an edge-case mismatch with `auto_tap_lands`, the game could proceed with unpaid costs. `CastSpell` has the same pattern, so this is pre-existing, but worth noting.

3. **`is_commander` matches by `card_def_id`, not object identity.** In `src/game/mod.rs:1284-1293`, `is_commander` checks if `inst.card_def_id == commander_card_id`. If a player somehow has two copies of the same card (e.g., via Clone effects), both would be treated as the commander. Not a problem today since Clone isn't implemented, but will be fragile if/when it is.

4. **`count_mana_symbols` over-counts.** In `src/scryfall.rs:948-951`, `count_mana_symbols` counts *all* `{` characters in the text, not just mana symbols. Oracle text with ability costs like `{2}, {T}: ...` would inflate the count. This affects the `AddMana` effect parsing.

5. **`parse_buff` doesn't handle mixed signs.** `src/scryfall.rs:930-944` only matches `+N/+N` or `-N/-N`, not `+N/-N` (e.g., "target creature gets +3/-1 until end of turn"). The loop iterates signs `["+", "-"]` and uses the same sign for both power and toughness.

### Design / Architecture

6. **`ureq` is a synchronous HTTP client added as a non-optional dependency.** This adds 535 lines to `Cargo.lock` and substantial compile-time cost (ring, rustls, url, idna, icu_*). Since Scryfall fetching is only used for deck import (not during simulation), this should be behind a Cargo feature flag (e.g., `[features] scryfall = ["ureq"]`) so CI/simulation builds aren't burdened.

7. **`run_commander_game_inner` is a near-copy of `run_game_inner`.** `src/simulation/mod.rs` duplicates ~60 lines of game-loop logic. The only differences are `new_commander` vs `new`, `setup_commander_game` vs `setup_game`, and a verbose branch that handles `CastCommander`. These should share a common inner loop or be parameterized by format.

8. **`setup_commander_game` duplicates `setup_game` logic.** `src/rules/mod.rs:1949-2010` — the two functions have the same shuffle/draw/phase-entry pattern. Consider extracting a shared helper.

9. **Snapshot doesn't persist `commander_tax` / `commander_damage_received` on players.** `GameStateSnapshot` now includes `format`, but the snapshot is just a clone of player state vectors (which *do* include commander fields since `PlayerState` is `Clone`). This works correctly today. However, `GameStateSnapshot` explicitly lists its fields — the new `format` field was added, but there's no verification that all new `PlayerState` commander fields are tested through snapshot/restore. A test that snapshots mid-commander-game and restores would be prudent.

10. **`validate_commander_deck` does not check color identity.** Commander decks require all cards to share the commander's color identity. This is a core deckbuilding rule (CR 903.4) that's missing entirely.

### Minor / Style

11. **`urlencod` is misspelled.** `src/scryfall.rs:954` — should be `urlencode` or `url_encode`. Also, the encoding is incomplete (only handles space, apostrophe, comma). Consider using the `url` crate's encoding since it's already a transitive dependency.

12. **`ManaCost::parse` is assumed to exist.** The Scryfall module calls `ManaCost::parse(s)` (`src/scryfall.rs:429`). I can see it compiles, but I couldn't find where this method was added — verify it handles all Scryfall mana cost formats (hybrid, phyrexian, X costs, etc.).

13. **Type line parsing with em-dash byte offset.** `src/scryfall.rs:498` uses `&type_line[idx + 5..]` for the em-dash (`" — "`). The em-dash `—` is 3 bytes in UTF-8, so `" — "` is `1 + 3 + 1 = 5` bytes. This is correct but fragile — a comment noting the byte math or using `str::split_once` would be safer.

14. **Magic numbers.** `next_card_id: 10_000` in the Scryfall fetcher (`src/scryfall.rs:158`) assumes sample card IDs stay below 10,000. A constant would be clearer.

---

## Verdict

**Approve with requested changes.** The feature implementation is solid, well-tested, and correctly integrated into the existing architecture. The commander mechanics work as expected across all tests. The Scryfall integration is a practical addition for expanding the card pool.

The mandatory items before merge are:
- **Issue #6:** Put `ureq` behind a feature flag to avoid bloating non-network builds.
- **Issue #10:** Add color identity validation to `validate_commander_deck` (core Commander rule).
- **Issue #1:** Add a prominent doc comment about the forced command-zone redirect simplification.

Everything else (dedup of simulation/setup code, `urlencod` rename, `count_mana_symbols` fix) can be follow-up work.
