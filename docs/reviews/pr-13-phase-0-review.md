# PR #13 Review: Phase 0 — Shared Interface Contract

**Branch**: `claude/implement-phase-0-NFgI6` -> `mainline`
**Commits**: 2 (`c03b4d9` initial implementation, `2baf8ca` fixes)
**Files changed**: 4 (+1390 / -20)
**Tests**: 46 pass, 0 fail, 0 warnings

---

## Summary

This PR implements Phase 0 of the Consolidated Strategy: the shared interface contract that both Track A (Rules Engine) and Track B (MCCFR Solver) will code against. It adds the Observation API (`PlayerView`), the Canonical Action Mapping (`canonical.rs`), and the Snapshot Contract (documentation on `GameState`).

A follow-up commit (`2baf8ca`) fixes the two issues from the first review pass: hand-duplicate disambiguation and the `PlayerView.objects` information leak.

---

## Deliverable-by-Deliverable Assessment

### 0.1 — Observation API: `GameState::visible_state(player)` — PASS

**Strategy spec** (lines 62-106): Defines `PlayerView<'a>` with public, per-player, and private information fields, plus `visible_state()` returning it.

**What the PR delivers** (`src/game/mod.rs`):
- `PlayerView<'a>` struct with all specified fields: `phase`, `active_player`, `turn_number`, `battlefield`, `stack`, `combat`, `pending_triggers`, `my_life`/`opp_life`, `my_graveyard`/`opp_graveyard`, `my_exile`/`opp_exile`, `opp_hand_size`, `opp_library_size`, `my_hand`, `my_mana_pool`, `my_land_plays_remaining`, `objects`, `card_db`.
- `visible_state()` correctly maps `self.opponent(player)` to populate the "my" vs "opp" distinction.
- Adds `priority_player` field not in the original spec — a reasonable addition since the MCCFR solver needs to know who has priority.
- `objects` field is now a **filtered** `HashMap<ObjectId, &'a CardInstance>` (not the full map) — excludes opponent hand contents and both libraries. Good fix from the first review.

**Test coverage**: 5 integration tests verify information partitioning:
- `test_player_view_basic_fields`
- `test_player_view_hides_opponent_hand_contents`
- `test_player_view_graveyard_and_exile_visible`
- `test_player_view_objects_excludes_opponent_hand` (new)
- `test_player_view_objects_excludes_libraries` (new)

**Assessment**: Fully satisfies 0.1. The struct compiles, `visible_state()` returns it correctly, and the objects map now enforces information boundaries at the data level.

### 0.2 — Canonical Action Mapping — PASS

**Strategy spec** (lines 108-132): Defines `CanonicalAction` enum, `canonicalize()` and `resolve()` functions with the round-trip invariant.

**What the PR delivers** (`src/action/canonical.rs`, ~744 lines):
- `CanonicalAction` enum covers all 12 `Action` variants.
- `CanonicalTarget` enum for stable target references.
- `canonicalize()` / `resolve()` with full round-trip support.
- `PlayLand`, `CastSpell`, `Discard` now include `hand_index` for duplicate disambiguation (fixed in `2baf8ca`).
- Helper: `hand_instance_index()` mirrors `battlefield_instance_index()` for hand zone.

**Round-trip invariant verification**: Tested by 11 unit tests + 4 integration tests including:
- `test_canonical_hand_duplicate_disambiguation` — 2 Mountains in hand, verifies distinct canonical forms and exact ObjectId recovery
- `test_canonical_discard_hand_duplicate_disambiguation` — 8 Mountains, verifies all 8 produce distinct canonical forms
- `test_canonical_roundtrip_full_game_all_actions` — full game, all legal actions at every state

**Assessment**: Fully satisfies 0.2. The round-trip invariant now holds at strict ObjectId equality, even for duplicate cards in hand.

### 0.3 — Snapshot Contract — PASS

**Strategy spec** (lines 134-141): Document clone vs. shared vs. reconstructed rules.

**What the PR delivers**: Doc comment on `GameState` documenting:
- **Always cloned**: objects, players, battlefield, stack, combat, pending_triggers, all scalars
- **Shared via Arc**: card_db
- **Reconstructed after clone**: event bus (Phase 1A), continuous effects caches (Phase 2A), dirty flags
- **Rule**: Derivable fields must use `#[serde(skip)]` and be excluded from equality/hashing

**Assessment**: Satisfies 0.3.

---

## Acceptance Criteria Checklist

