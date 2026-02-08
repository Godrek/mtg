# Consolidated Development Strategy: Rules Engine + MCCFR

## Current State (Post Pre-Requisite Fixes)

The codebase is ready for parallel development. Four foundational fixes have landed:

| Fix | PR | What Changed |
|---|---|---|
| `Arc<CardDatabase>` | #9 | `GameState.card_db` is now `Option<Arc<CardDatabase>>` — clone is O(1) for the DB |
| Deterministic discard | #8 | `Action::Discard { object_id }` replaces `discard_random()` — cleanup discard is a player choice |
| Trigger ordering surfaced | #10 | `Action::OrderTriggers { ordering }` — simultaneous triggers are a player decision, not silent FIFO |
| Combat abstraction | #11 | `CombatAbstraction::Bucketed` reduces attacker/blocker space from O(2^n) to ~7 buckets |

### What We Have Now

```
src/
├── action/mod.rs      # Action enum (12 variants), legal_actions(), legal_actions_abstracted()
├── card/mod.rs        # CardDef, CardInstance, Effect enum, TriggeredAbility
├── card/sample.rs     # ~40 hardcoded card definitions
├── game/mod.rs        # GameState, PlayerState, CombatState, Arc<CardDatabase>
├── rules/mod.rs       # apply_action(), SBA, triggers, combat damage, phase management
├── strategy/mod.rs    # Strategy trait, RandomStrategy, GreedyStrategy
├── simulation/mod.rs  # run_game(), simulate() with rayon parallelism
├── mana/mod.rs        # ManaPool, ManaCost, Color
├── deck_import.rs     # Deck importing from text files
└── lib.rs
```

### Key Interfaces Already Stable

- **`Strategy` trait** (`src/strategy/mod.rs:9-12`): `choose_action(&self, state: &GameState, player: PlayerIndex) -> Action` — clean boundary between solver and engine
- **`Action` enum** (`src/action/mod.rs:22-75`): 12 variants including `Discard`, `OrderTriggers` — complete for current rules coverage
- **`legal_actions()` / `legal_actions_abstracted()`** (`src/action/mod.rs:97-99`): Full and bucketed modes — MCCFR uses bucketed, simulation uses full
- **`apply_action()`** (`src/rules/mod.rs:9-242`): Deterministic state transitions for all action types
- **`GameState::clone()`**: O(n) on objects HashMap, O(1) on card_db via Arc

---

## Development Tracks

Two parallel tracks with a shared interface milestone at the start.

### Track A: Rules Engine (from PR #7)

Correctness and fidelity of game rules. Makes the engine produce correct state transitions for increasingly complex card interactions.

### Track B: MCCFR Solver (from PR #6)

GTO strategy computation. Builds the solver infrastructure that consumes the engine as a black box through `Strategy` + `legal_actions()` + `apply_action()`.

---

## Phase 0: Shared Interface Contract

**Goal**: Define the observation and action APIs both tracks code against. Do this before any Phase 1 work.

**Duration**: Single PR, no code changes to existing modules.

### Deliverables

#### 0.1 — Observation API: `GameState::visible_state(player)`

Add to `src/game/mod.rs`:

```rust
/// Everything a player can observe — the information set boundary.
/// Rules engine writes to GameState; MCCFR reads through this view.
pub struct PlayerView<'a> {
    // Public information (both players can see)
    pub phase: Phase,
    pub active_player: PlayerIndex,
    pub turn_number: u32,
    pub battlefield: &'a [ObjectId],       // all permanents, both players
    pub stack: &'a [StackEntry],
    pub combat: &'a CombatState,
    pub pending_triggers: &'a [PendingTrigger],

    // Per-player public info
    pub my_life: i32,
    pub opp_life: i32,
    pub my_graveyard: &'a [ObjectId],
    pub opp_graveyard: &'a [ObjectId],
    pub my_exile: &'a [ObjectId],
    pub opp_exile: &'a [ObjectId],
    pub opp_hand_size: usize,
    pub opp_library_size: usize,

    // Private information (only the viewing player sees)
    pub my_hand: &'a [ObjectId],

    // Mana
    pub my_mana_pool: &'a ManaPool,
    pub my_land_plays_remaining: u32,

    // Object lookup (shared, read-only)
    pub objects: &'a HashMap<ObjectId, CardInstance>,
    pub card_db: &'a CardDatabase,
}

impl GameState {
    pub fn visible_state(&self, player: PlayerIndex) -> PlayerView<'_> { ... }
}
```

This decouples MCCFR's information set computation from GameState internals. When the rules engine adds fields (event bus, effects manager), MCCFR doesn't break — only `visible_state()` needs updating.

#### 0.2 — Canonical Action Mapping

