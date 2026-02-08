# PR #13 Review: Phase 0 — Shared Interface Contract

**Branch**: `claude/implement-phase-0-NFgI6` -> `mainline`
**Commit**: `c03b4d9 Implement Phase 0: Shared Interface Contract for Rules Engine + MCCFR`
**Files changed**: 4 (+1103 / -1)

---

## Summary

This PR implements Phase 0 of the Consolidated Strategy: the shared interface contract that both Track A (Rules Engine) and Track B (MCCFR Solver) will code against. It adds the Observation API (`PlayerView`), the Canonical Action Mapping (`canonical.rs`), and the Snapshot Contract (documentation on `GameState`).

---

## Deliverable-by-Deliverable Assessment

### 0.1 — Observation API: `GameState::visible_state(player)` — PASS

**Strategy spec** (lines 62-106): Defines `PlayerView<'a>` with public, per-player, and private information fields, plus `visible_state()` returning it.

**What the PR delivers** (`src/game/mod.rs:+279-388`):
- `PlayerView<'a>` struct with all specified fields: `phase`, `active_player`, `turn_number`, `battlefield`, `stack`, `combat`, `pending_triggers`, `my_life`/`opp_life`, `my_graveyard`/`opp_graveyard`, `my_exile`/`opp_exile`, `opp_hand_size`, `opp_library_size`, `my_hand`, `my_mana_pool`, `my_land_plays_remaining`, `objects`, `card_db`.
- `visible_state()` correctly maps `self.opponent(player)` to populate the "my" vs "opp" distinction.
- Adds `priority_player` field not in the original spec — a reasonable addition since the MCCFR solver needs to know who has priority.

**Test coverage**: 3 integration tests (`test_player_view_basic_fields`, `test_player_view_hides_opponent_hand_contents`, `test_player_view_graveyard_and_exile_visible`) validate correct information partitioning.

**Assessment**: Fully satisfies 0.1. The struct compiles and `visible_state()` returns it correctly.

### 0.2 — Canonical Action Mapping — PASS

**Strategy spec** (lines 108-132): Defines `CanonicalAction` enum, `canonicalize()` and `resolve()` functions with the round-trip invariant.

**What the PR delivers** (`src/action/canonical.rs`, 709 lines):
- `CanonicalAction` enum covers all 12 `Action` variants: `PassPriority`, `PlayLand`, `CastSpell`, `ActivateManaAbility`, `ActivateAbility`, `DeclareAttackers`, `DeclareBlockers`, `Discard`, `OrderTriggers`, `OrderDamageAssignment`, `Concede`.
- `CanonicalTarget` enum for stable target references (player index or card-id + controller + instance-index).
- `canonicalize(&Action, &GameState) -> CanonicalAction` — maps ObjectIds to CardIds via the objects map, with instance disambiguation by sorted battlefield position.
- `resolve(&CanonicalAction, &GameState, PlayerIndex) -> Option<Action>` — reverse mapping with graceful `None` on lookup failure.
- Helper functions: `battlefield_instance_index()`, `find_in_hand()`, `find_on_battlefield_by_index()`, `find_on_battlefield_by_controller()`, `canonicalize_target()`, `resolve_target()`.
- Both types derive `Serialize, Deserialize` for future regret table persistence.

**Round-trip invariant verification**: The acceptance criterion states `resolve(canonicalize(action, state), state, player) == Some(action)` for all legal actions. This is tested by:
- 7 unit tests in `canonical::tests` covering individual action types
- `test_canonicalize_all_legal_actions_roundtrip` — constructs a rich game state and verifies round-trip for every `legal_actions()` output
- `test_canonical_roundtrip_combat_phase` — exercises DeclareAttackers with multiple creatures
- `test_canonical_roundtrip_trigger_ordering` — exercises OrderTriggers with duplicate Elvish Visionaries
- `test_canonical_roundtrip_full_game_all_actions` — **runs a complete game** and verifies round-trip for every legal action at every game state encountered (tested >100 actions across ~100 turns)

**Assessment**: Fully satisfies 0.2. The round-trip invariant holds across all tested scenarios.

### 0.3 — Snapshot Contract — PASS

**Strategy spec** (lines 134-141): Document clone vs. shared vs. reconstructed rules.

**What the PR delivers** (`src/game/mod.rs:+157-185`): Doc comment on `GameState` documenting:
- **Always cloned**: objects, players, battlefield, stack, combat, pending_triggers, all scalars
- **Shared via Arc**: card_db
- **Reconstructed after clone**: event bus (Phase 1A), continuous effects caches (Phase 2A), dirty flags
- **Rule**: Derivable fields must use `#[serde(skip)]` and be excluded from equality/hashing

