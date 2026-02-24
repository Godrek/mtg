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
| 2 | Multiplayer (4-Player) | Not started (deferred) | 0/6 |
| 3 | Comprehensive Keywords | Not started | 0/5 |
| 4 | Comprehensive Effects | **Done** | 4/4 |
| 5 | Auras & Equipment | Not started | 0/5 |
| 6 | Planeswalker Support | Not started | 0/4 |
| 7 | Scryfall Oracle Pipeline | Not started | 0/4 |
| 8 | Mana System Overhaul | Not started | 0/4 |
| 9 | Goldfish First-Class | Not started | 0/5 |
| 10 | Commander Rules Completion | Not started | 0/5 |
| 11 | Advanced Mechanics | Not started | 0/7 |
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

## Phase 2: Multiplayer Support — 4-Player Commander (0/6 done)

**Goal:** Support N players (primarily 4 for Commander). This is structural and touches every player-indexed reference.

> **Note:** Deferred — goldfish is single-player, current 2-player model works fine for it. Implement after Phase 9.

- [ ] **Step 2.1: Replace `opponent()` with `opponents()` iterator**
  - `fn opponents(&self, player: PlayerIndex) -> Vec<PlayerIndex>` — returns all other non-eliminated players
  - `fn next_player(&self, player: PlayerIndex) -> PlayerIndex` — clockwise turn order, skipping eliminated players
  - Keep `fn opponent()` as a convenience that returns `opponents()[0]` for 2-player backward compat

- [ ] **Step 2.2: Fix turn order**
  - Turn rotation: active player advances clockwise via `next_player()`
  - Priority passing: APNAP ordering for N players (active player, then clockwise)
  - `consecutive_passes` must track per-player or count to N before stack resolves

- [ ] **Step 2.3: Fix combat for multiplayer**
  - Attacker must declare which opponent/planeswalker each creature attacks
  - Defending player for blocking is determined per-attacker
  - Multiple defending players can assign blockers independently
  - Combat damage dealt to different opponents tracked separately

- [ ] **Step 2.4: Fix "each opponent" / "target opponent" semantics**
  - Effects with `TargetSpec::Opponent` → must enumerate all opponents
  - Effects like "each opponent loses N life" → iterate all opponents
  - Commander damage tracked per-opponent (already `Vec<i32>`)

- [ ] **Step 2.5: Elimination**
  - When a player loses (life ≤ 0, commander damage ≥ 21, etc.), they are eliminated
  - Their permanents leave the battlefield (triggers fire for each)
  - Their spells/abilities are removed from the stack
  - Game continues until 1 player remains (or goldfish: player 0 wins when goldfish dies)

- [ ] **Step 2.6: Add multiplayer tests**
  - 4-player game runs to completion
  - Player elimination removes their permanents
  - Turn order skips eliminated players
  - "Each opponent" hits all opponents

---

## Phase 3: Comprehensive Keyword Support (0/5 done)

**Goal:** Implement all ~180 MTG keyword abilities. Organized by complexity.

- [ ] **Step 3.1: Simple keywords (no parameters, affect combat/targeting/SBA)**
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

- [ ] **Step 3.2: Parameterized keywords**
  - `Protection(ProtectionFrom)` — full version: from color, from CMC, from creature type, from everything
    - `ProtectionFrom::Color(Color)`, `ProtectionFrom::Everything`, `ProtectionFrom::CreatureType(Subtype)`
    - Can't be blocked by, targeted by, dealt damage by, or enchanted/equipped by sources with that quality
  - `Hexproof` → `HexproofFrom(Option<Color>)` — some hexproof variants
  - `Ward(ManaCost)` — counter targeting spell unless controller pays
  - `Annihilator(N)` — defending player sacrifices N permanents

- [ ] **Step 3.3: Keyword actions (not permanent abilities)**
  - `Scry(N)` — look at top N, put any on bottom in any order
  - `Surveil(N)` — like scry but to graveyard instead of bottom
  - `Investigate` — create a Clue token
  - `Explore` — reveal top card: if land, put in hand; otherwise +1/+1 counter
  - `Amass(N)` — create or buff an Army token
  - `Adapt(N)` — if no +1/+1 counters, put N +1/+1 counters
  - `Proliferate` — add a counter to any number of permanents/players that already have one
  - `Populate` — create a copy of a creature token you control
  - `Bolster(N)` — put N +1/+1 counters on creature with least toughness you control
  - `Manifest` — put top card face-down as 2/2
  - `Create a Food/Treasure/Clue/Blood token` — predefined token templates

