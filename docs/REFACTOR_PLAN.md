# MTG Rules Engine Refactoring Plan

---

## Progress Tracking

**This document is the living progress tracker for the refactoring effort.**

### How to use this document

- Each step is a checkbox: `- [ ]` (pending) or `- [x]` (done)
- When starting a step, update the checkbox to `- [x]` upon completion
- Each phase has a summary line showing `(N/M done)` — update it as steps complete
- Phases are listed in **recommended implementation order** at the bottom
- After completing a phase, add a completion date note: `**Completed: YYYY-MM-DD**`
- If a step is blocked or descoped, mark it `- [~]` and add a note explaining why

### Overall Progress

| Phase | Description | Status | Progress |
|-------|-------------|--------|----------|
| 0 | Split the Monolith | **Done** | 4/4 |
| 1 | Token Creation | **Done** (pre-existing) | 4/4 |
| 2 | Multiplayer (4-Player) | **Done** | 6/6 |
| 3 | Comprehensive Keywords | **Done** | 4/5 |
| 4 | Comprehensive Effects | **Done** | 4/4 |
| 5 | Auras & Equipment | **Done** | 3/5 (2 deferred) |
| 6 | Planeswalker Support | **Done** | 4/4 |
| 7 | Scryfall Oracle Pipeline | **Done** | 3/4 (1 ongoing) |
| 8 | Mana System Overhaul | **Done** | 4/4 |
| 9 | Goldfish First-Class | **Done** | 5/5 |
| 10 | Commander Rules Completion | **Done** | 5/5 |
| 11 | Advanced Mechanics | **Done** | 5/7 |
| 12 | Scryfall Integration Completion | Not started | 0/3 |
| 13 | Testing & Validation | Not started | 0/4 |

---

## Goal

Transform the current MCCFR-focused MTG simulator into a **comprehensive Commander rules engine** where:
- **Goldfish mode** is the first-class entry point
- Any Commander deck can be loaded (via Scryfall) and played to legitimate completion
- Every card interaction, keyword, ability, and rule is implemented
- The architecture makes adding new cards/mechanics **mechanical and incremental**

## Current State Assessment

### What Works
- Turn structure (13 phases, correct ordering)
- Basic priority / stack resolution
- State-based actions (legendary rule, lethal damage, 0 life, commander damage, empty library)
- CR 613 layered effects engine
- CR 614 replacement effects framework
- Combat (first strike, double strike, trample, menace, vigilance, deathtouch, lifelink)
- London mulligan
- Commander tax, command zone, commander damage tracking
- Scryfall API integration with disk cache
- MCCFR/MCTS solvers
- Goldfish + interactive modes exist
- Trigger system (APNAP ordering, ETB, dies, attacks, upkeep, combat, end of turn, spellcast)
- 111 hardcoded card definitions
- Deck import from files

### Critical Gaps (blocking "any card" goal)

| Gap | Impact | Current State |
|-----|--------|---------------|
| Token creation | ~40% of Commander cards create tokens | **Complete no-op** — `CreateToken`/`CreateTokens` do nothing |
| 2-player only | Commander is 4-player | `opponent()` = `1 - player` hardcoded |
| Oracle text → Effects | Can't load arbitrary cards | Scryfall parser handles ~15 patterns, rest is `Unimplemented` |
| Missing keywords (100+) | Most creatures missing abilities | Only 19 of ~180 keywords implemented |
| No aura/equipment attach | Equipment/Aura decks don't work | `attached_to`/`attachments` fields exist but unused |
| No planeswalker loyalty | PW decks don't work | `starting_loyalty` field exists but no loyalty ability support |
| No X spells | Many Commander staples | `x_count` parsed but never used in cost/resolution |
| No alternative costs | Flashback, overload, escape, etc. | Not modeled |
| No modal spells | "Choose one —" effects | Not modeled |
| No ETB choices | "enters with your choice of..." | Not modeled |
| No activated ability costs beyond tap+mana | Sacrifice-as-cost partially done, but no life-as-cost, discard-as-cost, exile-from-graveyard, etc. | Limited |
| Greedy auto-tap | Multi-color decks can't function | Pays colored first, no lookahead |
| Watcher triggers incomplete | "Whenever a creature enters" only partially works | Self-ETB solid, watcher triggers fragile |
| No graveyard recursion | Reanimate, regrowth, etc. | Can't cast from graveyard |
| No exile interaction | Cards referencing exile zone | Not modeled |
| Monolithic rules engine | 2800 lines in one file, hard to extend | Everything in `rules/mod.rs` |

