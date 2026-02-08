# xMage Rules Engine Review (High-Level)

## Access Constraints

During this review, network access to `github.com/magefree/mage` was blocked in the execution environment (HTTP 403 via HTTPS CONNECT tunnel). As a result, the conclusions below are based on public, high-level knowledge of xMage/Mage architecture patterns and common MTG engine design practices rather than direct repository inspection. The requirements section translates those high-level patterns into actionable requirements for our simulator, but should be validated against the upstream repository when access is available.

## High-Level Approach (Inferred)

### 1) Engine-Core vs. Card Implementations
xMage is widely known to separate a reusable game rules engine (core) from per-card implementations. The rules engine provides:
- A canonical game state (zones, players, turns, phases, stack, continuous effects).
- A rules processor that drives turn structure, priority, and spell/ability resolution.
- Centralized mechanisms for continuous effects (layers), triggered abilities, replacement effects, and state-based actions (SBAs).

Card implementations are typically written as individual classes (one per card) that reference the engine’s effect/ability APIs. This allows adding thousands of cards without forking or rewriting core rules logic.

### 2) Effects/Abilities as Composable Objects
Cards define their behavior by composing effect and ability objects provided by the engine. Examples of common patterns:
- **Static abilities** (continuous effects) expressed via layer-based modifiers.
- **Triggered abilities** registered to event hooks (enter-the-battlefield, dies, combat damage, upkeep, etc.).
- **Activated abilities** with costs and targets, using standardized targeting APIs.
- **Replacement effects** that intercept events (e.g., “if it would die, exile instead”).

This structure allows a small number of engine primitives to cover many card behaviors, including custom logic in card-specific subclasses when necessary.

### 3) Rules Engine Handles Comprehensive Rules Complexity
The engine models critical MTG rules interactions:
- **Layer system** for continuous effects.
- **State-based actions** applied until stable.
- **Trigger ordering** and APNAP (active player, non-active player) resolution.
- **Priority and stack resolution**, including spell/ability counters.

Card code stays declarative, while the engine resolves interactions and conflicts.

### 4) Scalability for 30,000+ Cards
xMage’s scale suggests:
- **Strong separation between data and rules logic**, avoiding bespoke logic in the engine for single cards.
- **Reusable effect/ability primitives** to minimize per-card code size.
- **Automated testing and regression safety** via engine-level tests plus card-level tests.
- **Data-driven card definitions** where possible (even if Java classes are still used), often aided by scripting and code generation.

### 5) Extensibility and Maintenance
To support many cards and frequent updates:
- The engine offers extension points (interfaces/abstract classes) to define new abilities/effects without modifying core logic.
- Card code is modular and isolated by set, helping incremental updates and bug fixes.
- Rules engine stays stable; card packages evolve rapidly.

## Requirements for Our Simulator (Translated)

Below are concrete requirements for our Rust-based simulator, derived from these high-level patterns. Each requirement is framed as a capability or architectural constraint.

### A) Core Engine vs. Card Layer
1. **Strict Engine/Card Separation**
   - The engine must expose stable APIs for phases, stack, targeting, effects, and triggers.
   - Card definitions should not modify engine internals; they must compose engine primitives.
2. **Card Behavior as Data + Reusable Effects**
   - Prefer data-driven definitions using a small, extensible set of effect/ability building blocks.
   - Allow card-specific custom logic, but keep it behind a controlled extension trait/interface.

### B) Rules Engine Completeness
3. **Layer System for Continuous Effects**
   - Implement a formal layer pipeline (copy, control, text/type, color, abilities, P/T, etc.).
   - Ensure deterministic ordering and stable reapplication each time state changes.
4. **State-Based Actions Loop**
   - Apply SBAs repeatedly until no more changes.
   - Trigger dies triggers only after SBA stabilization (consistent with current design).
5. **Trigger Queue + APNAP Ordering**
   - Centralized trigger collection, then flush in APNAP order.
   - Support event-scoped triggers plus global watchers.
6. **Priority + Stack Resolution**
   - Maintain priority passes, stack-based spell/ability resolution, and counter logic.

### C) Scalability & Maintainability
7. **Extensible Effect/Ability Registry**
   - Provide a registry for effect types and target specs.
   - Avoid engine branching for specific cards where possible.
8. **Data-Driven Card Ingestion**
   - Support loading large card pools from external JSON or CSV sources, mapping to core effects.
   - Enable code generation for card definitions to reduce boilerplate.
9. **Performance in Bulk Simulation**
   - Use immutable read + batched write phases to avoid borrow conflicts and reduce overhead.
   - Maintain efficient state copies for simulation branches (e.g., compact snapshots or diff-based).
10. **Automated Regression Testing**
   - Expand integration tests with representative rules interactions.
   - Ensure new cards can be added with minimal engine regression risk.

### D) Governance & Workflow
11. **Stable Engine API Contracts**
   - Version engine APIs, allowing card sets to target stable interfaces.
12. **Card Validation Pipeline**
   - Provide linting/validation to detect invalid targeting, undefined effects, or rule violations.

## Next Steps
- **Validate** the above points directly against the xMage repository once GitHub access is available.
- **Map** each requirement to current modules (rules, action, card, game) and create a prioritized roadmap.
- **Prototype** a data-driven card ingestion path to reduce per-card Rust code.
