# GTO Strategy Planning: MCCFR-First Approach

## Core Problem Statement

Game Theory Optimal play in Magic: The Gathering is fundamentally **matchup-dependent**. A strategy is not a function of game state alone — it's a function of:

1. **The current game state** (board, hands, life totals, stack)
2. **My deck composition** (what I could draw, what's left in my library)
3. **The opponent's deck composition** (what they could have, what they've already played)

Two nearly identical red aggro decks with slightly different curves will have different optimal sequencing. The same deck plays entirely differently against control vs. aggro vs. midrange. This means:

- There is no single "GTO strategy" — there is a GTO strategy **per deck pair**
- Training produces a policy for a specific matchup, not a universal player
- Even within a matchup, the policy must be a **mixed strategy** (probability distributions over actions) because Nash equilibria in imperfect information games are generally mixed

## Why MCCFR First

Monte Carlo Counterfactual Regret Minimization is the right starting point because:

1. **Provable convergence** — MCCFR converges to Nash equilibrium in two-player zero-sum games. MTG is two-player zero-sum (one player wins, the other loses). We get theoretical guarantees that deep RL cannot provide.

2. **No function approximation needed initially** — We can start with tabular MCCFR (exact regret tables) on simplified game states and progressively add abstraction. Deep RL requires designing neural architectures, training pipelines, and hyperparameter tuning from day one.

3. **Natural handling of mixed strategies** — CFR's regret matching directly produces probability distributions over actions. This is exactly what GTO play requires (e.g., "bluff-attack with this 2/2 into their 3/3 some percentage of the time").

4. **Information set framework is built in** — CFR is designed around information sets. Implementing MCCFR forces us to build the information set abstraction layer, which is a prerequisite for any approach (including deep RL).

5. **Incremental value** — Even a partially-converged MCCFR policy on a simplified game is a meaningful improvement over heuristics. We can validate the infrastructure with small card pools before scaling.

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                   Training System                    │
│                                                     │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │  MCCFR      │  │  Information  │  │  Regret /  │ │
│  │  Traversal  │──│  Set Builder  │──│  Strategy  │ │
│  │  Engine     │  │  & Abstractor │  │  Tables    │ │
│  └──────┬──────┘  └──────────────┘  └─────┬──────┘ │
│         │                                  │        │
│         ▼                                  ▼        │
│  ┌─────────────┐                   ┌────────────┐  │
│  │  Game       │                   │  Serialized │  │
│  │  Simulator  │                   │  Policy     │  │
│  │  (existing) │                   │  (.bin)     │  │
│  └─────────────┘                   └──────┬─────┘  │
│                                           │        │
└───────────────────────────────────────────┼────────┘
                                            │
                    ┌───────────────────────┐│
                    │   Deployment          ││
                    │                       ▼│
                    │  ┌──────────────────┐  │
                    │  │  McfrStrategy    │  │
                    │  │  (impl Strategy) │  │
                    │  │                  │  │
                    │  │  Loads policy,   │  │
                    │  │  looks up info   │  │
                    │  │  set, samples    │  │
                    │  │  from distrib.   │  │
                    │  └──────────────────┘  │
                    └───────────────────────┘
```

The existing `Strategy` trait works as-is for deployment. `choose_action()` internally:
1. Computes the current information set from the `GameState`
2. Looks up the trained action distribution for that information set
3. Samples one action from the distribution and returns it

Training does not use the `Strategy` trait — it traverses the game tree directly.

## Requirement 1: Information Sets

### What a Player Can Observe

An information set represents everything a player **knows** at a decision point. In MTG:

**Fully observable (public information):**
- Battlefield (all permanents, their state: tapped/untapped, counters, damage)
- Graveyard contents (both players)
- Exile zone contents (both players)
- Stack (all spells/abilities waiting to resolve)
- Life totals
- Current phase, turn number, active player
- Number of cards in opponent's hand (but not which cards)
- Number of cards in opponent's library
- Combat state (declared attackers, declared blockers)
- Mana pools (both players)
- Lands played this turn

**Private information (known only to the owning player):**
- Cards in own hand
- Order of own library (in practice, players don't know this either after shuffling)

**Hidden information (known to neither player):**
- Order of both libraries
- Which specific cards remain in opponent's library (partially inferrable from what's been played/drawn)

### Information Set Representation

```rust
/// An information set captures everything a player knows at a decision point.
/// Two game states that look identical from a player's perspective
/// belong to the same information set.
#[derive(Hash, Eq, PartialEq, Clone)]
struct InformationSet {
    // Public game state
    phase: Phase,
    turn_number: u32,  // may want to bucket this
    active_player: PlayerIndex,
    my_life: i32,
    opp_life: i32,

