# Architecture Review: MCCFR Strategy (PR #6) vs. Rules Engine Epic (PR #7)

## Summary

PR #6 proposes a GTO strategy system built on MCCFR (Monte Carlo Counterfactual Regret Minimization). PR #7 proposes a ground-up rewrite of the rules engine with an event system, CR 613 layered effects, proper SBA/trigger loops, and a composable card framework. Both are well-structured plans. There are **no hard architectural blockers** between them, but there are several **coupling points, sequencing dependencies, and shared-interface risks** that need explicit coordination to avoid rework.

The `Strategy` trait (`src/strategy/mod.rs:9-12`) is a clean separation boundary — MCCFR consumes the rules engine as a black box through `choose_action(&self, state: &GameState, player: PlayerIndex) -> Action`. The architectural intent is already aligned. But the implicit coupling through `GameState`, `Action`, and `legal_actions()` means developing them in isolation will lead to integration pain.

---

## Coupling Points & Dependencies

### 1. `GameState` Is the Central Shared Interface — Both PRs Reshape It

**Current state** (`src/game/mod.rs:157-210`): All fields are `pub`. MCCFR's `compute_info_set()` and PR #7's event system will both directly read/write these fields.

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
- PR #7 should expose a `visible_state(player) -> PartialGameState` (PR #6 explicitly requests this in its Phase 0)
- Both PRs should code against this agreed-upon observation interface, not raw struct fields

### 2. `legal_actions()` Semantics Will Diverge

**Current state**: `legal_actions()` at `src/action/mod.rs:78-222` is a monolithic ~150-line function that generates actions based on phase, priority, and board state. It caps attacker subsets at 10 (`generate_subsets(&eligible, 10)` on line 104) and uses a greedy mana check (`can_potentially_pay()` on line 175).

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

## Concrete Issues Found in the Current Codebase

These are existing problems that **neither PR addresses** but that will bite during integration:

### Issue A: Engine Nondeterminism Breaks MCCFR (Critical)

The current engine uses `rand::thread_rng()` in two places that are **not player decisions**:

- `discard_random()` at `src/rules/mod.rs:843-853` — randomly selects cards to discard
- `draw_cards()` doesn't randomize, but `setup_game()` at line 1183 shuffles with `rand::thread_rng()`

MCCFR requires that `apply_action(state, action)` be a **deterministic** function: same state + same action = same resulting state. Currently, if the engine forces a random discard (e.g., cleanup step discard to 7 at line 796-799), two calls with the same state produce different results.

**Fix needed before either PR**: The RNG must be part of `GameState` (a seeded `StdRng` or similar), or discard selection must be surfaced as a player `Action` (which it should be per the rules — players choose which cards to discard). This is a prerequisite for MCCFR correctness.

### Issue B: `card_db` Clone Cost Is Already a Problem

`GameState` at line 162 stores `card_db: Option<CardDatabase>`, where `CardDatabase` is `HashMap<CardId, CardDef>`. Every `state.clone()` deep-clones this entire database — all card names (Strings), oracle text, keyword vectors, ability lists.

The `#[serde(skip)]` annotation prevents serialization but **not cloning**. For MCCFR's millions of clones, this is the single largest allocation cost.

**Fix needed before MCCFR**: Change `Option<CardDatabase>` to `Arc<CardDatabase>`. This is a one-line type change that makes clones O(1) for the database while preserving shared immutable access. The `#[serde(skip)]` already prevents it from serializing, so `Arc` just needs `Clone` (which it has).

### Issue C: `flush_triggers()` Silently Orders Player Decisions

At `src/rules/mod.rs:519-553`, `flush_triggers()` puts triggers on the stack in FIFO order within each player's group. The comment at line 518 says "the controller chooses (simplified: FIFO)."

This is a real player decision that MCCFR must see. When a player has multiple triggered abilities, the order they go on the stack determines resolution order (LIFO), which can matter strategically. PR #7 plans to fix this with proper APNAP ordering, but the fix must surface as an `Action` variant, not just a different silent ordering.

### Issue D: `auto_tap_lands()` Hides Mana Decisions