- [ ] **Step 3.4: Alternative cost keywords**
  - `Flashback(Cost)` — may cast from graveyard for flashback cost
  - `Overload(Cost)` — may cast for overload cost; if you do, replace "target" with "each"
  - `Escape { cost, exile_count }` — cast from graveyard by paying cost + exiling cards
  - `Retrace` — cast from graveyard by discarding a land
  - `Buyback(Cost)` — pay extra to return to hand instead of graveyard
  - `Kicker(Cost)` — may pay extra for enhanced effect
  - `Multikicker(Cost)` — may pay any number of times
  - `Entwine(Cost)` — pay extra to choose all modes
  - `Splice(Cost)` — pay to add effect to arcane spell
  - `Emerge(Cost)` — sacrifice creature, reduce cost by sacrificed creature's CMC
  - `Evoke(Cost)` — cast for evoke cost, sacrifice when ETB
  - `Morph(Cost)` — cast face-down as 2/2, turn face-up for morph cost
  - `Foretell(Cost)` — exile face-down for {2}, cast later for foretell cost
  - `Mutate(Cost)` — cast on top/under creature
  - `Bestow(Cost)` — cast as aura enchantment
  - `Disturb(Cost)` — cast transformed from graveyard
  - `Encore(Cost)` — exile from graveyard, create token copies attacking each opponent

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

## Phase 5: Auras, Equipment, and Attachments (0/5 done)

**Goal:** Implement the attachment system so Auras and Equipment work.

- [ ] **Step 5.1: Aura implementation**
  - When an Aura spell resolves, it must target a legal permanent
  - Attach the Aura to the target: set `attached_to` on the Aura, add to `attachments` on the target
  - If the enchanted permanent leaves the battlefield, the Aura goes to graveyard (SBA)
  - Aura's static abilities generate continuous effects on the attached permanent
  - Auras that lose their legal target fall off (SBA)

- [ ] **Step 5.2: Equipment implementation**
  - Equipment enters as a regular artifact
  - "Equip {N}" is an activated ability (sorcery speed) that attaches it to a creature you control
  - Equipped creature gains the listed abilities/buffs via continuous effects
  - If creature leaves, equipment stays on battlefield (unattached)
  - If equipment leaves, it detaches

- [ ] **Step 5.3: Fortification**
  - Same as Equipment but for lands (rare but covers the pattern)

- [ ] **Step 5.4: Reconfigure**
  - Equipment that can be a creature or equipment (living weapons)

- [ ] **Step 5.5: Add attachment-related SBAs**
  - Aura enchanting illegal permanent → graveyard
  - Aura enchanting nothing → graveyard
  - Equipment not attached to creature → just stays (no SBA needed)
  - Token attachment cleanup

---

## Phase 6: Planeswalker Support (0/4 done)

**Goal:** Loyalty abilities, planeswalker damage redirection, and planeswalker-specific rules.

- [ ] **Step 6.1: Loyalty counter system**
  - Planeswalkers enter with `starting_loyalty` counters (new field on CardInstance: `loyalty_counters: u32`)
  - Add loyalty as a counter type alongside +1/+1 and -1/-1

- [ ] **Step 6.2: Loyalty abilities**
  - New ability type: `LoyaltyAbility { cost: i32, effect, description }`
  - Players may activate one loyalty ability per planeswalker per turn, at sorcery speed
  - Adding loyalty: add counters, put ability on stack
  - Removing loyalty: remove counters (must have enough), put ability on stack

- [ ] **Step 6.3: Planeswalker damage**
  - Attackers can target planeswalkers (expand combat to allow attacking planeswalkers)
  - Damage directly targets planeswalker (current rules, post-Dominaria)
  - Planeswalker at 0 loyalty → SBA destroys it

- [ ] **Step 6.4: Planeswalker uniqueness rule**
  - Already implemented in SBA — verify it works with new loyalty system