    // Battlefield — sorted canonical representation
    my_permanents: Vec<PermanentSummary>,  // card_id, tapped, damage, counters
    opp_permanents: Vec<PermanentSummary>,

    // Known zones
    my_hand: Vec<CardId>,  // sorted for canonical ordering
    my_graveyard: Vec<CardId>,
    opp_graveyard: Vec<CardId>,
    opp_hand_size: usize,

    // Stack
    stack_summary: Vec<StackSummary>,

    // Combat
    combat_summary: Option<CombatSummary>,

    // Mana
    my_mana_pool: ManaPool,
}
```

### Information Set Abstraction (Bucketing)

Raw information sets are too numerous for tabular storage. We need abstraction:

**Phase 1 — Lossless (small card pools):**
Use exact information sets. With 10-15 unique cards and short games (5-8 turns), the number of reachable information sets is tractable for tabular MCCFR.

**Phase 2 — Lossy abstraction (larger card pools):**
- **Life total bucketing**: {1-5, 6-10, 11-15, 16-20, 21+} instead of exact values
- **Board state abstraction**: Represent board by aggregate stats (total power, total toughness, creature count, mana available) rather than exact permanents
- **Hand abstraction**: Categorize hand cards by role (land, cheap creature, expensive creature, removal, pump spell) rather than exact card identities
- **Turn bucketing**: {early (1-3), mid (4-6), late (7+)} instead of exact turn number
- **Ignore damage on creatures / counters when they don't affect lethal calculations**

The abstraction function maps `GameState × PlayerIndex → AbstractInfoSet`, and the granularity is a tunable knob trading memory/compute for strategy quality.

## Requirement 2: MCCFR Algorithm

### Variant Selection: External Sampling MCCFR

We'll use **External Sampling MCCFR** because:
- Only samples chance nodes and opponent actions (not the traversing player's actions)
- The traversing player explores all their own actions at each decision point
- Lower variance than outcome sampling, much faster than vanilla CFR
- Standard choice for large imperfect-information games

### Algorithm Sketch

```
function external_sampling_mccfr(state, traversing_player):
    if state.is_terminal():
        return utility(state, traversing_player)  // +1 win, -1 loss, 0 draw

    player = state.current_player()
    info_set = compute_info_set(state, player)
    actions = legal_actions(state)

    if player == traversing_player:
        // Traverse all actions, compute counterfactual regrets
        strategy = regret_matching(info_set)
        action_utilities = {}

        for action in actions:
            child_state = state.clone()
            apply_action(child_state, action)
            action_utilities[action] = external_sampling_mccfr(
                child_state, traversing_player
            )

        node_utility = sum(strategy[a] * action_utilities[a] for a in actions)

        // Update regrets
        for action in actions:
            regret = action_utilities[action] - node_utility
            cumulative_regret[info_set][action] += regret

        return node_utility

    else:
        // Opponent or chance node: sample according to current strategy
        if is_chance_node(state):
            // Sample a random outcome (e.g., draw a card)
            action = sample_chance(state)
        else:
            strategy = regret_matching(info_set)
            action = sample_from(strategy)

        child_state = state.clone()
        apply_action(child_state, action)
        return external_sampling_mccfr(child_state, traversing_player)

function regret_matching(info_set):
    regrets = cumulative_regret[info_set]
    positive_regrets = {a: max(0, r) for a, r in regrets}
    total = sum(positive_regrets.values())
    if total > 0:
        return {a: r / total for a, r in positive_regrets}
    else:
        return uniform distribution over actions
```

### Chance Nodes in MTG

MTG has several sources of randomness that MCCFR must handle:
- **Library shuffle** at game start (initial deck order)
- **Draw step** (which card is drawn)
- **Any shuffle effects** (fetch lands, tutors — not yet implemented)

For external sampling, we sample these once per traversal. The initial library order is sampled at the start of each MCCFR iteration.

### Handling the MTG Decision Structure

MTG's priority system creates a subtlety: the game frequently alternates between players within a single turn. This is unlike poker where players act in strict rounds. MCCFR handles this naturally — at each decision point, we check who has priority and treat them as the acting player.

However, some "decisions" are trivially forced:
- When the only legal action is PassPriority, skip the CFR node entirely
- When a player has priority but nothing meaningful to do (no castable spells, stack empty), auto-pass
- This dramatically reduces the effective game tree depth

## Requirement 3: Regret & Strategy Storage

### Data Structures

```rust
/// Per-information-set data for MCCFR
struct InfoSetData {
    /// Cumulative regret for each action index
    cumulative_regret: Vec<f64>,

