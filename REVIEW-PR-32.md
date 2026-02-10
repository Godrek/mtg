# PR #32 Review: Reshuffle Opening Hands + London Mulligan

## Summary

Two commits: (1) fix the 0% win rate bug by reshuffling between MCCFR iterations, (2) add London Mulligan support for Commander games. Both are well-motivated and correctly structured. Several issues below, ranging from a potential game-breaking bug to minor suggestions.

---

## Commit 1: Reshuffle Opening Hands (`6dd8ae2`)

### Verdict: Approve with comments

The diagnosis is correct — training on a single fixed shuffle means canonical actions only cover one set of 7 cards, making the learned strategy useless on any other draw. The fix is targeted and applied consistently across all three training entry points (single-threaded `train_goldfish_with_progress`, parallel shards, and `warm_start_from_greedy`).

### Issues

#### [Medium] `reshuffle_opening_hand` doesn't clear the battlefield, graveyard, exile, stack, or game event state

`reshuffle_opening_hand` resets hands, libraries, mana, and phase/turn, but doesn't touch:
- `state.battlefield` / `state.objects` (permanents from a prior iteration would persist)
- `state.graveyard` / `state.exile`
- `state.stack`
- `state.game_over` / `state.players[*].has_lost`
- `state.consecutive_passes`
- `state.players[*].life`

This is currently safe **only** because the function is called immediately after `initial_state.clone()` (before any game actions modify the state). But the function's doc comment says "Resets turn/phase/mana state so the game starts cleanly from Turn 1" — which is misleading. If anyone ever calls `reshuffle_opening_hand` on a mid-game state, it would produce a corrupt game.

**Suggestion:** Either:
1. Add a doc-comment caveat: "Must only be called on a freshly-cloned initial state (before any game actions)." — or
2. Also reset `game_over`, `has_lost`, life totals, battlefield, graveyard, exile, stack, and consecutive_passes for robustness.

#### [Low] Hardcoded player count = 2

```rust
draw_cards(state, 0, 7);
draw_cards(state, 1, 7);
```

The hand-drain loop correctly iterates `0..state.players.len()`, but redrawing is hardcoded to players 0 and 1. This is fine for the current 1v1 goldfish use case but inconsistent within the same function.

**Suggestion:** Use a loop:
```rust
for player in 0..state.players.len() {
    draw_cards(state, player, 7);
}
```

---

## Commit 2: London Mulligan (`ec5c478`)

### Verdict: Request changes (one bug, rest are suggestions)

The overall architecture is clean — `Phase::Mulligan` as a pre-game state machine with `advance_mulligan()` handling transitions is the right design. The canonical action support, info set integration, and strategy implementations are all well done.

### Issues

#### [Bug / High] `advance_phase` will panic if `PassPriority` is ever issued during Mulligan phase

`legal_actions_with` has a fallback `return vec![Action::PassPriority]` at the end of the Mulligan branch (line ~186 in the diff). If this path is ever hit, `handle_priority_pass` → `advance_phase` is called, which does:

```rust
fn advance_phase(state: &mut GameState) {
    let current_idx = Phase::TURN_ORDER
        .iter()
        .position(|&p| p == state.phase)
        .unwrap_or(0);  // ← returns 0 (Untap) since Mulligan isn't in TURN_ORDER
    ...
}
```

`Phase::Mulligan` is not in `TURN_ORDER`, so `position()` returns `None`, `.unwrap_or(0)` maps it to index 0 (Untap), and the game would skip the mulligan entirely and jump to Upkeep. This is a silent correctness bug.

While the comment says "should not reach here," defensive code should not silently break the game state.

**Fix:** Replace the `PassPriority` fallback with either `unreachable!("mulligan advance_mulligan should handle all transitions")` or add a guard in `advance_phase`/`handle_priority_pass` to not advance from Mulligan.

#### [Medium] `Phase::Mulligan` not in `TURN_ORDER` — could cause issues in other code paths

`Phase::TURN_ORDER` is a `[Phase; 13]` constant that doesn't include `Mulligan`. This is intentional (Mulligan isn't a turn phase), but any code that iterates `TURN_ORDER` to check "is this phase valid" or "what's the next phase" will silently ignore Mulligan. The `advance_phase` issue above is one consequence.

Consider adding a comment to `TURN_ORDER` noting that `Mulligan` is intentionally excluded as a pre-game phase.

#### [Medium] `MulliganMulligan` action doesn't check `MAX_MULLIGANS`

In `apply_action` for `Action::MulliganMulligan`:
```rust
state.players[player].mulligan_count += 1;
```

