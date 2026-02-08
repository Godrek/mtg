# PR #10 Review: Surface trigger ordering as Action::OrderTriggers for MCCFR visibility

## Review round 2 (post-fix)

**Verdict: Approve.** All three bugs from the initial review have been fixed. The fix commit also integrates the cleanup discard changes from PR #8 cleanly. Two minor items remain (see below), neither blocking.

### Status of previously reported bugs

| Bug | Status | How it was fixed |
|-----|--------|------------------|
| `fire_triggers()` ignores `flush_triggers()` return | **Fixed** | `fire_triggers` now returns `bool` with `#[must_use]`. Upkeep and EndStep callers use the return value to conditionally set `priority_player`. ETB, DestroyTarget, and SBA death callers explicitly suppress with `let _ =` and document that the game loop will catch pending triggers. |
| Death trigger ordering not surfaced | **Fixed** | DestroyTarget (`src/rules/mod.rs:435-446`) and SBA deaths (`src/rules/mod.rs:790-803`) now batch-check before a single flush, with comments explaining the pause-and-resume pattern. |
| Missing `#[must_use]` on `flush_triggers()` | **Fixed** | Added to both `flush_triggers` (line 601) and `fire_triggers` (line 674). |

### Status of previously reported test gaps

| Gap | Status |
|-----|--------|
| No APNAP test | **Fixed** — `test_apnap_both_players_multiple_triggers` verifies AP orders first, NAP orders second, all 4 triggers end up on the stack in correct APNAP order. |
| No >6 triggers fallback test | **Fixed** — `test_more_than_six_triggers_fifo_fallback` verifies 7 triggers produce exactly 1 FIFO ordering. |
| No natural game flow trigger test | **Fixed** — `test_etb_multiple_triggers_through_natural_game_flow` exercises the spell-resolution ETB path (single trigger auto-flush case). |

### New changes: Cleanup discard (from PR #8 merge)

The fix commit integrates cleanup discard from the merged PR #8:

- `Action::Discard { object_id }` — new action variant for player-chosen cleanup discard
- `legal_actions` offers `Discard` actions (no `PassPriority`) when hand > 7 during Cleanup phase
- `apply_action(Discard)` validates phase/player/hand, calls `finalize_cleanup` when hand reaches 7
- `GreedyStrategy` picks first available discard action
- Tests: `test_cleanup_requires_discard_action`, `test_cleanup_allows_pass_at_seven`, `test_cleanup_multiple_discards`

This is a solid improvement over the previous `discard_random` approach for cleanup — it surfaces discard choice as a player decision that MCCFR can optimize.

---

## Remaining minor items (non-blocking)

### 1. `GreedyStrategy` discard heuristic picks arbitrarily

`src/strategy/mod.rs:126-131`:
```rust
// Priority 5: Cleanup discard if forced
for action in &actions {
    if let Action::Discard { .. } = action {
        return action.clone();
    }
}
```

This picks the first `Discard` action offered, which depends on hand ordering rather than card value. A smarter heuristic (e.g., discard lowest-CMC card, or prefer discarding lands when hand is land-heavy) would improve the greedy baseline. Not blocking since MCCFR explores all options anyway.

### 2. Flaky test: `test_random_vs_greedy_greedy_wins_more`

This test hit exactly 40.0% which fails the `> 40%` assertion. It passes on rerun (3/3 subsequent runs succeeded). The boundary condition (`>` vs `>=`) makes this inherently flaky with 200 games. Consider either `>= 40%`, a lower threshold, or a larger sample size.

---

## Initial review (round 1)

<details>
<summary>Click to expand original review</summary>

### Summary

This PR surfaces trigger ordering as an explicit `Action::OrderTriggers` decision point, replacing the previous silent FIFO ordering inside `flush_triggers()`. When a player controls multiple simultaneous triggered abilities, MTG rules require them to choose the stack ordering — this is a real strategic decision that MCCFR should observe and optimize over.

### What the PR does well

1. **Correct motivation**: Trigger ordering is a genuine strategic decision in MTG. Surfacing it as an `Action` variant gives MCCFR a decision node it was previously blind to.

2. **Clean refactor of `flush_triggers()`**: The split into `check_triggers` (queue) → `flush_triggers` (push-to-stack with pause) → `push_trigger_to_stack` (helper) is a clean decomposition. The pause-and-resume pattern — where `flush_triggers` returns `false` to indicate it's waiting for an ordering decision — is a reasonable state-machine approach.

3. **DeclareAttackers path is correct**: The batch-check-then-flush pattern at `src/rules/mod.rs:168-183` is the right approach. Calling `check_triggers` for each attacker first, then a single `flush_triggers`, groups all simultaneous attack triggers for one ordering decision. The guard condition change (`&& state.combat.attackers.is_empty()`) in `legal_actions` correctly prevents re-offering attacker selection while waiting for trigger ordering.

4. **Permutation cap at 6 items**: Pragmatic choice. 6! = 720 is the upper bound on orderings, avoiding combinatorial explosion while covering the vast majority of real game situations.

5. **Tests cover the primary case**: `test_order_triggers_surfaced_for_multiple_simultaneous_triggers` validates the core flow end-to-end.

### Bugs (all now fixed in round 2)

1. `fire_triggers()` silently ignores `flush_triggers()` return value (high severity)
2. Death trigger ordering not surfaced (medium severity)
3. Missing `#[must_use]` on `flush_triggers()` (low severity)

### Design concerns

- **Phase overloading during trigger ordering**: The state machine is implicit — no explicit state distinguishes "waiting to declare attackers" from "waiting to order triggers." The guard `state.combat.attackers.is_empty()` in `legal_actions` handles the DeclareAttackers case. For other phases, `legal_actions`'s pending-trigger check catches it. This works but is fragile.

- **`OrderTriggers` doesn't advance phase**: After ordering, the code relies on the pass-priority cycle to resolve the stack and advance. This is correct per MTG rules but implicit.

</details>
