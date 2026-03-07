# MTG GTO Simulator

A Magic: The Gathering game simulator built in Rust, designed for approximating Game Theory Optimal (GTO) play between two decks via high-volume simulation. Supports both 60-card constructed and Commander (EDH) formats.

## Overview

True GTO for Magic is computationally intractable due to the game's enormous state space, but we can approximate win rates by simulating many games with heuristic or solver-derived strategies. This project provides:

- A rules engine covering the core MTG turn structure, priority system, stack, combat, state-based actions, triggers, replacement effects, and the layered continuous effects system (CR 613)
- MCTS (Monte Carlo Tree Search) and MCCFR (Monte Carlo Counterfactual Regret Minimization) solvers
- Automatic combo discovery that finds infinite loops via exhaustive DFS
- Commander format support with command zone, commander tax, and commander damage
- 274 card definitions across 6 prebuilt decks (2 constructed, 4 Commander)
- Parallel simulation via Rayon

## Quick Start

```bash
cargo build --release
cargo run --release                          # Demo: Greedy vs MCCFR
DECK=kinnan cargo run --release --bin mcts_goldfish   # MCTS goldfish optimization
DECK=kinnan cargo run --release --bin combo_list      # Discover all combos in a deck
cargo run --release --bin interactive        # Manual goldfish play
cargo test                                   # Full test suite (~312 tests)
```

## Binaries

### `combo_list` -- Combo Discovery

Finds all infinite combos in a deck using exhaustive ability activation search with DFS cycle detection.

```bash
DECK=kinnan cargo run --release --bin combo_list
DECK=ashcoat cargo run --release --bin combo_list
DECK=kinnan MAX_PIECES=2 cargo run --release --bin combo_list
```

Options (environment variables):
- `DECK` -- Deck name: `red`, `green`, `kinnan`, `brimaz`, `ashcoat` (default: `kinnan`)
- `MAX_PIECES` -- Max cards per combo (default: 3)
- `MAX_DEPTH` -- DFS search depth (default: 20)
- `MAX_MANA` -- Max startup mana to try (default: 10)
- `MAX_CREATURES` -- Max startup creature tokens to try (default: 5)

Output includes category classification (Infinite Mana, Infinite Tokens, Infinite Damage, etc.) and a summary of all combos grouped by category.

### `mcts_goldfish` -- MCTS Goldfish Optimizer

Runs goldfish (solitaire) games using Monte Carlo Tree Search to find optimal play sequences.

```bash
DECK=kinnan ITERATIONS=500 cargo run --release --bin mcts_goldfish
```

Options: `DECK`, `ITERATIONS`, `EXPLORE`, `DEPTH`, `THREADS`, `GAMES`, `CHECKPOINT`

### `interactive` -- Manual Goldfish

Play goldfish games manually with human-readable action selection.

```bash
DECK=kinnan cargo run --release --bin interactive
```

### `bfs_goldfish` -- Exhaustive Goldfish Solver

Finds the minimum-turn goldfish kill for a Commander deck using DFS with branch-and-bound. Exhaustively searches all play lines with aggressive pruning to prove optimal kill turns per shuffle.

```bash
SEEDS=20 MAX_TURN=10 TIMEOUT=30 VERBOSE=1 cargo run --release --bin bfs_goldfish
```

Options: `SEEDS`, `SEED_START`, `MAX_TURN`, `TIMEOUT` (seconds per seed), `MAX_STATES`, `VERBOSE`, `TRACE`

Pruning strategies: phase auto-pass (only Upkeep + PreCombatMain matter), dead card filter (counterspells, opponent-dependent cards), combat skip, fetch land handling, mana ability removal (engine auto-taps), combo macro instant-win detection. Safety limits prevent OOM and runaway searches.

### `commander_goldfish` -- Commander Goldfish

Goldfish simulation for Commander decks with command zone and commander tax.

## Architecture