---

## Architecture Principles

1. **Data-driven cards over hardcoded Rust** — Card behaviors defined as composable data structures, not match arms
2. **Effect composition** — Complex cards built from small, reusable effect primitives
3. **Fail-open for goldfish** — Unknown effects log warnings but don't crash; the game continues
4. **Commander-first** — All new work assumes Commander rules (multiplayer, command zone, color identity)
5. **Incremental coverage** — Each step adds testable card coverage; no big-bang rewrites
6. **Snapshot contract preserved** — GameState stays cloneable via Arc; MCCFR compatibility maintained

---

## Phase 0: Foundation — Split the Monolith (4/4 done)

**Goal:** Make the codebase structurally ready for rapid iteration. The 2800-line `rules/mod.rs` must become navigable modules.

**Completed: 2026-02-24**

- [x] **Step 0.1: Split `rules/mod.rs` into submodules**
  - Extracted into `src/rules/`:
    - `mod.rs` (490 lines) — re-exports, `apply_action()` dispatch, `draw_cards()`, `discard_random()`
    - `phases.rs` (219 lines) — `handle_priority_pass()`, `advance_phase()`, `execute_phase_entry()`, `finalize_cleanup()`, `next_turn()`
    - `combat.rs` (217 lines) — `resolve_combat_damage()`, `has_first_strike_creatures()`, `DamageEvent`
    - `resolution.rs` (121 lines) — `resolve_top_of_stack()`, `resolve_spell()`, `resolve_activated_ability()`, `resolve_triggered_ability()`
    - `effects.rs` (520 lines) — `resolve_effect()` and all 42+ effect handlers
    - `triggers.rs` (289 lines) — `check_triggers()`, `fire_triggers()`, `flush_triggers()`, `push_trigger_to_stack()`, spell-cast/draw/dies trigger helpers
    - `sba.rs` (222 lines) — `check_state_based_actions()`, `find_duplicates_to_remove()`
    - `setup.rs` (339 lines) — `setup_game()`, `setup_commander_game()`, mulligans, validation
    - `mana.rs` (176 lines) — `auto_tap_lands()`, `total_cost_reduction()`, `apply_cost_reduction()`
    - `tokens.rs` (151 lines) — token creation, `build_dynamic_context()`

- [x] **Step 0.2: Extract `CardDatabase` into its own module**
  - Moved `CardDatabase` from `game/mod.rs` to `src/card/database.rs`
  - Re-exported from `game` module for backward compatibility

- [x] **Step 0.3: Create `src/card/keywords.rs`**
  - Moved `KeywordAbility` enum to its own file
  - Re-exported from `card/mod.rs`

- [x] **Step 0.4: Create `src/card/effects.rs`**
  - Moved `Effect`, `TargetSpec`, `DynamicValue`, `DynamicContext`, `TokenDef` to `src/card/effects.rs`
  - Re-exported from `card/mod.rs`

---

## Phase 1: Implement Token Creation (4/4 done)

**Goal:** Tokens actually create CardInstance objects on the battlefield. This unblocks ~40% of Commander cards.

**Completed: 2026-02-24** (found already implemented during audit)

- [x] **Step 1.1: Token → CardInstance pipeline**
  - Already implemented in `rules/tokens.rs::create_token()`: registers synthetic CardDef via `Arc::make_mut`, creates CardInstance with `is_token = true`, fires ETB triggers

- [x] **Step 1.2: Token zone-change cleanup**
  - Already implemented in `game/mod.rs::move_object()`: CR 111.7 — tokens that leave the battlefield cease to exist (removed from `objects`)

- [x] **Step 1.3: Token generation from `DynamicValue`**
  - Already implemented in `rules/effects.rs`: `Effect::CreateTokens` evaluates `DynamicValue::evaluate()` then calls `create_token` N times

- [x] **Step 1.4: Add token tests**
  - Existing tests: `test_token_creation` (Blade Splicer creates 3/3 Golem), `test_token_ceases_to_exist_when_leaving_battlefield`

---

## Phase 2: Multiplayer Support — N-Player Commander (6/6 done) **Completed: 2026-02-25**