Add `src/action/canonical.rs`:

```rust
/// Stable action identifier independent of ObjectId assignment.
/// Two game states in the same information set map concrete Actions
/// to the same CanonicalAction.
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum CanonicalAction {
    PassPriority,
    PlayLand { card_id: CardId },
    CastSpell { card_id: CardId, target: CanonicalTarget },
    ActivateAbility { source_card_id: CardId, ability_index: usize },
    DeclareAttackers { attacker_card_ids: Vec<CardId> },  // sorted
    DeclareBlockers { assignments: Vec<(CardId, CardId)> }, // sorted
    Discard { card_id: CardId },
    OrderTriggers { source_card_ids: Vec<(CardId, usize)> },
    OrderDamageAssignment { /* ... */ },
    Concede,
}

pub fn canonicalize(action: &Action, state: &GameState) -> CanonicalAction { ... }
pub fn resolve(canonical: &CanonicalAction, state: &GameState, player: PlayerIndex) -> Option<Action> { ... }
```

#### 0.3 — Snapshot Contract

Document the rules for what gets cloned vs. reconstructed:

- **Always cloned**: `objects`, `players`, `battlefield`, `stack`, `combat`, `pending_triggers`, all scalar fields
- **Shared via Arc**: `card_db` (already done)
- **Reconstructed after clone**: Future event bus state, continuous effects caches, dirty flags — these are derived from game state and must not be part of `Clone`
- **New rule**: Any field added to `GameState` that is derivable from other fields must be marked `#[serde(skip)]` and excluded from equality/hashing. The canonical game state is the minimal set of fields needed to reconstruct the full state.

### Acceptance Criteria for Phase 0

- `PlayerView` struct compiles and `visible_state()` returns it
- `canonicalize()` round-trips: `resolve(canonicalize(action, state), state, player) == Some(action)` for all legal actions
- Both tracks agree this is the interface they code against

---

## Phase 1A: Rules Engine Foundations

**Goal**: Correct SBA/trigger loop, event system skeleton, lay groundwork for continuous effects.

**Depends on**: Phase 0 complete.

### 1A.1 — SBA/Trigger Recurrence Loop

Current gap: `check_state_based_actions()` (`src/rules/mod.rs:607-686`) fires dies triggers at the end but doesn't re-check whether those triggers caused new SBAs.

Fix: Implement the CR 704.3 loop:
```
loop {
    perform_all_SBAs()
    check_and_queue_triggers()
    if no_SBAs_performed && no_triggers_queued { break }
    put_triggers_on_stack()  // may pause for OrderTriggers
}
// only now grant priority
```

This changes state transition behavior. MCCFR policies trained against the current engine won't transfer — acceptable because MCCFR training hasn't started.

### 1A.2 — Event System Skeleton

Add `src/events/mod.rs`:

```rust
/// A game event that just happened. Events are transient — they fire,
/// handlers process them, and they're discarded. They are NOT part of
/// GameState and do NOT affect Clone/snapshot.
pub enum GameEvent {
    DamageDealt { source: ObjectId, target: Target, amount: u32 },
    ZoneChange { object: ObjectId, from: ZoneType, to: ZoneType },
    LifeChanged { player: PlayerIndex, old: i32, new: i32 },
    SpellCast { object: ObjectId, controller: PlayerIndex },
    AbilityTriggered { source: ObjectId, ability_index: usize },
    CounterPlaced { object: ObjectId, counter_type: CounterType, count: i32 },
}
```

Key design constraint: **events are transient, not stateful**. The event system is a notification mechanism within `apply_action()` and `resolve_effect()`. It does not store subscribers or callbacks in `GameState`. This preserves cheap `Clone`.

Event handlers are registered at the engine level (not per-game-state) and receive `(&mut GameState, &GameEvent)`. This is a function-pointer table, not a dynamic dispatch system.

### 1A.3 — Replacement Effect Framework

Add `Action::ChooseReplacementOrder` and the corresponding infrastructure for when multiple replacement effects could apply to the same event. This is a player decision that MCCFR must see.

This extends `Action` and `CanonicalAction` — coordinate with Phase 0's canonical action contract.

### Acceptance Criteria for Phase 1A

- SBA loop correctly handles recursive triggers (test: creature with "when ~ dies, deal 2 damage to each player" killing another creature at 2 toughness)
- Events fire for all zone changes, damage, and life changes
- Event handlers do not affect `GameState::clone()` cost
- All existing tests pass

---

## Phase 1B: MCCFR Infrastructure

**Goal**: End-to-end MCCFR working on a tiny card pool. Validate the information set framework, regret tables, and training loop.

