# MTG GTO Simulator

A Magic: The Gathering game simulator built in Rust, designed for approximating Game Theory Optimal (GTO) play between two decks via high-volume simulation.

## Overview

True GTO for Magic is computationally intractable due to the game's enormous state space, but we can approximate win rates by simulating many games with heuristic or solver-derived strategies. This project provides:

- A rules engine that handles the core MTG turn structure, priority system, stack, combat, and state-based actions
- A trigger queue system with proper APNAP (Active Player, Non-Active Player) ordering
- Parallel game simulation via Rayon for fast iteration
- Pluggable strategy interface for building AI opponents and eventually CFR-based solvers

## Quick Start

```bash
cargo build --release
cargo run --release
```

This runs 1,000-game simulations across three matchup configurations (Greedy vs Greedy, Greedy vs Random, Random vs Greedy) using the built-in Mono-Red Aggro and Mono-Green Stompy decks.

```bash
cargo test
```

Runs the full test suite (unit + integration tests).

## Architecture

```
src/
  lib.rs              # Module declarations
  main.rs             # Entry point: runs sample simulations
  mana/mod.rs         # Color, ManaCost, ManaPool with payment logic
  card/mod.rs         # CardDef, CardInstance, effects, abilities, keywords
  card/sample.rs      # 23 sample cards, 2 prebuilt 60-card decks
  game/mod.rs         # GameState, phases, zones, stack, combat, CardDatabase
  action/mod.rs       # Action enum, legal_actions() enumeration
  rules/mod.rs        # Turn loop, stack resolution, combat, SBA, triggers
  strategy/mod.rs     # Strategy trait, RandomStrategy, GreedyStrategy
  simulation/mod.rs   # Parallel game runner via Rayon
tests/
  integration_test.rs # End-to-end tests for game setup, triggers, simulation
```

### Key Modules

**`mana`** -- Color enum (WUBRG), `ManaCost` with parsing (`"{2}{W}{U}"`), and `ManaPool` with `can_pay()`/`pay()` logic.

**`card`** -- Card definitions (`CardDef`) and per-game instances (`CardInstance`). Supports 18 keyword abilities, 10 trigger conditions, targeted effects (damage, destroy, bounce, buff, counter, draw, discard, tokens), and activated/triggered abilities.

**`game`** -- The complete `GameState` struct, cloneable for tree search. Tracks all zones (library, hand, battlefield, graveyard, exile, stack), player state, combat state, and pending triggers. Two-player only for now.

**`action`** -- Enumerates all legal actions for the priority player: land plays, spell casting with target enumeration, mana abilities, activated abilities, attacker/blocker declarations. Handles Hexproof/Shroud targeting restrictions and Flying/Fear/Intimidate/Menace blocking restrictions.

**`rules`** -- The core rules engine. Handles the full turn structure (13 phases), priority passing, stack resolution, combat damage (first strike, double strike, trample, deathtouch, lifelink), state-based actions, and the trigger queue system. Uses a two-phase read-then-write pattern to satisfy the Rust borrow checker.

**`strategy`** -- The `Strategy` trait is the interface for decision-making. `RandomStrategy` picks uniformly at random. `GreedyStrategy` uses heuristics (play lands, cast biggest spell, attack with all, block favorably).

**`simulation`** -- `simulate()` runs N games in parallel using Rayon and aggregates win rates, average turn count, and average actions per game.

## Rules Engine Coverage

### Implemented
- Full turn structure with 13 phases
- Priority system with consecutive pass tracking
- Stack: spells, activated abilities, triggered abilities
- Combat: declare attackers/blockers, first strike, double strike, trample, deathtouch, lifelink, vigilance
- Blocking restrictions: Flying/Reach, Fear, Intimidate, Menace (2+ blockers)
- Targeting restrictions: Hexproof, Shroud
- State-based actions: life check, lethal damage, toughness <= 0
- Trigger queue: ETB, dies, attacks, beginning of upkeep, end of turn
- APNAP ordering for simultaneous triggers
- Auto-tap lands for mana payment
- Mana abilities (tap for color, colorless, any, choice)
- `enters_tapped` support
- Indestructible, Defender, Haste, Flash

### Not Yet Implemented
- Token creation (effect is stubbed, `CreateToken` no-ops)
- Planeswalker loyalty and abilities
- Enchantment auras and equipment
- ETB watcher triggers (e.g., "whenever a creature enters the battlefield")
- Prowess
- Protection (partially modeled)
- Mulligan system
- Exile-based effects (Swords to Plowshares exiles but is modeled as destroy)
- Multi-color mana costs with proper color fixing
- Suspend, cascade, and other alternate casting

## Sample Cards (23)

**Lands:** Mountain, Forest, Plains, Island, Swamp

**Creatures:** Gray Ogre, Goblin Guide, Monastery Swiftspear, Shivan Dragon, Grizzly Bears, Llanowar Elves, Elvish Mystic, Kalonian Tusker, Leatherback Baloth, Savannah Lions, Serra Angel, Elvish Visionary, Blade Splicer, Siege-Gang Commander

**Instants/Sorceries:** Lightning Bolt, Shock, Lava Spike, Rift Bolt, Giant Growth, Swords to Plowshares, Counterspell

## Sample Decks

- **Mono-Red Aggro** (60 cards): Mountains, Goblin Guide, Monastery Swiftspear, Gray Ogre, Lightning Bolt, Shock, Lava Spike, Rift Bolt
- **Mono-Green Stompy** (60 cards): Forests, Llanowar Elves, Elvish Mystic, Grizzly Bears, Kalonian Tusker, Leatherback Baloth, Giant Growth

## Extending

### Adding Cards

Add new `CardDef` entries in `src/card/sample.rs` (or load from JSON/Scryfall API). Each card specifies types, keywords, mana cost, power/toughness, effects, and triggered/activated abilities.

### Adding Strategies

Implement the `Strategy` trait:

```rust
pub trait Strategy: Send + Sync {
    fn choose_action(&self, state: &GameState, player: PlayerIndex) -> Action;
    fn name(&self) -> &str;
}
```

### Future: GTO Solving

The `GameState` is fully cloneable and serializable, designed to support tree search algorithms like MCCFR (Monte Carlo Counterfactual Regret Minimization). The `Strategy` trait is the interface that a CFR solver would implement, mapping information sets to action distributions.

## Dependencies

- **serde** + **serde_json**: Serialization for game state snapshots
- **rand 0.8**: Random number generation for shuffling, random strategies
- **rayon 1.8**: Data parallelism for running simulations across CPU cores

## License

This project is for research and educational purposes.