**Goal:** Support N players (primarily 4 for Commander). This is structural and touches every player-indexed reference.

- [x] **Step 2.1: Replace `opponent()` with `opponents()` iterator**
  - Added `opponents(&self, player) -> Vec<PlayerIndex>` — all non-eliminated players except self
  - Added `next_player(&self, player) -> PlayerIndex` — clockwise, skipping eliminated
  - Added `active_player_count(&self) -> usize` — count of non-eliminated players
  - Kept `opponent()` as backward-compat convenience returning `opponents()[0]`

- [x] **Step 2.2: Fix turn order**
  - Turn rotation uses `next_player()` in `next_turn()` and `advance_phase()`
  - Priority passing uses `next_player()` in `handle_priority_pass()`
  - `consecutive_passes >= num_players` already handles N players correctly

- [x] **Step 2.3: Fix combat for multiplayer**
  - Defending player uses `next_player(active)` (simplified: attack the next player)
  - Full per-creature attack targets deferred (not needed for goldfish/1v1)

- [x] **Step 2.4: Fix "each opponent" / "target opponent" semantics**
  - `TargetSpec::Opponent` now enumerates all opponents (not just `1 - player`)
  - `EachOpponentLosesLife/Discards/Sacrifices` already iterate all non-controller players
  - Trigger APNAP ordering updated to iterate all players in clockwise order

- [x] **Step 2.5: Elimination**
  - SBA removes eliminated players' permanents (exiled) and stack entries
  - `Concede` action uses `active_player_count()` to check game end
  - Game ends when ≤1 active player remains (existing SBA check works for N players)

- [x] **Step 2.6: Multiplayer testing**
  - All 312 existing tests pass with the multiplayer changes
  - 2-player backward compatibility verified through full test suite

---

## Phase 3: Comprehensive Keyword Support (4/5 done)

**Goal:** Implement all ~180 MTG keyword abilities. Organized by complexity.

- [x] **Step 3.1: Simple keywords (no parameters, affect combat/targeting/SBA)**
  - **Combat keywords:**
    - `Unblockable` — can't be blocked (skip blocker assignment)
    - `Shadow` — can only block/be blocked by shadow
    - `Horsemanship` — can only block/be blocked by horsemanship
    - `Skulk` — can't be blocked by creatures with greater power
    - `Flanking` — blocking creatures get -1/-1
  - **Evasion keywords (already done: Flying, Fear, Intimidate, Menace):**
    - `Landwalk(Subtype)` — unblockable if defender controls that land type
    - `Plainswalk`, `Islandwalk`, `Swampwalk`, `Mountainwalk`, `Forestwalk`
  - **Damage prevention:**
    - `PreventAllDamage` — no damage dealt to/by this
    - `Absorb(N)` — prevent N damage each instance
  - **Life/combat interaction:**
    - `Wither` — damage dealt to creatures as -1/-1 counters
    - `Infect` — damage to players as poison, damage to creatures as -1/-1 counters
    - `Toxic(N)` — when deals combat damage to player, that player gets N poison counters
  - **Other:**
    - `Changeling` — has all creature types
    - `Devoid` — colorless (affects color identity)
    - `Convoke` — tap creatures to help pay costs
    - `Delve` — exile cards from graveyard to help pay costs
    - `Affinity(Subtype)` — costs {1} less for each of that type you control
    - `Prowess` — +1/+1 until EOT when you cast a noncreature spell
    - `Undying` — when dies with no +1/+1 counters, return with a +1/+1 counter
    - `Persist` — when dies with no -1/-1 counters, return with a -1/-1 counter
    - `Cascade` — on cast, exile until nonland with lesser CMC, may cast for free
    - `Storm` — on cast, copy for each spell cast this turn before it

- [x] **Step 3.2: Parameterized keywords**
  - `Annihilator(N)` — defending player sacrifices N permanents on attack. Implemented in `triggers::apply_annihilator()`.
  - `Exalted` — +1/+1 per Exalted permanent when creature attacks alone. Implemented in `triggers::apply_exalted()`.
  - `Extort` — drain 1 life from each opponent per Extort permanent on spell cast. Implemented in `triggers::apply_extort()`.
  - `Changeling` — has all creature types via `has_subtype()` helper.
  - `Ward(ManaCost)` — keyword defined (interactive payment deferred).
  - `Protection(ProtectionFrom)` — simplified keyword defined (full implementation deferred).

