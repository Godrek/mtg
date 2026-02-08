# PR #14 Review: Track A — Phase 1A Rules Engine Foundations

**Branch**: `claude/implement-track-a-SQGvj`
**Commit**: `a206205` — "Implement Phase 1A: Rules Engine Foundations (Track A)"
**Reviewed against**: `docs/CONSOLIDATED_STRATEGY.md`, Phase 1A (sections 1A.1, 1A.2, 1A.3)

---

## Summary

This PR implements the three Phase 1A deliverables from the consolidated strategy:
- **1A.1**: SBA/Trigger Recurrence Loop (CR 704.3)
- **1A.2**: Event System Skeleton
- **1A.3**: Replacement Effect Framework

Total: +1,534 lines / -68 lines across 9 files. All 68 tests pass, 0 failures, 0 warnings.

---

## 1A.1 — SBA/Trigger Recurrence Loop

### Strategy Requirement

> Implement the CR 704.3 loop: perform all SBAs, check and queue triggers, break only when no SBAs performed AND no triggers queued.

### What Was Implemented

The `check_state_based_actions()` function in `src/rules/mod.rs` was restructured from a flat single-pass SBA loop into a nested two-level loop:

- **Inner loop** (`src/rules/mod.rs:~850-905`): Repeatedly performs SBAs (player life check, creature lethal damage) until no more SBAs apply in a single pass.
- **Outer loop** (`src/rules/mod.rs:~845-940`): After the inner SBA loop stabilizes, queues death triggers, checks the CR 704.3 exit condition (`!any_sba && !triggers_queued`), flushes triggers to the stack, then re-iterates.
- **OrderTriggers pause**: If `flush_triggers` returns `false` (player has >1 simultaneous trigger needing ordering), the function returns early. The game loop presents `Action::OrderTriggers`, and after the player orders, `check_state_based_actions` is called again to resume the outer loop.

### Assessment: MEETS SPECIFICATION

The loop structure matches the strategy's pseudocode exactly. The interleaving of SBA checks and trigger queuing is correct per CR 704.3.

### Issues

1. **Duplicate trigger firing risk** (`src/rules/mod.rs:895-899`): For each dead creature, the code calls:
   ```rust
   check_triggers(state, TriggerCondition::Dies, Some(obj_id)); // dead creature's own trigger
   check_triggers(state, TriggerCondition::Dies, None);          // battlefield watchers
   ```
   The `None` variant scans ALL battlefield permanents for `Dies` triggers. If there are N deaths, every battlefield permanent with a `Dies` trigger gets checked N times. This is correct for "whenever a creature dies" watchers (like Blood Artist), but the current `TriggerCondition::Dies` doesn't distinguish between "when THIS creature dies" and "when ANY creature dies." A surviving creature with "when ~ dies" on the battlefield would have its trigger erroneously queued once per death event. This is a **pre-existing design limitation** in the trigger system, not introduced by this PR, but the new SBA loop amplifies it since it processes deaths iteratively.

   **Recommendation**: Track this as a Phase 2A item. The fix is to add a `TriggerCondition::AnyCreatureDies` variant or add a `self_only: bool` field to `TriggeredAbility`.

2. **No recursive SBA test**: The acceptance criterion explicitly calls for "creature with 'when ~ dies, deal 2 damage to each player' killing another creature at 2 toughness." The test `test_sba_recurrence_dies_trigger_deals_damage_to_players` validates the dies-trigger-deals-damage path, but does not set up a scenario where the trigger's damage causes a SECOND creature death via SBA, which would test the actual recurrence. The test verifies the loop structure works but doesn't exercise the full cascading scenario from the acceptance criteria.

   **Recommendation**: Add a test with two Fiery Conclusion Elementals where the first's death trigger deals lethal damage to the second, causing a second SBA death and a second trigger.

---

## 1A.2 — Event System Skeleton

### Strategy Requirement