---

## Phase 7: Scryfall Oracle Text → Effect Pipeline (0/4 done)

**Goal:** Automatically convert any card's oracle text into our Effect/Ability representation. This is the key to "load any deck."

- [ ] **Step 7.1: Improve the oracle text parser**
  - **Keyword detection:** All keywords from Steps 3.1-3.4, parameterized keywords, keyword with magnitude
  - **Activated ability detection:** `"{cost}: {effect}"` pattern, multiple costs, restrictions
  - **Triggered ability detection:** "When"/"Whenever"/"At" patterns, all trigger conditions, effect portion
  - **Static ability detection:** "Creatures you control get...", "Other creatures ... have ...", "Spells ... cost ... less"

- [ ] **Step 7.2: Two-tier card resolution**
  - Tier 1: Hand-authored `CardDef` for complex/staple cards (override Scryfall)
  - Tier 2: Auto-parsed from Scryfall oracle text → best-effort `CardDef`
  - Priority chain: exact pattern → regex extraction → keyword scanning → fallback `Unimplemented`

- [ ] **Step 7.3: Card coverage database**
  - `CardCoverage` struct tracking fully/partially/stub implemented cards
  - CLI command: `cargo run --bin card_coverage -- "decklist.txt"` reporting coverage

- [ ] **Step 7.4: Hand-authored overrides for Commander staples**
  - Create `src/card/overrides.rs` for top ~200 most-played Commander cards
  - Sol Ring, Mana Crypt, Rhystic Study, Cyclonic Rift, Smothering Tithe, Dockside Extortionist, Mystic Remora, Demonic Tutor, Swords to Plowshares, Path to Exile, Beast Within, all fetch/shock/dual lands, mana dorks, etc.

---

## Phase 8: Mana System Overhaul (0/4 done)

**Goal:** Multi-color mana works correctly. Auto-tap makes intelligent decisions.

- [ ] **Step 8.1: Mana ability improvements**
  - `ManaAbility::TapForAny` — player chooses color (currently defaults to colorless)
  - `ManaAbility::TapForColorOrColorless(Color)` — e.g., pain lands
  - `ManaAbility::ConditionalMana { color, condition }` — e.g., "only for creature spells"
  - Treasure tokens: `ManaAbility::Sacrifice` — sacrifice artifact to add any color

- [ ] **Step 8.2: Mana pool tracking improvements**
  - Track mana restrictions: "spend only to cast creature spells" (e.g., Cavern of Souls)
  - Track mana source for Kinnan-style bonuses
  - Snow mana ({S}) tracking

