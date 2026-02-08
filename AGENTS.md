# AGENTS.md

Instructions for AI agents working on this codebase.

## Project Overview

This is an MTG (Magic: The Gathering) game simulator written in Rust, designed for approximating GTO (Game Theory Optimal) play via large-scale simulation. The codebase implements a rules engine, card definitions, strategy interface, and parallel simulation harness.

## Tech Stack

- **Language:** Rust (edition 2021)
- **Dependencies:** serde + serde_json (serialization), rand 0.8 (RNG), rayon 1.8 (parallelism)
- **Structure:** Binary + library crate (`src/main.rs` + `src/lib.rs`)

## Build & Test

```bash
cargo build          # Compile
cargo test           # Run all tests (unit + integration)
cargo run --release  # Run sample simulations
```

All tests must pass with 0 warnings before committing. Run `cargo test` after every change.

## Architecture

| Module | Path | Purpose |
|--------|------|---------|
| `mana` | `src/mana/mod.rs` | Color, ManaCost, ManaPool |
| `card` | `src/card/mod.rs` | CardDef, CardInstance, effects, abilities, keywords |
| `card::sample` | `src/card/sample.rs` | Sample card definitions and prebuilt decks |
| `game` | `src/game/mod.rs` | GameState, phases, zones, stack, combat, CardDatabase |
| `action` | `src/action/mod.rs` | Action enum, `legal_actions()` enumeration |
| `rules` | `src/rules/mod.rs` | Turn loop, stack resolution, combat, SBA, triggers |
| `strategy` | `src/strategy/mod.rs` | Strategy trait, RandomStrategy, GreedyStrategy |
| `simulation` | `src/simulation/mod.rs` | Parallel game runner via Rayon |
| tests | `tests/integration_test.rs` | End-to-end integration tests |

## Critical Patterns

### Borrow Checker: Two-Phase Read-Write

The `GameState` owns both the `CardDatabase` (via `card_db: Option<CardDatabase>`) and mutable game data. Calling `state.card_db()` borrows `state` immutably, so you cannot mutate `state` while holding that reference.

**Always separate reads from writes:**

```rust
// Phase 1: Read (immutable borrow)
let def = {
    let db = state.card_db();
    let inst = &state.objects[&obj_id];
    db.get(inst.card_def_id).unwrap().clone()
};

// Phase 2: Write (mutable borrow)
if let Some(inst) = state.objects.get_mut(&obj_id) {
    inst.damage_marked += def.power.unwrap_or(0) as u32;
}
```

For bulk operations (combat damage, auto-tap lands), collect decisions into a `Vec` during the read phase, then apply them in a separate write loop.

### Combat Phase Flow

`DeclareAttackers` and `DeclareBlockers` actions directly advance the phase (not via priority pass cycle). This prevents infinite loops where the phase never advances.

### Trigger Queue System

Triggers are queued as `PendingTrigger` entries, then flushed to the stack in APNAP order (active player's triggers first). Use:

- `fire_triggers(state, condition, source_hint)` -- checks and flushes in one call
- `check_triggers(state, condition, source_hint)` -- only queues (for batching multiple events)
- `flush_triggers(state)` -- puts queued triggers on the stack

When `source_hint` is `Some(id)`, only that object is checked. When `None`, all battlefield permanents are checked.

For dies triggers specifically, call both `check_triggers(state, Dies, Some(dying_id))` (for the creature's own triggers) and `check_triggers(state, Dies, None)` (for battlefield watchers), then flush once.

### State-Based Actions

SBA run in a loop until no more apply. Dies triggers are fired *after* the SBA loop stabilizes, not during it. This matches MTG comprehensive rules.

## Adding Cards

Add new `CardDef` entries to `src/card/sample.rs`:

1. Add a constant ID in the `ids` module
2. Insert the `CardDef` in `build_sample_db()`
3. All fields are required -- use existing cards as templates
4. Set `enters_tapped: false` unless the card specifically enters tapped

## Adding Effects

To add a new effect type:

1. Add a variant to the `Effect` enum in `src/card/mod.rs`
2. Handle it in `resolve_effect()` in `src/rules/mod.rs`
3. If it requires targeting, add a `TargetSpec` variant and handle in `targets_for_spec()` in `src/action/mod.rs`

## Adding Keywords

1. Add to `KeywordAbility` enum in `src/card/mod.rs`
2. Implement the game rule in the relevant location:
   - Combat keywords: `resolve_combat_damage()` or `can_block()` in `src/action/mod.rs`
   - Targeting keywords: `can_target_permanent()` in `src/action/mod.rs`
   - Other: `legal_actions()`, `apply_action()`, or `check_state_based_actions()`

## Known Limitations

- **Token creation is stubbed**: `CreateToken` effect is a no-op. Blade Splicer and Siege-Gang Commander triggers fire but produce no tokens.
- **ETB watcher triggers not supported**: Only self-ETB ("when THIS enters") works. "Whenever A creature enters" (like Soul Warden) requires separate handling.
- **Two players only**: `opponent()` assumes `1 - player`.
- **Simplified mana**: Auto-tap is greedy, not optimal. No multi-color fixing.
- **No mulligan**: Both players keep 7-card opening hands.
- **Swords to Plowshares**: Modeled as destroy, not exile + life gain.

## Testing Guidelines

- Write integration tests in `tests/integration_test.rs`
- For testing specific game interactions, manually set up a `GameState` with cards in specific zones and phases, then call `apply_action()` and assert on the result
- See `test_etb_trigger_elvish_visionary` for the pattern
- Simulation tests should use at least 100 games for statistical stability

## Common Pitfalls

1. **Don't hold `state.card_db()` across mutations** -- clone what you need first
2. **Don't forget `enters_tapped: false`** when adding new cards
3. **Combat actions advance phases directly** -- don't expect priority pass to handle it
4. **`legal_actions()` generates exponential attacker/blocker subsets** -- capped at 10 attackers and 6 blockers per attacker to keep it manageable
5. **SBA fires dies triggers after loop stabilizes** -- not during each iteration