```
src/
  lib.rs                # Module declarations
  main.rs               # Entry point: Greedy vs MCCFR demo

  mana/mod.rs           # Color, ManaCost, ManaPool with payment logic
  card/mod.rs           # CardDef, CardInstance, effects, abilities, keywords
  card/sample.rs        # 274 card definitions, 6 prebuilt decks
  deck_import.rs        # Deck importing from text files

  game/mod.rs           # GameState, phases, zones, stack, combat, CardDatabase
  action/mod.rs         # Action enum (14 variants), legal_actions() enumeration
  action/canonical.rs   # CanonicalAction for information set abstraction
  rules/mod.rs          # Turn loop, stack resolution, combat, SBA, triggers
  events/mod.rs         # GameEvent enum, EventBus, EventLog
  replacement/mod.rs    # Replacement effects (CR 614)
  layers/mod.rs         # Continuous effects layered system (CR 613)

  combo.rs              # ComboRegistry, ComboCategory, ComboEffect, detection & application
  combo_discovery.rs    # Exhaustive DFS combo discovery with cycle detection

  strategy/mod.rs       # Strategy trait + 5 implementations
  simulation/mod.rs     # Parallel game runner via Rayon
  info_set/mod.rs       # Information sets, abstraction, bucketing
  solver/mod.rs         # RegretTable, policy framework
  solver/mcts.rs        # MCTS with UCB1 selection
  solver/mccfr.rs       # MCCFR traversal, parallel training, checkpointing

  bin/combo_list.rs     # Combo discovery CLI
  bin/mcts_goldfish.rs  # MCTS goldfish optimizer
  bin/bfs_goldfish.rs   # Exhaustive DFS goldfish solver
  bin/interactive.rs    # Manual goldfish play
  bin/commander_goldfish.rs  # Commander goldfish simulation

tests/
  integration_test.rs   # 129 end-to-end rules engine tests
  mccfr_test.rs         # 36 MCCFR solver + training scenario tests
  deck_import_test.rs   # Deck import tests
```

### Key Modules

**`mana`** -- Color enum (WUBRG), `ManaCost` with parsing (`"{2}{W}{U}"`), and `ManaPool` with `can_pay()`/`pay()` logic.

**`card`** -- Card definitions (`CardDef`) and per-game instances (`CardInstance`). Supports 18 keyword abilities, 19 trigger conditions, targeted effects (damage, destroy, bounce, buff, counter, draw, discard, tokens), activated/triggered abilities, equipment, and static abilities.

**`game`** -- The complete `GameState` struct, cloneable for tree search. Tracks all zones (library, hand, battlefield, graveyard, exile, stack, command), player state, combat state, and pending triggers. Two-player. `Arc<CardDatabase>` for zero-cost cloning.

**`action`** -- Enumerates all legal actions for the priority player: land plays, spell casting with target enumeration, mana abilities, activated abilities, attacker/blocker declarations, combo macro-actions. Handles Hexproof/Shroud targeting restrictions and combat restrictions (Flying, Menace, etc.).

**`rules`** -- The core rules engine. Handles the full turn structure (13 phases), priority passing, stack resolution, combat damage (first strike, double strike, trample, deathtouch, lifelink), state-based actions (CR 704.5), the trigger queue system with APNAP ordering, and replacement effects.

**`combo`** / **`combo_discovery`** -- Automatic infinite combo detection. The discovery engine uses exhaustive DFS to find cycles in ability activation sequences. Discovered combos are classified into categories (`InfiniteMana`, `InfiniteTokens`, `InfiniteDamage`, `InfiniteLifeGain`, `InfiniteDraw`) and registered as macro-actions that the solver can activate as a single decision during main phases.

**`strategy`** -- Five strategy implementations:
- `RandomStrategy` -- Uniform random legal actions
- `GreedyStrategy` -- Heuristic-based (play lands, cast biggest spell, attack all)
- `GoldfishStrategy` -- Passive opponent for goldfish testing
- `McfrStrategy` -- MCCFR-trained with regret table lookup
- `AbstractedMcfrStrategy` -- MCCFR-trained with information set abstraction

**`solver`** -- MCTS with UCB1 selection for goldfish optimization. MCCFR for two-player GTO approximation with parallel training, checkpointing, and warm-starting from GreedyStrategy.