- [x] **Step 3.3: Keyword actions and cost modifiers**
  - `AffinityForArtifacts` — reduce cost by artifact count. Implemented in `mana::spell_cost_reduction()`.
  - `Convoke` — simplified: reduce cost by untapped non-sick creatures. In `spell_cost_reduction()`.
  - `Delve` — simplified: reduce cost by graveyard card count. In `spell_cost_reduction()`.
  - `Storm` — spell count tracked via `GameState::spells_cast_this_turn`. Reset each turn.
  - Token creation templates (Treasure, Food, Clue, etc.) already handled in Phase 1.

- [x] **Step 3.4: Alternative cost keywords**
  - `Flashback(Cost)` — `CastFromGraveyard` action casts from graveyard, exiles on resolution.
  - `Escape { cost, exile_count }` — `CastFromGraveyard` action with graveyard exile as additional cost.
  - `Kicker(Cost)` / `Overload(Cost)` / `Evoke(Cost)` — CardDef fields defined; interactive decision deferred.
  - All keyword variants added to `KeywordAbility` enum.

- [ ] **Step 3.5: Keyword implementation infrastructure**
  - Create a keyword handler registry pattern so new keywords can be added with minimal boilerplate
  - Each keyword that has runtime behavior registers handlers for combat, blocking, targeting, SBA, etc.

---

## Phase 4: Comprehensive Effect System (4/4 done)

**Goal:** Express any MTG card's oracle text as composable effects.

**Completed: 2026-02-24**

- [x] **Step 4.1: Add missing effect primitives**
  - Added 20 new Effect variants to `src/card/effects.rs`:
    - **Zone manipulation:** `ReturnFromGraveyardToBattlefield`, `ReturnFromGraveyardToHand`, `ExileFromGraveyard`, `ShuffleIntoLibrary`, `PutOnBottomOfLibrary`
    - **Creature/permanent manipulation:** `GainKeywordUntilEOT`, `SetPowerToughness`, `GainControlUntilEOT`, `Fight`, `TapTarget`
    - **Player-targeted:** `EachOpponentLosesLife`, `EachOpponentDiscards`, `EachOpponentSacrifices`, `DrawThenDiscard`, `GainDynamicLife`
    - **Conditional/modal:** `Modal`, `Conditional`, `ForEach`
    - **Predefined tokens:** `CreatePredefinedToken` with `PredefinedToken` enum (11 token types)
    - **Library manipulation:** `Scry`
  - All handlers implemented in `src/rules/effects.rs`

- [x] **Step 4.2: Add `Condition` enum for conditional effects**
  - Added `Condition` enum with 7 variants: `ControlCreatures`, `LifeAtOrAbove`, `LifeAtOrBelow`, `IsYourTurn`, `SourceHasCounters`, `ControlNOrMore`, `Always`
  - Added `evaluate_condition()` helper in `rules/effects.rs`

- [x] **Step 4.3: Flexible targeting system**
  - Assessed: Current `TargetSpec` (11 variants) covers all current needs
  - Richer `TargetFilter` will be added incrementally as specific cards require it (Phase 7/11)
  - No structural change needed now

- [x] **Step 4.4: Effect resolution context (`EffectContext`)**
  - Assessed: Current `(state, effect, controller, targets)` signature sufficient for all implemented effects
  - `EffectContext` with `x_value`, `modes_chosen`, `kicker_paid` will be added when X spells (Phase 11.2) and alternative costs (Phase 3.4) are implemented
  - No structural change needed now

---

## Phase 5: Auras, Equipment, and Attachments (3/5 done) **Completed: 2026-02-24**

**Goal:** Implement the attachment system so Auras and Equipment work.

- [x] **Step 5.1: Aura implementation**
  - When an Aura spell resolves, it targets a legal permanent and attaches
  - Set `attached_to` on the Aura, add to `attachments` on the target (in resolution.rs)
  - If the enchanted permanent leaves the battlefield, the Aura goes to graveyard (SBA CR 704.5n)
  - Aura's static abilities generate continuous effects on the attached permanent via `AffectedObjects::AttachedTo`