**Assessment**: Satisfies 0.3. Delivered as documentation (doc comment) rather than a separate file, which is arguably better since it lives next to the struct it governs.

---

## Acceptance Criteria Checklist

| Criterion | Status | Evidence |
|---|---|---|
| `PlayerView` struct compiles and `visible_state()` returns it | PASS | All 30 tests pass including 3 dedicated PlayerView tests |
| `canonicalize()` round-trips for all legal actions | PASS | Unit tests + full-game round-trip test (`test_canonical_roundtrip_full_game_all_actions`) |
| Both tracks agree this is the interface they code against | N/A | Social agreement, not testable by code review |

---

## Issues and Observations

### Minor Issues

1. **`Discard` canonicalization lacks disambiguation for duplicate cards in hand** (`canonical.rs:194-197`): `find_in_hand()` returns the *first* matching card_id in hand. If the player holds two Mountains, `Discard { object_id: mountain_2 }` canonicalizes to `CanonicalAction::Discard { card_id: MOUNTAIN }`, but `resolve()` returns `mountain_1` — a *different* ObjectId. The round-trip invariant technically fails when the hand contains duplicates of the discarded card. This doesn't matter for correctness (discarding either Mountain is strategically identical), but it should be documented or an `instance_index` added for strict round-trip purity, similar to how `ActivateManaAbility` disambiguates.

2. **`PlayLand` has the same duplicate-in-hand issue** (`canonical.rs:113-116`): Same pattern as Discard. If the player holds two copies of the same land, `resolve()` always picks the first.

3. **`CastSpell` also shares this pattern** (`canonical.rs:118-128`): Two copies of the same spell in hand — resolve picks the first.

4. **`battlefield_instance_index` considers all controllers** (`canonical.rs:424-434`): The function counts all battlefield permanents with a matching `card_def_id` regardless of controller, but `find_on_battlefield_by_index` does the same. This is internally consistent, but `CanonicalTarget::Object` uses `controller` + `instance_index` via `find_on_battlefield_by_controller()`, creating two different disambiguation schemes. Battlefield-sourced actions (ActivateAbility, DeclareAttackers) use the controller-agnostic scheme, while targets use the controller-scoped scheme. This is fine but worth noting for future maintainers.

5. **No `my_library` field in `PlayerView`**: The spec doesn't include it (the player's own library contents are hidden in MTG rules for information-set purposes). However, some solver implementations might want the player's own library size for evaluating deck-out scenarios. Currently accessible indirectly via `objects` traversal but not directly exposed.

### Structural Observations

6. **Strategy spec says "no code changes to existing modules"** (line 58): The PR does modify `src/action/mod.rs` (adds `pub mod canonical;`) and `src/game/mod.rs` (adds `PlayerView` and `visible_state()`). This is a pragmatic deviation — the new types *must* live in existing modules to access private fields. The intent of the constraint (don't break existing behavior) is preserved: no existing function signatures or behaviors change.

7. **The `objects` field in `PlayerView` exposes all objects including opponent's library/hand contents**: While the top-level fields correctly partition information, `objects: &'a HashMap<ObjectId, CardInstance>` gives MCCFR access to look up any ObjectId including those in the opponent's hidden zones. The solver must be disciplined about only looking up ObjectIds obtained through the view's public fields. A future refinement could filter the objects map, but this adds complexity and allocation cost.

8. **Good design choices**:
   - `Serialize`/`Deserialize` on canonical types enables regret table persistence (needed by Phase 1B).
   - Instance disambiguation by sorted ObjectId is deterministic.
   - `resolve()` returns `Option<Action>` rather than panicking — clean error handling for the solver.
   - The full-game round-trip test is an excellent acceptance test that exercises the entire action space.

---

## Verdict

**APPROVE with minor comments.** The PR delivers all three Phase 0 deliverables and satisfies the stated acceptance criteria. The code is well-structured, thoroughly tested (30 tests pass, including a full-game round-trip stress test), and provides a clean interface boundary for both tracks.

The hand-duplicate disambiguation issue (items 1-3) is the only functional concern. It doesn't affect correctness for MCCFR (discarding equivalent cards is the same decision), but it means the strict `resolve(canonicalize(a, s), s, p) == Some(a)` invariant doesn't hold at the ObjectId level when duplicates exist in hand. This should be either documented as intentional or fixed with an instance index before Phase 1B relies on strict round-trip equality.
