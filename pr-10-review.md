# PR #10 Review: Surface trigger ordering as Action::OrderTriggers for MCCFR visibility

## Summary

This PR surfaces trigger ordering as an explicit `Action::OrderTriggers` decision point, replacing the previous silent FIFO ordering inside `flush_triggers()`. When a player controls multiple simultaneous triggered abilities, MTG rules require them to choose the stack ordering — this is a real strategic decision that MCCFR should observe and optimize over.

**Verdict: Approve with requested changes.** The core design is correct and well-motivated. The DeclareAttackers trigger path is implemented properly. However, the same fix is not applied consistently to other trigger sites, and there's a subtle bug where `fire_triggers()` silently ignores the new `flush_triggers()` return value.

---

## What the PR does well

1. **Correct motivation**: Trigger ordering is a genuine strategic decision in MTG. Surfacing it as an `Action` variant gives MCCFR a decision node it was previously blind to.

2. **Clean refactor of `flush_triggers()`**: The split into `check_triggers` (queue) → `flush_triggers` (push-to-stack with pause) → `push_trigger_to_stack` (helper) is a clean decomposition. The pause-and-resume pattern — where `flush_triggers` returns `false` to indicate it's waiting for an ordering decision — is a reasonable state-machine approach.

3. **DeclareAttackers path is correct**: The batch-check-then-flush pattern at `src/rules/mod.rs:168-183` is the right approach. Calling `check_triggers` for each attacker first, then a single `flush_triggers`, groups all simultaneous attack triggers for one ordering decision. The guard condition change (`&& state.combat.attackers.is_empty()`) in `legal_actions` correctly prevents re-offering attacker selection while waiting for trigger ordering.

4. **Permutation cap at 6 items**: Pragmatic choice. 6! = 720 is the upper bound on orderings, avoiding combinatorial explosion while covering the vast majority of real game situations.

5. **Tests cover the primary case**: `test_order_triggers_surfaced_for_multiple_simultaneous_triggers` validates the core flow end-to-end — pending triggers → OrderTriggers actions → apply ordering → triggers on stack → normal priority.

---

## Bugs

### 1. `fire_triggers()` silently ignores `flush_triggers()` return value (high severity)

`src/rules/mod.rs:628-631`:
```rust
pub fn fire_triggers(state: &mut GameState, condition: TriggerCondition, source_hint: Option<ObjectId>) {
    check_triggers(state, condition, source_hint);
    flush_triggers(state); // return value silently dropped
}
```

`fire_triggers()` is called at 4 trigger sites that haven't been updated to handle the paused state:

| Call site | Line | Trigger condition |
|-----------|------|-------------------|
| ETB resolution | 326 | `EntersBattlefield` |
| Upkeep entry | 815 | `BeginningOfUpkeep` |
| End step entry | 863 | `EndOfTurn` |

Each of these calls `fire_triggers()` and continues to the next statement, assuming all triggers were flushed. If a player controls >1 simultaneous trigger at any of these sites, the triggers remain in `pending_triggers` and the calling code proceeds with potentially inconsistent state.

**Why it still "works" (mostly):** The game loop calls `legal_actions()` after every action, and `legal_actions()` checks `pending_triggers` first. So the triggers are eventually caught. But:
- The code between `fire_triggers()` returning and the next `legal_actions()` call runs with unordered triggers in `pending_triggers` — SBA checks, phase transitions, etc. may interact with this partial state.
- The contract of `fire_triggers()` has changed (it used to guarantee all triggers were flushed) but callers haven't been updated.

**Fix:** Either (a) update `fire_triggers()` to return the `bool` and update all callers to handle `false`, or (b) apply the same batch-check-then-flush-then-guard pattern used for DeclareAttackers to all trigger sites.

### 2. Death trigger ordering not surfaced (medium severity)

`src/rules/mod.rs:398-403` (DestroyTarget effect) and `src/rules/mod.rs:745-751` (SBA creature deaths) call `check_triggers(Dies, ...)` then `flush_triggers(state)`, ignoring the return value:

```rust
// DestroyTarget (line 398-403)
for &id in &destroyable {
    check_triggers(state, TriggerCondition::Dies, Some(id));
    check_triggers(state, TriggerCondition::Dies, None);
}
if !destroyable.is_empty() {
    flush_triggers(state); // return value ignored
}
```

If a board wipe kills two creatures with "When ~ dies" abilities, this is exactly the same class of problem this PR fixes for attack triggers — the ordering is silently FIFO'd instead of surfaced to the player.

**Fix:** Apply the same pause-and-check pattern here. After `flush_triggers`, check whether triggers are still pending and let the game loop handle them rather than continuing.

### 3. Missing `#[must_use]` on `flush_triggers()` (low severity)

Since `flush_triggers()` now returns a meaningful `bool` (true = complete, false = paused), it should have `#[must_use]` to catch accidental ignoring of the return value:

```rust
#[must_use]
fn flush_triggers(state: &mut GameState) -> bool {
```

This would have caught bugs #1 and #2 at compile time.

---

## Design concerns

### Phase overloading during trigger ordering

When attack triggers need ordering, the phase remains `DeclareAttackers` with `combat.attackers` non-empty. This is handled by the guard `state.combat.attackers.is_empty()` in `legal_actions`. For other trigger sites (ETB during Upkeep, Deaths during any phase), the phase also stays unchanged, and `legal_actions`'s pending-trigger check catches it.

This works, but the state machine is now implicit: there's no explicit state distinguishing "we're in DeclareAttackers waiting to declare" from "we're in DeclareAttackers waiting to order triggers." Future changes to phase handling or `legal_actions` could easily miss this implicit invariant. A comment in the `Phase` enum or `GameState` documenting this pattern would help.

### `OrderTriggers` after ordering doesn't advance phase

After `apply_action(OrderTriggers)`, the code doesn't advance the phase — it relies on the next pass-priority cycle to resolve the stack and eventually advance. This is actually correct per MTG rules (players get priority after triggers are placed on the stack), but the implicit nature of this could be documented.

---

## Test gaps

- **No APNAP test**: No test covers the case where both players have simultaneous triggers (e.g., AP has 2 death triggers AND NAP has 2 death triggers from the same board wipe). The `flush_triggers` function handles this in theory (pause for AP ordering, then pause for NAP ordering), but it's untested.
- **No test for >6 triggers fallback**: The `generate_permutations` cap at 6 items returns a single FIFO ordering, but there's no test verifying this behavior.
- **No test for trigger ordering during ETB/Upkeep/EndStep**: All tests focus on the manually-queued triggers path. None exercise `fire_triggers()` with >1 simultaneous trigger through the natural game flow.

---

## Nits

- `src/simulation/mod.rs:125-127`: The new match arm for `OrderTriggers` in the verbose logging is fine, but consider matching it earlier with the other structured actions for consistency.
- The commit message claims "RandomStrategy picks randomly" — while true (it picks uniformly from `legal_actions` which now includes `OrderTriggers`), no code change was made to `RandomStrategy`. The statement is accurate but could mislead readers into looking for a RandomStrategy change.