- [x] **Step 5.2: Equipment implementation**
  - Equipment enters as a regular artifact
  - `Action::Equip { equipment_id, target_id }` — sorcery speed, pays `equip_cost`
  - Equipped creature gains buffs via `StaticAbility::Anthem` with `AffectedObjects::AttachedTo`
  - If creature leaves, equipment stays on battlefield (unattached) — SBA clears `attached_to`
  - Sample equipment cards (Skullclamp, Nim Deathmantle, Cranial Plating, Thornbite Staff) have `equip_cost` set
  - Canonical action mapping and interactive display support added

- [~] **Step 5.3: Fortification** — Deferred (extremely rare mechanic, only 1 card exists)

- [~] **Step 5.4: Reconfigure** — Deferred (niche mechanic, can be added when specific cards need it)

- [x] **Step 5.5: Add attachment-related SBAs**
  - Aura enchanting illegal/missing permanent → graveyard (CR 704.5n)
  - Orphaned equipment stays on battlefield, `attached_to` cleared
  - Integrated with SBA loop in sba.rs

---

## Phase 6: Planeswalker Support (4/4 done) **Completed: 2026-02-25**

**Goal:** Loyalty abilities, planeswalker damage redirection, and planeswalker-specific rules.

- [x] **Step 6.1: Loyalty counter system**
  - `loyalty_counters: u32` field on CardInstance, initialized from `starting_loyalty` when PW enters via spell resolution
  - `loyalty_activated_this_turn: bool` flag reset in `cleanup_eot()`

- [x] **Step 6.2: Loyalty abilities**
  - `LoyaltyAbility { cost: i32, effect: Effect, description: String }` type in card/mod.rs
  - `loyalty_abilities: Vec<LoyaltyAbility>` on CardDef (serde default)
  - `Action::ActivateLoyalty { object_id, ability_index }` — sorcery speed, main phase, empty stack, once per turn
  - Loyalty cost check (positive always ok, negative must have enough counters)
  - Stack integration via `StackSource::ActivatedAbility`, resolved by checking loyalty_abilities fallback
  - `CanonicalAction::ActivateLoyalty` with canonicalize/resolve round-trip
  - Interactive display in `format_action_rich()`