    /// Cumulative strategy (for computing average strategy)
    cumulative_strategy: Vec<f64>,

    /// Number of times this info set was visited
    visit_count: u64,
}

/// The complete regret table
struct RegretTable {
    /// Maps information set hash -> per-action regret data
    /// Action indices correspond to sorted legal_actions() output
    data: HashMap<u64, InfoSetData>,
}
```

### Action Indexing Problem

`legal_actions()` returns different action lists depending on game state. Two states in the same information set should have the same legal actions (since they look identical to the acting player), but the `ObjectId`s in the actions will differ.

**Solution**: Canonicalize actions relative to the information set. Instead of storing regret for `CastSpell { object_id: 47 }`, store regret for `CastSpell { card: "Lightning Bolt", target: "opponent" }`. This requires an action abstraction layer:

```rust
/// A canonical action identifier that's stable across game states
/// within the same information set.
#[derive(Hash, Eq, PartialEq, Clone)]
enum CanonicalAction {
    PassPriority,
    PlayLand { card_id: CardId },
    CastSpell { card_id: CardId, target: CanonicalTarget },
    AttackWith { attacker_set: Vec<CardId> },  // sorted by CardId
    Block { assignments: Vec<(CardId, CardId)> },
    ActivateAbility { source_card: CardId, ability_index: usize },
}
```

### Serialization & Checkpointing

Regret tables must be serializable for:
- Checkpointing during long training runs
- Loading a trained policy into `McfrStrategy` for deployment
- Analyzing convergence across training iterations

Use `serde` with bincode for compact binary serialization (the existing serde dependency works).

## Requirement 4: Game State Feature Extraction

Even before deep RL, we need feature extraction for:
- Information set hashing (compact representation for table lookup)
- Evaluation heuristics (for pruning or warm-starting)
- Future neural network input

### Feature Categories

```rust
struct GameFeatures {
    // Resource features
    my_life: f32,
    opp_life: f32,
    my_cards_in_hand: f32,
    opp_cards_in_hand: f32,
    my_lands_untapped: f32,
    my_lands_total: f32,
    my_mana_available: f32,  // total mana from untapped sources

    // Board features
    my_creature_count: f32,
    opp_creature_count: f32,
    my_total_power: f32,
    opp_total_power: f32,
    my_total_toughness: f32,
    opp_total_toughness: f32,

    // Tempo features
    turn_number: f32,
    cards_played_this_turn: f32,

    // Derived features
    life_differential: f32,       // my_life - opp_life
    board_advantage: f32,         // my_total_power - opp_total_power
    can_attack_for_lethal: bool,  // sum of eligible attacker power >= opp_life
    opponent_can_lethal: bool,    // sum of their eligible power >= my_life
}
```

## Requirement 5: Training Infrastructure

### Training Loop

```rust
struct McfrTrainer {
    regret_table: RegretTable,
    card_db: CardDatabase,
    deck0: Vec<CardId>,
    deck1: Vec<CardId>,
    iterations: u64,
    abstraction: Box<dyn InfoSetAbstraction>,
}

impl McfrTrainer {
    fn train(&mut self, num_iterations: u64) {
        for i in 0..num_iterations {
            // Alternate which player is the traversing player
            let traverser = (i % 2) as PlayerIndex;

            // Sample a new game (random deck order)
            let state = self.new_game();

            // Run one external sampling traversal
            self.traverse(state, traverser);

            // Periodic logging/checkpointing
            if i % 10_000 == 0 {
                self.checkpoint();
                self.log_convergence();
            }
        }
    }
}
```

### Convergence Monitoring

Track exploitability over time:
- Every N iterations, compute a best-response against the current average strategy
- Exploitability = value of best response. Should decrease toward 0.
- For simplified games, we can compute this exactly. For full games, approximate via simulation.

### Integration with Existing Simulation Harness

The trained policy deploys via a new strategy implementation:

```rust
struct McfrStrategy {
    policy: HashMap<u64, Vec<(CanonicalAction, f64)>>,  // info_set_hash -> distribution
    abstraction: Box<dyn InfoSetAbstraction>,
}

