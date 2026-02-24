# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

MTG GTO Simulator — a Rust-based Magic: The Gathering game simulator for approximating Game Theory Optimal play via MCCFR (Monte Carlo Counterfactual Regret Minimization) and high-volume parallel simulation.

## Build & Test

```bash
cargo build                    # Compile
cargo test                     # All tests (unit + integration), must pass with 0 warnings
cargo test --test integration_test  # Integration tests only
cargo test test_name           # Run a single test by name
cargo test --features scryfall # Include Scryfall API tests
cargo run --release            # Run sample simulations (1K games)
cargo run --release --bin commander_goldfish  # Commander goldfish simulator
```

CI runs `cargo build --verbose && cargo test --verbose` on push/PR to `mainline`.

## Architecture

Library + binary crate. The main branch is `mainline`.

| Module | Purpose |
|--------|---------|
| `src/mana/` | Color (WUBRG), ManaCost parsing (`"{2}{W}{U}"`), ManaPool payment |
| `src/card/mod.rs` | CardDef, CardInstance, Effect enum, KeywordAbility, abilities |
| `src/card/sample.rs` | 111 card definitions, prebuilt decks (Mono-Red, Mono-Green, Commander) |
| `src/card/catalog.rs` | Card catalog metadata & implementation status |
| `src/game/mod.rs` | GameState (fully cloneable via Arc\<CardDatabase\>), phases, zones, combat |
| `src/action/mod.rs` | Action enum, `legal_actions()`, target/blocker enumeration |
| `src/action/canonical.rs` | CanonicalAction for stable solver action IDs |
| `src/rules/mod.rs` | Turn loop (13 phases), priority, stack resolution, combat damage, SBA, triggers |
| `src/strategy/mod.rs` | Strategy trait, Random/Greedy/MCCFR/Goldfish strategies |
| `src/simulation/mod.rs` | Parallel game runner via Rayon |
| `src/solver/` | MCCFR regret table, training, checkpointing |
| `src/info_set/` | Information sets and abstraction for solver |
| `src/layers/` | CR 613 layered effects engine |
| `src/replacement/` | CR 614 replacement effects |
| `src/combo.rs` | Combo detection & macro-actions |
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

- `fire_triggers(state, condition, source_hint)` — checks and flushes in one call
- `check_triggers(state, condition, source_hint)` — queues only (for batching)
- `flush_triggers(state)` — puts queued triggers on the stack
- Dies triggers: call `check_triggers` with `Some(dying_id)` AND with `None` (battlefield watchers), then flush once
- SBA fires dies triggers **after** the SBA loop stabilizes, not during

### Combat Phase

`DeclareAttackers` and `DeclareBlockers` actions directly advance the phase — don't expect priority passing to handle phase transitions.

### Legal Actions

`legal_actions()` generates exponential attacker/blocker subsets — capped at 10 attackers and 6 blockers per attacker.

## Extending the Codebase

**Adding cards:** Add constant ID in `src/card/sample.rs` `ids` module → insert `CardDef` in `build_sample_db()`. Set `enters_tapped: false` unless specifically required.

**Adding effects:** Add `Effect` variant in `src/card/mod.rs` → handle in `resolve_effect()` in `src/rules/mod.rs` → add `TargetSpec` variant and handle in `targets_for_spec()` if targeting needed.

**Adding keywords:** Add to `KeywordAbility` enum → implement in relevant location (combat: `resolve_combat_damage()`/`can_block()`, targeting: `can_target_permanent()`, other: `legal_actions()`/`apply_action()`/`check_state_based_actions()`).

**Adding strategies:** Implement the `Strategy` trait (`choose_action` + `name`).

## Known Limitations

- Token creation is stubbed (CreateToken no-ops)
- Only self-ETB triggers work ("when THIS enters"), not watcher triggers ("whenever a creature enters")
- Two players only (`opponent()` assumes `1 - player`)
- Auto-tap is greedy, no multi-color mana fixing
- Swords to Plowshares modeled as destroy, not exile + life gain

## Snapshot Contract

Canonical state is always cloneable. Derived fields use `#[serde(skip)]`. CardDatabase shared via `Arc` for O(1) clone. This supports MCCFR tree search which clones GameState at every decision point.
