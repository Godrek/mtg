# Architecture Review: MCCFR Strategy (PR #6) vs. Rules Engine Epic (PR #7)

## Summary

PR #6 proposes a GTO strategy system built on MCCFR (Monte Carlo Counterfactual Regret Minimization). PR #7 proposes a ground-up rewrite of the rules engine with an event system, CR 613 layered effects, proper SBA/trigger loops, and a composable card framework. Both are well-structured plans. There are **no hard architectural blockers** between them, but there are several **coupling points, sequencing dependencies, and shared-interface risks** that need explicit coordination to avoid rework.

---

## Coupling Points & Dependencies

### 1. `GameState` Is the Central Shared Interface — Both PRs Reshape It

**PR #7 (Rules Engine)** will fundamentally restructure `GameState`:
- Adds an event bus, continuous effects manager, zone-change counters
- Restructures how effects are represented and resolved
- Changes how SBAs and triggers fire (recurrence loop before priority)
- Adds a batched read/compute/apply mutation pipeline
- Adds `GameStateSnapshot` for fast copy/restore

**PR #6 (MCCFR)** depends heavily on stable `GameState` semantics:
- `compute_info_set(state, player) -> InformationSet` reads from GameState fields
- `legal_actions(state)` must return consistent, canonical action lists
- MCCFR traversal calls `state.clone()` millions of times per training run
- Feature extraction reads life, board, hand, mana from GameState

**Risk**: If the rules engine refactor lands first (or concurrently), every MCCFR module that touches GameState needs to be rewritten against the new API. If MCCFR lands first against the current GameState, the rules engine refactor will break the MCCFR information set computation, feature extraction, and canonical action mapping.

**Recommendation**: Define the `GameState` public API contract first as a shared interface document. Specifically:
- Which fields/methods will MCCFR use to observe game state?
- PR #7 should expose a `visible_state(player) -> PartialGameState` (PR #6 explicitly requests this)
- Both PRs should code against this agreed-upon observation interface, not raw struct fields

### 2. `legal_actions()` Semantics Will Diverge

**Current state**: `legal_actions()` in `src/action/mod.rs` is a monolithic 150-line function that generates actions based on phase, priority, and board state. It caps attacker subsets at 10 and uses a simple mana check.

**PR #7** will change when and how legal actions are generated:
- The SBA/trigger recurrence loop means priority isn't granted until the game reaches a stable state — `legal_actions()` may never be called in states that the current engine allows
- New effect types (replacement effects, continuous effects modifying costs/restrictions) will change what actions are legal
- The event system may add new action types (choosing replacement effect ordering, choosing trigger ordering)

