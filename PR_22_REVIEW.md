# PR #22 Re-Review: Commander Format + Scryfall API Integration

**Commits:** 3 (`880a4c6` Commander support, `fb537bc` Scryfall integration, `9027790` Review fixes)
**Files changed:** 14 | **Build:** Compiles cleanly in both `default` and `--features scryfall` configs
**Tests:** 206 pass without `scryfall`, 223 pass with `--features scryfall`, 0 fail

---

## Previous review status

All 3 must-fix items and 6 other items from the first review have been addressed. Verification below.

---

## Must-fix items: RESOLVED

### 1. Feature flag for `ureq` — FIXED

`Cargo.toml` now has `[features] scryfall = ["ureq"]` with `ureq` marked `optional = true`. The `scryfall` module is gated with `#[cfg(feature = "scryfall")]` in `src/lib.rs`, and `tests/scryfall_test.rs` has `#![cfg(feature = "scryfall")]`. Default builds skip all HTTP/TLS dependencies entirely. Verified: `cargo check` (no feature) compiles in 11s vs previous 20s with all the TLS deps.

### 2. Color identity validation — FIXED

`validate_commander_deck` in `src/rules/mod.rs:2053-2071` now checks every card's color identity is a subset of the commander's identity (CR 903.4). The `CardDef::color_identity()` method was enhanced (`src/card/mod.rs:387-406`) to include colors from both mana cost and mana abilities (`TapForColor`, `TapForChoice`), so dual lands and basic land subtypes contribute correctly. Uses `HashSet<Color>` for subset checking. Implementation is correct.

**Note:** Color identity from mana symbols in oracle text (e.g., `{R}` in Alesha's activated ability text) is not yet scanned — the doc comment acknowledges this limitation. This is acceptable for the current card pool but should be tracked for future work.

### 3. Command-zone redirect documented — FIXED

`move_object` in `src/game/mod.rs:832-847` now has a thorough doc comment explaining the GTO simplification, referencing CR 903.9a, explaining why always-redirect is the dominant strategy, and noting the future `Action::ChooseCommanderZone` extension point. Excellent documentation.

---

## Other fixes: RESOLVED

### 4. Deduplicated game loops — FIXED

Extracted `run_game_loop()` in `src/simulation/mod.rs` as a shared function. Both `run_game_inner` and `run_commander_game_inner` now just set up their respective `GameState` and delegate to the common loop. The verbose logging branch now also handles `CastCommander` and `OrderTriggers` uniformly. Clean net reduction of ~60 lines.

### 5. `is_commander` uses object identity — FIXED

`PlayerState` now has `commander_object_id: Option<ObjectId>` (`src/game/mod.rs:187-189`). `is_commander()` checks by `commander_object_id` first, falling back to `card_def_id` only when `commander_object_id` is `None` (backward compat). `setup_commander_game` sets `commander_object_id` during setup (`src/rules/mod.rs:1977,1992`). Tests updated to use object identity (`tests/commander_test.rs:32-34`). The redirect tests now properly move the actual commander object rather than creating a new card with the same `card_def_id`, which is a better test pattern.

### 6. `count_mana_symbols` — FIXED

Now explicitly counts only `{w}`, `{u}`, `{b}`, `{r}`, `{g}`, `{c}` symbols (`src/scryfall.rs:951-956`). `{T}`, `{2}`, etc. are correctly excluded.

### 7. `parse_buff` mixed signs — FIXED

Iterates all four sign combinations (`+/+`, `-/-`, `+/-`, `-/+`) with independent `sign_p` and `sign_t` loops (`src/scryfall.rs:931-946`).

### 8. `urlencode` — FIXED

Renamed from `urlencod` to `urlencode` (`src/scryfall.rs:955`). Now does proper RFC 3986 percent-encoding: unreserved characters (`A-Za-z0-9-_.~`) pass through, spaces become `+`, everything else is percent-encoded byte-by-byte. Handles multi-byte UTF-8 correctly.

### 9. Type line parsing — FIXED

Uses `split_once(" — ")` and `split_once(" - ")` instead of manual `find` + byte offset arithmetic (`src/scryfall.rs:496-501`). Eliminates the fragile `idx + 5` calculation.

---

## Remaining items (non-blocking, informational)

These items from the original review were not addressed in this round but are acceptable as follow-up work:

- **Mana payment verification in `apply_action`** (original #2): `CastCommander` (and pre-existing `CastSpell`) don't verify that mana payment succeeded after `auto_tap_lands`. Low risk since `can_potentially_pay` gates the action in `legal_actions`.
- **`setup_commander_game` duplication** (original #8): Still duplicates the shuffle/draw/phase-entry pattern from `setup_game`. Less critical now that the game loop itself is deduplicated.
- **Snapshot/restore test for commander state** (original #9): No dedicated test for snapshot mid-commander-game. Works correctly since `PlayerState` is `Clone`, but a test would add confidence.
- **`ManaCost::parse` format coverage** (original #12): Should verify handling of hybrid (`{W/U}`), phyrexian (`{W/P}`), and X costs.
- **Magic number `10_000`** (original #14): Starting card ID for Scryfall-fetched cards. A named constant would be clearer.

---

## Verdict

**Approve.** All must-fix items and requested changes have been properly addressed. The fixes are clean, well-implemented, and maintain backward compatibility. The feature flag correctly isolates the HTTP dependency. Color identity validation follows the comprehensive rules. The code deduplication and object-identity improvements strengthen the architecture.

223 tests pass across both build configurations. Ready to merge.
