# Solver & Simulation

Strategy computation and parallel simulation infrastructure.

## Strategy Trait

All play strategies implement the `Strategy` trait:

```rust
trait Strategy {
    fn choose_action(&self, state: &GameState, player: PlayerIndex) -> Action;
    fn name(&self) -> &str;
}
```

### Implementations

| Strategy | Description |
|----------|-------------|
| `RandomStrategy` | Uniform random from legal actions |
| `GreedyStrategy` | Heuristic: play lands, cast biggest spell, attack all |
| `GoldfishStrategy` | Passive opponent for goldfish testing (never blocks, never attacks, always passes) |
| `McfrStrategy` | MCCFR-trained with regret table lookup |
| `AbstractedMcfrStrategy` | MCCFR-trained with information set abstraction |

## MCTS Goldfish Optimizer

Monte Carlo Tree Search for finding optimal goldfish play sequences.

```bash
DECK=kinnan ITERATIONS=500 cargo run --release --bin mcts_goldfish
```

Uses UCB1 selection to balance exploration and exploitation. Finds play lines that minimize goldfish kill turn.

Options: `DECK`, `ITERATIONS`, `EXPLORE`, `DEPTH`, `THREADS`, `GAMES`, `CHECKPOINT`

## MCCFR Solver

Monte Carlo Counterfactual Regret Minimization for two-player GTO approximation.

### Key Components

- **RegretTable**: `HashMap<u64, InfoSetData>` mapping information set hashes to action regrets
- **InformationSet**: Observable game state from one player's perspective (via `PlayerView`)
- **CanonicalAction**: Stable action identifiers that abstract away instance-specific IDs
- **InfoSetAbstraction**: Configurable bucketing to reduce state space
  - `IdentityAbstraction` -- no-op passthrough
  - `BucketedAbstraction` -- life/turn/board/hand bucketing
  - `CardAwareBucketedAbstraction` -- enhanced with card role classification
  - `MultiPhaseAbstraction` -- fine-grained for strategic phases, coarse for others

### Training

```rust
// Basic training
train(&mut regret_table, &deck_a, &deck_b, &config);

// Parallel training with sharded tables
train_parallel(&mut regret_table, &deck_a, &deck_b, &config);

// Warm-started from GreedyStrategy
train_warm_started(&mut regret_table, &deck_a, &deck_b, &config);
```

Features:
- External sampling MCCFR traversal
- Depth-limited solving with rollout strategies
- Parallel training via Rayon (sharded tables, merged after)
- Checkpointing (save/load via bincode)
- Convergence tracking via `approximate_exploitability()`
- Warm-starting from GreedyStrategy to seed regret tables

## Simulation Runner

```rust
// Run N games in parallel
let results = simulate(n_games, &deck_a, &deck_b, &strategy_a, &strategy_b);
```

Aggregates win rates, average turn count, and average actions per game. Uses Rayon for parallelism (~2700 games/sec with GreedyStrategy in debug mode).

## Combo Discovery

Finds infinite combos via exhaustive DFS:

```bash
DECK=kinnan cargo run --release --bin combo_list
```

1. Generates all card subsets up to `MAX_PIECES` (default 3)
2. DFS explores all ability activation sequences
3. Cycle detection: state returns to previous configuration with equal or greater resources
4. Classification: `InfiniteMana`, `InfiniteTokens`, `InfiniteDamage`, `InfiniteLifeGain`, `InfiniteDraw`

Discovered combos become **macro-actions** (`Action::ActivateMacro`) that collapse infinite loops into single decisions in the game tree.

## Exhaustive Goldfish Solver

Finds minimum-turn goldfish kills via DFS with branch-and-bound:

```bash
SEEDS=20 MAX_TURN=10 TIMEOUT=30 cargo run --release --bin bfs_goldfish
```

Pruning: phase auto-pass, dead card filter, combat skip, fetch land handling, mana ability removal, combo macro instant-win detection.

## Opponent Modeling

`OpponentModel` with Bayesian updating for deck archetype inference:
- `observe_card()` -- update beliefs based on observed cards
- `most_likely_archetype()` -- current best guess
- `distribution()` -- full probability distribution over archetypes