- [ ] **Step 8.3: Smart auto-tap**
  - Replace greedy auto-tap with search-based approach
  - Most-constrained-first: tap lands with fewer color options first
  - Prefer tapping lands that only produce the needed color (don't waste dual lands)
  - Tap colorless-only sources for generic costs
  - Fallback: iterate permutations up to a cap (e.g., 12 lands) for correctness

- [ ] **Step 8.4: Mana ability choice surfacing**
  - When a land can produce multiple colors, player chooses which color
  - Goldfish mode: heuristic (produce what's needed)
  - Interactive mode: prompt player

---

## Phase 9: Goldfish Mode as First-Class Entry Point (0/5 done)

**Goal:** `cargo run --release --bin goldfish` is the primary way to use the engine. Load any Commander deck, goldfish it, see every card work.

- [ ] **Step 9.1: New unified goldfish binary**
  - Create `src/bin/goldfish.rs` replacing `commander_goldfish.rs` as primary entry point
  - CLI: `--deck path/to/deck.txt`, `--interactive`, `--strategy greedy|mccfr`, `--games N`, `--iterations N`
  - Loads any deck file (downloads cards from Scryfall automatically)
  - Reports card coverage, runs simulation, interactive mode, verbose mode

- [ ] **Step 9.2: Automatic Scryfall-backed deck loading**
  - Check hand-authored overrides → Scryfall cache → Scryfall API → auto-parse → log warnings → proceed

- [ ] **Step 9.3: Goldfish opponent simplification**
  - Goldfish opponent has very high life (e.g., 10,000) to avoid incidental self-damage ending game
  - Goldfish never blocks, never plays spells
  - Track: turns to kill, total damage dealt per turn, mana spent per turn, cards drawn

- [ ] **Step 9.4: Game trace output**
  - Human-readable per-turn trace: "Turn 1: Play Forest. Cast Llanowar Elves."
  - Helps players verify deck works and debug card interactions

- [ ] **Step 9.5: Card coverage report**
  - Before starting: print deck name, commander, coverage breakdown (fully/partially/stub)
  - List unimplemented cards with reason

---

## Phase 10: Commander-Specific Rules Completion (0/5 done)

**Goal:** All Commander-specific rules work correctly.

- [ ] **Step 10.1: Color identity enforcement**
  - Validate all cards match commander's color identity when loading deck
  - Reject or warn on cards outside color identity
  - Basic lands: only lands matching color identity (or colorless lands)

- [ ] **Step 10.2: Singleton enforcement**
  - Validate no more than 1 copy of any non-basic-land card
  - Exception: cards that say "you may have any number" (e.g., Relentless Rats)

- [ ] **Step 10.3: Commander death replacement**
  - When commander would go to graveyard or exile, owner may redirect to command zone
  - This is a replacement effect (CR 903.9a)
  - Each redirection increases commander tax by {2}
  - Surface `Action::ChooseCommanderZone { object_id }` when commander dies/exiled

- [ ] **Step 10.4: Partner commanders**
  - Two commanders with Partner keyword
  - Combined color identity
  - Either can be cast from command zone (separate tax for each)
  - Each tracks commander damage separately

- [ ] **Step 10.5: Commander-specific win conditions**
  - Commander damage ≥ 21 from a single commander → that player loses (already implemented)
  - Verify interaction with damage prevention, life gain, etc.
  - Last player standing wins

---

## Phase 11: Advanced Mechanics (0/7 done)

**Goal:** Cover the remaining mechanics needed for most Commander decks.

- [ ] **Step 11.1: Counters (beyond +1/+1)**
  - Loyalty counters (planeswalkers)
  - Poison counters (players) — 10 poison = lose
  - -1/-1 counters (interact with +1/+1: they annihilate)
  - Charge counters, lore counters, time counters, etc.
  - `Proliferate` — add one counter of a type already present

- [ ] **Step 11.2: X spells**
  - Parse `{X}` in mana cost
  - Player chooses X when casting
  - X value stored in EffectContext
  - On stack, CMC includes chosen X
  - In all other zones, X = 0

- [ ] **Step 11.3: Casting from non-hand zones**
  - Graveyard: flashback, escape, retrace, disturb
  - Exile: foretell, adventure, impulse draw ("exile top, may play until end of turn")
  - Top of library: future sight effects
  - Command zone: already done (commander)

- [ ] **Step 11.4: Copy effects**
  - Copy a spell on the stack (e.g., Fork, Twincast)
  - Copy a creature as a token (e.g., Clone, Spark Double)
  - Copy an artifact (e.g., Sculpting Steel)
  - Mutate-style overlays

- [ ] **Step 11.5: Replacement effects (build on existing framework)**
  - "If would draw, instead..." (Notion Thief, Spirit of the Labyrinth)
  - "If would die, instead exile" (Rest in Peace)
  - "If would enter, enters with N counters" (Doubling Season)
  - "If tokens would be created, create twice that many" (Anointed Procession, Doubling Season)
  - "If damage would be dealt, prevent it" (damage prevention shields)
  - "If would be put into graveyard from anywhere, exile instead" (Leyline of the Void)

- [ ] **Step 11.6: Cost modification**
  - Additional costs: "As an additional cost, sacrifice a creature"
  - Cost reduction extensions: Affinity, Convoke, Delve, Emerge
  - Alternative costs: force spike effects, "you may pay {0} instead"
  - Tax effects: "Spells cost {1} more" (Thalia), "Noncreature spells cost {1} more"

- [ ] **Step 11.7: Special actions**
  - Morph / Megamorph / Disguise — cast face-down, turn up
  - Manifest — top card face-down as 2/2
  - Foretell — exile face-down, cast later
  - These require a "face-down" state on CardInstance

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
