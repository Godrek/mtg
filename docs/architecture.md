# Architecture

MTG Commander Goldfish Simulator -- library + binary crate. The main branch is `mainline`.

## Source Tree

```
src/
  lib.rs                    # Module declarations
  main.rs                   # Default binary entry point

  tui.rs                    # Ratatui TUI: zone browser, action panel, card detail, undo, SVG export

  mana/mod.rs               # Color, ManaCost, ManaPool with payment logic
  card/mod.rs               # CardDef, CardInstance, types
  card/effects.rs           # Effect, TargetSpec, DynamicValue, Condition, TokenDef
  card/keywords.rs          # KeywordAbility enum (34+ keywords)
  card/database.rs          # CardDatabase (HashMap<CardId, CardDef>)
  card/sample.rs            # 275+ card definitions, 4 Commander prebuilt decks
  card/catalog.rs           # CoverageLevel, DeckCoverage, implementation tracking
  deck_import.rs            # Deck importing from text files

  game/mod.rs               # GameState, PlayerState, phases, zones, stack, combat, CardDatabase
  action/mod.rs             # Action enum (14 variants), legal_actions() enumeration
  action/canonical.rs       # CanonicalAction for information set abstraction

  rules/mod.rs              # apply_action() dispatch, draw_cards(), discard_random()
  rules/phases.rs           # Phase transitions, priority passing
  rules/combat.rs           # Combat damage resolution
  rules/resolution.rs       # Stack resolution (spells, activated, triggered abilities)
  rules/effects.rs          # Effect resolution handlers (42+ effects)
  rules/triggers.rs         # Trigger system, APNAP ordering
  rules/sba.rs              # State-based actions (CR 704.5)
  rules/setup.rs            # Game setup, setup_commander_game(), mulligans
  rules/mana.rs             # Smart auto-tap, cost reduction/increase
  rules/tokens.rs           # Token creation, DynamicContext

  events/mod.rs             # GameEvent enum, EventBus, EventLog
  layers/mod.rs             # CR 613 layered continuous effects engine
  replacement/mod.rs        # CR 614 replacement effects

  combo.rs                  # ComboRegistry, ComboCategory, detection & macro-action application
  combo_discovery.rs        # Exhaustive DFS combo discovery with cycle detection

  strategy/mod.rs           # Strategy trait + 5 implementations
  simulation/mod.rs         # Parallel game runner via Rayon
  info_set/mod.rs           # InformationSet, InfoSetAbstraction, BucketedAbstraction
  solver/mod.rs             # RegretTable, policy framework
  solver/mcts.rs            # MCTS with UCB1 selection
  solver/mccfr.rs           # MCCFR traversal, parallel training, checkpointing

  scryfall.rs               # Scryfall API integration (behind `scryfall` feature flag)

  bin/tui.rs                # TUI goldfish player
  bin/tui_snapshot.rs       # Headless SVG screenshot generator
  bin/goldfish.rs           # Unified goldfish simulator CLI
  bin/interactive.rs        # Manual goldfish play (text-based)
  bin/commander_goldfish.rs # Commander goldfish batch simulation
  bin/bfs_goldfish.rs       # Exhaustive DFS goldfish solver
  bin/mcts_goldfish.rs      # MCTS goldfish optimizer
  bin/combo_list.rs         # Combo discovery CLI

tests/
  integration_test.rs       # End-to-end rules engine tests
  mccfr_test.rs             # MCCFR solver + training scenario tests
  rules_test.rs             # Targeted rule tests
  card_regression_test.rs   # Card-specific regression tests
  goldfish_test.rs          # Goldfish completion and kill speed tests
  deterministic_test.rs     # Deterministic replay tests
  deck_import_test.rs       # Deck import tests
  benchmark_test.rs         # Throughput benchmarks

decks/                      # Commander deck files (.txt, one card per line)
```

## Key Design Decisions

### GameState Cloneability

`GameState` is fully cloneable for tree search (MCCFR/MCTS). The `CardDatabase` is shared via `Arc` for O(1) clone cost. Derived/transient fields use `#[serde(skip)]` and are excluded from serialization.

### Two-Phase Read-Write

Rust's borrow checker requires separating reads from writes when accessing `state.card_db()` (which borrows `state` immutably) and mutating `state`. Always clone what you need in a read phase, then apply in a write phase.

### Commander-Only

All game setup uses Commander rules: 100-card singleton decks, command zone, commander tax, commander damage, color identity validation. The `setup_commander_game()` path is the standard entry point.

### Data-Driven Cards

Card behaviors are defined as composable data structures (`Effect`, `TriggeredAbility`, `ActivatedAbility`, `StaticAbility`, `KeywordAbility`), not bespoke match arms. Adding a card is a data definition, not a code change.

### Two-Tier Card Resolution

Cards are first looked up in the hand-authored sample database (275+ cards). If not found, the Scryfall API is queried and the oracle text is auto-parsed into effects. This enables loading arbitrary Commander decks.

### Event System

`GameEvent` is a transient notification mechanism, not part of `GameState`. Events fire, get processed, and are discarded. This preserves cheap `Clone`.

### Layered Effects (CR 613)

Continuous effects are applied via the comprehensive rules layer system. `compute_characteristics()` recomputes power, toughness, types, colors, and abilities from scratch each time, ensuring determinism.