| Criterion | Status | Evidence |
|---|---|---|
| `PlayerView` struct compiles and `visible_state()` returns it | PASS | 46 tests pass including 5 dedicated PlayerView tests |
| `canonicalize()` round-trips for all legal actions | PASS | Unit tests + full-game round-trip test, including hand-duplicate edge cases |
| Both tracks agree this is the interface they code against | N/A | Social agreement, not testable by code review |

---

## Previously Reported Issues — Status

### Issue 1-3: Hand-duplicate disambiguation — FIXED

`PlayLand`, `CastSpell`, and `Discard` canonical forms now include a `hand_index` field computed via `hand_instance_index()` (sorted by ObjectId). Tests verify strict ObjectId-level round-trip equality with 2 and 8 duplicates.

### Issue 7: PlayerView.objects information leak — FIXED

Changed from `&'a HashMap<ObjectId, CardInstance>` (full map) to `HashMap<ObjectId, &'a CardInstance>` (filtered). `visible_state()` explicitly collects only objects from visible zones. Two tests verify opponent hand and library objects are excluded.

---

## New Issues Found (Second Pass)

### BUG: `visible_state()` only adds `StackSource::Spell` objects, ignoring ability sources — MEDIUM

**Location**: `src/game/mod.rs`, `visible_state()` stack handling block.

**Problem**: The `StackSource` enum has three variants:
```rust
pub enum StackSource {
    Spell(ObjectId),
    ActivatedAbility { source_id: ObjectId, ability_index: usize },
    TriggeredAbility { source_id: ObjectId, ability_index: usize },
}
```

But `visible_state()` only matches `Spell`:
```rust
for entry in &self.stack {
    if let StackSource::Spell(id) = entry.source {
        // Only Spell handled — ActivatedAbility and TriggeredAbility are skipped
    }
}
```

When an activated or triggered ability is on the stack, its source permanent's ObjectId is not added to the visible objects map via this code path. In practice, the source is usually still on the battlefield (already included), but there are MTG scenarios where it isn't (e.g., a creature's dies trigger on the stack after the creature moved to the graveyard — though the graveyard is also collected). The fix is simple — use a `match` to extract the ObjectId from all three variants.

**Impact**: Low in practice (sources are typically in other visible zones), but it's an incomplete pattern match that could silently drop objects in edge cases. Easy fix.

### OBSERVATION: `canonicalize()` for CastSpell assumes spell is still in hand — LOW

**Location**: `src/action/canonical.rs`, `canonicalize()` for `Action::CastSpell`.

`hand_instance_index(state, inst.owner, *object_id)` searches the player's hand for the spell being cast. If `canonicalize()` were ever called *after* `apply_action()` moves the spell to the stack, the index would default to 0 via `unwrap_or(0)`, producing an incorrect canonical form.

**Current safety**: All call sites in tests and the intended MCCFR flow canonicalize *before* applying the action. The full-game round-trip test confirms this works. However, this is an implicit contract.

**Recommendation**: Add a doc comment on `canonicalize()` stating it must be called on the game state *before* the action is applied (i.e., the state in which the action is legal).

### OBSERVATION: Two disambiguation schemes for battlefield objects

`battlefield_instance_index()` counts all permanents with a matching `card_def_id` regardless of controller. `CanonicalTarget::Object` uses `controller` + `instance_index` via `find_on_battlefield_by_controller()`. Both are internally consistent, but having two schemes adds cognitive load. This is a stylistic note, not a bug.

### OBSERVATION: `combat.blockers` values not added to visible objects

`visible_state()` iterates `combat.blockers` but only adds the *keys* (blocker ObjectIds). The *values* (attacker ObjectIds being blocked) are not explicitly added. This is fine because attackers are already added via `combat.attackers`, but it's fragile — if the attacker list and blocker map ever diverge, some objects could be missing. Consider adding both keys and values for robustness.

### POSITIVE: `canonicalize()`/`resolve()` correctly use `GameState.objects`, not `PlayerView.objects`

The canonical action module takes `&GameState` (not `&PlayerView`), ensuring lookups against the full object map. This is correct — canonicalization is an engine-level operation, not an information-set operation.

---

## Verdict

**APPROVE.** The fix commit successfully addresses both issues from the first review. All 46 tests pass with no warnings.

The one new finding of substance is the incomplete `StackSource` pattern match in `visible_state()` — it should handle `ActivatedAbility` and `TriggeredAbility` source IDs, not just `Spell`. This is low risk (sources are usually in other visible zones) and easy to fix, so it should not block merge. The `canonicalize()` timing contract should be documented.

Overall, this is a clean, well-tested implementation of all three Phase 0 deliverables. The interface is ready for Phase 1A and 1B to code against.
