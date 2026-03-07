# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

MTG Commander Goldfish Simulator -- a Rust-based Magic: The Gathering Commander rules engine with an interactive TUI for goldfish (solitaire) play. Includes MCCFR/MCTS solvers for GTO approximation.

## Build & Test

```bash
cargo build                    # Compile
cargo test                     # All tests (unit + integration), must pass with 0 warnings
cargo test --test integration_test  # Integration tests only
cargo test test_name           # Run a single test by name
cargo test --features scryfall # Include Scryfall API tests
cargo run --release --features tui --bin tui  # TUI goldfish player
cargo run --release --bin goldfish -- --preset kinnan --trace  # CLI goldfish
```

CI runs `cargo build --verbose && cargo test --verbose` on push/PR to `mainline`.

## Architecture

Library + binary crate. The main branch is `mainline`. Commander-only -- all decks are 100-card singleton with a commander.

| Module | Purpose |
|--------|---------|
| `src/tui.rs` | Ratatui-based TUI: zone browser, action panel, card detail, undo |
| `src/mana/` | Color (WUBRG), ManaCost parsing (`"{2}{W}{U}"`), ManaPool payment |
| `src/card/mod.rs` | CardDef, CardInstance, Effect enum, KeywordAbility, abilities |
| `src/card/sample.rs` | 275+ card definitions, 4 Commander prebuilt decks |
| `src/card/catalog.rs` | Card catalog metadata & implementation status |
| `src/game/mod.rs` | GameState (fully cloneable via Arc\<CardDatabase\>), phases, zones, combat |
| `src/action/mod.rs` | Action enum, `legal_actions()`, target/blocker enumeration |
| `src/action/canonical.rs` | CanonicalAction for stable solver action IDs |
| `src/rules/mod.rs` | Turn loop (13 phases), priority, stack resolution, combat damage, SBA, triggers |
| `src/rules/setup.rs` | Game setup, `setup_commander_game()`, mulligans |
| `src/rules/effects.rs` | Effect resolution handlers (42+ effects) |
| `src/rules/triggers.rs` | Trigger system, APNAP ordering |
| `src/rules/combat.rs` | Combat damage resolution |
| `src/rules/sba.rs` | State-based actions (CR 704.5) |
| `src/rules/mana.rs` | Smart auto-tap, cost reduction |
| `src/rules/tokens.rs` | Token creation |
| `src/layers/` | CR 613 layered effects engine |
| `src/replacement/` | CR 614 replacement effects |
| `src/events/` | GameEvent enum, EventBus, EventLog |
| `src/strategy/mod.rs` | Strategy trait, Random/Greedy/MCCFR/Goldfish strategies |
| `src/simulation/mod.rs` | Parallel game runner via Rayon |
| `src/solver/` | MCCFR regret table, MCTS, training, checkpointing |
| `src/info_set/` | Information sets and abstraction for solver |
| `src/combo.rs` | Combo detection & macro-actions |
| `src/combo_discovery.rs` | Exhaustive DFS combo discovery |
| `src/deck_import.rs` | Deck file parsing (.txt from `decks/`) |
| `src/scryfall.rs` | Scryfall API integration (behind `scryfall` feature flag) |

## Critical Patterns

### Borrow Checker: Two-Phase Read-Write

`GameState` shares CardDatabase via `Arc` (O(1) clone). `state.card_db()` borrows immutably, so you **cannot mutate state while holding that reference**. Always clone what you need first:

```rust
// Phase 1: Read
let def = {
    let db = state.card_db();
    db.get(state.objects[&obj_id].card_def_id).unwrap().clone()
};
// Phase 2: Write
state.objects.get_mut(&obj_id).unwrap().damage_marked += def.power.unwrap_or(0) as u32;
```

For bulk operations, collect decisions into a `Vec` during reads, then apply in a write loop.

### Trigger Queue (APNAP ordering)

- `fire_triggers(state, condition, source_hint)` -- checks and flushes in one call
- `check_triggers(state, condition, source_hint)` -- queues only (for batching)
- `flush_triggers(state)` -- puts queued triggers on the stack
- Dies triggers: call `check_triggers` with `Some(dying_id)` AND with `None` (battlefield watchers), then flush once
- SBA fires dies triggers **after** the SBA loop stabilizes, not during

### Combat Phase

`DeclareAttackers` and `DeclareBlockers` actions directly advance the phase -- don't expect priority passing to handle phase transitions.

### Legal Actions

`legal_actions()` generates exponential attacker/blocker subsets -- capped at 10 attackers and 6 blockers per attacker.

## Extending the Codebase

**Adding cards:** Add constant ID in `src/card/sample.rs` `ids` module -> insert `CardDef` in `build_sample_db()`. Set `enters_tapped: false` unless specifically required.

**Adding effects:** Add `Effect` variant in `src/card/effects.rs` -> handle in `resolve_effect()` in `src/rules/effects.rs` -> add `TargetSpec` variant and handle in `targets_for_spec()` if targeting needed.

**Adding keywords:** Add to `KeywordAbility` enum in `src/card/keywords.rs` -> implement in relevant location (combat: `resolve_combat_damage()`/`can_block()`, targeting: `can_target_permanent()`, other: `legal_actions()`/`apply_action()`/`check_state_based_actions()`).

**Adding strategies:** Implement the `Strategy` trait (`choose_action` + `name`).

## Known Limitations

- Auto-tap is most-constrained-first heuristic, not optimal for all multi-color situations
- Copy effects (Clone, Fork) not yet implemented
- Morph/Manifest/Foretell not yet implemented (require face-down state)
- Swords to Plowshares modeled as destroy, not exile + life gain

## Snapshot Contract

Canonical state is always cloneable. Derived fields use `#[serde(skip)]`. CardDatabase shared via `Arc` for O(1) clone. This supports MCCFR tree search which clones GameState at every decision point.
