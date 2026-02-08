# Rules Engine Proposal: Fully-Featured, Testable, Performance-Optimized

*Synthesized from the [Claude xMage review](claude-xmage-rules-engine-review.md), the [Codex xMage review](codex-xmage-rules-engine-review.md), and a full audit of the current mtg-gto codebase.*

---

## Table of Contents

1. [Goals & Constraints](#1-goals--constraints)
2. [Current Architecture Summary](#2-current-architecture-summary)
3. [Proposed Architecture](#3-proposed-architecture)
4. [Module-by-Module Design](#4-module-by-module-design)
   - [4.1 Event Bus](#41-event-bus)
   - [4.2 Continuous Effects & Layer System](#42-continuous-effects--layer-system)
   - [4.3 Replacement Effects](#43-replacement-effects)
   - [4.4 Priority Loop & SBA Engine](#44-priority-loop--sba-engine)
   - [4.5 Effect & Ability Trait System](#45-effect--ability-trait-system)
   - [4.6 Targeting & Filters](#46-targeting--filters)
   - [4.7 Card Composition Framework](#47-card-composition-framework)
   - [4.8 Watcher System](#48-watcher-system)
   - [4.9 Game State Optimizations](#49-game-state-optimizations)
5. [Edge-Case Testing Strategy](#5-edge-case-testing-strategy)
6. [Performance Optimization Strategy](#6-performance-optimization-strategy)
7. [Implementation Roadmap](#7-implementation-roadmap)
8. [Risk Analysis](#8-risk-analysis)

---

## 1. Goals & Constraints

### Goals

| Goal | Description |
|------|-------------|
| **Fully-featured rules engine** | Correct implementation of MTG Comprehensive Rules: layers, replacement effects, SBAs, priority, stack, combat, zones, triggers. Sufficient to handle any Standard/Modern-legal card. |
| **Testable for all edge cases** | Every rules subsystem can be unit-tested in isolation. Edge cases (e.g., "Humility + Opalescence", timestamp-dependent layer interactions, replacement effect chains) have dedicated test cards that exercise specific rules corners. |
| **Performance-optimized for GTO simulation** | State cloning, action enumeration, and effect recalculation must be fast enough for millions of MCTS/CFR rollouts. Target: <10μs per state clone, <100μs per `apply_effects()` call with typical board states. |

### Constraints

- **Rust** — We use Rust's ownership model for safety and performance. No garbage collection pauses during simulation.
- **Single-binary, no network** — Unlike xMage's client-server model, we run everything in-process. Player decisions come from `Strategy` trait implementations, not network calls.
- **Clone-heavy AI search** — Our MCTS/CFR approach clones `GameState` millions of times. Every byte added to `GameState` has a multiplicative cost. Minimizing state size is a first-class concern.
- **No UI** — We don't need rendering hints, icons, or display text on effects. We can strip everything xMage carries for its GUI.

### Design Principles (from both reviews)

Both the Claude and Codex reviews converge on the same core principles:

1. **Strict engine/card separation** — The engine provides primitives; cards compose them. Card code never touches engine internals.
2. **Cards are data, not behavior** — Card definitions are declarative. The engine interprets them.
3. **Event-driven architecture** — Actions emit events; triggers, replacements, and watchers react. This decouples "what happens" from "what reacts."
4. **Layer-ordered determinism** — Continuous effects recalculated from scratch in strict layer order. No mutable caching of intermediate state.
5. **Immutable-read, batched-write** — The pattern we already use in combat damage. Extend it everywhere to avoid borrow conflicts and enable rollback.

---

## 2. Current Architecture Summary

### What We Have (Strengths)

| Component | Status | Notes |
|-----------|--------|-------|
| **Turn structure** | 13-phase flat enum | Correct phase ordering, auto-advance for untap/cleanup |
| **Stack** | Working | Push/resolve/counter for spells, activated, and triggered abilities |
| **Mana system** | Working | 5-color + colorless, auto-tap heuristic, cost parsing |
| **Combat** | Working | Attackers, blockers, first/double strike, deathtouch, trample, lifelink, evasion (flying/fear/intimidate/menace) |
| **Triggered abilities** | Basic | ETB, dies, attacks, upkeep, end-of-turn with APNAP ordering |
| **Card definitions** | 23 sample cards | Clean `CardDef`/`CardInstance` split, 12-variant `Effect` enum |
| **Action enumeration** | Working | Full legal action generation with attacker/blocker subset enumeration (capped at 2^10) |
| **Simulation harness** | Working | Parallel Monte Carlo via Rayon, `Strategy` trait for AI |
| **State cloning** | Working | `GameState` derives `Clone`; designed for MCTS |
| **Test suite** | 12 tests | Mana unit tests, integration tests for game flow, ETB trigger test |

### What We're Missing (Gaps)

| Gap | Severity | Blocking |
|-----|----------|----------|
| **Continuous effects / layer system** | Critical | Enchantments, equipment, anthems, Humility, Opalescence — all wrong or impossible |
| **Event system** | Critical | Foundation for replacement effects, triggers, watchers |
| **Replacement effects** | Critical | "Enters tapped", "if this would die, exile instead", damage prevention |
| **SBA/trigger recurrence loop** | High | Can miss triggers; illegal states possible |
| **Composable targeting/filters** | High | Multi-target, modal spells, filtered targeting not possible |
| **Dynamic values** | High | Tarmogoyf, "damage equal to creature count", X spells |
| **Token creation** | Medium | `CreateToken` effect is stubbed |
| **Fizzle checking** | Medium | Targets not re-validated on resolution |
| **Zone change counters** | Medium | Effects can't detect stale object references |
| **Delayed triggers** | Medium | "At beginning of next end step, sacrifice it" |
| **Legend rule, aura SBAs** | Medium | Multiple legendaries, unattached auras not handled |
| **Watchers** | Medium | No derived state tracking (storm count, "this turn" tracking) |

---

## 3. Proposed Architecture

### High-Level Module Map

```
src/
├── lib.rs                  # Crate root
├── main.rs                 # Entry point
├── mana/mod.rs             # [KEEP] ManaCost, ManaPool, Color
├── card/
│   ├── mod.rs              # [EVOLVE] CardDef, CardInstance, types, keywords
│   ├── sample.rs           # [KEEP] Sample card database
│   ├── effects.rs          # [NEW] Effect trait + common implementations
│   ├── abilities.rs        # [NEW] Ability trait hierarchy (spell, activated, triggered, static)
│   ├── targets.rs          # [NEW] Target/Filter composable predicate system
│   └── dynamic_value.rs    # [NEW] DynamicValue trait + common implementations
├── game/
│   ├── mod.rs              # [EVOLVE] GameState, PlayerState, zones
│   ├── event.rs            # [NEW] GameEvent enum + EventBus
│   ├── continuous.rs        # [NEW] ContinuousEffects manager + Layer system
│   ├── replacement.rs       # [NEW] Replacement effect engine
│   └── watcher.rs          # [NEW] Watcher trait + registry
├── action/mod.rs           # [EVOLVE] Action enum, legal_actions()
├── rules/mod.rs            # [EVOLVE] apply_action(), priority loop, SBAs, combat, stack
├── strategy/mod.rs         # [KEEP] Strategy trait, Random, Greedy
├── simulation/mod.rs       # [KEEP] run_game(), simulate()
└── deck_import.rs          # [KEEP] Deck file parsing
```

### Data Flow

```
Player Action
    │
    ▼
rules::apply_action()
    │
    ├── Pre-event fired ──► replacement::check() ──► modify/cancel
    │
    ├── Action executed (state mutated)
    │
    ├── Post-event fired ──► watcher::observe()
    │                    ──► trigger::check() ──► pending_triggers queue
    │
    ├── continuous::apply_effects() ──► reset + layer 1..7e reapply
    │
    └── rules::check_state_and_triggered() ──► SBA loop + trigger flush
            │
            ▼
        Next player receives priority
```

---

## 4. Module-by-Module Design

### 4.1 Event Bus

**File**: `src/game/event.rs`

The event system is the foundation for replacement effects, triggers, and watchers. We define a typed event enum and a lightweight bus that routes events.

```rust
/// Every game action that other systems may observe or intercept.
#[derive(Debug, Clone)]
pub enum GameEvent {
    // Zone changes
    ZoneChange { object_id: ObjectId, from: ZoneType, to: ZoneType, source: ObjectId },
    EntersBattlefield { object_id: ObjectId, controller: PlayerIndex },
    Dies { object_id: ObjectId, owner: PlayerIndex },

    // Damage
    DamageDealt { source: ObjectId, target: Target, amount: u32, is_combat: bool },

    // Life
    LifeGained { player: PlayerIndex, amount: u32, source: ObjectId },
    LifeLost { player: PlayerIndex, amount: u32, source: ObjectId },

    // Spells and abilities
    SpellCast { object_id: ObjectId, controller: PlayerIndex },
    AbilityActivated { source: ObjectId, ability_index: usize, controller: PlayerIndex },
    AbilityTriggered { source: ObjectId, ability_index: usize, controller: PlayerIndex },
    SpellCountered { object_id: ObjectId, source: ObjectId },

    // Combat
    AttackerDeclared { attacker: ObjectId, defending: PlayerIndex },
    BlockerDeclared { blocker: ObjectId, attacker: ObjectId },

    // Permanents
    Tapped { object_id: ObjectId },
    Untapped { object_id: ObjectId },
    CounterAdded { object_id: ObjectId, counter_type: CounterType, amount: u32 },
    CounterRemoved { object_id: ObjectId, counter_type: CounterType, amount: u32 },

    // Tokens
    TokenCreated { object_id: ObjectId, controller: PlayerIndex },

    // Turn structure
    TurnBegin { player: PlayerIndex, turn_number: u32 },
    PhaseBegin { phase: Phase },
    PhaseEnd { phase: Phase },

    // Drawing
    CardDrawn { player: PlayerIndex, object_id: ObjectId },
    CardDiscarded { player: PlayerIndex, object_id: ObjectId },
}
```

**Pre/post pattern**: Rather than doubling the enum, we use a phase tag:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventPhase { Pre, Post }

/// Fired event with metadata.
#[derive(Debug, Clone)]
pub struct FiredEvent {
    pub event: GameEvent,
    pub phase: EventPhase,
    pub applied_replacements: Vec<ObjectId>,  // Rule 614.5 tracking
}
```

**Design rationale**: A flat enum with `#[derive(Clone)]` is cheaply cloneable and doesn't require heap allocation. The `applied_replacements` vec is typically empty or very small (1-2 entries), so the overhead is minimal.

### 4.2 Continuous Effects & Layer System

**File**: `src/game/continuous.rs`

This is the biggest new subsystem. It implements MTG Comprehensive Rules 613.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    Copy,                     // 1a
    FaceDown,                 // 1b
    Control,                  // 2
    Text,                     // 3
    Type,                     // 4
    Color,                    // 5
    AbilityAddRemove,         // 6
    PtCharacteristicDefining, // 7a
    PtSet,                    // 7b
    PtModify,                 // 7c
    PtCounters,               // 7d
    PtSwitch,                 // 7e
}

/// A single active continuous effect.
#[derive(Debug, Clone)]
pub struct ActiveEffect {
    pub source_id: ObjectId,
    pub effect_id: u64,          // unique ID for dependency tracking
    pub layer: Layer,
    pub timestamp: u64,          // monotonically increasing
    pub duration: Duration,
    pub apply: ContinuousEffectKind,
}

#[derive(Debug, Clone)]
pub enum Duration {
    WhileOnBattlefield,    // source must be on battlefield
    UntilEndOfTurn,
    UntilYourNextTurn,
    UntilEndOfCombat,
    Permanent,             // e.g., +1/+1 counters (handled differently, but modeled)
    Custom(u64),           // keyed to a condition check
}

#[derive(Debug, Clone)]
pub enum ContinuousEffectKind {
    // Layer 6: ability modification
    AddKeyword { target: EffectTarget, keyword: KeywordAbility },
    RemoveKeyword { target: EffectTarget, keyword: KeywordAbility },
    RemoveAllAbilities { target: EffectTarget },
    AddAbility { target: EffectTarget, ability: Box<TriggeredAbility> },

    // Layer 7b: set P/T
    SetPowerToughness { target: EffectTarget, power: DynValue, toughness: DynValue },

    // Layer 7c: modify P/T
    ModifyPowerToughness { target: EffectTarget, power: DynValue, toughness: DynValue },

    // Layer 7e: switch P/T
    SwitchPowerToughness { target: EffectTarget },

    // Layer 2: control change
    ChangeControl { target: EffectTarget, new_controller: PlayerIndex },

    // Layer 4: type change
    AddType { target: EffectTarget, card_type: CardType },
    RemoveType { target: EffectTarget, card_type: CardType },

    // Layer 5: color change
    AddColor { target: EffectTarget, color: Color },
    SetColor { target: EffectTarget, colors: Vec<Color> },

    // Combat restrictions/requirements
    CantAttack { target: EffectTarget },
    CantBlock { target: EffectTarget },
    MustAttack { target: EffectTarget },
    MustBlock { target: EffectTarget, must_block: Option<ObjectId> },

    // Cost modification
    ReduceCost { filter: SpellFilter, reduction: ManaCost },
    IncreaseCost { filter: SpellFilter, increase: ManaCost },

    // AsThough permissions
    AsThoughHasFlash { filter: SpellFilter },
    AsThoughPlayFromZone { player: PlayerIndex, zone: ZoneType, filter: SpellFilter },
}

/// What objects an effect applies to.
#[derive(Debug, Clone)]
pub enum EffectTarget {
    Source,                              // the card that created this effect
    Permanent(ObjectId),                 // a specific permanent
    AllMatching(PermanentFilter),        // all permanents matching a filter
    Controller(PlayerIndex),             // player-level effect
}

/// The continuous effects manager.
#[derive(Debug, Clone)]
pub struct ContinuousEffects {
    effects: Vec<ActiveEffect>,
    next_timestamp: u64,
    next_effect_id: u64,
}
```

**The `apply_effects()` algorithm:**

```rust
impl ContinuousEffects {
    /// Recalculate all game state from base values + effects in layer order.
    /// Called after every action and before every priority decision.
    pub fn apply_effects(&self, state: &mut GameState) {
        // 1. Remove expired effects
        self.prune_expired(state);

        // 2. Reset all permanents to base (printed) characteristics
        for obj_id in &state.battlefield {
            let inst = state.objects.get_mut(obj_id).unwrap();
            inst.reset_to_base();
        }

        // 3. Apply effects in layer order
        for layer in Layer::all() {
            let mut layer_effects: Vec<&ActiveEffect> = self.effects
                .iter()
                .filter(|e| e.layer == layer)
                .collect();

            // Sort by timestamp within layer (Rule 613.7)
            layer_effects.sort_by_key(|e| e.timestamp);

            // TODO: dependency resolution for same-layer effects (Rule 613.8)

            for effect in layer_effects {
                effect.apply(state);
            }
        }

        // 4. Apply counter-based P/T (Layer 7d) from actual counters
        for obj_id in &state.battlefield {
            let inst = state.objects.get_mut(obj_id).unwrap();
            // +1/+1 and -1/-1 counters apply here
            inst.apply_counter_pt();
        }
    }
}
```

**Performance note**: This function runs on every priority check. With a typical board of 10-20 permanents and 5-10 active effects, the overhead is small. We should profile but this is unlikely to be a bottleneck compared to action enumeration.

**`CardInstance::reset_to_base()`**: A new method that clears all computed fields back to what the `CardDef` specifies — clearing temp mods, resetting keyword sets, etc. This is the "wipe slate clean" step that makes layers work correctly.

### 4.3 Replacement Effects

**File**: `src/game/replacement.rs`

Replacement effects intercept pre-events and modify them before the action executes. This implements Rules 614-616.

```rust
/// A registered replacement effect.
#[derive(Debug, Clone)]
pub struct ReplacementEffect {
    pub source_id: ObjectId,
    pub id: u64,
    pub applies_to: ReplacementCondition,
    pub replacement: ReplacementAction,
    pub self_only: bool,  // true for self-replacement (e.g., "enters tapped")
}

#[derive(Debug, Clone)]
pub enum ReplacementCondition {
    /// "If ~ would enter the battlefield..."
    EntersBattlefield { filter: Option<PermanentFilter> },
    /// "If damage would be dealt to..."
    DamageDealt { target_filter: Option<TargetFilter> },
    /// "If a creature would die..."
    WouldDie { filter: Option<PermanentFilter> },
    /// "If you would draw a card..."
    WouldDraw { player: Option<PlayerIndex> },
    /// "If you would gain life..."
    WouldGainLife { player: Option<PlayerIndex> },
    /// Generic: matches any event passing a custom predicate
    Custom(u64),  // keyed to a registered check function
}

#[derive(Debug, Clone)]
pub enum ReplacementAction {
    /// Modify the event (e.g., "enters tapped" adds a flag)
    EnterTapped,
    /// Redirect to a different zone (e.g., "exile instead")
    RedirectZone(ZoneType),
    /// Prevent the event entirely
    Prevent,
    /// Modify the amount (e.g., "prevent 3 damage")
    ReduceAmount(u32),
    /// Double the amount (e.g., "if you would gain life, gain twice that much")
    DoubleAmount,
    /// Custom replacement logic (keyed to registered function)
    Custom(u64),
}

/// The replacement effect engine.
#[derive(Debug, Clone)]
pub struct ReplacementEngine {
    effects: Vec<ReplacementEffect>,
    next_id: u64,
}

impl ReplacementEngine {
    /// Check and apply replacement effects to a pre-event.
    /// Returns the (possibly modified) event, or None if prevented entirely.
    pub fn apply(&self, event: &mut FiredEvent, state: &GameState) -> bool {
        loop {
            let applicable: Vec<&ReplacementEffect> = self.effects
                .iter()
                .filter(|r| r.matches(&event.event)
                    && !event.applied_replacements.contains(&r.source_id))
                .collect();

            if applicable.is_empty() {
                return true; // event proceeds
            }

            // Rule 616.1c: affected player chooses if multiple apply
            // For simulation: apply self-replacement first, then by timestamp
            let chosen = if applicable.len() == 1 {
                applicable[0]
            } else {
                self.choose_replacement(&applicable, state)
            };

            // Mark as applied (Rule 614.5)
            event.applied_replacements.push(chosen.source_id);

            // Apply the replacement
            if !chosen.apply(&mut event.event) {
                return false; // event prevented
            }
        }
    }
}
```

**Integration with the engine**: Every game action in `rules/mod.rs` that mutates state will be wrapped:

```rust
// Before:
fn deal_damage(state: &mut GameState, source: ObjectId, target: Target, amount: u32) {
    // directly apply damage
}

// After:
fn deal_damage(state: &mut GameState, source: ObjectId, target: Target, amount: u32) {
    let mut event = FiredEvent::pre(GameEvent::DamageDealt { source, target, amount, is_combat: false });

    if !state.replacements.apply(&mut event, state) {
        return; // damage prevented
    }

    let actual_amount = event.amount(); // may have been modified
    // apply the damage

    state.fire_post_event(GameEvent::DamageDealt { source, target, amount: actual_amount, is_combat: false });
}
```

### 4.4 Priority Loop & SBA Engine

**File**: Changes to `src/rules/mod.rs`

The current priority system uses a `consecutive_passes` counter. We need to evolve it into the full SBA+trigger recurrence loop.

**Current** (simplified):
```
player passes → increment counter → if both passed → resolve or advance
```

**Proposed**:
```rust
/// The core priority loop. Called at every point where players get priority.
fn play_priority(state: &mut GameState) {
    state.continuous.apply_effects(state);

    loop {
        // Reset pass flags
        for p in &mut state.players {
            p.passed = false;
        }

        loop {
            let player = state.priority_player;

            // SBA + trigger recurrence (Rule 704.3)
            loop {
                let sba_happened = check_state_based_actions(state);
                let triggers_fired = check_and_flush_triggers(state);
                if !sba_happened && !triggers_fired {
                    break;
                }
                state.continuous.apply_effects(state);
            }

            // Player gets priority — strategy chooses action
            // (In simulation, this returns to the game loop for the strategy to decide)
            // If the player passes, mark passed and continue to next player
            // If the player acts, reset all pass flags and loop
            return; // yield to the game loop
        }
    }
}
```

Since our simulation loop is externally driven (the game loop calls `legal_actions()` then `apply_action()`), we integrate the recurrence into `apply_action()`:

```rust
pub fn apply_action(state: &mut GameState, action: &Action) {
    match action {
        Action::PassPriority => handle_priority_pass(state),
        _ => {
            // Execute the action
            execute_action(state, action);

            // After any non-pass action:
            state.consecutive_passes = 0;
            state.continuous.apply_effects(state);

            // SBA + trigger recurrence before yielding priority
            run_sba_trigger_loop(state);
        }
    }
}

fn run_sba_trigger_loop(state: &mut GameState) {
    loop {
        let sba_happened = check_state_based_actions(state);
        let triggers_fired = check_and_flush_triggers(state);
        if !sba_happened && !triggers_fired {
            break;
        }
        state.continuous.apply_effects(state);
    }
}
```

**Extended SBAs** to add:

```rust
fn check_state_based_actions(state: &mut GameState) -> bool {
    let mut changed = false;
    loop {
        let mut this_pass = false;

        // Existing: life <= 0, lethal damage, empty library
        // ...

        // NEW: Legend rule (704.5j)
        this_pass |= check_legend_rule(state);

        // NEW: Planeswalker loyalty <= 0 (704.5i)
        this_pass |= check_planeswalker_loyalty(state);

        // NEW: Unattached aura (704.5n)
        this_pass |= check_unattached_auras(state);

        // NEW: +1/+1 and -1/-1 counter annihilation (704.5q)
        this_pass |= check_counter_annihilation(state);

        // NEW: Token not on battlefield (704.5d)
        this_pass |= check_tokens_off_battlefield(state);

        // NEW: Deathtouch damage (704.5h)
        this_pass |= check_deathtouch_damage(state);

        if !this_pass { break; }
        changed = true;
    }
    changed
}
```

### 4.5 Effect & Ability Trait System

**File**: `src/card/effects.rs`, `src/card/abilities.rs`

The current `Effect` enum has 12 variants. This works well for simple effects but doesn't scale to continuous effects, dynamic values, or custom card logic. We evolve it into a **trait-based system with an enum fast path**.

```rust
/// Core effect trait — anything that can mutate game state when resolved.
pub trait EffectImpl: std::fmt::Debug + Clone + Send + Sync {
    fn resolve(&self, state: &mut GameState, source: ObjectId, targets: &[Target]);
}

/// The Effect enum wraps common cases for cheap cloning and pattern matching,
/// with a Custom variant for trait objects.
#[derive(Debug, Clone)]
pub enum Effect {
    // === One-shot effects (resolve once) ===
    DealDamage { amount: DynValue, target: TargetSpec },
    GainLife { amount: DynValue },
    LoseLife { amount: DynValue, target: TargetSpec },
    DrawCards { count: DynValue },
    DestroyTarget { target: TargetSpec },
    DestroyAll { filter: PermanentFilter, no_regen: bool },
    ExileTarget { target: TargetSpec },
    BounceTo { zone: ZoneType, target: TargetSpec },
    Buff { power: DynValue, toughness: DynValue, duration: Duration },
    DiscardCards { count: DynValue, target: TargetSpec },
    CreateToken { token: TokenDef, count: DynValue },
    Counter { target: TargetSpec },
    SearchLibrary { filter: CardFilter, destination: ZoneType },
    MillCards { count: DynValue, target: TargetSpec },
    Sacrifice { filter: PermanentFilter, count: DynValue },
    Multiple(Vec<Effect>),

    // === Continuous effects (persist over time) ===
    ContinuousModify { kind: ContinuousEffectKind, duration: Duration },

    // === Custom escape hatch ===
    Custom(Arc<dyn EffectImpl>),

    // === Placeholder for unimplemented cards ===
    Unimplemented(String),
}
```

**DynValue — Dynamic Values:**

```rust
/// A value that can be static or computed at resolution time.
#[derive(Debug, Clone)]
pub enum DynValue {
    /// Fixed integer
    Static(i32),
    /// Count of permanents matching a filter
    PermanentCount(PermanentFilter),
    /// Cards in a zone matching a filter
    CardCount { zone: ZoneType, player: PlayerRef, filter: CardFilter },
    /// Card types in graveyards
    CardTypesInGraveyards { player: PlayerRef },
    /// X value from mana cost
    XValue,
    /// Power of a specific creature
    PowerOf(ObjectRef),
    /// Toughness of a specific creature
    ToughnessOf(ObjectRef),
    /// Life total of a player
    LifeTotal(PlayerRef),
    /// Custom computation
    Custom(Arc<dyn Fn(&GameState, ObjectId) -> i32 + Send + Sync>),
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerRef { Controller, Opponent, Active, Player(PlayerIndex) }

impl DynValue {
    pub fn evaluate(&self, state: &GameState, source: ObjectId) -> i32 { ... }
}
```

**Ability Hierarchy:**

```rust
/// Static abilities generate continuous effects while on the battlefield.
#[derive(Debug, Clone)]
pub struct StaticAbility {
    pub effects: Vec<ContinuousEffectKind>,
    pub condition: Option<Condition>,
}

/// Triggered abilities respond to events.
#[derive(Debug, Clone)]
pub struct TriggeredAbility {
    pub trigger: TriggerCondition,
    pub effect: Effect,
    pub target: Option<TargetSpec>,
    pub condition: Option<Condition>,
    pub description: String,
}

/// Activated abilities have costs and go on the stack.
#[derive(Debug, Clone)]
pub struct ActivatedAbility {
    pub cost: AbilityCost,
    pub effect: Effect,
    pub target: Option<TargetSpec>,
    pub timing: AbilityTiming,  // Sorcery speed or instant speed
    pub description: String,
}

/// Comprehensive cost modeling
#[derive(Debug, Clone)]
pub struct AbilityCost {
    pub mana: Option<ManaCost>,
    pub tap: bool,
    pub sacrifice: Option<PermanentFilter>,
    pub discard: Option<(u32, CardFilter)>,
    pub pay_life: Option<u32>,
    pub exile_from_graveyard: Option<(u32, CardFilter)>,
    pub loyalty: Option<i32>,  // +N or -N for planeswalkers
}
```

### 4.6 Targeting & Filters

**File**: `src/card/targets.rs`

Replace the flat `TargetSpec` enum with a composable filter system.

```rust
/// What a spell/ability can target.
#[derive(Debug, Clone)]
pub enum TargetSpec {
    NoTarget,
    Single(TargetFilter),
    Multiple { filter: TargetFilter, count: DynValue },
    UpTo { filter: TargetFilter, max: u32 },
    DividedDamage { filter: TargetFilter, total: DynValue, min_targets: u32 },
}

/// Composable predicate for what constitutes a legal target.
#[derive(Debug, Clone)]
pub enum TargetFilter {
    // Base filters
    AnyCreature,
    AnyPlayer,
    AnyPermanent,
    AnySpell,

    // Refined filters
    PermanentMatching(PermanentFilter),
    PlayerMatching(PlayerFilter),
    CardInZone { zone: ZoneType, filter: CardFilter },

    // Combinators
    Or(Box<TargetFilter>, Box<TargetFilter>),
    And(Box<TargetFilter>, Box<TargetFilter>),
    Not(Box<TargetFilter>),
}

/// Filter for permanents on the battlefield.
#[derive(Debug, Clone)]
pub enum PermanentFilter {
    Any,
    ControlledBy(PlayerRef),
    NotControlledBy(PlayerRef),
    HasType(CardType),
    HasSubtype(Subtype),
    HasKeyword(KeywordAbility),
    HasColor(Color),
    PowerLessOrEqual(DynValue),
    ToughnessLessOrEqual(DynValue),
    CmcLessOrEqual(DynValue),
    IsToken,
    IsNonToken,
    And(Box<PermanentFilter>, Box<PermanentFilter>),
    Or(Box<PermanentFilter>, Box<PermanentFilter>),
    Not(Box<PermanentFilter>),
}

/// Filter for cards (in any zone).
#[derive(Debug, Clone)]
pub enum CardFilter {
    Any,
    HasType(CardType),
    HasSubtype(Subtype),
    CmcLessOrEqual(u32),
    CmcEquals(u32),
    HasColor(Color),
    NameEquals(String),
    And(Box<CardFilter>, Box<CardFilter>),
    Or(Box<CardFilter>, Box<CardFilter>),
    Not(Box<CardFilter>),
}
```

**Fizzle checking** (R12.2): Add to `resolve_top_of_stack()`:

```rust
fn resolve_top_of_stack(state: &mut GameState) {
    let entry = state.stack.pop().unwrap();

    // Re-validate targets
    if !entry.targets.is_empty() {
        let all_illegal = entry.targets.iter().all(|t| !is_legal_target(state, &entry, t));
        if all_illegal {
            // Spell fizzles — move to graveyard without resolving
            if let StackSource::Spell(obj_id) = entry.source {
                move_to_graveyard(state, obj_id);
            }
            return;
        }
    }

    // ... resolve as before, skipping illegal targets
}
```

### 4.7 Card Composition Framework

**How cards are defined with the new system:**

```rust
// Lightning Bolt — simple instant
pub fn lightning_bolt() -> CardDef {
    CardDef {
        name: "Lightning Bolt".into(),
        mana_cost: Some(ManaCost::parse("{R}")),
        card_types: vec![CardType::Instant],
        spell_effect: Some(Effect::DealDamage {
            amount: DynValue::Static(3),
            target: TargetSpec::Single(TargetFilter::Or(
                Box::new(TargetFilter::AnyCreature),
                Box::new(TargetFilter::AnyPlayer),
            )),
        }),
        ..CardDef::default()
    }
}

// Tarmogoyf — dynamic P/T
pub fn tarmogoyf() -> CardDef {
    CardDef {
        name: "Tarmogoyf".into(),
        mana_cost: Some(ManaCost::parse("{1}{G}")),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Lhurgoyf".into())],
        power: Some(0),
        toughness: Some(1),
        static_abilities: vec![StaticAbility {
            effects: vec![ContinuousEffectKind::SetPowerToughness {
                target: EffectTarget::Source,
                power: DynValue::CardTypesInGraveyards { player: PlayerRef::All },
                toughness: DynValue::Custom(Arc::new(|state, source| {
                    // toughness = card types in graveyards + 1
                    count_card_types_in_graveyards(state) + 1
                })),
            }],
            condition: None,
        }],
        ..CardDef::default()
    }
}

// Wrath of God — board wipe
pub fn wrath_of_god() -> CardDef {
    CardDef {
        name: "Wrath of God".into(),
        mana_cost: Some(ManaCost::parse("{2}{W}{W}")),
        card_types: vec![CardType::Sorcery],
        spell_effect: Some(Effect::DestroyAll {
            filter: PermanentFilter::HasType(CardType::Creature),
            no_regen: true,
        }),
        ..CardDef::default()
    }
}

// Snapcaster Mage — ETB trigger with targeting
pub fn snapcaster_mage() -> CardDef {
    CardDef {
        name: "Snapcaster Mage".into(),
        mana_cost: Some(ManaCost::parse("{1}{U}")),
        card_types: vec![CardType::Creature],
        subtypes: vec![Subtype("Human".into()), Subtype("Wizard".into())],
        keywords: vec![KeywordAbility::Flash],
        power: Some(2),
        toughness: Some(1),
        triggered_abilities: vec![TriggeredAbility {
            trigger: TriggerCondition::EntersBattlefield,
            effect: Effect::ContinuousModify {
                kind: ContinuousEffectKind::AddAbility {
                    target: EffectTarget::Permanent(/* resolved at trigger time */),
                    ability: Box::new(/* flashback ability */),
                },
                duration: Duration::UntilEndOfTurn,
            },
            target: Some(TargetSpec::Single(TargetFilter::CardInZone {
                zone: ZoneType::Graveyard,
                filter: CardFilter::Or(
                    Box::new(CardFilter::HasType(CardType::Instant)),
                    Box::new(CardFilter::HasType(CardType::Sorcery)),
                ),
            })),
            condition: None,
            description: "target instant or sorcery card in your graveyard gains flashback".into(),
        }],
        ..CardDef::default()
    }
}
```

**Data-driven card import** (from Codex review's recommendation): Once the effect/ability/target library is stable, we can add a JSON-based card loader:

```json
{
  "name": "Lightning Bolt",
  "mana_cost": "{R}",
  "types": ["Instant"],
  "spell_effect": {
    "type": "DealDamage",
    "amount": 3,
    "target": { "type": "CreatureOrPlayer" }
  }
}
```

This is a Phase 4 enhancement — getting the trait/enum system right in Rust first, then layering serialization on top.

### 4.8 Watcher System

**File**: `src/game/watcher.rs`

Watchers track derived state across events. They are used by triggered abilities and effects that need "this turn" or "since last turn" information.

```rust
/// Watcher trait — observes events, accumulates state, resets periodically.
pub trait WatcherImpl: std::fmt::Debug + Clone + Send + Sync {
    fn watch(&mut self, event: &GameEvent, state: &GameState);
    fn reset(&mut self);
}

/// Built-in watchers for common patterns.
#[derive(Debug, Clone)]
pub enum Watcher {
    /// Tracks spells cast this turn per player.
    SpellsCastThisTurn(HashMap<PlayerIndex, u32>),
    /// Tracks creatures that died this turn.
    CreaturesDiedThisTurn(Vec<ObjectId>),
    /// Tracks damage dealt this turn by source.
    DamageDealtThisTurn(HashMap<ObjectId, u32>),
    /// Tracks whether combat damage was dealt to a player this turn.
    CombatDamageToPlayer(HashMap<PlayerIndex, bool>),
    /// Tracks cards drawn this turn per player.
    CardsDrawnThisTurn(HashMap<PlayerIndex, u32>),
    /// Custom watcher via trait object.
    Custom(Arc<dyn WatcherImpl>),
}

/// Registry of active watchers on GameState.
#[derive(Debug, Clone)]
pub struct WatcherRegistry {
    watchers: Vec<(WatcherScope, Watcher)>,
}

#[derive(Debug, Clone, Copy)]
pub enum WatcherScope { Game, Turn, Step }
```

**Integration**: After every post-event, the watcher registry is notified:

```rust
impl WatcherRegistry {
    pub fn observe(&mut self, event: &GameEvent, state: &GameState) {
        for (_, watcher) in &mut self.watchers {
            watcher.watch(event, state);
        }
    }

    pub fn reset_turn(&mut self) {
        for (scope, watcher) in &mut self.watchers {
            if matches!(scope, WatcherScope::Turn) {
                watcher.reset();
            }
        }
    }
}
```

### 4.9 Game State Optimizations

The current `GameState` clones the entire `CardDatabase` with every clone (it's `Option<CardDatabase>` with `#[serde(skip)]`). This is the single biggest performance issue for MCTS.

**Change 1: Share CardDatabase via Arc**

```rust
pub struct GameState {
    pub card_db: Arc<CardDatabase>,  // shared, never cloned
    // ... everything else
}
```

This change alone eliminates cloning 23+ `CardDef` structs (with their `String` fields) on every state clone.

**Change 2: Compact object representation**

Current `CardInstance` has `Vec<KeywordAbility>` for temp keywords and `Vec<ObjectId>` for attachments. These heap-allocate on every clone.

```rust
/// Bitfield for keywords — 18 keywords fit in a u32.
pub type KeywordSet = u32;

pub struct CardInstance {
    pub object_id: ObjectId,
    pub card_def_id: CardId,
    pub owner: u8,              // PlayerIndex as u8 (max 255 players)
    pub controller: u8,
    pub tapped: bool,
    pub summoning_sick: bool,
    pub damage_marked: u16,     // max 65535 damage
    pub plus_counters: i16,
    pub minus_counters: i16,
    pub temp_power_mod: i16,
    pub temp_toughness_mod: i16,
    pub base_keywords: KeywordSet,   // from CardDef
    pub granted_keywords: KeywordSet, // from continuous effects
    pub attached_to: Option<ObjectId>,
    pub zone_change_counter: u16,     // for stale reference detection
    // Remove: attachments Vec, temp_keywords Vec
}
```

This makes `CardInstance` a fixed-size struct (~48 bytes) that clones with a single memcpy — no heap allocation.

**Change 3: SmallVec for zones**

Most zones (hand, graveyard, exile, stack) are small. Use `SmallVec<[ObjectId; 8]>` to avoid heap allocation for typical cases:

```rust
use smallvec::SmallVec;

pub struct PlayerState {
    pub library: Vec<ObjectId>,           // large, keep Vec
    pub hand: SmallVec<[ObjectId; 10]>,   // usually 0-7 cards
    pub graveyard: SmallVec<[ObjectId; 16]>, // usually 0-15 cards
    pub exile: SmallVec<[ObjectId; 4]>,   // usually 0-3 cards
    // ...
}
```

**Change 4: Incremental state hashing**

For MCTS transposition tables, we need fast state comparison. Maintain a Zobrist-style hash that updates incrementally as actions are applied, rather than hashing the full state.

```rust
pub struct GameState {
    pub hash: u64,  // Zobrist hash, updated on every mutation
    // ...
}
```

---

## 5. Edge-Case Testing Strategy

### Philosophy: "Test Cards"

Rather than trying to test abstract rules in isolation, we create **specific MTG cards (real or synthetic) that exercise specific rules corners**. Each test sets up a precise board state, performs an action, and asserts the resulting state. This matches xMage's approach (their test suite uses real cards in scripted scenarios).

### Test Categories

#### 5.1 Layer System Tests

| Test | Cards | What It Proves |
|------|-------|----------------|
| **Humility + Opalescence** | Humility (all creatures lose abilities and are 1/1), Opalescence (enchantments are creatures with P/T = CMC) | Layer 6 (remove abilities) vs Layer 7b (set P/T) — timestamp determines which "wins" for Humility itself |
| **Glorious Anthem stacking** | 2x Glorious Anthem (+1/+1 to your creatures) + Grizzly Bears | Layer 7c: Bears should be 4/4 (2+1+1) |
| **Turn to Frog + Giant Growth** | Turn to Frog ("becomes a 1/1 blue Frog"), Giant Growth (+3/+3 until EOT) | Layer 7b (set to 1/1) before 7c (modify +3/+3) = 4/4 |
| **Dress Down + Tarmogoyf** | Dress Down (creatures lose all abilities), Tarmogoyf | Layer 6 removes Tarmogoyf's CDA, so it falls back to printed 0/1 |
| **Control change + anthem** | Steal creature, opponent has anthem | Layer 2 (control change) before Layer 7c (anthem) — creature loses opponent's anthem, gains yours |

#### 5.2 Replacement Effect Tests

| Test | Cards | What It Proves |
|------|-------|----------------|
| **Double replacement** | Rest in Peace ("if a card would be put into a graveyard, exile it instead") + creature dies | Replacement correctly redirects zone change |
| **Replacement loop prevention** | Two replacement effects that could theoretically loop | `applied_replacements` tracking prevents re-application |
| **Enters tapped** | Tapped dual land | Self-replacement effect on ETB |
| **Damage prevention + redirect** | Fog + damage | Prevention replaces damage event with nothing |

#### 5.3 Priority & SBA Tests

| Test | Cards | What It Proves |
|------|-------|----------------|
| **SBA cascade** | Blood Artist (when a creature dies, drain 1) + two 1/1 creatures taking 1 damage each | Both creatures die as SBA, triggers go on stack, SBAs re-check (Blood Artist drains don't kill anyone), then priority |
| **Legend rule** | Play second copy of a legendary creature | SBA removes one, player chooses which to keep |
| **Counter annihilation** | Creature with +1/+1 and -1/-1 counters | SBA removes pairs |
| **Trigger during SBA** | Creature with "when this dies, create a token" + lethal damage | Token creation trigger fires during SBA, goes on stack |

#### 5.4 Combat Edge Cases

| Test | Cards | What It Proves |
|------|-------|----------------|
| **Trample + deathtouch** | 6/6 trample deathtouch vs 2/2 blocker | Only 1 damage assigned to blocker (deathtouch lethal), 5 tramples through |
| **First strike kills blocker** | 3/1 first strike vs 2/2 blocker | Blocker dies in first strike step, doesn't deal regular damage back |
| **Menace enforcement** | Menace creature attacks, only 1 potential blocker | Can't be blocked legally |
| **Double strike + lifelink** | Double strike lifelink creature | Life gained in both damage steps |

#### 5.5 Triggered Ability Tests

| Test | Cards | What It Proves |
|------|-------|----------------|
| **APNAP ordering** | Both players have ETB triggers at the same time | Active player's triggers go on stack first (resolve last) |
| **Delayed trigger** | "At the beginning of next end step, sacrifice this" | Trigger created by effect, fires at correct time |
| **"Whenever one or more" batch** | Token army dies to Wrath + "whenever one or more creatures die, draw a card" | Draws exactly 1 card, not N |

#### 5.6 Targeting Tests

| Test | Cards | What It Proves |
|------|-------|----------------|
| **Fizzle** | Lightning Bolt targeting creature, creature dies before resolution | Bolt fizzles (no damage to anything) |
| **Hexproof vs. own spells** | Hexproof creature + owner's targeted buff | Owner CAN target their own hexproof creature |
| **Shroud blocks all** | Shroud creature + owner's targeted buff | Owner CANNOT target their own shroud creature |

### Test Infrastructure

```rust
/// Helper for setting up test scenarios.
pub struct TestHarness {
    state: GameState,
    db: CardDatabase,
}

impl TestHarness {
    /// Create a fresh 2-player game with empty boards.
    pub fn new() -> Self { ... }

    /// Put a card directly onto the battlefield for a player.
    pub fn put_on_battlefield(&mut self, card_name: &str, player: PlayerIndex) -> ObjectId { ... }

    /// Put a card into a player's hand.
    pub fn put_in_hand(&mut self, card_name: &str, player: PlayerIndex) -> ObjectId { ... }

    /// Set the current phase.
    pub fn set_phase(&mut self, phase: Phase) { ... }

    /// Give a player mana.
    pub fn add_mana(&mut self, player: PlayerIndex, pool: ManaPool) { ... }

    /// Execute an action and return the state.
    pub fn act(&mut self, action: Action) -> &GameState { ... }

    /// Assert a permanent is on the battlefield with given P/T.
    pub fn assert_pt(&self, obj_id: ObjectId, power: i32, toughness: i32) { ... }

    /// Assert a permanent is in a specific zone.
    pub fn assert_zone(&self, obj_id: ObjectId, zone: ZoneType) { ... }

    /// Assert the stack has N entries.
    pub fn assert_stack_size(&self, n: usize) { ... }
}
```

---

## 6. Performance Optimization Strategy

### Performance Budget

For GTO simulation via MCTS/CFR, the critical path is:

```
clone state → enumerate legal actions → pick action → apply action → repeat
```

| Operation | Target | Current Estimate | Optimization |
|-----------|--------|-----------------|--------------|
| **State clone** | <10μs | ~50μs (HashMap clone + Vec clones + strings) | Arc<CardDB>, compact CardInstance, SmallVec |
| **apply_effects()** | <50μs (10 permanents, 5 effects) | N/A (not implemented) | Flat Vec iteration, no allocation |
| **legal_actions()** | <100μs (typical board) | ~200μs | Pre-filter eligible attackers/blockers |
| **apply_action()** | <20μs | ~10μs | Event firing adds overhead, keep lean |
| **Full rollout** (30 turns) | <10ms | ~15ms | Cumulative from above |

### Specific Optimizations

1. **`Arc<CardDatabase>`** — The card database is immutable after setup. Share it by reference, not by clone. This alone likely 5x-10x improves clone time.

2. **Keyword bitfield** — Replace `Vec<KeywordAbility>` with `u32` bitfield. 18 keywords fit easily. No heap allocation, O(1) check, single-word clone.

3. **Fixed-size CardInstance** — Eliminate all `Vec` fields on `CardInstance`. Attachments can be tracked via a separate `HashMap<ObjectId, SmallVec<[ObjectId; 4]>>` on `GameState`, or as a flat `Vec<(ObjectId, ObjectId)>` attachment list.

4. **SmallVec for zones** — Hand (7-10 cards), graveyard (0-15), exile (0-3) benefit from stack-allocated small vectors.

5. **Arena allocator for events** — During a single `apply_action()` call, many `FiredEvent` objects are created and dropped. Use a bump allocator or simply reuse a pre-allocated `Vec<FiredEvent>` to avoid repeated allocation.

6. **Lazy `apply_effects()`** — Track a "dirty" flag. Only recalculate when continuous effects have actually changed (new effect added, effect expired, source removed). In many game states, effects don't change between priority checks.

7. **Zobrist hashing** — Maintain an incremental hash for transposition table lookups in MCTS. Update on each zone change, counter change, life change, etc. Costs one XOR per mutation.

8. **Object pool for CardInstance** — Instead of `HashMap<ObjectId, CardInstance>`, use a `Vec<CardInstance>` with ObjectId as the index (or a slab allocator). HashMap lookup has pointer indirection; Vec is cache-friendly.

### Benchmarking Plan

Add criterion benchmarks:

```rust
// benches/state_clone.rs
fn bench_state_clone(c: &mut Criterion) {
    let state = setup_typical_game_state(); // 10 permanents, 2 players, 5 cards each hand
    c.bench_function("state_clone", |b| b.iter(|| state.clone()));
}

// benches/apply_effects.rs
fn bench_apply_effects(c: &mut Criterion) {
    let mut state = setup_state_with_effects(); // 3 anthems, 15 creatures
    c.bench_function("apply_effects", |b| b.iter(|| {
        state.continuous.apply_effects(&mut state)
    }));
}

// benches/legal_actions.rs
fn bench_legal_actions(c: &mut Criterion) {
    let state = setup_combat_state(); // 5 attackers, 4 blockers
    c.bench_function("legal_actions", |b| b.iter(|| legal_actions(&state)));
}
```

---

## 7. Implementation Roadmap

### Phase 0: Performance Foundation (1 step, minimal risk)

| Task | Files | Description |
|------|-------|-------------|
| **0.1** Arc CardDatabase | `game/mod.rs` | Replace `Option<CardDatabase>` with `Arc<CardDatabase>`. Update clone, setup, and all call sites. |
| **0.2** Keyword bitfield | `card/mod.rs` | Replace `Vec<KeywordAbility>` with `KeywordSet` (u32). Update all `has_keyword()` checks. |
| **0.3** Add criterion benchmarks | `benches/` | Baseline benchmarks for state clone, legal_actions, and full game rollout. |

*Validates: Performance baseline established before architecture changes.*

### Phase 1: Event System (foundation for everything else)

| Task | Files | Description |
|------|-------|-------------|
| **1.1** Define GameEvent enum | `game/event.rs` (new) | All event variants, FiredEvent struct, EventPhase |
| **1.2** Wire events into rules engine | `rules/mod.rs` | Add `fire_event()` calls at every state mutation point. Initially events are fire-and-forget (no observers). |
| **1.3** Connect triggers to events | `rules/mod.rs` | Refactor `check_triggers()` to match on `GameEvent` variants instead of `TriggerCondition` enum matching. |

*Test: Existing tests still pass. New test verifying events fire in correct order for a Bolt → creature dies scenario.*

### Phase 2: Continuous Effects & Layers

| Task | Files | Description |
|------|-------|-------------|
| **2.1** ContinuousEffects manager | `game/continuous.rs` (new) | Layer enum, ActiveEffect, ContinuousEffects struct, `apply_effects()` method |
| **2.2** CardInstance::reset_to_base() | `card/mod.rs` | New method that clears computed state to CardDef values |
| **2.3** Wire apply_effects into priority | `rules/mod.rs` | Call `apply_effects()` after every action and before priority |
| **2.4** Static abilities → continuous effects | `card/mod.rs`, `rules/mod.rs` | When a permanent with static abilities enters the battlefield, register its continuous effects |
| **2.5** Duration tracking + cleanup | `game/continuous.rs` | Prune expired effects (end of turn, left battlefield) |

*Test: Glorious Anthem test (creature gets +1/+1 while anthem on battlefield, loses it when anthem is removed). Turn to Frog + Giant Growth test.*

### Phase 3: Replacement Effects

| Task | Files | Description |
|------|-------|-------------|
| **3.1** ReplacementEngine | `game/replacement.rs` (new) | ReplacementEffect, ReplacementCondition, ReplacementAction, `apply()` method with loop prevention |
| **3.2** Wrap state mutations with pre-events | `rules/mod.rs` | Every zone change, damage application, and life change fires a pre-event through the replacement engine |
| **3.3** Enters-tapped support | `rules/mod.rs` | Self-replacement for `enters_tapped` flag on CardDef |
| **3.4** Prevention effects | `game/replacement.rs` | Damage prevention as a replacement effect subtype |

*Test: Rest in Peace test (creatures go to exile instead of graveyard). Enters-tapped land test. Fog damage prevention test.*

### Phase 4: Priority Loop & SBAs

| Task | Files | Description |
|------|-------|-------------|
| **4.1** SBA/trigger recurrence loop | `rules/mod.rs` | Implement `run_sba_trigger_loop()` — repeat SBAs and trigger checks until stable |
| **4.2** Extended SBAs | `rules/mod.rs` | Legend rule, planeswalker loyalty, counter annihilation, unattached auras, tokens |
| **4.3** Delayed triggered abilities | `rules/mod.rs`, `game/mod.rs` | `DelayedTrigger` struct on GameState, checked during trigger processing |

*Test: SBA cascade test (Blood Artist + mass damage). Legend rule test. Counter annihilation test.*

### Phase 5: Card Composition & Scaling

| Task | Files | Description |
|------|-------|-------------|
| **5.1** DynValue system | `card/dynamic_value.rs` (new) | DynValue enum with `evaluate()`, common implementations |
| **5.2** Composable filters | `card/targets.rs` (new) | PermanentFilter, CardFilter, TargetFilter with AND/OR/NOT combinators |
| **5.3** Evolve Effect enum | `card/effects.rs` (new) | Expanded Effect enum with DynValue, DestroyAll, Exile, Sacrifice, etc. |
| **5.4** Fizzle checking | `rules/mod.rs` | Re-validate targets on resolution, fizzle if all illegal |
| **5.5** Token creation | `rules/mod.rs` | Implement `CreateToken` effect — create CardInstance with is_token flag |
| **5.6** Add 50+ cards | `card/sample.rs` | Using the new composition system, add enough cards for a Standard-like metagame |

*Test: Tarmogoyf dynamic P/T. Wrath of God destroys all creatures. Fizzle test (Bolt target removed). Token creation test.*

### Phase 6: Watchers & Polish

| Task | Files | Description |
|------|-------|-------------|
| **6.1** Watcher system | `game/watcher.rs` (new) | WatcherRegistry, built-in watchers, event integration |
| **6.2** Zone change counters | `card/mod.rs`, `game/mod.rs` | Counter on CardInstance, updated on zone change, checked by effects |
| **6.3** Game state metadata | `game/mod.rs` | `values: HashMap<String, i64>` for storm count, monarch, etc. |
| **6.4** State clone optimizations | `game/mod.rs`, `card/mod.rs` | SmallVec for zones, compact CardInstance, object pool |
| **6.5** Zobrist hashing | `game/mod.rs` | Incremental hash updated on mutations |

*Test: Storm count watcher. Zone change counter staleness detection. Benchmark: <10μs state clone.*

---

## 8. Risk Analysis

| Risk | Impact | Mitigation |
|------|--------|------------|
| **apply_effects() too slow** | Blocks MCTS performance | Profile early (Phase 2). Lazy dirty-flag optimization. For typical boards (<20 permanents, <10 effects), should be <50μs. |
| **Effect enum gets too large** | Maintainability | The `Custom(Arc<dyn EffectImpl>)` escape hatch keeps the enum bounded. Only add common effects to the enum; rare ones use Custom. |
| **Replacement effect ordering non-deterministic** | Rules correctness in simulation | For GTO simulation, we can use a deterministic ordering (timestamp-based) rather than player choice. Add a `Strategy::choose_replacement()` method for when it matters. |
| **State clone size growth** | MCTS throughput | Each new field on GameState costs ~N bytes × millions of clones. Audit state size after each phase. Arc-share immutable data. |
| **Borrow checker conflicts in event system** | Implementation difficulty | Use the read-then-write pattern: collect events into a Vec during state mutation, process observers after mutation is complete. Never fire events while holding mutable borrows. |
| **Scope creep toward xMage feature parity** | Never ship | We don't need 28,000 cards or a client-server architecture. Scope to: correct Standard-legal rules engine + 100-200 card composition system + fast simulation. |

---

*Proposal synthesized from [Claude xMage review](claude-xmage-rules-engine-review.md), [Codex xMage review](codex-xmage-rules-engine-review.md), and full audit of the mtg-gto codebase (Feb 2026).*
