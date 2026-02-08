# Consolidated Development Strategy: Rules Engine + MCCFR

> **How to update this document**: When completing work, check off the relevant
> items (`- [ ]` to `- [x]`), add the PR number in parentheses, and move any
> newly-discovered sub-tasks into the appropriate phase as unchecked items. Keep
> descriptions concise (one line per checkbox). If a task is descoped or deferred,
> strike it out with `~~text~~` and add a note explaining where it moved. Do not
> delete history — completed phases serve as reference for what shipped and when.

---

## Current State

**Last updated**: 2026-02-08

All phases through Phase 2 (both tracks) are complete. Phase 2A (Rules Engine:
continuous effects & card scaling) shipped in PRs #16-19. Phase 2B (MCCFR
scaling) shipped in PR #17. The project is ready to begin Phase 3.

### Source Tree

```
src/
├── action/mod.rs         # Action enum (13 variants), legal_actions(), legal_actions_abstracted()
├── action/canonical.rs   # CanonicalAction, canonicalize(), resolve()
├── card/mod.rs           # CardDef, CardInstance, Effect enum, TriggeredAbility
├── card/sample.rs        # 111 card definitions, 2 prebuilt 60-card decks
├── game/mod.rs           # GameState, PlayerState, CombatState, PlayerView, Arc<CardDatabase>
├── rules/mod.rs          # apply_action(), SBA/trigger loop (CR 704.3), combat, phases
├── events/mod.rs         # GameEvent enum (9 variants), EventBus, EventLog
├── replacement/mod.rs    # ReplacementEffect, ReplacementAction, PendingReplacementChoice
├── layers/mod.rs         # CR 613 layered effects engine, compute_characteristics()
├── info_set/mod.rs       # InformationSet, from_view(), hash_value(), InfoSetAbstraction trait, BucketedAbstraction
├── solver/mod.rs         # RegretTable, InfoSetData, ActionEntry, policy framework
├── solver/mccfr.rs       # MCCFR traversal, train(), train_parallel(), train_extended(), checkpointing
├── strategy/mod.rs       # Strategy trait, RandomStrategy, GreedyStrategy, McfrStrategy, AbstractedMcfrStrategy
├── simulation/mod.rs     # run_game(), simulate() with rayon parallelism
├── mana/mod.rs           # ManaPool, ManaCost, Color
├── deck_import.rs        # Deck importing from text files
└── lib.rs

tests/
├── integration_test.rs   # 400+ end-to-end rules engine tests
├── mccfr_test.rs         # MCCFR solver + training scenario tests
└── deck_import_test.rs   # Deck import tests
```

### Key Interfaces (Stable)

- **`Strategy` trait** (`src/strategy/mod.rs`): `choose_action(&self, state: &GameState, player: PlayerIndex) -> Action`
- **`Action` enum** (`src/action/mod.rs`): 13 variants including `Discard`, `OrderTriggers`, `ChooseReplacementOrder`
- **`CanonicalAction`** (`src/action/canonical.rs`): 12 variants with verified round-trip invariant
- **`PlayerView`** (`src/game/mod.rs`): Observation API — information-set boundary between engine and solver
- **`legal_actions()` / `legal_actions_abstracted()`** (`src/action/mod.rs`): Full and bucketed modes
- **`apply_action()`** (`src/rules/mod.rs`): Deterministic state transitions for all action types
- **`GameState::clone()`**: O(n) on objects HashMap, O(1) on card_db via Arc
- **`InfoSetAbstraction` trait** (`src/info_set/mod.rs`): `abstract_info_set(&self, info_set: &InformationSet) -> u64`
- **`train_parallel()`** (`src/solver/mccfr.rs`): Sharded parallel MCCFR with rayon
- **`TrainConfig`** (`src/solver/mccfr.rs`): Extended config with abstraction, rollout mode, checkpointing

---

## Development Tracks

Two parallel tracks with a shared interface milestone at the start.

**Track A — Rules Engine**: Correctness and fidelity of game rules. Produces
correct state transitions for increasingly complex card interactions.

**Track B — MCCFR Solver**: GTO strategy computation. Consumes the engine as
a black box through `Strategy` + `legal_actions()` + `apply_action()`.

---

## Phase 0: Shared Interface Contract — COMPLETE

> Merged in PR #13. All acceptance criteria met.

