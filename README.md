# MTG Commander Goldfish Simulator

A Rust-based Magic: The Gathering Commander rules engine with an interactive TUI for goldfish (solitaire) play. Load any Commander deck, pilot it through a full game, and see every card interaction play out.

## Quick Start

```bash
# Interactive TUI goldfish (recommended)
cargo run --release --features tui --bin tui

# CLI goldfish with a preset deck
cargo run --release --bin goldfish -- --preset kinnan --trace

# CLI goldfish with a deck file
cargo run --release --bin goldfish -- --deck decks/kinnan_bonder_prodigy.txt

# Manual interactive play (text-based)
cargo run --release --bin interactive

# Run the test suite
cargo test
```

## TUI Goldfish Player

The primary interface is a terminal UI for piloting a Commander deck in goldfish mode (solitaire against a passive opponent at 40 life).

```bash
cargo run --release --features tui --bin tui
```

**Controls:**
- **WASD / Arrow keys** -- Navigate between zones (Hand, Battlefield, Command Zone, Graveyard, Exile, Stack)
- **Space** -- Select card / confirm action
- **U** -- Undo last action
- **Q** -- Quit (shows action log)

**Features:**
- Zone-based card browser (Hand, Battlefield, Command Zone, Stack, Graveyard, Exile)
- Legal action panel with card detail view
- Tapped / summoning sick / damage / counter indicators
- Undo support for exploring lines
- Auto-advance for non-interactive phases
- SVG snapshot export via `tui_snapshot` binary

### TUI Snapshots

Generate SVG screenshots of game states for documentation or analysis:

```bash
cargo run --release --features tui --bin tui_snapshot -- --output-dir snapshots/
```

## Binaries

| Binary | Description |
|--------|-------------|
| `tui` | Interactive TUI goldfish player (requires `--features tui`) |
| `tui_snapshot` | Headless SVG screenshot generator (requires `--features tui`) |
| `goldfish` | CLI goldfish simulator with deck loading, traces, and coverage reports |
| `interactive` | Text-based manual goldfish play |
| `commander_goldfish` | Batch Commander goldfish simulation with statistics |
| `bfs_goldfish` | Exhaustive DFS solver -- finds minimum-turn goldfish kills |
| `mcts_goldfish` | MCTS optimizer for goldfish play sequences |
| `combo_list` | Automatic infinite combo discovery via exhaustive DFS |

### Goldfish CLI

```bash
# Run 100 goldfish games with the Kinnan deck
cargo run --release --bin goldfish -- --preset kinnan --games 100

# Trace a single game
cargo run --release --bin goldfish -- --preset ashcoat --trace

# Check card coverage for a deck
cargo run --release --bin goldfish -- --deck my_deck.txt --coverage
```

Presets: `kinnan`, `brimaz`, `ashcoat`, `thrun`

### Combo Discovery

Finds infinite combos in a deck using exhaustive ability activation search with DFS cycle detection.

```bash
DECK=kinnan cargo run --release --bin combo_list
DECK=ashcoat cargo run --release --bin combo_list
```

Combos are classified by category (Infinite Mana, Infinite Tokens, Infinite Damage, Infinite Life Gain, Infinite Draw) and registered as macro-actions for the solver.

### Exhaustive Goldfish Solver

Finds the minimum-turn goldfish kill via DFS with branch-and-bound:

```bash
SEEDS=20 MAX_TURN=10 TIMEOUT=30 cargo run --release --bin bfs_goldfish
```

## Commander Decks

Four prebuilt 100-card Commander decks:

- **Kinnan, Bonder Prodigy** -- Simic mana combo (Basalt Monolith + Kinnan infinite mana)
- **Brimaz, King of Oreskos** -- Mono-white tokens/aggro
- **Ashcoat of the Shadow Swarm** -- Mono-black rat tribal (Marrow-Gnawer + Thornbite Staff combos)
- **Thrun, Breaker of Silence** -- Mono-green voltron

Deck files in `decks/` use standard text format (one card per line). Any Commander deck can be loaded -- cards not in the 275+ hand-authored database are auto-parsed via Scryfall.

## Rules Engine

The engine covers the core MTG comprehensive rules relevant to Commander goldfish play:

- Full turn structure (13 phases) with priority system
- Stack: spells, activated abilities, triggered abilities
- Combat: first strike, double strike, trample, deathtouch, lifelink, vigilance, menace, flying/reach
- State-based actions (CR 704.5 suite)
- Trigger queue with APNAP ordering (19 trigger conditions)
- Replacement effects (CR 614) and continuous effects / layered system (CR 613)
- Token creation with ETB triggers
- Planeswalker loyalty abilities
- Equipment and Aura attachments
- Commander rules: command zone, commander tax, commander damage, color identity, singleton, partner commanders
- 34+ keyword abilities including Flashback, Escape, Cascade, Storm, Undying, Persist
- Smart auto-tap with most-constrained-first mana payment
- Scryfall-backed card loading (275+ hand-authored cards + auto-parse fallback)

See [docs/rules-engine.md](docs/rules-engine.md) for full coverage details.

## Project Structure

See [docs/architecture.md](docs/architecture.md) for the full module layout and design patterns.

## Build & Test

```bash
cargo build                    # Compile
cargo test                     # All tests (~312 tests), must pass with 0 warnings
cargo test --features scryfall # Include Scryfall API tests
cargo run --release            # Demo simulation
```

CI runs `cargo build --verbose && cargo test --verbose` on push/PR to `mainline`.

## Dependencies

- **ratatui** + **crossterm** (optional, `tui` feature): Terminal UI
- **serde** + **serde_json**: Serialization for game state snapshots
- **rand 0.8**: RNG for shuffling
- **rayon 1.8**: Parallel simulation
- **bincode**: Binary serialization for solver checkpoints
- **ureq** (optional, `scryfall` feature): Scryfall API for card data import

## License

This project is for research and educational purposes.