impl Strategy for McfrStrategy {
    fn choose_action(&self, state: &GameState, player: PlayerIndex) -> Action {
        let info_set = self.abstraction.compute(state, player);
        let hash = info_set.hash();

        if let Some(distribution) = self.policy.get(&hash) {
            // Sample from trained distribution
            let action = sample_from_distribution(distribution);
            // Map canonical action back to concrete action with ObjectIds
            resolve_canonical_action(action, state, player)
        } else {
            // Unseen info set — fall back to uniform random
            let actions = legal_actions(state);
            actions.choose(&mut rand::thread_rng()).cloned()
                .unwrap_or(Action::PassPriority)
        }
    }
}
```

## Phased Implementation Plan

### Phase 0: Foundation (prerequisites)

**Goal**: Build the infrastructure that all approaches need.

- [ ] **Information set module** (`src/info_set/mod.rs`)
  - `InformationSet` struct with canonical representation
  - `compute_info_set(state, player) -> InformationSet`
  - Hashing (use `std::hash` with a quality hasher like `FxHash`)
  - Unit tests: two states that differ only in hidden info produce the same info set

- [ ] **Canonical action mapping** (`src/action/canonical.rs`)
  - `CanonicalAction` enum
  - `canonicalize(action, state, player) -> CanonicalAction`
  - `resolve(canonical, state, player) -> Action` (reverse mapping)
  - Unit tests: round-trip canonicalize → resolve preserves semantics

- [ ] **Game state feature extraction** (`src/features/mod.rs`)
  - `extract_features(state, player) -> GameFeatures`
  - Used by info set abstraction, heuristic evaluation, and future NN input

- [ ] **Hidden information handling in GameState**
  - Add `fn visible_state(&self, player: PlayerIndex) -> PartialGameState`
  - Or: info set computation reads only public + own-private zones
  - Ensure MCCFR traversal uses full state but info set lookup uses partial view

### Phase 1: Tabular MCCFR on Simplified Games

**Goal**: End-to-end MCCFR working on a tiny card pool to validate the infrastructure.

- [ ] **Create a minimal test scenario**
  - 3-4 unique cards per deck (e.g., Mountains + Lightning Bolts vs. Mountains + Gray Ogres)
  - 10-15 card decks (not full 60) to keep tree size manageable
  - Games should end in 3-5 turns

- [ ] **MCCFR trainer module** (`src/solver/mccfr.rs`)
  - `RegretTable` with cumulative regret and strategy storage
  - `regret_matching()` function
  - External sampling traversal
  - Training loop with iteration counting

- [ ] **Trivial forced-action pruning**
  - Skip CFR nodes where only one action is legal
  - Auto-pass when no meaningful actions available
  - This is critical for performance — most priority passes in MTG are trivially forced

- [ ] **McfrStrategy implementation**
  - Loads trained regret table
  - Implements `Strategy` trait
  - Falls back to random for unseen information sets

- [ ] **Validation**
  - Train on the simplified game until convergence
  - Verify exploitability decreases over iterations
  - `McfrStrategy` should beat `GreedyStrategy` in simulation
  - `McfrStrategy` mirror match should converge to ~50% win rate

### Phase 2: Scaling to Realistic Decks

**Goal**: Handle the existing 60-card sample decks.

- [ ] **Information set abstraction layer**
  - `trait InfoSetAbstraction { fn abstract(&self, info_set: InformationSet) -> u64; }`
  - Implement life bucketing, board summarization, hand role classification
  - Make abstraction granularity configurable

- [ ] **Memory-efficient regret storage**
  - Switch from `HashMap` to more compact storage if needed
  - Consider pruning regrets that haven't been visited recently
  - Regret table size monitoring and logging

- [ ] **Parallelized MCCFR iterations**
  - External sampling MCCFR is embarrassingly parallel across iterations
  - Use Rayon (already a dependency) for parallel traversals
  - Lock-free regret table updates (atomic operations or sharded tables)

- [ ] **Depth-limited solving with rollout policy**
  - Full game tree is too deep — limit MCCFR traversal to N turns of lookahead
  - Use a rollout policy (GreedyStrategy or RandomStrategy) to estimate value beyond the horizon
  - The depth limit is a tunable parameter

- [ ] **Validation at scale**
  - Train Mono-Red vs. Mono-Green matchup
  - Compare against GreedyStrategy baseline
  - Measure convergence rate and memory usage
  - Profile for performance bottlenecks

### Phase 3: Refinements

**Goal**: Improve strategy quality and usability.

- [ ] **Warm-starting from heuristics**
  - Initialize regret tables using GreedyStrategy's implicit preferences
  - Avoids cold-start problem where early iterations are pure random

- [ ] **Opponent modeling / deck inference**
  - As the game progresses, narrow down possible opponent hands based on observed plays
  - Bayesian updating on opponent's likely deck composition
  - This is where information sets get really interesting

- [ ] **Multi-street abstraction**
  - Different abstraction granularity for different phases
  - Main phase decisions need more detail than "auto-pass priority during opponent's turn"
  - Combat decisions may benefit from exact representation (smaller branching)

- [ ] **Policy visualization**
  - For a given board state, show the action distribution the policy recommends
  - Useful for debugging and for human players studying GTO play
  - Export key decision points with their distributions

## Future: Deep RL Path (Phase 4+)

Once the MCCFR infrastructure is proven, deep RL becomes viable:

### What MCCFR gives us that RL needs
- **Information set framework** — reused directly as the observation space
- **Feature extraction** — becomes the neural network input layer
- **Canonical actions** — becomes the action space for the policy network
- **Simulation harness** — generates training episodes
- **Baseline policies** — MCCFR policies serve as opponents for self-play and as comparison baselines

### Deep CFR (Neural Network + CFR)
Replace the regret table with a neural network that predicts regrets given features:
- `regret_network(features) -> Vec<f64>` (one regret per action)
- Train the network on (info_set_features, actual_regrets) pairs collected during traversal
- Generalizes to unseen information sets — the key advantage over tabular MCCFR
- Published approach: "Deep Counterfactual Regret Minimization" (Brown et al., 2019)

### AlphaZero-style Self-Play
- Policy + value network: `(features) -> (action_probs, state_value)`
- Train via self-play with MCTS for action selection
- No convergence guarantees to Nash equilibrium, but scales better
- Can learn cross-matchup patterns if trained on multiple deck pairs

### Hybrid: MCCFR Subgames + Neural Net Global Policy
- Neural network for high-level decisions (what to cast, when to be aggressive)
- MCCFR solving for tactical subgames (combat math, stack interaction bluffs)
- Similar architecture to Libratus/Pluribus in poker
- Most promising long-term approach but requires both systems working first

## Open Questions

1. **How coarse can information set abstraction be before strategy quality degrades unacceptably?** This is empirical — we need to test multiple granularities.

2. **How deep should MCCFR traverse before using a rollout policy?** MTG games are 5-15 turns with many decision points per turn. Full traversal is infeasible for 60-card decks. 2-3 turns of lookahead with heuristic rollouts may be the practical sweet spot.

3. **Should we handle the draw step as a chance node or determinize?** Determinization (assume a specific draw, average over many samples) is simpler but theoretically weaker. True chance-node handling is correct but increases the tree size by a factor of |library| at each draw.

4. **Can we exploit MTG's structure to prune the action space?** Many legal actions are obviously bad (Lightning Bolt your own creature, not attacking when opponent is at 1 life). Domain-specific pruning could dramatically reduce branching factor without losing strategy quality. But we must be careful — "obviously bad" actions are sometimes correct in GTO (bluffs, information hiding).

5. **Multi-game strategies (sideboarding, metagaming)**: GTO for a single game is already hard. Best-of-three with sideboarding is a meta-game on top. Table for later.

## Technical Dependencies

**Existing (no new crates needed for Phase 0-1):**
- `serde` / `serde_json` — serialization for regret tables and policies
- `rand` — sampling in MCCFR traversal
- `rayon` — parallel MCCFR iterations

**Likely needed for Phase 2+:**
- `bincode` — compact binary serialization for large regret tables
- `fxhash` or `ahash` — fast hashing for information set lookup
- `indicatif` — progress bars for long training runs

**Needed for Phase 4 (Deep RL):**
- `tch-rs` (PyTorch bindings) or `candle` (Rust-native ML) — neural network training
- Or: Python training loop via `pyo3`, with Rust game engine called from Python