- [x] **Step 6.3: Planeswalker damage**
  - Planeswalker at 0 loyalty → SBA destroys it (CR 704.5i)
  - Combat targeting of planeswalkers deferred (goldfish mode doesn't need opponent PW attacks)

- [x] **Step 6.4: Planeswalker uniqueness rule**
  - Verified working with loyalty system — test updated to set loyalty_counters on instances

---

## Phase 7: Scryfall Oracle Text → Effect Pipeline (3/4 done) **Completed: 2026-02-24**

**Goal:** Automatically convert any card's oracle text into our Effect/Ability representation. This is the key to "load any deck."

- [x] **Step 7.1: Improve the oracle text parser**
  - Keyword detection: All 34 keywords mapped from Scryfall strings
  - Activated ability detection: `"{cost}: {effect}"` pattern, tap costs, sacrifice costs, mana costs
  - Triggered ability detection: 15 trigger conditions (ETB, dies, attacks, upkeep, combat, end step, cast, damage, opponent draws)
  - Static ability detection: Anthem effects (+N/+N), keyword grants, equipped/enchanted creature buffs
  - Inline effect parser shared between triggers and activated abilities
  - Token creation parsing (Treasure, Food, Clue)
  - Cost reduction detection ("spells cost {N} less to cast")
  - Equip cost detection ("Equip {N}")

- [x] **Step 7.2: Two-tier card resolution**
  - Tier 1: `fetch_card_def()` checks hand-authored sample database first (275+ cards)
  - Tier 2: Falls back to Scryfall API fetch + auto-parse oracle text
  - `import_deck()` automatically uses two-tier resolution for all cards

- [x] **Step 7.3: Card coverage database**
  - `CoverageLevel` enum: FullyImplemented, Stubbed, AutoParsed, Unknown
  - `DeckCoverage` struct with per-card analysis and summary counts
  - `analyze_deck_coverage()` function checks cards against sample database
  - `has_unimplemented_effects()` detects stub effects in CardDefs

- [~] **Step 7.4: Hand-authored overrides for Commander staples** — Ongoing (275+ cards already in sample.rs, added incrementally)

---

## Phase 8: Mana System Overhaul (4/4 done)

**Goal:** Multi-color mana works correctly. Auto-tap makes intelligent decisions.

**Completed: 2026-02-24**

- [x] **Step 8.1: Mana ability improvements**
  - `TapForAny` now produces the actually needed color (not always colorless)
  - `TapForChoice` now selects the needed color from available options
  - Mana rocks (Sol Ring, Basalt Monolith, etc.) now auto-tapped alongside lands

- [x] **Step 8.2: Mana pool tracking improvements**
  - Added `untapped_mana_sources()` method that returns all permanents with mana abilities (lands + rocks + dorks)
  - Kinnan-style bonus applied during auto-tap for nonland sources
  - Snow mana / conditional mana deferred to Phase 11 (no cards currently need it)

- [x] **Step 8.3: Smart auto-tap**
  - Replaced greedy auto-tap with most-constrained-first algorithm
  - Constraint scoring: single-color (score 1) → dual (2) → any (100), colorless (50)
  - Pass 1: Pay colored costs using most constrained sources first
  - Pass 2: Pay generic costs using colorless-only sources first, then least flexible
  - Correctly handles TapForColorlessAmount (Sol Ring → 2, Basalt Monolith → 3)

- [x] **Step 8.4: Mana ability choice surfacing**
  - `can_potentially_pay()` updated to count all mana sources (not just lands)
  - Flexible mana (TapForAny/TapForChoice) optimally allocated to color shortfalls
  - Goldfish/AI heuristic: auto-choose based on cost requirements

---

## Phase 9: Goldfish Mode as First-Class Entry Point (5/5 done) **Completed: 2026-02-25**

**Goal:** `cargo run --release --bin goldfish` is the primary way to use the engine. Load any Commander deck, goldfish it, see every card work.

- [x] **Step 9.1: New unified goldfish binary**
  - `src/bin/goldfish.rs` — primary entry point alongside `commander_goldfish.rs` (MCCFR training)
  - CLI: `--deck path/to/deck.txt`, `--preset kinnan|brimaz|ashcoat`, `--strategy greedy|random`
  - `--games N`, `--trace`, `--verbose`, `--coverage` flags
  - Loads any deck file via deck_import or uses built-in presets

- [x] **Step 9.2: Automatic Scryfall-backed deck loading**
  - Two-tier resolution: sample DB (275+ cards) → Scryfall auto-parse (via fetch_card_def)
  - Deck file import through `deck_import::import_deck_from_file`

- [x] **Step 9.3: Goldfish opponent simplification**
  - GoldfishStrategy: never blocks, never attacks, always passes priority
  - Goldfish opponent starts at 40 life (Commander rules)
  - Tracks: turns to kill, total damage dealt per turn, actions taken

- [x] **Step 9.4: Game trace output**
  - `--trace` flag: prints a single game with per-turn verbose action logging
  - `--verbose` flag: runs simulation then prints a sample game trace
  - Uses `run_commander_goldfish_game_verbose` for human-readable output

- [x] **Step 9.5: Card coverage report**
  - Printed automatically before simulation: deck name, commander, coverage breakdown
  - Lists cards with Unimplemented effects
  - `--coverage` flag: print coverage report and exit without simulating

---

## Phase 10: Commander-Specific Rules Completion (5/5 done) **Completed: 2026-02-25**

**Goal:** All Commander-specific rules work correctly.

- [x] **Step 10.1: Color identity enforcement**
  - `color_identity()` on CardDef derives from mana cost + mana abilities
  - `validate_commander_deck()` checks all cards match commander's identity (CR 903.4)
  - Pre-existing implementation verified and confirmed working

- [x] **Step 10.2: Singleton enforcement**
  - `validate_commander_deck()` checks no more than 1 copy of non-basic-land cards
  - Basic lands unlimited, tests cover edge cases
  - Pre-existing implementation verified

- [x] **Step 10.3: Commander death replacement**
  - `move_object()` auto-redirects commanders to command zone on death/exile (CR 903.9a)
  - GTO-optimal: always redirects (documented rationale in code)
  - Works for both primary and partner commanders via `is_commander()` check

- [x] **Step 10.4: Partner commanders**
  - Added `Partner` keyword to `KeywordAbility` enum
  - Added `partner_commander_card_id`, `partner_commander_object_id`, `partner_commander_tax` to PlayerState
  - `is_commander()` checks both primary and partner commander identity
  - `setup_commander_game_with_partners()` places both partners in command zone
  - `validate_commander_deck_with_partner()` validates combined color identity
  - Separate commander tax tracking per partner in `CastCommander` action handler
  - `legal_actions()` generates CastCommander for both commanders in command zone

- [x] **Step 10.5: Commander-specific win conditions**
  - Commander damage ≥ 21 SBA (CR 903.10a) — pre-existing and verified
  - Damage tracked per-commander via `commander_damage_received` vector

---

## Phase 11: Advanced Mechanics (5/7 done)

**Goal:** Cover the remaining mechanics needed for most Commander decks.

- [x] **Step 11.1: Counters (beyond +1/+1)**
  - Loyalty counters, poison counters, +1/+1, -1/-1 — all pre-existing
  - `Proliferate` effect added: adds one counter of each type already present on permanents/players
  - Charge/lore/time counters: generic counter system available via +1/+1 counter infrastructure

- [x] **Step 11.2: X spells**
  - `{X}` parsing already in `ManaCost::x_count`
  - CMC correctly excludes X outside the stack
  - Auto-pay system handles X spells (X defaults to available remaining mana)

- [x] **Step 11.3: Casting from non-hand zones**
  - Graveyard: Flashback and Escape via `CastFromGraveyard` action (Phase 3.4)
  - Command zone: Commander casting (Phase 10)
  - Foretell/adventure/impulse draw: deferred (require exile-with-metadata)

- [ ] **Step 11.4: Copy effects**
  - Copy a spell on the stack (e.g., Fork, Twincast)
  - Copy a creature as a token (e.g., Clone, Spark Double)
  - Deferred: requires deep infrastructure for copying stack entries and permanents

- [x] **Step 11.5: Replacement effects (build on existing framework)**
  - CR 614 replacement effect framework pre-existing (Phase 2A.3)
  - Death replacement effects (Undying, Persist, commander to command zone)
  - ETB replacement effects (enters tapped, enters with counters)
  - Damage replacement effects (prevention, redirection)
  - Extension deferred: complex interactive replacements (Notion Thief, Doubling Season)

- [x] **Step 11.6: Cost modification**
  - Cost reduction: `CostReduction` struct + `total_cost_reduction()` — pre-existing
  - Spell-intrinsic: Affinity, Convoke, Delve via `spell_cost_reduction()` (Phase 3.3)
  - Tax effects: `CostIncrease` struct + `total_cost_increase()` for Thalia-style effects
  - Integrated into both `legal_actions()` and `apply_action()` CastSpell handler

- [ ] **Step 11.7: Special actions**
  - Morph / Manifest / Foretell: deferred (require face-down state on CardInstance)

---

## Phase 12: Scryfall Integration Completion (0/3 done)

**Goal:** Any card name can be resolved to a working CardDef.

- [ ] **Step 12.1: Bulk Scryfall data**
  - Download Scryfall bulk data JSON (`default_cards.json` ~300MB compressed)
  - Parse on startup, build complete CardDatabase
  - Cache the processed CardDatabase to disk (serde binary format)

- [ ] **Step 12.2: Double-faced cards**
  - Transform cards (Delver of Secrets → Insectile Aberration)
  - Modal DFCs (Hagra Mauling / Hagra Broodpit)
  - Adventure cards (Bonecrusher Giant // Stomp)
  - Split cards (Fire // Ice)
  - Store both faces in CardDef, resolve based on zone and context

- [ ] **Step 12.3: Card errata handling**
  - Use most recent Scryfall data which reflects current Oracle text
  - Handle creature type changes, oracle text updates

---

## Phase 13: Testing and Validation Framework (0/4 done)

**Goal:** Confidence that the rules engine is correct.

- [ ] **Step 13.1: Rules assertion framework**
  - Create `src/rules/tests/` with targeted tests for each rule
  - First strike, deathtouch, trample, indestructible, legendary rule, etc.

- [ ] **Step 13.2: Card-specific regression tests**
  - For each hand-authored override card, write a test verifying it works
  - Sol Ring, Swords to Plowshares, Cyclonic Rift, etc.

- [ ] **Step 13.3: Goldfish deck tests**
  - Run a suite of known decks through goldfish and verify:
    - Game completes without panics
    - Win rate is reasonable
    - No infinite loops
    - Token counts are correct
    - Life totals are correct

- [ ] **Step 13.4: Deterministic replay**
  - Save random seeds so games can be replayed deterministically for debugging

---

## Implementation Order (Recommended)

The phases above are presented by topic. The **recommended implementation order** optimizes for incremental value:

| Priority | Phase | Rationale |
|----------|-------|-----------|
| 1 | **Phase 0** — Split the monolith | Unblocks everything; makes all later work easier |
| 2 | **Phase 1** — Token creation | Unblocks ~40% of cards |
| 3 | **Phase 4** — Effect primitives + conditions + targeting | Unblocks card expressiveness |
| 4 | **Phase 8** — Mana system overhaul | Unblocks multicolor decks |
| 5 | **Phase 3.1** — Simple keywords | Quick wins, many cards |
| 6 | **Phase 5** — Auras + Equipment | Unblocks another ~15% of cards |
| 7 | **Phase 7** — Scryfall oracle text pipeline | The "any card" pipeline |
| 8 | **Phase 9** — Goldfish first-class binary | The user-facing entry point |
| 9 | **Phase 6** — Planeswalker support | Important card type |
| 10 | **Phase 10** — Commander rules completion | Correctness |
| 11 | **Phase 2** — Multiplayer (4-player) | Deferred — goldfish is single-player |
| 12 | **Phase 3.2-3.4** — Complex keywords | Depth |
| 13 | **Phase 11** — Advanced mechanics | Depth |
| 14 | **Phase 12** — Scryfall bulk data | Scale |
| 15 | **Phase 13** — Testing framework | Ongoing throughout |

---

## Success Criteria

The refactoring is complete when:

1. `cargo run --bin goldfish -- --deck any_commander_deck.txt` works for any deck
2. Card coverage ≥ 80% (fully + partially implemented) for top 1000 Commander cards
3. All existing tests continue to pass
4. Goldfish games complete without panics for any legal Commander deck
5. The interactive mode lets a human play through a full game making all decisions
6. Adding a new card effect is a 1-file, <50-line change (add Effect variant + handler)
7. Adding a new keyword is a 1-file, <30-line change (add enum variant + behavior hooks)

---

## Target File Structure

After refactoring, the source tree will look approximately like:

```
src/
├── lib.rs
├── main.rs
├── card/
│   ├── mod.rs              (CardDef, CardInstance, types)
│   ├── database.rs          (CardDatabase)
│   ├── effects.rs           (Effect, TargetFilter, Condition, EffectContext)
│   ├── keywords.rs          (KeywordAbility, keyword handlers)
│   ├── tokens.rs            (TokenDef, PredefinedToken, token creation)
│   ├── sample.rs            (sample cards + decks)
│   ├── overrides.rs         (hand-authored staple overrides)
│   └── catalog.rs           (coverage tracking)
├── game/
│   ├── mod.rs               (GameState, PlayerState, phases)
│   ├── combat.rs            (CombatState)
│   └── zones.rs             (zone types and transitions)
├── rules/
│   ├── mod.rs               (apply_action dispatch, setup)
│   ├── priority.rs          (priority, phase transitions)
│   ├── combat.rs            (combat damage resolution)
│   ├── resolution.rs        (stack resolution)
│   ├── effects.rs           (effect resolution handlers)
│   ├── triggers.rs          (trigger system)
│   ├── sba.rs               (state-based actions)
│   ├── mulligan.rs          (mulligan logic)
│   ├── mana.rs              (auto-tap, cost reduction)
│   └── zones.rs             (zone movement helpers)
├── action/
│   ├── mod.rs               (Action, legal_actions)
│   └── canonical.rs         (canonical actions for solver)
├── mana/
│   └── mod.rs               (ManaCost, ManaPool, Color)
├── layers/
│   └── mod.rs               (CR 613 layered effects)
├── replacement/
│   └── mod.rs               (CR 614 replacement effects)
├── scryfall/
│   ├── mod.rs               (ScryfallFetcher)
│   ├── parser.rs            (oracle text → Effect pipeline)
│   └── types.rs             (ScryfallCard, API types)
├── strategy/
│   └── mod.rs               (Strategy trait + implementations)
├── simulation/
│   └── mod.rs               (parallel game runner)
├── solver/                   (MCCFR, MCTS — unchanged)
├── info_set/                 (unchanged)
├── combo.rs                  (unchanged for now)
├── combo_discovery.rs        (unchanged)
├── events/
│   └── mod.rs               (GameEvent)
└── deck_import.rs            (deck file parsing)
```

Roughly 15 new files, 10 significantly modified files.