> Add `src/events/mod.rs` with `GameEvent` enum. Events are transient, not stateful. Event handlers registered at engine level, not per-game-state. Preserves cheap Clone.

### What Was Implemented

**`src/events/mod.rs`** (321 lines): New module with:
- `GameEvent` enum with 10 variants: `DamageDealt`, `ZoneChange`, `LifeChanged`, `SpellCast`, `AbilityTriggered`, `CounterChanged`, `Tapped`, `Untapped`, `CardDrawn`, `TurnStarted`
- `EventBus` struct: function-pointer table (`Vec<EventHandler>` where `type EventHandler = fn(&mut GameState, &GameEvent)`)
- `EventLog` struct: log-only collector for testing/debugging
- `Zone` enum: event-specific zone type (avoids circular dependency with `card::ZoneType`)
- Unit tests for bus creation, subscribe/emit, event log, zone conversion

**Event emission sites** in `src/rules/mod.rs`:
- `apply_action` → `CastSpell`: emits `SpellCast` + `ZoneChange` (hand→stack)
- `resolve_effect` → `DealDamage`: emits `DamageDealt` + `LifeChanged`
- `resolve_effect` → `GainLife`: emits `LifeChanged`
- `resolve_effect` → `LoseLife`: emits `LifeChanged`
- `push_trigger_to_stack`: emits `AbilityTriggered`
- `next_turn`: emits `TurnStarted`
- `draw_cards`: emits `CardDrawn` + `ZoneChange` (library→hand)
- `resolve_combat_damage`: emits `DamageDealt` + `LifeChanged` for combat damage
- `GameState::move_object`: emits `ZoneChange`

**`GameState.pending_events`** (`src/game/mod.rs`):
- Added `#[serde(skip)] pub pending_events: Vec<GameEvent>` field
- `emit_event()` and `drain_events()` methods
- Events accumulate during `apply_action()`, drained by external code between actions

### Assessment: MEETS SPECIFICATION WITH DESIGN DIVERGENCE

The implementation matches the strategy's requirements with one notable design choice:

**Design divergence: `pending_events` lives INSIDE `GameState`**, not outside as specified. The strategy says:

> The event system is a notification mechanism within `apply_action()`. It does not store subscribers or callbacks in `GameState`. This preserves cheap `Clone`.