**Depends on**: Phase 0 complete. Can run in parallel with Phase 1A if both code against the Phase 0 interfaces.

### 1B.1 — Information Set Module

Add `src/info_set/mod.rs`:

```rust
pub struct InformationSet { /* from PlayerView */ }

impl InformationSet {
    pub fn from_view(view: &PlayerView, card_db: &CardDatabase) -> Self { ... }
    pub fn hash(&self) -> u64 { ... }
}
```

Uses `PlayerView` from Phase 0 — does not read raw `GameState` fields.

### 1B.2 — Regret Table

Add `src/solver/mod.rs`:

```rust
pub struct RegretTable {
    data: HashMap<u64, InfoSetData>,  // info_set_hash -> per-action data
}

struct InfoSetData {
    cumulative_regret: Vec<f64>,     // indexed by canonical action
    cumulative_strategy: Vec<f64>,
    visit_count: u64,
}
```

Self-contained data structure — no dependency on rules engine internals.

### 1B.3 — External Sampling MCCFR Traversal

Add `src/solver/mccfr.rs`:

The traversal function:
1. Calls `legal_actions_abstracted(state)` (bucketed combat)
2. Calls `canonicalize()` to map actions to regret table indices
3. Calls `state.clone()` + `apply_action()` to explore branches
4. Calls `visible_state(player)` to compute information sets
5. Skips CFR nodes where only one action is legal (trivial pass-through)

Uses `legal_actions_abstracted()` not `legal_actions()` — combat branching is bounded.

### 1B.4 — McfrStrategy Implementation

```rust
pub struct McfrStrategy {
    policy: RegretTable,  // or a frozen policy extracted from it
    abstraction: InfoSetAbstraction,
}

impl Strategy for McfrStrategy {
    fn choose_action(&self, state: &GameState, player: PlayerIndex) -> Action {
        let view = state.visible_state(player);
        let info_set = InformationSet::from_view(&view, state.card_db());
        let actions = legal_actions_abstracted(state);
        let canonical: Vec<_> = actions.iter().map(|a| canonicalize(a, state)).collect();
        // look up distribution, sample, resolve back to concrete Action
    }
}
```

### 1B.5 — Minimal Training Scenario

- **Decks**: 15-card decks. Mountains + Lightning Bolts vs. Mountains + Gray Ogres.
- **Target**: Games end in 3-5 turns. Reachable info sets fit in memory (<100K entries).
- **Validation**:
  - Exploitability decreases over iterations
  - McfrStrategy beats GreedyStrategy after sufficient training
  - McfrStrategy mirror match converges to ~50% win rate

### Acceptance Criteria for Phase 1B

- Training loop runs and converges on the minimal scenario
- Regret table serializes/deserializes via serde + bincode
- McfrStrategy implements `Strategy` and plays legal games
- `cargo test` passes for all new modules

---

## Phase 2A: Rules Engine — Continuous Effects & Card Scaling

**Depends on**: Phase 1A complete.

### 2A.1 — CR 613 Layered Effects Engine

Implement the layer system for continuous effects:
- Layer 1: Copy effects
- Layer 2: Control-changing effects
- Layer 3: Text-changing effects
- Layer 4: Type-changing effects
- Layer 5: Color-changing effects
- Layer 6: Ability-adding/removing effects
- Layer 7: P/T-setting and modifying effects (sublayers a-e)

Replace the current `temp_power_mod` / `temp_toughness_mod` / `temp_keywords` fields on `CardInstance` with the layered recomputation model.

Design constraint for MCCFR: The effects engine must be **deterministic** — same `GameState` input produces the same computed characteristics. No caching that persists across clones (dirty flags are local optimization, not state).

### 2A.2 — Composable Card Framework

Replace the monolithic `Effect` enum with composable building blocks:
- `effects::common` library (deal damage, gain life, draw, destroy, bounce)
- `abilities::common` (ETB, dies, upkeep triggers)
- `targets::common` (any creature, any player, creature or player)
- `DynamicValue` trait for runtime-computed values (e.g., "damage equal to the number of creatures you control")

Card definitions become declarative compositions. The `Effect` enum stays as the internal representation but is constructed from the library rather than hardcoded.

### 2A.3 — Expand Card Pool to 100+

Using the composable framework, express 50-100 cards declaratively. Priority:
1. Cards already in `sample.rs` (fix any that use `Effect::Unimplemented`)
2. Format staples that test edge cases (protection, regeneration, auras)
3. Cards that exercise the layered effects engine (Glorious Anthem, Crusade, Humility)

### Acceptance Criteria for Phase 2A

- Layered effects resolve correctly for test cases: Humility + Opalescence, multiple anthem effects, timestamp ordering
- At least 50 cards expressible without engine changes
- `CardInstance::effective_power/toughness` uses the layer engine, not raw modifiers