There's no guard against exceeding `MAX_MULLIGANS`. The guard exists in `legal_actions_with` (line ~170: `if ps.mulligan_count < MAX_MULLIGANS`), but `apply_action` trusts the caller. This is fine for the solver (which only picks from legal actions), but if `apply_action` is ever called directly with a crafted action, it would allow infinite mulligans.

**Suggestion:** Add a debug_assert or comment noting that the caller is responsible for checking legality.

#### [Medium] `GoldfishStrategy` always keeps — no bottom-card handling

`GoldfishStrategy::choose_action` returns `MulliganKeep` when `phase == Mulligan`, but since the goldfish opponent never mulligans, it will never need to bottom cards. This is correct for goldfish mode. However, if `GoldfishStrategy` is ever used as a real opponent strategy (not just goldfish), a mulligan-then-keep path would hit the bottom-card phase with no handling.

The current code would fall through to the "Handle mandatory actions" section and likely pick a wrong action.

**Suggestion:** Add a `MulliganBottomCard` handler in `GoldfishStrategy` (e.g., bottom the first card in hand) for defensive completeness, or add a comment noting this is goldfish-only.

#### [Low] `GreedyStrategy` mulligan heuristic hardcodes 7-card hand assumption

```rust
if (2..=5).contains(&land_count) || ps.mulligan_count >= 2 {
    return Action::MulliganKeep;
}
```

The 2-5 land range is reasonable for a 7-card hand but becomes increasingly permissive for 5-6 card hands after mulligans. After 1 mulligan (keeping 6 cards), keeping with 2 lands means 33% lands which is low. After 2 mulligans it auto-keeps regardless.

This is a minor heuristic concern, not a bug — the MCCFR solver will learn its own policy. But worth noting for baseline accuracy.

#### [Low] `MulliganBottomCard` canonical action includes `hand_index`

```rust
CanonicalAction::MulliganBottomCard {
    card_id: CardId,
    hand_index: usize,
}
```

Including `hand_index` in the canonical action means two identical cards at different hand positions produce different canonical actions. This is consistent with how other hand-based canonical actions work in this codebase (e.g., `PlayLand`), but for mulligan bottoming, the position in hand is typically irrelevant — only the card identity matters.

This inflates the info set space slightly. Not a bug, but worth considering whether `hand_index` is needed here.

#### [Low] `reshuffle_opening_hand` updated to be mulligan-aware — good

The second commit correctly updates `reshuffle_opening_hand` to reset `mulligan_count` and `mulligan_decided`, and routes Commander games through `Phase::Mulligan` while non-Commander goes straight to `Phase::Untap`. This is well-handled.

#### [Nit] `setup_commander_game` no longer calls `execute_phase_entry`

Before this PR, `setup_commander_game` ended with `execute_phase_entry(state)` which auto-advanced through Untap → Upkeep → Draw. Now it sets `phase = Phase::Mulligan` and returns. The mulligan phase entry in `execute_phase_entry` just sets `priority_player = active` — but `setup_commander_game` already sets both `active_player` and `priority_player` to 0 on the lines above. So not calling `execute_phase_entry` is fine, but it's a subtle behavioral change worth noting in the commit message.

---

## Testing

The integration test is updated to include `my_mulligan_count: 0` in the `InformationSet` constructor. However, there are no new tests for the mulligan mechanics themselves:

- No test for the keep → bottom N cards flow
- No test for mulligan → redraw → keep flow
- No test for `advance_mulligan` transitioning to Turn 1 after all players decide
- No test for `GoldfishStrategy` / `GreedyStrategy` mulligan behavior
- No test verifying `reshuffle_opening_hand` resets mulligan fields

**Recommendation:** Add at least a basic integration test that exercises the full mulligan flow (mulligan once → keep → bottom 1 card → verify hand size = 6 and game advances to Untap).

---

## Overall Assessment

| Aspect | Rating |
|--------|--------|
| Correctness | Good (1 bug in PassPriority/advance_phase interaction) |
| Architecture | Strong — Mulligan as a Phase with state machine is clean |
| MCCFR integration | Thorough — info sets, canonical actions, both abstractions updated |
| Strategy support | Good — GreedyStrategy heuristic is reasonable, MCCFR learns from training |
| Test coverage | Weak — no mulligan-specific tests |
| Code clarity | Good — comments and doc strings are helpful |

**Recommendation:** Fix the `PassPriority` bug in the Mulligan fallback path, add mulligan integration tests, and this is ready to merge.