**PR #6** needs `legal_actions()` to be:
- **Deterministic**: Same information set must produce the same action list
- **Canonicalizable**: Actions must map to stable `CanonicalAction` identifiers
- **Complete**: All meaningful choices must be represented (PR #6 warns about "trivially forced" decisions needing pruning)

**Risk**: The rules engine adds new decision points (replacement effect ordering, APNAP trigger ordering) that PR #6's `CanonicalAction` enum doesn't account for. These are real game decisions that affect GTO play.

**Recommendation**: PR #6's `CanonicalAction` needs a variant for player-choice ordering decisions (e.g., `ChooseReplacementOrder`, `OrderTriggers`). These should be designed upfront even if the rules engine implements them later.

### 3. `Clone` / Snapshot Performance Is a Shared Bottleneck

**PR #6** requires millions of `state.clone()` calls per MCCFR training session. Current `GameState::clone()` is O(n) on the objects HashMap — workable but not cheap.

**PR #7** proposes adding:
- An event bus with subscribers
- A continuous effects manager with active effects lists and dependency graphs
- Zone-change counters
- A `GameStateSnapshot` for fast copy/restore

**Risk**: If the rules engine adds event bus state, effects manager state, and subscriber lists to `GameState`, the clone cost increases significantly. The event bus pattern (with closures/callbacks) may not even be `Clone`-able without careful design.

**Recommendation**:
- PR #7's event bus must be designed as either stateless (recomputed from game state) or excluded from snapshots
- PR #7's proposed `GameStateSnapshot` should be the mechanism MCCFR uses for branching, not raw `Clone`
- Consider a two-tier design: hot path state (zones, life, mana) that's cheap to snapshot, and cold path state (event registrations, effect metadata) that's reconstructed

### 4. Effect Resolution Determinism

**PR #6** requires deterministic game simulation. MCCFR traversal must produce identical outcomes for identical (state, action) pairs so that regret values are meaningful.

**PR #7** introduces:
- Replacement effects with **player choice ordering** — this is a new source of nondeterminism from the player's perspective
- Batch events for simultaneous actions — resolution order matters
- Continuous effects with timestamp ordering and dependency resolution

**Risk**: If replacement effect ordering or trigger ordering is handled inconsistently (e.g., arbitrary ordering in the engine vs. player-choice per the rules), MCCFR training will either miss real decisions or be corrupted by phantom nondeterminism.

**Recommendation**: Every point where the rules engine makes a player-visible ordering choice must surface as an `Action` variant in `legal_actions()`. The engine must not make these choices silently. This is critical for MCCFR correctness.

### 5. Card Definition Format

**PR #7** proposes a composable card framework with `DynamicValue`, `CustomEffect`, and declarative compositions. This replaces the current hardcoded `CardDef` + `Effect` enum approach.

**PR #6** uses `CardId` extensively in `InformationSet` and `CanonicalAction`. The MCCFR system needs to map between game objects and abstract card identities.

**Risk**: Low. The card definition format is internal to the rules engine. As long as cards still have a stable `CardId` and discoverable properties (name, types, CMC), the MCCFR system doesn't care how effects are implemented.

**Non-risk**: This is the one area where the two PRs are cleanly decoupled.

---

## Sequencing Analysis

### Can they be developed in parallel?

**Partially yes, with constraints:**

| MCCFR Component | Depends on Rules Engine? |
|---|---|
| InformationSet struct & hashing | Yes — reads GameState fields |
| CanonicalAction mapping | Yes — depends on Action enum and legal_actions() |
| RegretTable / storage | No — self-contained data structure |
| MCCFR algorithm (traversal) | Yes — calls clone(), legal_actions(), apply_action() |
| Feature extraction | Yes — reads GameState fields |
| McfrStrategy (deployment) | Yes — calls legal_actions(), returns Action |
| Training loop & checkpointing | No — orchestration layer |

The MCCFR data structures (RegretTable, training loop, serialization) can be built independently. But every component that touches game state or actions is coupled to the rules engine's API.

### Recommended sequencing

1. **Shared interface definition** (do first): Agree on the observation/action API that both systems code against
2. **PR #7 Phase 1** (SBA loop, event system foundations) — this stabilizes game state transitions
3. **PR #6 Phase 0** (info sets, canonical actions, features) — built against the new interfaces
4. **PR #7 Phase 2** (continuous effects, composable cards) — can proceed in parallel with:
5. **PR #6 Phase 1** (tabular MCCFR on simplified games) — uses the stabilized engine

---

## Specific Issues in Each PR

### PR #6 (MCCFR Strategy Plan)

1. **Attacker subset explosion**: `legal_actions()` already caps at 10 creatures. The plan doesn't address how MCCFR handles this — with 10+ creatures, the current code generates `2^10 = 1024` attack combinations. MCCFR will need action abstraction for combat declarations, not just information set abstraction.

2. **The `card_db` skip on serde is a problem for training serialization**: If MCCFR checkpoints `GameState` via serde, the card_db won't be included. Training needs to handle this (reconstruct card_db on deserialization or use a separate checkpoint format).

3. **Draw-step chance node handling**: The plan asks "should we determinize or treat as chance node?" — for correctness, treat as a chance node. Determinization introduces strategy artifacts where the solver "knows" the draw even though the player doesn't. External sampling handles this naturally.

4. **Missing: stack interaction decisions**: PR #6 doesn't discuss how MCCFR handles priority decisions during stack resolution (e.g., "opponent casts a spell, do I respond?"). These are critical for GTO play with instants and are the most complex decision trees in MTG.

### PR #7 (Rules Engine Epic)

1. **Event bus allocation pattern**: "Arena-allocated vectors or smallvec-like patterns" for events — this conflicts with `Clone`-based snapshotting. Arena allocators don't clone well. Need to decide whether events are transient (not part of state) or persistent.

2. **"Dirty flags per object to skip reapply"** for continuous effects — this optimization assumes effects change infrequently. During MCCFR traversal where the engine explores many branches, dirty flags will thrash. The optimization may be counterproductive for the AI use case.

3. **"Batched read/compute/apply phases"** — good for single-threaded correctness, but MCCFR needs to fork state at arbitrary points. The batch pipeline must be interruptible/skippable for AI traversal.

4. **Acceptance criterion "≥2x current games/sec"** is underspecified. Measured how? With which decks? Under MCCFR traversal (many clones, rollbacks) or end-to-end simulation?

---

## Verdict: No Hard Blockers, But Real Coordination Needed

The two proposals are architecturally **compatible** — MCCFR consumes the rules engine as a black box, and the rules engine doesn't need to know about MCCFR. The `Strategy` trait is a clean separation boundary.

However, the **implicit coupling through `GameState`, `Action`, and `legal_actions()`** means that developing them in isolation will lead to integration pain. The highest-priority action is defining the shared observation/action interface that both PRs commit to, before either proceeds to implementation.

### Three things to decide now

1. **Will `GameState` expose a `visible_state(player)` method?** Both PRs want this. Define the type signature and semantics.
2. **Will ordering choices (replacement effects, triggers) be `Action` variants?** MCCFR needs them to be. The rules engine needs to surface them.
3. **What is the snapshot/clone contract?** MCCFR needs cheap branching. The rules engine adds state. Agree on what gets snapshotted vs. reconstructed.