The `EventBus` (subscribers) is correctly kept outside `GameState`. However, `pending_events: Vec<GameEvent>` was added to `GameState` itself. While marked `#[serde(skip)]` and documented as transient, this means:
- `GameState::clone()` copies the pending events vec (though it's O(0) in practice since events are drained between actions)
- The event data co-travels with state during MCCFR traversal

This is a pragmatic choice — emitting events deep inside `resolve_effect()` and `resolve_combat_damage()` is much simpler when `GameState` carries the accumulator. The alternative (threading an `&mut EventBus` through every function call) would require significant signature changes.

**Acceptable for Phase 1A.** The `#[serde(skip)]` annotation and drain-between-actions pattern keep the cost O(0) in the MCCFR hot path.

### Issues

1. **`source: 0` placeholder in DamageDealt events** (`src/rules/mod.rs:~440,458,1310,1320`): Combat damage and effect damage emit `DamageDealt { source: 0, ... }`. The comment says "source tracking deferred to Phase 2A." This means events are structurally complete but semantically incomplete for damage source tracking. Downstream handlers can't correlate damage to its source.

   **Recommendation**: Acceptable for Phase 1A skeleton. Must be resolved before Phase 2A cards that care about damage sources (e.g., protection, damage prevention shields).

2. **Duplicate `ZoneChange` events for spells**: When `CastSpell` is processed, the code manually emits `ZoneChange { from: Hand, to: Stack }` at `src/rules/mod.rs:91-95`, but then calls `state.players[player].hand.retain(...)` without going through `move_object`. So spells cast from hand get their `ZoneChange` emitted manually, while other zone transitions use `move_object` (which also emits `ZoneChange`). This dual path could cause issues if `move_object` is later called for spell casting — it would double-emit.

   **Recommendation**: Consider routing all zone transitions through `move_object` for consistency, or at minimum add a comment noting this is intentional.

3. **`Zone` enum duplicates `ZoneType`**: The `events::Zone` enum is a copy of `card::ZoneType` with a `From<ZoneType>` impl. The justification is avoiding circular dependencies, but both types are in the same crate. This adds maintenance burden (any new zone must be added in both places).

   **Recommendation**: Consider making `GameEvent` use `card::ZoneType` directly. If the concern is about the `events` module depending on `card`, that dependency already exists (it imports `ObjectId` from `card`).

---

## 1A.3 — Replacement Effect Framework

### Strategy Requirement

> Add `Action::ChooseReplacementOrder` and corresponding infrastructure for when multiple replacement effects could apply to the same event. This is a player decision that MCCFR must see.

### What Was Implemented

**`src/replacement/mod.rs`** (205 lines): New module with:
- `ReplacementEventKind` enum: 7 variants (EntersBattlefield, DamageDealt, CardDraw, WouldDie, LifeGain, LifeLoss, CounterPlacement)
- `ReplacementAction` enum: 5 variants (Prevent, RedirectToZone, ModifyAmount, EntersModified, Custom)
- `ReplacementEffect` struct: source_id, controller, applies_to, action, is_self_replacement, description
- `PendingReplacementChoice` struct: chooser, applicable_effects, event_kind
- `find_applicable_replacements()`: separates self-replacement effects from player-choice effects per CR 614.16a
- Serde serialization support
- Unit tests for finding applicable replacements and serde round-trip

**`Action::ChooseReplacementOrder`** (`src/action/mod.rs:72-82`):
- New action variant: `ChooseReplacementOrder { ordering: Vec<(ObjectId, usize)> }`
- Display impl, canonical mapping, and round-trip in `src/action/canonical.rs`

**`apply_action` handler** (`src/rules/mod.rs:290-301`):
- Stub implementation: `let _ = ordering; state.consecutive_passes = 0;`
- Comment: "Phase 2A will wire this into the actual replacement application logic."

### Assessment: MEETS SPECIFICATION — SKELETON ONLY

The framework establishes the correct data structures and the MCCFR-visible action variant. The self-replacement vs. player-choice distinction (CR 614.16a) is correctly modeled. However, this is purely declarative infrastructure — no replacement effects are actually applied anywhere in the rules engine yet.

### Issues

1. **No integration with game rules**: `find_applicable_replacements()` exists but is never called from `apply_action()`, `resolve_effect()`, or `check_state_based_actions()`. The `apply_action` handler for `ChooseReplacementOrder` is a no-op stub. This means replacement effects have zero runtime impact.

   **Recommendation**: Acceptable for Phase 1A. The strategy explicitly says "This extends `Action` and `CanonicalAction` — coordinate with Phase 0's canonical action contract." The infrastructure-first approach is correct. Phase 2A must wire this in.

2. **`GreedyStrategy` doesn't handle `ChooseReplacementOrder`**: The strategy module (`src/strategy/mod.rs`) was not updated. If `legal_actions()` ever returns `ChooseReplacementOrder`, the `GreedyStrategy` would fall through to `Action::PassPriority`, which may not be a legal action at that point. Since `legal_actions()` never generates this variant yet, this is not a runtime bug, but it's a landmine.

   **Recommendation**: Add a handler in `GreedyStrategy` (similar to `OrderTriggers` — pick the first ordering) before Phase 2A wires in the replacement logic.

3. **`ReplacementEffect` stores `source_id: ObjectId`**: This means replacement effects are tied to specific object instances. If the source permanent leaves and re-enters the battlefield (getting a new ObjectId), the replacement effect becomes stale. This is fine for the current design (effects would be rebuilt from the board state) but should be documented.

---

## Phase 0 Interface Compatibility

### Canonical Action Round-Trip

`CanonicalAction::ChooseReplacementOrder` is correctly added to the canonical mapping. The `canonicalize()` and `resolve()` functions handle the new variant using the same `battlefield_instance_index` / `find_on_battlefield_by_index` pattern as `OrderTriggers`. The integration test `test_replacement_order_canonical_roundtrip` verifies the round-trip.

### PlayerView / Snapshot Contract

No changes to `PlayerView` or the snapshot contract. The `pending_events` field is `#[serde(skip)]`, consistent with Phase 0's rule that "any field added to `GameState` that is derivable from other fields must be marked `#[serde(skip)]`."

---

## Acceptance Criteria Checklist

| Criterion | Status | Notes |
|---|---|---|
| SBA loop correctly handles recursive triggers | PARTIAL | Loop structure is correct. Test validates basic dies→trigger→damage flow but doesn't test cascading (trigger-caused death → re-SBA → second trigger). |
| Events fire for all zone changes | YES | `move_object` emits `ZoneChange`, `draw_cards` emits `ZoneChange`, `CastSpell` emits `ZoneChange`. |
| Events fire for damage | YES | `resolve_effect` and `resolve_combat_damage` emit `DamageDealt`. Source is placeholder `0`. |
| Events fire for life changes | YES | All life mutation sites emit `LifeChanged`. |
| Event handlers do not affect `GameState::clone()` cost | YES | `EventBus` is outside `GameState`. `pending_events` is inside but O(0) between actions. |
| All existing tests pass | YES | 68 tests pass, 0 failures, 0 warnings. 18 new unit tests + 16 new integration tests added. |

---

## Files Changed

| File | Lines | What |
|---|---|---|
| `src/events/mod.rs` | +321 | New: GameEvent enum, EventBus, EventLog |
| `src/replacement/mod.rs` | +205 | New: Replacement effect framework |
| `src/rules/mod.rs` | +291/-68 | SBA recurrence loop, event emissions |
| `tests/integration_test.rs` | +664 | 16 new integration tests |
| `src/game/mod.rs` | +39/-1 | pending_events field, emit/drain methods, ZoneChange in move_object |
| `src/card/sample.rs` | +38 | Fiery Conclusion Elemental test card |
| `src/action/canonical.rs` | +27 | ChooseReplacementOrder canonical mapping |
| `src/action/mod.rs` | +12 | ChooseReplacementOrder action variant |
| `src/lib.rs` | +2 | Module declarations |

---

## Recommendations Summary

### Must-Fix Before Merge (P0)

None. The PR is structurally sound and all tests pass.

### Should-Fix (P1)

1. **Add cascading SBA test**: Write a test where a dies trigger kills another creature, triggering a second round of SBAs and triggers. This is the explicit acceptance criterion scenario.
2. **Add `ChooseReplacementOrder` handling to `GreedyStrategy`**: Defensive measure before Phase 2A wires in replacement effects.

### Nice-to-Have (P2)

3. **Unify `Zone` and `ZoneType`**: Eliminate the duplicate enum in `events`.
4. **Route all zone transitions through `move_object`**: Avoid the dual-path issue for spell casting zone changes.
5. **Track the `Dies` trigger disambiguation**: File an issue to distinguish "when THIS creature dies" vs "when ANY creature dies" before the card pool grows in Phase 2A.
6. **Fill in `source: 0` placeholders**: Track as a Phase 2A prerequisite for damage-source-aware cards.

---

## Verdict

**Approve with minor comments.** The PR delivers all three Phase 1A deliverables as specified in the consolidated strategy. The SBA loop is correctly structured per CR 704.3, the event system skeleton preserves the cheap-clone invariant, and the replacement effect framework establishes the right MCCFR-visible interface. The only gap is a test coverage shortfall on the cascading SBA scenario, which should be addressed before or shortly after merge.