---

## Phase 2B: MCCFR — Scaling to Realistic Decks

**Depends on**: Phase 1B complete. Can run in parallel with Phase 2A.

### 2B.1 — Information Set Abstraction

Implement lossy abstraction for larger card pools:
- Life bucketing: {1-5, 6-10, 11-15, 16-20, 21+}
- Board state: aggregate stats (total power, total toughness, creature count, mana available)
- Hand: categorize by role (land, cheap creature, expensive creature, removal)
- Turn: {early 1-3, mid 4-6, late 7+}

`trait InfoSetAbstraction { fn abstract_info_set(&self, info_set: &InformationSet) -> u64; }`

### 2B.2 — Parallel MCCFR Training

External sampling MCCFR is embarrassingly parallel across iterations. Use rayon (already a dependency):
- Each thread runs independent traversals
- Regret table updates use atomic operations or sharded tables
- Checkpoint every N iterations

### 2B.3 — Depth-Limited Solving with Rollouts

Full game trees are too deep for 60-card decks. Limit MCCFR to N turns of lookahead, then use GreedyStrategy or RandomStrategy for rollout evaluation.

### 2B.4 — Scale Validation

Train on the Mono-Red vs. Mono-Green matchup from the existing sample decks:
- Measure convergence rate and memory usage
- Compare win rate against GreedyStrategy baseline
- Profile clone/legal_actions/apply_action as the performance-critical path

### Acceptance Criteria for Phase 2B

- MCCFR trains on 60-card decks without OOM
- Trained policy beats GreedyStrategy by measurable margin
- Training parallelizes across cores with near-linear speedup

---

## Phase 3: Integration & Quality

**Depends on**: Both Phase 2 tracks complete.

### 3A — Rules Engine Polish
- Full 704.5 SBA suite
- Combat requirement/restriction integration (must-attack, can't-block)
- Turn structure extras (extra turns, skip steps)
- Zone-change counters on all objects
- `GameStateSnapshot` struct for optimized copy/restore

### 3B — MCCFR Polish
- Warm-starting from GreedyStrategy heuristics
- Opponent modeling / deck inference (Bayesian updating on opponent's likely hands)
- Policy visualization (action distributions for key decision points)
- Multi-abstraction: different granularity for main phase vs. combat

### 3C — Benchmarks
Two benchmark suites:
1. **Engine throughput**: games/sec for end-to-end simulation with GreedyStrategy vs. GreedyStrategy
2. **Solver throughput**: states-evaluated/sec (clone + legal_actions_abstracted + apply_action + is_terminal)

Target: engine throughput >= 2x pre-rules-engine-refactor baseline.

---

## Dependency Graph

```
Phase 0 (Shared Interfaces)
    ├──> Phase 1A (Rules: SBA loop, events)
    │        └──> Phase 2A (Rules: layers, card framework)
    │                 └──> Phase 3A (Rules: polish)
    │
    └──> Phase 1B (MCCFR: info sets, regret tables, training)
             └──> Phase 2B (MCCFR: abstraction, parallel, depth-limited)
                      └──> Phase 3B (MCCFR: polish)
                               │
                               └──> Phase 3C (Benchmarks — needs both)
```

Tracks A and B are independent after Phase 0, with one constraint: if Phase 1A changes the SBA/trigger loop behavior (which it will), Phase 1B must train against the post-1A engine. Any policies trained against the pre-1A engine are invalidated.

---

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Layered effects engine too slow for MCCFR traversal | Solver throughput drops | Opt-in dirty flags; "always recompute" path for solver |
| Info set space too large even with abstraction | OOM during training | Regret pruning (drop unvisited entries); configurable abstraction granularity |
| Rules engine changes invalidate trained policies | Wasted training compute | Version the engine; only retrain after Phase 1A stabilizes |
| `auto_tap_lands()` hides mana decisions | Sub-optimal GTO play | Accept as approximation for Phase 1-2; surface as Action in Phase 3+ if needed |
| Replacement effect ordering explodes action space | MCCFR branching too high | Cap permutations at 6 (720 orderings), FIFO fallback beyond that (same pattern as `OrderTriggers`) |

---

## What Not To Build

- **GUI/client** — out of scope, simulation-only
- **Multiplayer beyond 2 players** — MCCFR is designed for 2-player zero-sum
- **Full 28k-card parity** — target 100-200 cards that exercise the rules engine
- **Deep RL (Phase 4+)** — infrastructure first, neural nets later. MCCFR provides the info set framework, feature extraction, and baselines that deep RL needs.
- **Sideboarding / best-of-three** — single-game GTO is hard enough