At `src/rules/mod.rs:1055-1175`, `auto_tap_lands()` uses a greedy algorithm to decide which lands to tap. In GTO play, mana sequencing matters (e.g., tapping a dual land for the wrong color can lock you out of casting an instant on the opponent's turn).

For Phase 1 MCCFR (simplified games with basic lands), this doesn't matter. But for Phase 2+ (realistic decks with dual lands), this either needs to become a player decision or be acknowledged as an approximation.

### Issue E: Combat Action Space Explosion

`generate_subsets()` at `src/action/mod.rs:403-423` generates `2^n` attacker subsets (capped at `n=10`, so up to 1024). `generate_blocking_assignments()` at lines 479-580 generates single blocks, multi-blocks, and cross-blocks.

In MCCFR external sampling, the traversing player explores **all** their own actions at each decision point. With 1024 attack combinations, each requiring a full recursive traversal, a single combat step can create 1024 subtrees. Combined with blocking responses, combat is by far the most expensive part of the game tree.

PR #6 mentions action abstraction but doesn't propose a specific combat abstraction. This needs a concrete plan: e.g., bucket attacks into "all-out", "leave back biggest", "leave back all", "no attack" rather than enumerating all subsets.

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

0. **Pre-requisite fixes** (do immediately):
   - `Arc<CardDatabase>` instead of `Option<CardDatabase>` — unblocks MCCFR clone performance
   - Deterministic RNG in GameState — unblocks MCCFR correctness
   - Surface discard-to-hand-size as a player Action — rules correctness + MCCFR correctness
1. **Shared interface definition**: Agree on the observation/action API that both systems code against
2. **PR #7 Phase 1** (SBA loop, event system foundations) — this stabilizes game state transitions
3. **PR #6 Phase 0** (info sets, canonical actions, features) — built against the new interfaces
4. **PR #7 Phase 2** (continuous effects, composable cards) — can proceed in parallel with:
5. **PR #6 Phase 1** (tabular MCCFR on simplified games) — uses the stabilized engine

---

## Specific Issues in Each PR

### PR #6 (MCCFR Strategy Plan)

1. **Attacker subset explosion**: `legal_actions()` already caps at 10 creatures but still generates up to 1024 attack combinations. The plan mentions action abstraction but doesn't propose a specific combat abstraction. MCCFR traverses ALL actions for the traversing player — 1024 subtrees per combat step is intractable even for simplified games if both decks can generate boards with 5+ creatures.

2. **The `card_db` skip on serde is a problem for training serialization**: If MCCFR checkpoints `GameState` via serde, the card_db won't be included. Training needs to handle this (reconstruct card_db on deserialization or use a separate checkpoint format). Switching to `Arc<CardDatabase>` (recommended above) also solves this since the Arc can be reattached after deserialization.

3. **Draw-step chance node handling**: The plan asks "should we determinize or treat as chance node?" — for correctness, treat as a chance node. Determinization introduces strategy artifacts where the solver "knows" the draw even though the player doesn't. External sampling handles this naturally.

4. **Missing: stack interaction decisions**: PR #6 doesn't discuss how MCCFR handles priority decisions during stack resolution (e.g., "opponent casts a spell, do I respond?"). The current engine already supports this — `legal_actions()` generates `CastSpell` actions for instants when the stack is non-empty (line 164-168). But the MCCFR plan doesn't address the branching factor of instant-speed interaction, which is where the deepest game trees occur.

5. **Missing: cleanup discard is currently random, not a player choice**: `discard_random()` at `rules/mod.rs:843` randomly discards, but in MTG the player chooses. MCCFR needs this as an Action variant or it's training against a wrong game.

### PR #7 (Rules Engine Epic)

1. **Event bus allocation pattern**: "Arena-allocated vectors or smallvec-like patterns" for events — this conflicts with `Clone`-based snapshotting. Arena allocators don't clone well. Need to decide whether events are transient (not part of state) or persistent. Recommendation: events should be transient — fire, process, discard. Only the resulting state changes persist in the snapshot.

2. **"Dirty flags per object to skip reapply"** for continuous effects — this optimization assumes effects change infrequently. During MCCFR traversal where the engine explores many branches from the same base state, dirty flags will thrash (every branch dirties everything). The optimization may be counterproductive for the AI use case. Consider making dirty-flag optimization opt-in, with a simple "always recompute" path for solver traversal.

3. **"Batched read/compute/apply phases"** — good for single-threaded correctness, but MCCFR needs to fork state at arbitrary points mid-batch. The batch pipeline must support clone-at-any-point or the solver can't branch.

4. **Acceptance criterion "≥2x current games/sec"** is underspecified. Measured how? With which decks? Under MCCFR traversal (many clones, rollbacks) or end-to-end simulation? Recommend two benchmarks: (a) games/sec for end-to-end simulation with GreedyStrategy, and (b) states-evaluated/sec for tree traversal (clone, legal_actions, apply_action, check_terminal).

5. **SBA loop currently doesn't re-check after trigger resolution**: `check_state_based_actions()` at `rules/mod.rs:607-681` fires dies triggers at the end but doesn't loop back to check if those triggers caused new SBAs (e.g., a dies trigger that deals damage, reducing another creature's controller's life to 0). PR #7's recurrence loop fixes this, but it changes state transition behavior. Any MCCFR policies trained against the current engine won't transfer to the corrected engine — this reinforces the "rules engine Phase 1 first" sequencing.

---

## Verdict: No Hard Blockers, But Real Coordination Needed

The two proposals are architecturally **compatible** — MCCFR consumes the rules engine as a black box, and the rules engine doesn't need to know about MCCFR. The `Strategy` trait is a clean separation boundary, and the README's framing of cloneable GameState + strategy interface as the solver contract is the right architecture.

However, the **implicit coupling through `GameState`, `Action`, and `legal_actions()`** means that developing them in isolation will lead to integration pain. The highest-priority actions are:

### Three interface decisions to make now

1. **Will `GameState` expose a `visible_state(player)` method?** Both PRs want this. Define the type signature and semantics. This is the observation API that MCCFR codes against — it decouples info set computation from internal engine state.

2. **Will ordering choices (replacement effects, triggers) be `Action` variants?** MCCFR needs them to be. The rules engine needs to surface them. Also: discard-to-hand-size must become a player choice, not random.

3. **What is the snapshot/clone contract?** MCCFR needs cheap branching. The rules engine adds state. Agree on what gets snapshotted vs. reconstructed. Immediate win: `Arc<CardDatabase>`.

### Three pre-requisite fixes before either PR lands

1. **`Arc<CardDatabase>`** — one-line type change, massive clone cost reduction
2. **Seeded RNG in GameState** — deterministic transitions for MCCFR
3. **Discard-to-hand-size as Action** — rules correctness and MCCFR correctness
