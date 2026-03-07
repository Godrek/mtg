# xMage Rules Engine Review & Requirements for mtg-gto

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [xMage Project Overview](#2-xmage-project-overview)
3. [xMage Architecture Deep Dive](#3-xmage-architecture-deep-dive)
   - [3.1 Module Structure](#31-module-structure)
   - [3.2 Core Rules Engine](#32-core-rules-engine)
   - [3.3 Game Loop & Priority System](#33-game-loop--priority-system)
   - [3.4 Turn Structure](#34-turn-structure)
   - [3.5 The Stack](#35-the-stack)
   - [3.6 Continuous Effects & Layer System](#36-continuous-effects--layer-system)
   - [3.7 Event System & Replacement Effects](#37-event-system--replacement-effects)
   - [3.8 Combat System](#38-combat-system)
   - [3.9 Card Implementation Pattern](#39-card-implementation-pattern)
   - [3.10 Ability & Effect Composition](#310-ability--effect-composition)
   - [3.11 Watcher System](#311-watcher-system)
   - [3.12 Game State Management](#312-game-state-management)
4. [Key Design Patterns](#4-key-design-patterns)
5. [Gap Analysis: mtg-gto vs xMage](#5-gap-analysis-mtg-gto-vs-xmage)
6. [Requirements for mtg-gto Rules Engine](#6-requirements-for-mtg-gto-rules-engine)
   - [R1: Priority System](#r1-priority-system)
   - [R2: State-Based Actions](#r2-state-based-actions)
   - [R3: Continuous Effects & Layers](#r3-continuous-effects--layers)
   - [R4: Replacement Effects](#r4-replacement-effects)
   - [R5: Event System](#r5-event-system)
   - [R6: Triggered Abilities](#r6-triggered-abilities)
   - [R7: Combat System](#r7-combat-system)
   - [R8: Card Composition Framework](#r8-card-composition-framework)
   - [R9: Game State Architecture](#r9-game-state-architecture)
   - [R10: Zone Management](#r10-zone-management)
   - [R11: Turn Structure](#r11-turn-structure)
   - [R12: Targeting System](#r12-targeting-system)
7. [Implementation Priorities](#7-implementation-priorities)

---

## 1. Executive Summary

This document reviews how xMage — an open-source, Java-based MTG implementation with 28,000+ cards — architects its rules engine, and translates those patterns into actionable requirements for our Rust-based `mtg-gto` simulator.

**Key finding:** xMage succeeds at scale through three architectural pillars:

1. **Composition over inheritance** — Cards are declarative assemblies of reusable abilities, effects, targets, and costs. Card authors don't write game logic; they compose pre-built components.
2. **Event-driven rules engine** — Every game action fires typed events. Triggered abilities, replacement effects, and watchers all observe the same event stream, enabling clean separation between "what happens" and "what reacts."
3. **Strict layer-ordered effect application** — Continuous effects are recalculated from scratch in MTG's 7-layer order every time game state changes, preventing stale or incorrectly-ordered modifications.

Our `mtg-gto` simulator already handles basic phases, stack resolution, targeting, and combat. The major gaps are: the **continuous effects layer system**, **replacement effects**, **a scalable card composition framework**, **the SBA/trigger recurrence loop**, and **batch event processing**. These are the areas where xMage's architecture provides the most valuable guidance.

---

## 2. xMage Project Overview

| Attribute | Value |
|-----------|-------|
| **Repository** | [github.com/magefree/mage](https://github.com/magefree/mage) |
| **Language** | Java (~99.9% of codebase) |
| **Build System** | Maven (multi-module) |
| **Cards Implemented** | 28,000+ unique, 73,000+ reprints |
| **Formats Supported** | Standard, Modern, Commander, Pauper, Brawl, Oathbreaker, etc. |
| **License** | MIT |
| **Architecture** | Client-server with plugin system |
| **AI** | Multiple implementations including Monte Carlo Tree Search |

xMage is a full rules-enforcing engine — it does not allow illegal plays. Every interaction goes through the rules engine, which validates legality, manages the stack, resolves effects, and handles all timing and priority.

---

## 3. xMage Architecture Deep Dive

### 3.1 Module Structure

xMage is organized into 11 Maven modules with clear separation of concerns:

| Module | Role | Analogy in mtg-gto |
|--------|------|---------------------|
| **Mage/** | Core rules engine — game logic, abilities, effects, events, turn structure, stack, combat, zones | `src/rules/`, `src/game/`, `src/action/` |
| **Mage.Sets/** | 28,000+ individual card implementations + set definitions | `src/card/sample.rs` |
| **Mage.Common/** | Shared client/server interfaces, DTOs, utilities | N/A (single binary) |
| **Mage.Server/** | Game server hosting matches | `src/simulation/` |
| **Mage.Server.Plugins/** | Format-specific rules, AI players, deck validators, tournament types | `src/strategy/` |
| **Mage.Client/** | Swing GUI | N/A |
| **Mage.Tests/** | Integration and unit tests | `tests/integration_test.rs` |
| **Mage.Verify/** | Card data verification tools | N/A |

The critical insight is the separation of **Mage/** (engine, ~50 packages) from **Mage.Sets/** (card data, ~28,000 files). The engine provides the vocabulary; cards are written in that vocabulary. This is what enables 28,000 cards without the engine itself becoming unmanageable.

### 3.2 Core Rules Engine

The core engine (`Mage/src/main/java/mage/`) contains ~14 packages:

```
mage/
├── game/                  # Game state, lifecycle, zones
│   ├── turn/              # Phase/Step hierarchy (Turn > Phase > Step)
│   ├── stack/             # The MTG stack (SpellStack, Spell, StackAbility)
│   ├── combat/            # Combat orchestration (Combat, CombatGroup)
│   ├── events/            # ~87 event types with pre/post pairs
│   ├── permanent/         # Battlefield permanents
│   └── mulligan/          # Mulligan variants
├── abilities/             # The ability/effect type system
│   ├── effects/           # One-shot, continuous, replacement, prevention effects
│   │   └── common/        # Hundreds of reusable effect implementations
│   ├── keyword/           # Keyword abilities (Flying, Trample, etc.)
│   ├── common/            # Reusable ability templates
│   ├── costs/             # Cost modeling (mana, tap, sacrifice, etc.)
│   ├── triggers/          # Trigger condition definitions
│   ├── condition/         # Boolean conditions
│   └── dynamicvalue/      # Runtime-computed values
├── cards/                 # Card model hierarchy (split, DFC, adventure, etc.)
├── filter/                # Predicate-based targeting filters
├── target/                # Target selection and validation
├── watchers/              # Observer pattern for derived state tracking
├── counters/              # Counter types (+1/+1, loyalty, etc.)
└── constants/             # Enums and constants
```

### 3.3 Game Loop & Priority System

The game loop is the heart of xMage's rules engine. It implements **Comprehensive Rules section 117** (Timing and Priority).

#### The Priority Algorithm

```
playPriority(activePlayerId):
    applyEffects()                          // recalculate all continuous effects
    LOOP:
        reset all players' "passed" flags
        FOR EACH player in APNAP order:
            WHILE player has not passed AND can respond:
                checkStateAndTriggered()     // SBAs + triggered abilities
                applyEffects()               // recalculate continuous effects
                player takes action or passes
                IF action taken:
                    applyEffects()
        IF all players passed consecutively:
            IF stack is non-empty:
                resolve top of stack
                reset all "passed" flags     // everyone gets priority again
                CONTINUE LOOP
            ELSE:
                BREAK                        // phase/step advances
```

**Critical details:**

- **SBA + Trigger loop**: `checkStateAndTriggered()` runs state-based actions and checks for triggered abilities in a loop, repeating until neither produces new results. Only then does a player receive priority. This is required by Rule 704.3.
- **Continuous effects recalculation**: `applyEffects()` is called extremely frequently — after every action and before every priority decision. It resets all permanent and player characteristics to base values and re-applies all continuous effects in layer order from scratch.
- **APNAP ordering**: The Active Player gets priority first, then Non-Active Players in turn order. This matters for multiplayer and for simultaneous trigger ordering.

#### State-Based Actions (Rule 704)

xMage checks all SBAs as a batch:

| SBA | Rule | Description |
|-----|------|-------------|
| Life <= 0 | 704.5a | Player loses |
| Empty library draw | 704.5b | Player loses |
| 10+ poison counters | 704.5c | Player loses |
| Toughness <= 0 | 704.5f | Creature dies |
| Lethal damage | 704.5g | Creature dies |
| Deathtouch damage | 704.5h | Creature dies |
| 0 loyalty | 704.5i | Planeswalker dies |
| Legend rule | 704.5j | Owner keeps one, rest go to graveyard |
| Unattached aura | 704.5n | Goes to graveyard |
| +1/+1 and -1/-1 annihilation | 704.5q | Paired counters remove each other |
| Token not on battlefield | 704.5d | Ceases to exist |

### 3.4 Turn Structure

xMage models turns as a three-level hierarchy: **Turn > Phase > Step**.

```
Turn
├── Beginning Phase
│   ├── Untap Step           (no priority)
│   ├── Upkeep Step          (priority)
│   └── Draw Step            (priority)
├── Pre-Combat Main Phase
│   └── Main Step            (priority)
├── Combat Phase
│   ├── Beginning of Combat  (priority)
│   ├── Declare Attackers    (priority)
│   ├── Declare Blockers     (priority)
│   ├── First Strike Damage  (conditional, priority)
│   ├── Combat Damage        (priority)
│   └── End of Combat        (priority)
├── Post-Combat Main Phase
│   └── Main Step            (priority)
└── End Phase
    ├── End Step             (priority)
    └── Cleanup Step         (no priority, normally)
```

**Advanced turn manipulation:**

- **Extra turns**: Queued via `TurnMod` objects, processed by `playExtraTurns()` before each normal turn.
- **Extra phases/steps**: Processed after each normal phase/step via `playExtraPhases()` / `playExtraSteps()`.
- **Skip turn/phase/step**: Implemented via replacement effects — `replaceEvent()` intercepts the turn/phase/step event and cancels it.
- **End the turn** (e.g., Time Stop): Sets a flag that causes all remaining phases to be skipped, exiles the stack, clears combat, and jumps to cleanup.
- **Turn control** (e.g., Mindslaver): Checked at turn start; the controlling player makes all decisions for the controlled player.

Each Step tracks its execution state (`PRE`, `PRIORITY`, `POST`) and whether players get priority (`hasPriority` flag). The untap step and cleanup step normally skip priority.

### 3.5 The Stack

The stack (`SpellStack`) manages `StackObject` instances, which come in two flavors:

| Type | Class | Example |
|------|-------|---------|
| Spells | `Spell` | Casting Lightning Bolt |
| Abilities | `StackAbility` | Triggering "when this enters the battlefield" |

**Stack operations:**

- `push()` — Add a spell or ability to the top with a timestamp.
- `resolve()` — Remove and resolve the top object. For spells, this means applying effects and moving the card to the appropriate zone. For abilities, this means executing the ability's effects.
- `counter()` — Counter a stack object. Fires a `COUNTER` event (which can itself be replaced by "can't be countered" effects). The countered object is removed and placed in the appropriate zone (usually graveyard for spells).
- Stack objects know their controller, source, targets, and chosen modes.

### 3.6 Continuous Effects & Layer System

This is the most complex subsystem in xMage and the one our simulator currently lacks entirely. It implements **Comprehensive Rules section 613** (Interaction of Continuous Effects).

#### The Layer System

All continuous effects are applied in strict order across 7 layers with sublayers:

| Layer | Name | Example Effects |
|-------|------|-----------------|
| **1a** | Copy effects | Clone, Sakashima |
| **1b** | Face-down effects | Morph face-down characteristics |
| **2** | Control-changing | Gain control of target permanent |
| **3** | Text-changing | Change "Forest" to "Island" in rules text |
| **4** | Type-changing | "becomes an artifact in addition to its other types" |
| **5** | Color-changing | "is blue in addition to its other colors" |
| **6** | Ability add/remove | "gains flying", "loses all abilities" |
| **7a** | Characteristic-defining abilities (P/T) | Tarmogoyf's * / 1+* |
| **7b** | Set P/T | "becomes a 3/3" |
| **7c** | Modify P/T | +2/+2, -1/-1 effects |
| **7d** | Counter-based P/T | +1/+1 counters, -1/-1 counters |
| **7e** | Switch P/T | "switch power and toughness" |

**Additional effect categories managed by `ContinuousEffects`:**

| Category | Purpose |
|----------|---------|
| Replacement effects | Intercept and modify events before they happen |
| Prevention effects | Prevent damage or other effects |
| Requirement effects | "Must attack", "must block" mandates |
| Restriction effects | "Can't attack", "can't block" constraints |
| Cost modification effects | Reduce or increase costs |
| AsThough effects | Permission overrides ("cast as though it had flash") |
| Splice effects | Combine card text (Splice onto Arcane) |

#### How Layer Application Works

Every time `applyEffects()` is called (which is frequent):

1. **Reset** all permanents and players to their base/printed characteristics.
2. **Apply** each layer in order (1 through 7e), processing all active continuous effects in that layer.
3. **Within a layer**: effects are ordered by **timestamp** (the order they entered the game). Newer effects apply after older ones.
4. **Dependency resolution** (Rule 613.8): If effect A's existence or behavior depends on effect B, then B must be applied before A within the same layer. xMage implements this with a waiting-effects queue and iterative resolution, with special handling for circular dependencies.
5. **Layer 2 (control-changing)** is applied iteratively until no further control changes occur, since control changes can enable or disable other control-changing effects.

#### Implications for mtg-gto

This is the single biggest gap in our rules engine. Without a layer system, we cannot correctly handle interactions between multiple continuous effects — for example, a creature that "becomes a 3/3" (Layer 7b) with a +2/+2 enchantment (Layer 7c) must be a 5/5, not a 3/3.

### 3.7 Event System & Replacement Effects

#### Event Types

xMage defines ~200+ event types in the `GameEvent.EventType` enum, covering every possible game action. Events follow a **pre/post pattern**:

| Pre-Event (Before) | Action | Post-Event (After) |
|---------------------|--------|---------------------|
| `DRAW_CARD` | Card moves to hand | `DREW_CARD` |
| `DAMAGE_PLAYER` | Damage marked | `DAMAGED_PLAYER` |
| `DECLARE_ATTACKER` | Creature attacks | `ATTACKER_DECLARED` |
| `TAP` | Permanent taps | `TAPPED` |
| `LOSE_LIFE` | Life total decreases | `LOST_LIFE` |

Each `GameEvent` carries:
- `type` — What happened
- `targetId` — What it happened to
- `sourceId` — What caused it
- `playerId` — Which player
- `amount` — Numeric value (damage, life, etc.)
- `appliedEffects` — List of replacement effects already applied to this event (critical for Rule 614.5)

#### Replacement Effects (Rule 614)

Replacement effects intercept pre-events and modify them before they happen:

```
1. Pre-event fires (e.g., DAMAGE_PLAYER)
2. Engine collects all applicable replacement effects
3. Filter out effects already applied to this event (via appliedEffects list)
4. If multiple apply: affected player/controller chooses order (Rule 616.1c)
5. Apply chosen effect, mark it in appliedEffects
6. Repeat until no more replacement effects apply
7. Execute the (possibly modified) action
8. Post-event fires
```

The `appliedEffects` tracking is essential — it prevents infinite loops where a replacement effect re-replaces itself, while allowing different replacement effects to chain on the same event.

#### Batch Events

When multiple things happen simultaneously (e.g., Wrath of God destroying all creatures), xMage batches events:

- Individual events are queued via `addSimultaneousEvent()`
- All are processed at once via `handleSimultaneousEvent()`
- Batch event types (e.g., `DAMAGED_BATCH_FOR_PLAYERS`, `ZONE_CHANGE_BATCH`) aggregate the individual events so triggered abilities can see the full picture

This is important for cards like "Whenever one or more creatures die, draw a card" — they should trigger once, not once per creature.

### 3.8 Combat System

xMage's combat system (`Combat.java` + `CombatGroup.java`) handles the full complexity of MTG combat.

#### Combat Flow

```
Begin Combat Step
  └── Priority round

Declare Attackers Step
  ├── checkAttackRequirements()     // enforce "must attack" / goad
  ├── Player selects attackers
  ├── checkAttackRestrictions()     // validate "can't attack" effects
  ├── Tap attacking creatures (without vigilance)
  ├── Fire DECLARE_ATTACKER events per attacker
  └── Priority round

Declare Blockers Step
  ├── Player selects blockers
  ├── checkBlockRestrictions()      // validate "can't block" effects
  ├── checkBlockRequirements()      // enforce "must block" mandates
  ├── Validate menace (minBlockedBy = 2)
  ├── Validate maxBlockedBy constraints
  ├── Fire DECLARE_BLOCKER events per blocker
  ├── Determine blocked/unblocked attackers
  └── Priority round

First Strike Combat Damage Step (only if first/double strike creatures exist)
  ├── Assign damage from first strike / double strike creatures
  ├── Apply damage
  └── Priority round

Combat Damage Step
  ├── Assign damage from all remaining creatures
  ├── Handle trample (excess over lethal goes to defending player)
  ├── Apply damage
  └── Priority round

End of Combat Step
  ├── Clear combat state
  └── Priority round
```

#### Damage Assignment

Damage assignment follows strict MTG rules:

- **Attacker to blockers**: Damage is assigned to blockers in the order chosen by the attacking player. Each blocker must receive at least lethal damage before the next one receives any (unless the attacker has deathtouch, where lethal = 1).
- **Trample**: Once all blockers have been assigned lethal damage, excess goes to the defending player/planeswalker.
- **First strike**: Creatures with first strike or double strike deal damage in the first strike step. Creatures without first strike that didn't deal first-strike damage deal damage in the normal step. Double strike creatures deal damage in both steps.

### 3.9 Card Implementation Pattern

This is how xMage scales to 28,000+ cards. Every card follows an identical pattern:

```java
public final class LightningBolt extends CardImpl {

    // Primary constructor — declares what the card IS
    public LightningBolt(UUID ownerId, CardSetInfo setInfo) {
        super(ownerId, setInfo, new CardType[]{CardType.INSTANT}, "{R}");

        // Compose behavior from reusable building blocks:
        this.getSpellAbility().addTarget(new TargetAnyTarget());
        this.getSpellAbility().addEffect(new DamageTargetEffect(3));
    }

    // Copy infrastructure (required for zone changes, state snapshots)
    private LightningBolt(final LightningBolt card) { super(card); }

    @Override
    public LightningBolt copy() { return new LightningBolt(this); }
}
```

**Key properties of this pattern:**

| Property | Details |
|----------|---------|
| **All card classes are `final`** | No card subclasses another card; all extend `CardImpl` directly |
| **No game logic in card classes** | Cards declare what they are; the engine handles how rules work |
| **Constructor-only setup** | All card behavior is configured in the constructor |
| **Prototype pattern for cloning** | `copy()` + private copy constructor enables deep copying for zone changes and state snapshots |
| **Composition over inheritance** | Cards assemble abilities, effects, targets, and costs from a shared library |

#### Card Complexity Spectrum

**Simple instant** (Lightning Bolt): SpellAbility + Target + Effect
```java
this.getSpellAbility().addTarget(new TargetAnyTarget());
this.getSpellAbility().addEffect(new DamageTargetEffect(3));
```

**Creature with keywords** (Serra Angel): Subtypes + P/T + keyword singletons
```java
this.subtype.add(SubType.ANGEL);
this.power = new MageInt(4);
this.toughness = new MageInt(4);
this.addAbility(FlyingAbility.getInstance());
this.addAbility(VigilanceAbility.getInstance());
```

**Board wipe** (Wrath of God): Effect with static filter, no targets
```java
this.getSpellAbility().addEffect(
    new DestroyAllEffect(StaticFilters.FILTER_PERMANENT_CREATURES, true));
```

**ETB trigger** (Snapcaster Mage): Triggered ability wrapping an effect with a target
```java
Ability ability = new EntersBattlefieldTriggeredAbility(new SnapcasterMageEffect());
ability.addTarget(new TargetCardInYourGraveyard(filter));
this.addAbility(ability);
```

**Dynamic P/T** (Tarmogoyf): Static ability with a dynamic value
```java
this.addAbility(new SimpleStaticAbility(Zone.ALL,
    new SetBasePowerToughnessPlusOneSourceEffect(CardTypesInGraveyardCount.ALL)));
```

### 3.10 Ability & Effect Composition

#### Ability Type Hierarchy

xMage's ability system directly mirrors the MTG Comprehensive Rules:

| Ability Type | MTG Rule | xMage Class | When It Works |
|-------------|----------|-------------|---------------|
| Spell ability | 601 | `SpellAbility` | When the spell resolves |
| Activated ability | 602 | `ActivatedAbilityImpl` | Player pays cost, puts on stack |
| Triggered ability | 603 | `TriggeredAbilityImpl` | When condition met, goes on stack |
| Static ability | 604 | `StaticAbility` | Continuously while on battlefield |
| Mana ability | 605 | `ActivatedManaAbilityImpl` | Special: doesn't use the stack |
| Loyalty ability | — | `LoyaltyAbility` | Planeswalker-specific activated ability |
| Keyword ability | Various | `FlyingAbility`, etc. | Varies by keyword |

#### Effect Type Hierarchy

| Effect Type | Purpose | Example |
|------------|---------|---------|
| **OneShotEffect** | Resolves once when the spell/ability resolves | `DamageTargetEffect`, `DestroyAllEffect`, `CounterTargetEffect` |
| **ContinuousEffect** | Persists over time, applied in layers | `BoostSourceEffect` (+2/+2 until end of turn) |
| **ReplacementEffect** | Intercepts events before they happen | `EntersBattlefieldEffect` (enters tapped) |
| **PreventionEffect** | Prevents damage | `PreventDamageToTargetEffect` |
| **RestrictionEffect** | "Can't attack/block" | Defender keyword |
| **RequirementEffect** | "Must attack/block" | Goad mechanic |
| **CostModificationEffect** | Changes costs | `SpellsCostReductionControllerEffect` |
| **AsThoughEffect** | Permission override | "You may cast ~ as though it had flash" |

#### Reusable Effect Library

xMage provides **hundreds of reusable, parameterized effects** in `abilities.effects.common/`. Cards compose these rather than implementing custom logic:

```
DamageTargetEffect(amount)              — "Deal N damage to target"
DestroyTargetEffect(noRegen)            — "Destroy target permanent"
DrawCardSourceControllerEffect(count)   — "Draw N cards"
GainLifeEffect(amount)                  — "Gain N life"
ExileTargetEffect()                     — "Exile target"
CounterTargetEffect()                   — "Counter target spell"
CreateTokenEffect(token, count)         — "Create N tokens"
ReturnToHandTargetEffect()              — "Return target to hand"
BoostSourceEffect(power, toughness, duration) — "+N/+N until..."
GainAbilitySourceEffect(ability, duration)    — "Gains [keyword] until..."
```

When no reusable effect exists for a unique card, a custom inner class is written. But this is the exception, not the rule — most cards are fully composed from the shared library.

#### Targeting System

Targets are attached to abilities (not effects), matching MTG rules where targets are chosen when putting the spell/ability on the stack:

| Target Class | What It Targets |
|-------------|-----------------|
| `TargetAnyTarget` | Any creature, player, or planeswalker |
| `TargetCreaturePermanent` | A creature on the battlefield |
| `TargetPlayer` | A player |
| `TargetSpell` | A spell on the stack |
| `TargetCardInYourGraveyard(filter)` | A card in your graveyard matching criteria |
| `TargetPermanent(filter)` | A permanent matching criteria |

Filters use **composable predicates**:
```java
filter.add(Predicates.or(
    CardType.INSTANT.getPredicate(),
    CardType.SORCERY.getPredicate()));
```

#### Dynamic Values

For effects that depend on game state (e.g., Tarmogoyf), xMage uses `DynamicValue` — an interface with a `calculate(Game, Ability)` method that computes a value at resolution/application time:

```
CardTypesInGraveyardCount    — Count of card types in graveyards
PermanentsOnBattlefieldCount — Count of permanents matching a filter
CardsInControllerHandCount   — Number of cards in hand
GetXValue                    — Value of X in mana cost
```

### 3.11 Watcher System

Watchers are an observer pattern implementation for tracking derived game state. They watch the event stream and accumulate state that abilities and effects can query.

```
Watcher (abstract base)
├── watch(GameEvent, Game)  — called for every game event
├── reset()                 — called at appropriate intervals
├── conditionMet()          — query whether condition is satisfied
└── scope: GAME | PLAYER | CARD
```

**Example watchers:**

| Watcher | Tracks | Used By |
|---------|--------|---------|
| `SpellsCastThisTurnWatcher` | Count of spells cast per player this turn | "Storm" ability, "if you cast 2+ spells" |
| `CardsPutIntoGraveyardWatcher` | Cards that entered graveyards | "Whenever a card is put into a graveyard" |
| `CombatDamageWatcher` | Creatures that dealt combat damage | "Whenever this creature deals combat damage" |
| `FirstStrikeWatcher` | Creatures that dealt first-strike damage | Prevents double-counting in normal damage step |

Watchers decouple the "tracking" concern from the triggered abilities that use the data. Multiple triggered abilities can share the same watcher.

### 3.12 Game State Management

`GameState` is the central state container, holding everything needed to fully describe the game at any point:

| Category | Components |
|----------|------------|
| **Players** | Player objects, APNAP turn order, active player, priority holder, monarch, initiative |
| **Zones** | Battlefield, stack, exile, command zone (libraries, hands, graveyards are per-player) |
| **Zone tracking** | Object-to-zone mapping, zone change counters for stale reference detection |
| **Turn state** | Current turn/phase/step, turn mods (extra turns, skips) |
| **Effects** | All active continuous effects, triggered abilities (pending and delayed), triggered queue |
| **Combat** | Active combat groups, attackers, blockers, damage assignments |
| **Watchers** | All active watcher instances |
| **Card state** | Per-card state overrides, copied card references |
| **Game values** | Arbitrary key-value store for game-level state |

#### Snapshot and Restore

The entire `GameState` supports deep copy and restore:

```java
copy()                    // Full deep copy for save points, AI evaluation
restore(GameState)        // Overwrite current state (undo/rollback)
restoreForRollBack(state) // Restore but preserve turn counter
clearOnGameRestart()      // Full reset (Karn Liberated's restart-game ability)
```

This is critical for:
- **AI search**: MCTS and similar algorithms need to simulate futures without modifying the real game.
- **Undo**: Players can undo certain actions (before committing).
- **Rollback**: Judges/admins can rewind to a prior state.

#### Zone Change Counter

Every game object has a monotonically increasing zone change counter. When an object moves zones, its counter increments. Any effect or ability holding a reference to an object can compare the stored counter value with the current one — if they differ, the reference is stale (the object has moved zones since the reference was created). This implements MTG's rule that effects "lose track" of objects that change zones.

---

## 4. Key Design Patterns

| Pattern | Where Used | Purpose |
|---------|-----------|---------|
| **Composition over Inheritance** | Card implementations | Cards are data, not behavior — assembled from reusable parts |
| **Observer / Event-Driven** | Event system, Watchers, Triggers | Decouple "what happens" from "what reacts" |
| **Replacement / Interceptor** | Replacement effects | Modify game actions before they execute |
| **Layer / Chain of Responsibility** | Continuous effects | Apply effects in strict, deterministic order |
| **Prototype (Clone)** | `copy()` on every card/game object | Deep copying for zone changes and state snapshots |
| **State Snapshot (Memento)** | `GameState.copy()` / `restore()` | Undo, rollback, AI tree search |
| **Strategy** | Player / AI implementations | Pluggable decision-making |
| **Template Method** | Phase/Step execution | `prePriority()` → `priority()` → `postPriority()` skeleton |
| **Singleton** | Keyword abilities (`FlyingAbility.getInstance()`) | Stateless markers shared across instances |
| **Predicate Composition** | Filters and targets | Composable targeting constraints |
| **Timestamp Ordering** | Same-layer effect resolution | Newer effects override older ones at same layer |
| **Plugin Architecture** | Game formats, AI, validators | Extend without modifying the engine core |

---

## 5. Gap Analysis: mtg-gto vs xMage

| Feature | mtg-gto Status | xMage Status | Gap Severity |
|---------|---------------|--------------|-------------|
| **Turn structure** | 13 phases including combat substeps | Full 5-phase, 12-step hierarchy with extra turn/phase/step support | Low — mostly complete |
| **Priority system** | Basic pass/action loop | Full APNAP with SBA+trigger recurrence loop | **Medium** — missing recurrence loop |
| **Stack** | Basic push/resolve | Full with counter mechanics, copy handling | Low-Medium |
| **State-based actions** | Lethal damage, life <= 0, empty library | Full 704.5 suite (legend rule, aura check, counter annihilation, etc.) | Medium |
| **Continuous effects / layers** | **Not implemented** | Full 7-layer system with sublayers, timestamps, dependency resolution | **Critical** |
| **Replacement effects** | **Not implemented** | Full replacement effect chain with appliedEffects tracking | **Critical** |
| **Event system** | Basic trigger checking | ~200 typed events with pre/post pattern, batch events | **High** |
| **Triggered abilities** | Basic ETB, dies, attacks triggers | Full trigger infrastructure with APNAP ordering, delayed triggers | Medium |
| **Combat** | Attackers, blockers, damage, first strike | Full with requirements, restrictions, menace, trample, banding | Low-Medium |
| **Card composition** | Enum-based with hardcoded sample cards | Declarative composition from reusable ability/effect/target library | **High** |
| **Targeting** | Basic target enumeration | Predicate-composable filter system with legal target validation | Medium |
| **Game state snapshots** | Cloneable GameState (for AI) | Full copy/restore with zone change counters for stale detection | Low-Medium |
| **Keyword abilities** | 17 keywords as flags | Keyword abilities as first-class ability objects | Medium |
| **Dynamic values** | Not implemented | `DynamicValue` interface for runtime-computed amounts | Medium |
| **Watchers** | Not implemented | Observer pattern for derived state tracking | Medium |
| **Mana system** | 5-color + colorless, auto-tap | Full including conditional mana, mana restrictions | Low |
| **Card count** | ~20 sample cards | 28,000+ cards | Expected (different project goals) |

---

## 6. Requirements for mtg-gto Rules Engine

Based on the xMage review and our gap analysis, here are the requirements for evolving our rules engine. Each requirement includes what xMage does, what we currently have, and what we need.

### R1: Priority System

**MTG Rules**: 117 (Timing and Priority)

**Current state**: Our `handle_priority_pass()` advances phases or resolves the stack when a player passes, but does not implement the full SBA+trigger recurrence loop before granting priority.

**Requirements**:

1. **R1.1** — Before any player receives priority, run state-based actions and triggered ability checks in a loop until neither produces new results (Rule 704.3).
2. **R1.2** — Implement APNAP (Active Player, Non-Active Player) ordering for priority. The active player always receives priority first after any stack resolution or phase transition.
3. **R1.3** — Track "passed" state per player. When all players pass consecutively with the stack empty, the phase/step advances. When all pass with a non-empty stack, resolve the top object and restart priority.
4. **R1.4** — Recalculate continuous effects (R3) after every action and before every priority decision.

### R2: State-Based Actions

**MTG Rules**: 704 (State-Based Actions)

**Current state**: We check lethal damage, life <= 0, and empty library. Missing several SBAs.

**Requirements**:

1. **R2.1** — Implement all SBAs as a batch check that repeats until no new actions are generated:
   - Player life <= 0 → loses (704.5a)
   - Drew from empty library → loses (704.5b)
   - 10+ poison counters → loses (704.5c)
   - Token not on battlefield → ceases to exist (704.5d)
   - Creature toughness <= 0 → dies (704.5f)
   - Creature with lethal damage → dies (704.5g)
   - Creature with deathtouch damage → dies (704.5h)
   - Planeswalker with 0 loyalty → dies (704.5i)
   - Legend rule — if two+ legendaries with same name controlled by same player, owner keeps one (704.5j)
   - Unattached aura/equipment → graveyard (704.5n)
   - +1/+1 and -1/-1 counter annihilation (704.5q)
2. **R2.2** — After each batch of SBAs, check for triggered abilities. If any were generated, put them on the stack and repeat the SBA check.

### R3: Continuous Effects & Layers

**MTG Rules**: 613 (Interaction of Continuous Effects)

**Current state**: Not implemented. Keyword abilities are boolean flags. No system for persistent effects that modify game objects.

**Requirements**:

1. **R3.1** — Implement a `ContinuousEffects` manager that tracks all active continuous effects and applies them in the correct layer order:
   - Layer 1: Copy effects (1a: copy, 1b: face-down)
   - Layer 2: Control-changing effects
   - Layer 3: Text-changing effects
   - Layer 4: Type-changing effects
   - Layer 5: Color-changing effects
   - Layer 6: Ability adding/removing effects
   - Layer 7: P/T effects (7a: CDA, 7b: set, 7c: modify, 7d: counters, 7e: switch)
2. **R3.2** — The `apply_effects()` function must reset all permanents to base characteristics, then re-apply all continuous effects in layer order from scratch. This must be called after every game action and before every priority decision.
3. **R3.3** — Within the same layer, effects must be ordered by timestamp (monotonically increasing order based on when the effect entered the game).
4. **R3.4** — Implement dependency resolution for same-layer effects (Rule 613.8): if effect A depends on effect B, apply B first.
5. **R3.5** — Implement the following additional effect categories:
   - Restriction effects ("can't attack/block")
   - Requirement effects ("must attack/block")
   - Cost modification effects (reduce/increase spell costs)
   - AsThough effects ("may cast as though it had flash")

### R4: Replacement Effects

**MTG Rules**: 614-616 (Replacement Effects, Interaction)

**Current state**: Not implemented.

**Requirements**:

1. **R4.1** — Implement a replacement effect system that intercepts game events before they happen and allows registered effects to modify them.
2. **R4.2** — Track which replacement effects have already been applied to a given event (via an `applied_effects` list on the event object) to prevent a replacement effect from applying to the same event more than once (Rule 614.5).
3. **R4.3** — When multiple replacement effects apply to the same event, the affected player or controller chooses which to apply first (Rule 616.1c).
4. **R4.4** — Support self-replacement effects — effects that modify how they themselves work (e.g., "enters the battlefield tapped").
5. **R4.5** — Prevention effects (damage prevention) should be modeled as a subtype of replacement effects.

### R5: Event System

**MTG Rules**: Throughout the Comprehensive Rules

**Current state**: We have basic trigger checking but no formal event type system. Actions are applied directly without firing events.

**Requirements**:

1. **R5.1** — Define an `Event` enum or struct with typed event variants covering all game actions. Minimum required categories:
   - Zone changes (enters battlefield, dies, exiled, drawn, discarded, milled)
   - Damage (to creature, to player, to planeswalker, combat vs. non-combat)
   - Life changes (gain life, lose life)
   - Spells/abilities (spell cast, ability activated, ability triggered, spell countered)
   - Combat (attacker declared, blocker declared, creature blocked/unblocked)
   - Permanents (tapped, untapped, transformed, attached, counter added/removed)
   - Turn/phase (phase changed, step changed, turn begun/ended)
2. **R5.2** — Implement pre/post event pattern: fire a pre-event before the action (which replacement effects can intercept), execute the action, then fire a post-event (which triggers and watchers observe).
3. **R5.3** — Each event must carry: event type, target object ID, source object ID, controller player, numeric amount, and an applied-effects list.
4. **R5.4** — Implement batch events for simultaneous occurrences (e.g., Wrath of God destroying multiple creatures). Triggered abilities that care about "one or more" events should see the batch, not individual events.

### R6: Triggered Abilities

**MTG Rules**: 603 (Handling Triggered Abilities)

**Current state**: Basic trigger types (ETB, Dies, Attacks, Upkeep, EndOfTurn) with simple matching. No delayed triggers.

**Requirements**:

1. **R6.1** — Triggered abilities should register for specific event types and check a condition function when that event fires.
2. **R6.2** — When multiple triggered abilities trigger simultaneously, they are placed on the stack in APNAP order: active player's triggers first (in any order the player chooses), then each other player's in turn order.
3. **R6.3** — Implement delayed triggered abilities — triggers created by resolving spells/abilities that fire later (e.g., "At the beginning of the next end step, sacrifice it").
4. **R6.4** — Triggered abilities must be checked as part of the SBA+trigger recurrence loop (R1.1): after SBAs are processed, check for triggers, put them on the stack, then repeat until stable.
5. **R6.5** — Trigger conditions should support the full event payload (not just event type) for filtering — e.g., "whenever a creature YOU CONTROL dies" needs to check the controller.

### R7: Combat System

**MTG Rules**: 506-511 (Combat Phase)

**Current state**: Basic attack/block declaration, damage assignment with first strike and various evasion abilities. Missing requirement/restriction effect integration.

**Requirements**:

1. **R7.1** — Integrate combat with requirement/restriction effects from R3.5:
   - Restriction effects can prohibit specific creatures from attacking or blocking.
   - Requirement effects can force creatures to attack or block (e.g., goad).
2. **R7.2** — Validate blocking assignments against menace (`minBlockedBy`) and `maxBlockedBy` constraints after all blocks are declared.
3. **R7.3** — Damage assignment must enforce the ordered-blockers rule: each blocker must receive lethal damage (accounting for deathtouch) before the next in order receives any.
4. **R7.4** — Trample damage overflow must correctly apply to the defending player or planeswalker after all blockers have been assigned lethal damage.
5. **R7.5** — First strike / double strike damage must be handled as separate combat damage steps, with a full priority round between them.
6. **R7.6** — Fire appropriate events at each combat step so triggered abilities (R6) and replacement effects (R4) can interact with combat.

### R8: Card Composition Framework

**Current state**: Cards are defined as `CardDefinition` structs with enums for abilities and effects. Sample cards are hardcoded functions returning these structs. Scaling to hundreds of cards would require significant boilerplate.

**Requirements**:

1. **R8.1** — Evolve the card definition system toward a **composition-based model** where cards are assembled from reusable building blocks:
   - A library of reusable `Effect` implementations (parameterized: `DamageTarget(3)`, `DestroyAll(filter)`, `DrawCards(2)`, etc.)
   - A library of reusable `Ability` wrappers (`EntersBattlefieldTriggered(effect)`, `ActivatedAbility(cost, effect)`, `StaticAbility(continuous_effect)`, etc.)
   - A library of reusable `Target` specifications (`AnyTarget`, `CreaturePermanent`, `SpellOnStack`, `CardInGraveyard(filter)`, etc.)
2. **R8.2** — Card definitions should be **declarative** — describing what the card is, not how the rules engine processes it. The engine should handle all game logic.
3. **R8.3** — Support **parameterized effects** with dynamic values: effects like "deals damage equal to the number of creatures you control" should accept a `DynamicValue` that computes the amount at resolution time.
4. **R8.4** — Support **keyword abilities as composable objects** rather than boolean flags, so they can be added/removed by continuous effects (R3) and participate in the layer system.
5. **R8.5** — Support **custom effects** for unique cards that can't be composed from the standard library, while keeping the same interface so the engine handles them uniformly.
6. **R8.6** — All card objects must be cheaply cloneable for game state snapshots (critical for our MCTS/CFR AI goals).

### R9: Game State Architecture

**Current state**: `GameState` is cloneable and contains all zones, players, and game metadata. Missing zone change counters and formal state snapshot/restore.

**Requirements**:

1. **R9.1** — Implement **zone change counters**: each game object gets a monotonically increasing counter that increments whenever it changes zones. Effects and abilities store the counter value when they capture a reference; if the values differ later, the reference is stale.
2. **R9.2** — Support **full state snapshot and restore**: `save()` creates an immutable copy, `restore(snapshot)` overwrites the current state. This enables undo (within a turn) and AI tree search.
3. **R9.3** — The game state must include all active continuous effects, pending triggers, delayed triggers, and watcher state — not just zones and permanents.
4. **R9.4** — Implement a **values/metadata store** on the game state for arbitrary game-level state (e.g., monarch, initiative, day/night, storm count).

### R10: Zone Management

**MTG Rules**: 400-408 (Zones)

**Current state**: Library, hand, battlefield, graveyard, stack, and exile zones are implemented. Missing the command zone and formal zone-change event firing.

**Requirements**:

1. **R10.1** — Implement the command zone for Commander format support.
2. **R10.2** — Every zone change must fire a zone-change event that the event system (R5) can observe. This enables triggered abilities like "whenever a creature enters the battlefield" and replacement effects like "if this would die, exile it instead."
3. **R10.3** — Zone changes must update the zone change counter (R9.1) on the moving object.
4. **R10.4** — Tokens that exist in any zone other than the battlefield should cease to exist as an SBA (R2.1).

### R11: Turn Structure

**MTG Rules**: 500-514 (Turn Structure)

**Current state**: 13 phases modeled as an enum with sequential progression. Basic untap/draw/combat steps.

**Requirements**:

1. **R11.1** — Model turns as a **hierarchical structure** (Turn > Phase > Step) rather than a flat enum, enabling manipulation at each level (extra turns, extra phases, extra steps, skips).
2. **R11.2** — Support **extra turns** queued by spells/abilities, processed before/after the normal turn sequence.
3. **R11.3** — Support **phase/step skipping** via replacement effects (e.g., "skip your draw step").
4. **R11.4** — Support **"end the turn" effects** (e.g., Time Stop) that exile the stack, clear combat, skip remaining phases, and jump to cleanup.
5. **R11.5** — Each step should define whether players receive priority (untap and cleanup normally do not).
6. **R11.6** — Mana pools should empty at phase boundaries (with appropriate "mana doesn't empty" effect support).

### R12: Targeting System

**MTG Rules**: 115 (Targets)

**Current state**: Basic target enumeration with legality checks for hexproof/shroud and valid creature targets.

**Requirements**:

1. **R12.1** — Implement a **composable filter system** for target specifications. Filters should combine with AND/OR/NOT logic: "target creature or planeswalker an opponent controls" = `OR(Creature, Planeswalker) AND OpponentControls`.
2. **R12.2** — Targets must be chosen when placing a spell/ability on the stack, then re-validated on resolution. If a target becomes illegal between casting and resolution, the spell/ability is countered if ALL targets are illegal (Rule 608.2b).
3. **R12.3** — Distinguish between **targeted** effects ("target creature") and **non-targeted** effects ("all creatures"). Non-targeted effects bypass hexproof, shroud, and protection.
4. **R12.4** — Support **modal spells** where the player chooses one or more modes, each with potentially different targets.

---

## 7. Implementation Priorities

Based on impact to rules correctness and our GTO simulation goals, here is the recommended implementation order:

### Phase 1: Foundation (Critical for Correct Gameplay)

| Priority | Requirement | Rationale |
|----------|-------------|-----------|
| **P0** | R5: Event System | Foundation for R4, R6, and R11. Every other system depends on events. |
| **P0** | R3: Continuous Effects & Layers | Without layers, no continuous effect interaction is correct. Blocks all enchantments, equipment, anthems, etc. |
| **P0** | R1: Priority System (SBA+trigger loop) | Current priority system can miss triggered abilities and create illegal states. |

### Phase 2: Core Mechanics (Required for Non-Trivial Cards)

| Priority | Requirement | Rationale |
|----------|-------------|-----------|
| **P1** | R4: Replacement Effects | Needed for "enters tapped", "if this would die, exile instead", damage prevention, etc. |
| **P1** | R6: Triggered Abilities (full) | Delayed triggers, APNAP ordering, and proper recurrence needed for most cards. |
| **P1** | R8: Card Composition Framework | Needed to scale beyond 20 cards without massive boilerplate. |
| **P1** | R2: State-Based Actions (full suite) | Legend rule, aura checks, counter annihilation needed for broader card support. |

### Phase 3: Polish & Scale (Broader Card/Format Support)

| Priority | Requirement | Rationale |
|----------|-------------|-----------|
| **P2** | R12: Targeting System (composable filters) | Needed for cards with complex targeting requirements. |
| **P2** | R9: Game State Architecture (zone change counters) | Needed for correct "lose track" behavior and robust AI search. |
| **P2** | R7: Combat System (requirements/restrictions) | Needed for goad, menace validation, and combat-modifying effects. |
| **P2** | R11: Turn Structure (extra turns, skips) | Needed for Time Walk, extra combat effects, etc. |
| **P2** | R10: Zone Management (command zone, events) | Needed for Commander support. |

### Phase 3 Note: Card Count Scaling

xMage scales to 28,000 cards because of their composition framework. For our GTO simulator, we likely don't need all 28,000 cards — but we need enough to model interesting metagames. The card composition framework (R8) is what makes this tractable. With a good library of ~50-100 reusable effects and ~20 ability templates, we could express most Standard-legal cards declaratively.

---

*Document generated from analysis of [github.com/magefree/mage](https://github.com/magefree/mage) (xMage v1.4.58, MIT License)*
