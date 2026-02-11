# PR #41 Review: Add interactive goldfish mode for manual play and troubleshooting

## Summary
Single commit adding `src/bin/interactive.rs` (586 lines) — a new binary that lets you play a goldfish game manually, picking every action for player 0 while the goldfish auto-passes. The stated purpose is troubleshooting solver convergence by replaying optimal lines by hand.

## Verdict
**Approve with minor suggestions.** The code compiles cleanly, all 272 existing tests pass, and clippy reports no warnings on this file. The feature is useful and well-scoped. The issues below are non-blocking.

---

## Issues

### 1. Goldfish uses both decks — game is always a mirror match (Medium)
**`interactive.rs:64`** — Both players receive the same deck and commander:
```rust
rules::setup_commander_game(&mut s, &deck, &deck, cmd, cmd);
```
And at line 69:
```rust
rules::setup_game(&mut s, &deck, &deck);
```
Since the goldfish does nothing, the second deck is irrelevant for gameplay, but it does mean the goldfish's library/hand consumes memory with a full copy. This is fine for a debugging tool, but worth a comment explaining the choice (or passing a minimal empty deck for player 1 if the engine supports it).

### 2. Undo stack can grow unbounded (Low)
**`interactive.rs:87,182`** — Every human action clones the full `GameState` onto `undo_stack` with no cap. For a debugging tool this is acceptable, but a long session with a complex board state could use significant memory. Consider capping at ~50 undo levels or documenting the trade-off.

### 3. `format_action_description` is just a delegation wrapper (Low)
**`interactive.rs:530-533`** — `format_action_description` calls `state.card_db()` and then forwards to `format_action_rich`. The `_player` parameter is unused. This could be inlined at the two call sites, or if the intent is to later differentiate by player, add a `// TODO` noting that.

### 4. Graveyard display order is non-deterministic (Low)
**`interactive.rs:392-405`** — Graveyard cards are aggregated into a `HashMap<String, u32>`, then iterated. `HashMap` iteration order is random, so the graveyard display will vary between runs. Use a `BTreeMap` or sort the entries for consistent output.

### 5. Trailing space in stack entry formatting (Nit)
**`interactive.rs:375,381`** — The ability/trigger format strings have a trailing space:
```rust
format!("Ability of {} (#{}) ", d.name, ability_index)
format!("Trigger of {} (#{}) ", d.name, ability_index)
```
This is cosmetic but will look odd next to the `(P{})` suffix printed at line 384.

### 6. `ActivateManaAbility` display uses raw Debug for the ability (Low)
**`interactive.rs:449-452`** — The mana ability is formatted with `{:?}` (Debug), which will likely produce a verbose struct dump. Consider extracting just the mana production (e.g., show `"adds {G}"` instead of the full `ManaAbility { ... }` debug output).

### 7. `format_card_type` uses non-exhaustive match without wildcard (Note)
**`interactive.rs:563-571`** — If a new `CardType` variant is added in the future, this match will fail to compile. That's actually a *good* thing for a binary like this (forces update), so no change needed — just noting that it's a deliberate design choice.

---

## Positive observations

- **Undo support** is a great quality-of-life feature for a debugging tool. The clone-based approach is simple and correct.
- **Action log** printed at the end directly addresses the stated use case (replaying lines the solver should find).
- **Comprehensive action formatting** — `format_action_rich` covers all 17 `Action` variants with human-readable descriptions including card names, costs, targets, and P/T stats.
- **Clean separation** — display, formatting, and game loop are well-separated into logical sections.
- **Auto-skip for trivial decisions** (lines 96-102) keeps the interaction focused on meaningful choices.
- **All deck types supported** — red, green, kinnan, brimaz, ashcoat with proper commander/standard branching.
- **No Cargo.toml changes needed** — auto-discovery works correctly.
