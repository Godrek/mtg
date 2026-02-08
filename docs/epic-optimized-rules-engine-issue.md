# Epic: Optimized Rules Engine for High-Performance GTO Simulation

## Summary
Create a high-performance, composable rules engine architecture for our Rust MTG simulator that incorporates the critical xMage patterns (event system, continuous effect layers, replacement effects, robust SBA/trigger loop) while remaining optimized for large-scale GTO-approximate simulation (fast state snapshots, deterministic batching, and minimal per-action overhead). This Epic is informed by both the codex and claude xMage reviews and translates their requirements into an implementation roadmap tailored to our Rust codebase and simulation goals. 【F:docs/codex-xmage-rules-engine-review.md†L1-L94】【F:docs/claude-xmage-rules-engine-review.md†L1-L861】

## Goals
- Implement a **layered continuous effects system** (CR 613) with timestamp + dependency resolution that can be recalculated efficiently. 【F:docs/claude-xmage-rules-engine-review.md†L224-L274】
- Build a **formal event system** with pre/post events, batch events, and replacement effects. 【F:docs/claude-xmage-rules-engine-review.md†L275-L342】
- Enforce the **SBA + trigger recurrence loop** before priority is granted. 【F:docs/claude-xmage-rules-engine-review.md†L121-L170】
- Introduce a **composition-based card framework** that scales to larger card pools without engine changes. 【F:docs/claude-xmage-rules-engine-review.md†L374-L539】
- Optimize for **simulation throughput**: fast state cloning, minimal allocations, and deterministic, batchable state transitions. 【F:docs/codex-xmage-rules-engine-review.md†L36-L92】

## Non-Goals
- Full 28k-card parity with xMage (not required for our simulator). 【F:docs/claude-xmage-rules-engine-review.md†L857-L861】
- GUI/client implementation or multiplayer beyond 2 players.

## Epic Scope (Deliverables)
1. **Event & Replacement System**
   - Typed events with pre/post hooks.
   - Replacement effect pipeline with applied-effects tracking and player choice ordering. 【F:docs/claude-xmage-rules-engine-review.md†L275-L342】
   - Batch events for simultaneous actions (zone change batches, damage batches). 【F:docs/claude-xmage-rules-engine-review.md†L312-L321】

2. **Continuous Effects Layer Engine**
   - CR 613 layering model with sublayers and timestamp ordering.
   - Dependency-aware resolution within layers.
   - Reset-to-base + reapply model with caching where safe. 【F:docs/claude-xmage-rules-engine-review.md†L224-L274】

3. **Priority + SBA/Trigger Loop**
   - Recurrence loop: SBA → triggers → stack → repeat until stable.
   - APNAP ordering for triggered abilities and priority. 【F:docs/claude-xmage-rules-engine-review.md†L121-L170】

4. **Composable Card Framework**
   - Libraries of reusable effects/abilities/targets.
   - Dynamic values for runtime-calculated amounts.
   - Support custom effects without modifying engine core. 【F:docs/claude-xmage-rules-engine-review.md†L473-L523】

5. **High-Performance State Model**
   - Zone-change counters and robust snapshot/restore for AI search.
   - Efficient, deterministic state mutation pipeline (read → plan → apply). 【F:docs/claude-xmage-rules-engine-review.md†L540-L594】【F:docs/codex-xmage-rules-engine-review.md†L81-L92】

## Architecture Proposal (Optimized for GTO Simulation)

### A. Deterministic Event Pipeline (Core)
**Why:** Needed for triggers, replacement effects, and batch resolution with minimal recomputation. 【F:docs/claude-xmage-rules-engine-review.md†L275-L321】

**Plan:**
- Introduce `Event` + `EventBus` with `pre_event` and `post_event` hooks.
- Maintain a compact `EventContext` struct: `type`, `source`, `target`, `player`, `amount`, `applied_effects`.
- Add `SimultaneousEventBatch` for "one-or-more" triggers.
- Use arena-allocated vectors or smallvec-like patterns to minimize allocations in tight loops.

### B. Layered Continuous Effects Engine (CR 613)
**Why:** Correct continuous effect interaction is the biggest rules gap and required for realistic simulations. 【F:docs/claude-xmage-rules-engine-review.md†L224-L274】

**Plan:**
- Add `ContinuousEffects` manager with:
  - `active_effects: Vec<EffectInstance>`
  - Layered application pipeline with timestamps.
  - Dependency graph (lightweight) for same-layer ordering.
- Recompute layers after each action and before priority, but optimize with:
  - Stable base characteristics cache per object.
  - Dirty flags per object to skip reapply if no relevant effects changed.

### C. SBA + Trigger Recurrence Loop
**Why:** Ensures rules correctness and avoids illegal states. 【F:docs/claude-xmage-rules-engine-review.md†L121-L170】

**Plan:**
- Implement `check_state_and_triggers()` that loops:
  - Apply SBAs (full 704.5 suite).
  - Queue triggers.
  - If any changes or queued triggers, repeat.
- Only after stability grant priority.

### D. Composable Card System
**Why:** Scale card pool while keeping engine stable and minimal. 【F:docs/claude-xmage-rules-engine-review.md†L374-L539】

**Plan:**
- Introduce core libraries: `effects::common`, `abilities::common`, `targets::common`.
- Card definitions become declarative compositions of effect/ability structs.
- Add `DynamicValue` trait for runtime-computed values.
- Provide `CustomEffect` trait for outliers.

### E. Simulation-Optimized State
**Why:** GTO approximation needs fast branching and rollback. 【F:docs/claude-xmage-rules-engine-review.md†L540-L594】

**Plan:**
- Add zone-change counters on all objects.
- Add `GameStateSnapshot` struct for fast copy/restore.
- Use batched read/compute/apply phases to avoid borrow conflicts and reduce mutations.

## Milestones & Work Breakdown

### Phase 1 (P0): Foundations
- [ ] Implement event system + pre/post hooks + batch events.
- [ ] Implement layered continuous effects engine (CR 613).
- [ ] Add SBA + trigger recurrence loop in priority handling.

### Phase 2 (P1): Core Rules Scaling
- [ ] Replacement effects pipeline with player-choice ordering.
- [ ] Full triggered ability system (APNAP ordering, delayed triggers).
- [ ] Composable card framework + dynamic values.
- [ ] Expand SBA coverage to full 704.5 suite.

### Phase 3 (P2): Simulation & Quality
- [ ] Zone-change counters + state snapshot/restore.
- [ ] Target filter composition system.
- [ ] Combat requirement/restriction integration.
- [ ] Turn structure extras (extra turns, skip steps).

## Acceptance Criteria
- Engine correctly resolves layered effects in deterministic order.
- Replacement effects properly chain and do not re-apply to the same event.
- Triggers and SBAs reach a stable loop before priority is granted.
- At least 50–100 cards can be expressed declaratively using effect/ability libraries.
- Simulation throughput improves measurably on benchmark decks (target: ≥2x current games/sec).

## Risks & Mitigations
- **Risk:** Recomputing layers after every action may be too slow.
  - **Mitigation:** dirty flags + cached base stats, incremental recomputation when possible.
- **Risk:** Replacement effects create complex ordering bugs.
  - **Mitigation:** central ordering logic + applied-effects tracking + exhaustive tests.
- **Risk:** Card composition library grows too slowly to be useful.
  - **Mitigation:** define a prioritized list of 50–100 core effects for initial coverage.

## References
- codex xMage rules review. 【F:docs/codex-xmage-rules-engine-review.md†L1-L94】
- claude xMage rules review. 【F:docs/claude-xmage-rules-engine-review.md†L1-L861】