**`simulation`** -- `simulate()` runs N games in parallel using Rayon and aggregates win rates, average turn count, and average actions per game.

## Combo Discovery

The combo discovery system finds infinite loops among cards in a deck by:

1. **Generating all subsets** of cards up to `max_combo_pieces` (default 3)
2. **DFS exploration** of all ability activation sequences for each subset
3. **Cycle detection** -- when the state returns to a previous configuration with equal or greater resources, a combo is found
4. **Category classification** -- each combo is tagged with what it produces:

| Category | Example | Macro Effect |
|---|---|---|
| `InfiniteMana` | Basalt Monolith + Kinnan | Adds 1,000,000 colorless mana |
| `InfiniteTokens` | Marrow-Gnawer + Thornbite Staff | Creates 100 creature tokens |
| `InfiniteDamage` | Blood Artist + Marrow-Gnawer + Thornbite | Deals 1,000,000 damage (instant win) |
| `InfiniteLifeGain` | Ayara + Marrow-Gnawer + Thornbite | Gains 1,000,000 life |
| `InfiniteDraw` | (card draw per cycle) | Draws cards |

Combos can belong to multiple categories simultaneously. For example, Ayara + Marrow-Gnawer + Thornbite Staff is `[InfiniteTokens, InfiniteDamage, InfiniteLifeGain]`.

Discovered combos are registered as **macro-actions** (`Action::ActivateMacro`) that collapse an infinite loop into a single decision in the MCTS/MCCFR game tree. A proximity bonus incentivizes the solver to assemble combo pieces before the full combo is available.

## Rules Engine Coverage

### Implemented
- Full turn structure with 13 phases
- Priority system with consecutive pass tracking
- Stack: spells, activated abilities, triggered abilities
- Combat: declare attackers/blockers, first strike, double strike, trample, deathtouch, lifelink, vigilance
- Blocking restrictions: Flying/Reach, Fear, Intimidate, Menace (2+ blockers)
- Targeting restrictions: Hexproof, Shroud
- State-based actions: full CR 704.5 suite
- Trigger queue: 19 trigger conditions including ETB, dies, attacks, watcher triggers, cast triggers
- APNAP ordering for simultaneous triggers
- Replacement effects (CR 614)
- Continuous effects / layered system (CR 613)
- Token creation with ETB triggers
- Auto-tap lands for mana payment
- Mana abilities (tap for color, colorless, any, choice)
- `enters_tapped` support
- Indestructible, Defender, Haste, Flash, and 18 keyword abilities
- Equipment and static abilities
- Commander format: command zone, commander tax, commander damage

### Not Yet Implemented
- Planeswalker loyalty and abilities
- Enchantment auras
- Protection (partially modeled)
- Mulligan system
- Suspend, cascade, and other alternate casting
- Full multi-color mana fixing

## Sample Decks (6)

**60-Card Constructed:**
- **Mono-Red Aggro** -- Mountains, Goblin Guide, Monastery Swiftspear, Lightning Bolt, burn
- **Mono-Green Stompy** -- Forests, mana dorks, Kalonian Tusker, Leatherback Baloth

**Commander (EDH):**
- **Kinnan, Bonder Prodigy** -- Simic mana combo (Basalt Monolith + Kinnan infinite mana)
- **Brimaz, King of Oreskos** -- Mono-white tokens/aggro
- **Ashcoat of the Shadow Swarm** -- Mono-black rat tribal (Marrow-Gnawer + Thornbite Staff combos)
- **Thrun, Breaker of Silence** -- Mono-green voltron

## Dependencies

- **serde** + **serde_json**: Serialization for game state snapshots and checkpoints
- **rand 0.8**: Random number generation for shuffling, random strategies
- **rayon 1.8**: Data parallelism for simulations and MCCFR training
- **bincode**: Binary serialization for MCCFR checkpoints
- **ureq** (optional, `scryfall` feature): Scryfall API for card data import

## License

This project is for research and educational purposes.