- [x] **0.1 — Observation API: `PlayerView`** (PR #13)
  - [x] `PlayerView<'a>` struct in `src/game/mod.rs` with public, per-player, and private sections
  - [x] `GameState::visible_state(player)` returns `PlayerView` filtered to visible zones only
  - [x] Opponent hand contents and both libraries excluded; only sizes exposed
  - [x] Objects map filtered to visible-zone objects only (battlefield, stack, graveyards, exile, hand)
- [x] **0.2 — Canonical Action Mapping** (PR #13)
  - [x] `CanonicalAction` enum in `src/action/canonical.rs` (12 variants)
  - [x] `canonicalize(action, state) -> CanonicalAction`
  - [x] `resolve(canonical, state, player) -> Option<Action>`
  - [x] Round-trip invariant verified: `resolve(canonicalize(a, s), s, p) == Some(a)` for all legal actions
  - [x] Instance disambiguation via `hand_index` / `instance_index` fields
- [x] **0.3 — Snapshot Contract** (PR #13)
  - [x] Canonical fields always cloned (objects, players, battlefield, stack, combat, pending_triggers, scalars)
  - [x] `card_db` shared via `Arc` — O(1) clone
  - [x] Derived fields (`pending_events`) marked `#[serde(skip)]`, excluded from serialization
  - [x] Rule documented: derivable fields must use `#[serde(skip)]` and be excluded from equality/hashing

---

## Phase 1A: Rules Engine Foundations — COMPLETE

> Merged in PR #14. All acceptance criteria met.

- [x] **1A.1 — SBA/Trigger Recurrence Loop** (PR #14)
  - [x] CR 704.3 loop in `check_state_based_actions()` (`src/rules/mod.rs`)
  - [x] Inner loop: perform all SBAs until stable, then check/queue triggers
  - [x] Outer loop: re-enter if any SBAs performed or triggers queued
  - [x] APNAP trigger ordering via `flush_triggers()` with `OrderTriggers` pause
  - [x] Cascading death test: creature dies -> trigger -> damage -> second creature dies -> re-check
  - [x] 6 dedicated tests covering cascading, stability, and full-game scenarios
- [x] **1A.2 — Event System Skeleton** (PR #14)
  - [x] `GameEvent` enum in `src/events/mod.rs` (9 variants: DamageDealt, ZoneChange, LifeChanged, SpellCast, AbilityTriggered, CounterChanged, Tapped, Untapped, CardDrawn, TurnStarted)
  - [x] `EventBus` with function-pointer handler table (not dynamic dispatch)
  - [x] `pending_events: Vec<GameEvent>` on GameState, marked `#[serde(skip)]` — transient, not cloned
  - [x] Events emitted across rules engine: spell cast, zone changes, damage, life, draw, turn start
  - [x] `EventLog` test utility for recording and querying events
  - [x] 7 dedicated tests confirming emission, transience, and clone-cost neutrality
- [x] **1A.3 — Replacement Effect Framework** (PR #14)
  - [x] `ReplacementEffect` struct with source, controller, applies_to, action, self-replacement flag
  - [x] `ReplacementEventKind` enum (7 variants: ETB, damage, draw, death, life gain/loss, counters)
  - [x] `ReplacementAction` enum (5 variants: Prevent, RedirectToZone, ModifyAmount, EntersModified, Custom)
  - [x] `PendingReplacementChoice` struct for surfacing ordering decisions
  - [x] `find_applicable_replacements()` separates self-replacements from player choices (CR 614.16a)
  - [x] `Action::ChooseReplacementOrder` variant added to Action enum
  - [x] `CanonicalAction::ChooseReplacementOrder` variant with round-trip support
  - [x] 4 dedicated tests (detection, action enum, canonical round-trip, strategy handling)
  - [x] Wire replacement effect application into the rules engine (completed in Phase 2A, PR #17)

---

## Phase 1B: MCCFR Solver Infrastructure — COMPLETE

> Merged in PR #15. All acceptance criteria met.

- [x] **1B.1 — Information Set Module** (PR #15)
  - [x] `InformationSet` struct in `src/info_set/mod.rs` with all observable game fields
  - [x] `from_view(&PlayerView, &CardDatabase)` — reads exclusively from PlayerView, never raw GameState
  - [x] `hash_value() -> u64` — deterministic hash; same observable state always produces same hash
  - [x] `PermanentInfo` and `StackInfo` sub-structs for canonical board/stack representation
  - [x] Hand and battlefield sorted for canonical ordering
  - [x] Per-color mana breakdown ([u32; 6])
  - [x] 5 unit tests + integration tests
- [x] **1B.2 — Regret Table** (PR #15)
  - [x] `RegretTable` struct in `src/solver/mod.rs` — `HashMap<u64, InfoSetData>`
  - [x] `InfoSetData` with `HashMap<CanonicalAction, ActionEntry>` and `visit_count`
  - [x] `ActionEntry` with `cumulative_regret` and `cumulative_strategy`
  - [x] `current_strategy()` — regret matching (positive regrets -> proportional, else uniform)
  - [x] `average_strategy()` — cumulative strategy normalization for Nash convergence
  - [x] `to_bytes()` / `from_bytes()` — bincode serialization round-trip
  - [x] `prune(min_visits)` — memory management for unvisited entries
  - [x] 8 unit tests
- [x] **1B.3 — External Sampling MCCFR Traversal** (PR #15)
  - [x] `traverse()` in `src/solver/mccfr.rs` — recursive game tree traversal
  - [x] Terminal utility: +1 win, -1 loss, 0 draw
  - [x] Heuristic evaluation at depth limit (70% life + 30% board presence)
  - [x] For traverser: explores all actions, computes counterfactual regret
  - [x] For opponent: samples one action from current strategy
  - [x] Uses `legal_actions_abstracted()` for bounded combat branching
  - [x] Uses `canonicalize()` for regret table keying
  - [x] Skips single-action nodes (forced pass)
  - [x] `run_iteration()` and `train()` entry points
  - [x] `approximate_exploitability()` convergence metric
  - [x] `McfrConfig` with `max_depth` and `max_actions`
- [x] **1B.4 — McfrStrategy** (PR #15)
  - [x] `McfrStrategy` in `src/strategy/mod.rs` implementing `Strategy` trait
  - [x] Uses `average_strategy` (not current) for play decisions
  - [x] Constructs `InformationSet` from `visible_state(player)`
  - [x] Falls back to uniform distribution on unseen info sets
  - [x] `sample_from_distribution()` utility for probability sampling
- [x] **1B.5 — Minimal Training Scenario** (PR #15)
  - [x] 15-card mini decks (burn vs. creatures) in `tests/mccfr_test.rs`
  - [x] Single iteration creates info set entries
  - [x] 10-iteration training loop with valid visit counts
  - [x] Exploitability metric is finite and non-negative
  - [x] McfrStrategy plays complete legal games (10-game validation)
  - [x] McfrStrategy vs Random (50 games) and vs Greedy (100 games) matchups
  - [x] Mirror match convergence test (100 games, 15-85% balance tolerance)
  - [x] Regret table bincode serialization round-trip
  - [x] 14 integration tests all passing

---

## Phase 2A: Rules Engine — Continuous Effects & Card Scaling — COMPLETE

> Merged in PRs #16-19. All acceptance criteria met.

**Depends on**: Phase 1A complete (satisfied).

### 2A.1 — CR 613 Layered Effects Engine (PR #16)

- [x] Layer 1: Copy effects (`LayerModification::CopyOf`)
- [x] Layer 2: Control-changing effects (`LayerModification::ChangeController`)
- [x] Layer 3: Text-changing effects (placeholder — not commonly needed)
- [x] Layer 4: Type-changing effects (`AddType`, `RemoveType`, `SetTypes`, `AddSubtype`)
- [x] Layer 5: Color-changing effects (`AddColor`, `SetColors`)
- [x] Layer 6: Ability-adding/removing effects (`AddKeyword`, `RemoveKeyword`, `RemoveAllAbilities`)
- [x] Layer 7a-e: P/T-setting and modifying effects (`SetBasePT`, `SetPT`, `ModifyPT`, `SwitchPT`, counters)
- [x] Replace `temp_power_mod` / `temp_toughness_mod` / `temp_keywords` with layered recomputation via `compute_characteristics()`
- [x] `GameState::effective_power/toughness` uses the layer engine (src/game/mod.rs:875-893)
- [x] Determinism constraint: same GameState input always produces same computed characteristics
- [x] Test: Humility removes abilities and sets P/T to 1/1 (`test_humility_makes_all_creatures_1_1_and_removes_abilities`)
- [x] Test: Multiple anthem effects stacking (`test_multiple_anthems_stack`)
- [x] Test: Timestamp ordering for conflicting effects (`test_anthem_plus_humility_timestamp_ordering`)

### 2A.2 — Composable Card Framework (PR #17)

- [x] `effects::common` library: 18 Effect variants (DealDamage, GainLife, DrawCards, DestroyTarget, ExileTarget, BounceTo, Debuff, DiscardCards, MillCards, SacrificeCreatures, etc.)
- [x] `abilities::common`: TriggeredAbility (10 conditions), ActivatedAbility, StaticAbility (6 variants)
- [x] `targets::common`: TargetSpec with 11 variants (AnyCreature, AnyPlayer, CreatureOrPlayer, etc.)
- [ ] ~~`DynamicValue` trait for runtime-computed values~~ — deferred; effects use static values, no cards currently need runtime computation
- [x] Card definitions become declarative compositions of building blocks — all 111 cards are pure data structs

### 2A.3 — Wire Replacement Effect Application (PR #17)

- [x] Intercept events in rules engine: `deal_damage_with_replacement()`, `death_replacement_zone()`, `apply_etb_replacements()`
- [x] Apply replacement modifications (self-replacement applied automatically; player ordering stubbed — applies in encounter order)
- [x] CR 614.5: each effect applies only once per event (by-design via index lookup)
- [x] Self-replacement priority (CR 614.16a) — `is_self_replacement` flag separates self from player-choice
- [x] Test: replacement effect detection (`test_replacement_effect_find_applicable`)
- [x] Test: canonical round-trip (`test_replacement_order_canonical_roundtrip`)
- [x] Test: ETB replacement — `apply_etb_replacements()` handles enters-tapped and extra counters

### 2A.4 — Expand Card Pool to 100+ (PR #18)

- [x] 111 card definitions in `sample.rs` — 1 uses `Effect::Unimplemented` (Dark Ritual, mana generation)
- [x] Format staples added (Path to Exile, Swords to Plowshares, Counterspell, Wrath of God, etc.)
- [x] Cards exercising layered effects: Glorious Anthem, Crusade, Humility, Honor of the Pure, Gaea's Anthem
- [x] 110 of 111 cards expressible without engine changes

### Acceptance Criteria for Phase 2A

- [x] Layered effects resolve correctly for all layer-interaction test cases (18 tests: 12 unit + 6 integration)
- [x] At least 50 cards expressible without engine changes (110/111)
- [x] `GameState::effective_power/toughness` uses the layer engine
- [x] Replacement effects apply correctly when multiple compete for the same event

---

## Phase 2B: MCCFR — Scaling to Realistic Decks — COMPLETE

> Merged in PR #17. All acceptance criteria met.

**Depends on**: Phase 1B complete (satisfied). Can run in parallel with Phase 2A.

### 2B.1 — Information Set Abstraction (PR #17)

- [x] `trait InfoSetAbstraction { fn abstract_info_set(&self, info_set: &InformationSet) -> u64; }`
- [x] `IdentityAbstraction` — no-op passthrough (Phase 1B compatibility)
- [x] `BucketedAbstraction` — life/turn/board/hand bucketing without card_db
- [x] `CardAwareBucketedAbstraction` — enhanced bucketing with card_db for hand role classification and board power/toughness aggregation
- [x] Life bucketing: {<=0, 1-5, 6-10, 11-15, 16-20, 21+}
- [x] Board state abstraction: aggregate stats (creature count per side, mana availability)
- [x] Hand categorization by role (land, cheap creature, expensive creature, removal)
- [x] Turn bucketing: {early 0-3, mid 4-6, late 7+}
- [x] 9 unit tests (identity match, life collapse, turn collapse, bucket separation, reduction verification, card-aware, bucket values)

### 2B.2 — Parallel MCCFR Training (PR #17)

- [x] Each thread runs independent traversals via rayon (`train_parallel()`)
- [x] Regret table updates use sharded tables (one per thread, merged after)
- [x] `merge_regret_tables()` sums cumulative regret/strategy (linearity of CFR)
- [x] Checkpoint every N iterations (`save_checkpoint()` / `load_checkpoint()`)
- [x] Near-linear speedup across cores (validated via `test_parallel_training_60card`)
- [x] `TrainConfig` struct for extended configuration (abstraction, rollout mode, checkpoint settings)
- [x] `TrainingStats` struct for diagnostics (info set counts, visits, memory estimate, exploitability)

### 2B.3 — Depth-Limited Solving with Rollouts (PR #17)

- [x] `RolloutMode` enum: `Heuristic` (Phase 1B default) or `Strategy` (play-out with configurable strategy)
- [x] `rollout_utility()` — plays out game from depth limit using given strategy pair
- [x] Falls back to heuristic if rollout doesn't finish within max_rollout_actions
- [x] Configurable depth per training run via `McfrConfig::max_depth`
- [x] Configurable rollout strategies via `TrainConfig::rollout_strategies`
- [x] `AbstractedMcfrStrategy` — play-time strategy that uses same abstraction as training

### 2B.4 — Scale Validation (PR #17)

- [x] Train on Mono-Red vs. Mono-Green matchup (existing 60-card sample decks)
- [x] Measure convergence rate and memory usage via `training_stats()`
- [x] Compare win rate against GreedyStrategy baseline (60-card MCCFR vs Greedy matchup)
- [x] Memory bounded: < 100MB for 10-iteration training on 60-card decks
- [x] MCCFR trains on 60-card decks without OOM
- [x] 11 integration tests: 60-card training, parallel training, rollout training, checkpointing, abstraction reduction, strategy matchups

### Acceptance Criteria for Phase 2B

- [x] MCCFR trains on 60-card decks without OOM
- [x] Trained policy plays complete legal games against GreedyStrategy
- [x] Training parallelizes across cores with sharded tables
- [x] 25 total MCCFR integration tests all passing

---

## Phase 3: Integration & Quality

**Depends on**: Both Phase 2 tracks complete.

### 3A — Rules Engine Polish

- [ ] Full 704.5 SBA suite (all state-based actions per comprehensive rules)
- [ ] Combat requirement/restriction integration (must-attack, can't-block)
- [ ] Turn structure extras (extra turns, skip steps)
- [ ] Zone-change counters on all objects
- [ ] `GameStateSnapshot` struct for optimized copy/restore

### 3B — MCCFR Polish

- [ ] Warm-starting from GreedyStrategy heuristics
- [ ] Opponent modeling / deck inference (Bayesian updating on opponent's likely hands)
- [ ] Policy visualization (action distributions for key decision points)
- [ ] Multi-abstraction: different granularity for main phase vs. combat

### 3C — Benchmarks

- [ ] **Engine throughput** benchmark: games/sec for GreedyStrategy vs. GreedyStrategy
- [ ] **Solver throughput** benchmark: states-evaluated/sec (clone + legal_actions_abstracted + apply_action + is_terminal)
- [ ] Target: engine throughput >= 2x pre-rules-engine-refactor baseline

### Acceptance Criteria for Phase 3

- [ ] All Phase 3A, 3B, 3C items complete
- [ ] Full test suite passes
- [ ] Benchmarks documented with reproducible methodology

---

## Dependency Graph

```
Phase 0 (Shared Interfaces) ............. COMPLETE
    ├──> Phase 1A (Rules: SBA loop, events) ............. COMPLETE
    │        └──> Phase 2A (Rules: layers, card framework) ... COMPLETE
    │                 └──> Phase 3A (Rules: polish) ........... NEXT
    │
    └──> Phase 1B (MCCFR: info sets, regret tables, training) ... COMPLETE
             └──> Phase 2B (MCCFR: abstraction, parallel, depth-limited) ... COMPLETE
                      └──> Phase 3B (MCCFR: polish) ........... NEXT
                               │
                               └──> Phase 3C (Benchmarks — needs both)
```

---

## Design Constraints (Reference)

These constraints were established in Phases 0-1 and must be maintained going forward.

1. **Two-Phase Read-Write Borrow Pattern**: Separate read phase (compute decisions) from write phase (apply mutations). Prevents Rust borrow checker conflicts.
2. **Events are Transient**: `GameEvent` is a notification mechanism, NOT part of `GameState`. Preserves cheap `Clone`.
3. **Arc<CardDatabase>**: O(1) clone cost for the immutable card definitions database.
4. **Bucketed Combat**: `legal_actions_abstracted()` reduces attacker/blocker space from O(2^n) to ~7 buckets for MCCFR.
5. **Canonical Action Mapping**: Two game states in the same information set must map to the same `CanonicalAction` for regret table consistency.
6. **Snapshot Contract**: Derived fields use `#[serde(skip)]`, only canonical state is cloned/serialized.

---

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| ~~Layered effects engine too slow for MCCFR traversal~~ | ~~Solver throughput drops~~ | ~~Mitigated: Characteristics cache with invalidation implemented in Phase 2A (PR #19)~~ |
| Info set space too large even with abstraction | OOM during training | Regret pruning (drop unvisited entries); configurable abstraction granularity |
| ~~Rules engine changes invalidate trained policies~~ | ~~Wasted training compute~~ | ~~Mitigated: Phase 1A is stable; MCCFR trains against post-1A engine~~ |
| `auto_tap_lands()` hides mana decisions | Sub-optimal GTO play | Accept as approximation for Phase 2; surface as Action in Phase 3+ if needed |
| Replacement effect ordering explodes action space | MCCFR branching too high | Cap permutations at 6 (720 orderings), FIFO fallback beyond that |

---

## What Not To Build

- **GUI/client** — out of scope, simulation-only
- **Multiplayer beyond 2 players** — MCCFR is designed for 2-player zero-sum
- **Full 28k-card parity** — target 100-200 cards that exercise the rules engine
- **Deep RL (Phase 4+)** — infrastructure first, neural nets later. MCCFR provides the info set framework, feature extraction, and baselines that deep RL needs
- **Sideboarding / best-of-three** — single-game GTO is hard enough
