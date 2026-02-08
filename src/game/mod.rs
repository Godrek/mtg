use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::card::{CardDef, CardId, CardInstance, ObjectId, ZoneType};
use crate::events::GameEvent;
use crate::layers::{ComputedCharacteristics, ContinuousEffect};
use crate::mana::ManaPool;
use crate::replacement::{ReplacementEffect, ReplacementEventKind, ReplacementAction};

// ---------------------------------------------------------------------------
// Characteristics cache (Fix 1 + Fix 3)
// ---------------------------------------------------------------------------

/// Interior of the transient characteristics cache.
#[derive(Debug, Default)]
struct CharacteristicsCacheInner {
    entries: HashMap<ObjectId, ComputedCharacteristics>,
    /// Lazily-built set for O(1) battlefield membership tests.
    battlefield_set: Option<HashSet<ObjectId>>,
}

/// Transient cache for [`compute_characteristics`](crate::layers::compute_characteristics) results.
///
/// Uses `Mutex` for interior mutability so the read-only query methods
/// (`effective_power`, `has_keyword`, etc.) can populate the cache through
/// shared `&self` references. `Mutex` (rather than `RefCell`) is required
/// because `GameState` must be `Sync` for rayon parallel iteration.
/// Each clone gets its own empty cache, so contention never occurs.
///
/// Per the Snapshot Contract (Phase 0.3), this cache is:
/// - **NOT serialized** (`#[serde(skip)]`) — it's derived state
/// - **NOT cloned** — `Clone` produces an empty cache (cheap `GameState::clone()`)
/// - **Invalidated** whenever canonical state that affects characteristics changes
pub struct CharacteristicsCache(Mutex<CharacteristicsCacheInner>);

impl Clone for CharacteristicsCache {
    fn clone(&self) -> Self {
        // Per Snapshot Contract: derived/cached fields reset on clone.
        CharacteristicsCache(Mutex::new(CharacteristicsCacheInner::default()))
    }
}

impl Default for CharacteristicsCache {
    fn default() -> Self {
        CharacteristicsCache(Mutex::new(CharacteristicsCacheInner::default()))
    }
}

impl std::fmt::Debug for CharacteristicsCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CharacteristicsCache")
            .field("entries", &self.0.lock().map(|c| c.entries.len()).unwrap_or(0))
            .finish()
    }
}

/// Index into the players array (0 or 1 for a two-player game).
pub type PlayerIndex = usize;

/// Represents the phase/step within a turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Phase {
    // Beginning phase
    Untap,
    Upkeep,
    Draw,

    // Pre-combat main phase
    PreCombatMain,

    // Combat phase
    BeginningOfCombat,
    DeclareAttackers,
    DeclareBlockers,
    FirstStrikeDamage,
    CombatDamage,
    EndOfCombat,

    // Post-combat main phase
    PostCombatMain,

    // Ending phase
    EndStep,
    Cleanup,
}

impl Phase {
    /// Ordered list of all phases in a turn.
    pub const TURN_ORDER: [Phase; 13] = [
        Phase::Untap,
        Phase::Upkeep,
        Phase::Draw,
        Phase::PreCombatMain,
        Phase::BeginningOfCombat,
        Phase::DeclareAttackers,
        Phase::DeclareBlockers,
        Phase::FirstStrikeDamage,
        Phase::CombatDamage,
        Phase::EndOfCombat,
        Phase::PostCombatMain,
        Phase::EndStep,
        Phase::Cleanup,
    ];

    /// Can a player normally cast sorcery-speed spells here?
    pub fn is_main_phase(&self) -> bool {
        matches!(self, Phase::PreCombatMain | Phase::PostCombatMain)
    }

    /// Is this a combat step where damage is dealt?
    pub fn is_damage_step(&self) -> bool {
        matches!(self, Phase::FirstStrikeDamage | Phase::CombatDamage)
    }
}

/// A stack entry — a spell or ability waiting to resolve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackEntry {
    pub id: u64,
    pub source: StackSource,
    pub controller: PlayerIndex,
    pub targets: Vec<Target>,
}

/// What put this entry on the stack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StackSource {
    /// A card being cast (the object ID of the card).
    Spell(ObjectId),
    /// An activated ability from a permanent.
    ActivatedAbility {
        source_id: ObjectId,
        ability_index: usize,
    },
    /// A triggered ability.
    TriggeredAbility {
        source_id: ObjectId,
        ability_index: usize,
    },
}

/// A resolved target.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Target {
    Player(PlayerIndex),
    Object(ObjectId),
}

/// Per-player state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub life: i32,
    pub mana_pool: ManaPool,
    pub land_plays_remaining: u32,
    pub has_drawn_for_turn: bool,
    pub has_lost: bool,
    pub has_won: bool,

    // Zones (each is a list of ObjectIds).
    pub library: Vec<ObjectId>,
    pub hand: Vec<ObjectId>,
    pub graveyard: Vec<ObjectId>,
    pub exile: Vec<ObjectId>,
}

impl PlayerState {
    pub fn new() -> Self {
        PlayerState {
            life: 20,
            mana_pool: ManaPool::empty(),
            land_plays_remaining: 1,
            has_drawn_for_turn: false,
            has_lost: false,
            has_won: false,
            library: Vec::new(),
            hand: Vec::new(),
            graveyard: Vec::new(),
            exile: Vec::new(),
        }
    }
}

/// Combat state tracked during the combat phase.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CombatState {
    /// ObjectIds of creatures declared as attackers.
    pub attackers: Vec<ObjectId>,
    /// Map from blocker ObjectId -> the attacker ObjectId it's blocking.
    pub blockers: HashMap<ObjectId, ObjectId>,
    /// Map from attacker ObjectId -> list of blocker ObjectIds (derived from above).
    pub attacker_blockers: HashMap<ObjectId, Vec<ObjectId>>,
    /// Damage assignment for blocked creatures (attacker -> [(blocker, damage)]).
    pub damage_assignment: HashMap<ObjectId, Vec<(ObjectId, u32)>>,
}

impl CombatState {
    pub fn clear(&mut self) {
        self.attackers.clear();
        self.blockers.clear();
        self.attacker_blockers.clear();
        self.damage_assignment.clear();
    }
}

/// The complete game state — everything needed to determine legal actions and advance the game.
/// This must be cheaply cloneable for MCTS/CFR tree search.
///
/// # Snapshot Contract (Phase 0.3)
///
/// Defines what gets cloned vs. shared vs. reconstructed when `GameState::clone()` is called.
/// This contract ensures MCCFR traversal (which clones millions of states) stays fast while
/// the rules engine can freely add derived/cached fields without breaking the solver.
///
/// ## Always cloned (canonical state)
/// These fields define the unique game position. Two `GameState`s with identical values
/// for all canonical fields represent the same game state:
/// - `objects` — all card instances and their game-specific state
/// - `players` — life, zones (library, hand, graveyard, exile), mana, flags
/// - `battlefield`, `stack` — shared zones
/// - `combat` — attacker/blocker/damage assignment
/// - `pending_triggers` — triggers waiting to be placed on the stack
/// - All scalar fields: `active_player`, `phase`, `priority_player`, `turn_number`,
///   `consecutive_passes`, `next_object_id`, `next_stack_id`, `game_over`, `winner`
///
/// ## Shared via Arc (immutable reference data)
/// - `card_db` — card definitions are immutable after game setup; shared O(1) via `Arc`
///
/// ## Reconstructed after clone (derived / cached state)
/// Future fields that are derivable from canonical state must NOT be part of `Clone`:
/// - Event bus state (Phase 1A) — transient; not part of game state
/// - Continuous effects caches (Phase 2A) — recomputed from canonical state on demand
/// - Dirty flags / memoization caches — local optimization, not state
///
/// **Rule**: Any field added to `GameState` that is derivable from other fields must be
/// marked `#[serde(skip)]` and excluded from equality/hashing. The canonical game state
/// is the minimal set of fields needed to reconstruct the full state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Card definitions (shared, immutable). Wrapped in `Arc` so that
    /// `GameState::clone()` is O(1) for the DB — critical for MCTS/CFR search.
    #[serde(skip)]
    pub card_db: Option<Arc<CardDatabase>>,

    /// All card instances in the game, keyed by ObjectId.
    pub objects: HashMap<ObjectId, CardInstance>,

    /// The battlefield zone (shared — both players' permanents).
    pub battlefield: Vec<ObjectId>,

    /// The stack.
    pub stack: Vec<StackEntry>,

    /// Player states.
    pub players: Vec<PlayerState>,

    /// Whose turn is it?
    pub active_player: PlayerIndex,

    /// Current phase/step.
    pub phase: Phase,

    /// Which player has priority.
    pub priority_player: PlayerIndex,

    /// Turn number (starts at 1).
    pub turn_number: u32,

    /// Whether both players have passed priority in succession (stack resolves or phase advances).
    pub consecutive_passes: u32,

    /// Combat state (only meaningful during combat).
    pub combat: CombatState,

    /// Next object ID to assign.
    pub next_object_id: ObjectId,

    /// Next stack ID.
    pub next_stack_id: u64,

    /// Pending triggers waiting to be put on the stack.
    /// These accumulate during rule processing and are placed on the stack
    /// in APNAP order (active player's triggers first) before priority is given.
    pub pending_triggers: Vec<PendingTrigger>,

    /// Active continuous effects on the battlefield (Phase 2A.1).
    ///
    /// Continuous effects from static abilities are regenerated when the
    /// battlefield changes. Effects from resolved spells (until end of turn,
    /// permanent) are tracked here explicitly. The layer engine uses this
    /// list to compute characteristics on demand.
    pub continuous_effects: Vec<ContinuousEffect>,

    /// Next timestamp for continuous effect ordering (CR 613.7).
    pub next_timestamp: u32,

    /// Active replacement effects (Phase 2A.3).
    /// Replacement effects modify or replace events as they happen (CR 614).
    /// Self-replacement effects are applied automatically; competing player-choice
    /// replacements are surfaced as Action::ChooseReplacementOrder.
    pub replacement_effects: Vec<ReplacementEffect>,

    /// Game over flag.
    pub game_over: bool,

    /// Winner (if game is over). None = draw.
    pub winner: Option<PlayerIndex>,

    /// Transient event accumulator (Phase 1A.2).
    ///
    /// Events emitted during `apply_action()` and rule processing are
    /// collected here. External code (event bus, test harness) can drain
    /// this vec after each action to process events.
    ///
    /// This field is:
    /// - **NOT serialized** (`serde(skip)`) — events are transient
    /// - **NOT part of the canonical game state** — two states with
    ///   different pending_events but identical canonical fields represent
    ///   the same game position
    /// - **Cheap to clone** — should be empty between actions; any events
    ///   present at clone time are copied but this is O(0) in practice
    #[serde(skip)]
    pub pending_events: Vec<GameEvent>,

    /// Transient cache for `compute_characteristics` results.
    /// Avoids redundant recomputation (~90× per combat step) by caching
    /// the layer-engine output and a `HashSet` for O(1) battlefield membership.
    /// Reset on clone, not serialized, invalidated on canonical-state mutation.
    #[serde(skip)]
    pub characteristics_cache: CharacteristicsCache,
}

/// A trigger that has been queued but not yet placed on the stack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTrigger {
    pub source_id: ObjectId,
    pub ability_index: usize,
    pub controller: PlayerIndex,
    pub targets: Vec<Target>,
}

/// A simple card database that maps CardId -> CardDef.
#[derive(Debug, Clone, Default)]
pub struct CardDatabase {
    pub cards: HashMap<CardId, CardDef>,
}

impl CardDatabase {
    pub fn new() -> Self {
        CardDatabase {
            cards: HashMap::new(),
        }
    }

    pub fn insert(&mut self, card: CardDef) {
        self.cards.insert(card.id, card);
    }

    pub fn get(&self, id: CardId) -> Option<&CardDef> {
        self.cards.get(&id)
    }

    pub fn find_by_name(&self, name: &str) -> Option<CardId> {
        let target = name.trim();
        self.cards
            .values()
            .find(|card| card.name.eq_ignore_ascii_case(target))
            .map(|card| card.id)
    }
}

// ---------------------------------------------------------------------------
// Phase 0.1 — Observation API: PlayerView
// ---------------------------------------------------------------------------

/// Everything a player can observe — the information set boundary.
/// The rules engine writes to `GameState`; MCCFR reads through this view.
///
/// `PlayerView` exposes only information that the given player is entitled to
/// see under the MTG rules: public zones (battlefield, graveyard, exile, stack),
/// opponent's hand *size* and library *size* (but not contents), and the
/// player's own private hand.
///
/// When the rules engine adds internal fields (event bus, effects manager,
/// continuous effects cache), MCCFR is insulated — only `visible_state()`
/// needs updating.
pub struct PlayerView<'a> {
    // --- Public information (both players can see) ---
    /// Current game phase/step.
    pub phase: Phase,
    /// Whose turn it is.
    pub active_player: PlayerIndex,
    /// Current turn number (starts at 1).
    pub turn_number: u32,
    /// All permanents on the battlefield (both players).
    pub battlefield: &'a [ObjectId],
    /// The stack (spells and abilities waiting to resolve).
    pub stack: &'a [StackEntry],
    /// Combat state (attackers, blockers, damage assignment).
    pub combat: &'a CombatState,
    /// Triggered abilities waiting to be placed on the stack.
    pub pending_triggers: &'a [PendingTrigger],
    /// Which player currently has priority.
    pub priority_player: PlayerIndex,

    // --- Per-player public info ---
    /// Viewing player's life total.
    pub my_life: i32,
    /// Opponent's life total.
    pub opp_life: i32,
    /// Viewing player's graveyard.
    pub my_graveyard: &'a [ObjectId],
    /// Opponent's graveyard.
    pub opp_graveyard: &'a [ObjectId],
    /// Viewing player's exile zone.
    pub my_exile: &'a [ObjectId],
    /// Opponent's exile zone.
    pub opp_exile: &'a [ObjectId],
    /// Number of cards in the opponent's hand (contents hidden).
    pub opp_hand_size: usize,
    /// Number of cards in the opponent's library (contents hidden).
    pub opp_library_size: usize,

    // --- Private information (only the viewing player sees) ---
    /// The viewing player's hand (private — hidden from opponent).
    pub my_hand: &'a [ObjectId],

    // --- Mana ---
    /// Viewing player's current mana pool.
    pub my_mana_pool: &'a ManaPool,
    /// Remaining land plays this turn for the viewing player.
    pub my_land_plays_remaining: u32,

    // --- Object lookup (filtered, read-only) ---
    /// Card instances visible to the viewing player, keyed by ObjectId.
    /// Includes objects on the battlefield, stack, both graveyards, both exile
    /// zones, the viewing player's hand, and pending trigger sources.
    /// Excludes the opponent's hand contents and both libraries (hidden zones).
    pub objects: HashMap<ObjectId, &'a CardInstance>,
    /// Card definitions database (shared, immutable).
    pub card_db: &'a CardDatabase,
}

impl GameState {
    /// Build a `PlayerView` for the given player, exposing only information
    /// that player is entitled to see under the MTG rules.
    ///
    /// The `objects` map is filtered to only include card instances in visible
    /// zones: battlefield, stack, both graveyards, both exile zones, the viewing
    /// player's hand, pending trigger sources, and combat participants.
    /// Opponent hand contents and both libraries are excluded.
    pub fn visible_state(&self, player: PlayerIndex) -> PlayerView<'_> {
        let opp = self.opponent(player);

        // Collect ObjectIds from all visible zones into the filtered objects map.
        let mut visible = HashMap::new();

        // Battlefield — public
        for &id in &self.battlefield {
            if let Some(inst) = self.objects.get(&id) {
                visible.insert(id, inst);
            }
        }
        // Stack — spells/abilities are public
        for entry in &self.stack {
            let source_id = match entry.source {
                StackSource::Spell(id) => id,
                StackSource::ActivatedAbility { source_id, .. } => source_id,
                StackSource::TriggeredAbility { source_id, .. } => source_id,
            };
            if let Some(inst) = self.objects.get(&source_id) {
                visible.insert(source_id, inst);
            }
        }
        // Both graveyards — public
        for &id in &self.players[player].graveyard {
            if let Some(inst) = self.objects.get(&id) {
                visible.insert(id, inst);
            }
        }
        for &id in &self.players[opp].graveyard {
            if let Some(inst) = self.objects.get(&id) {
                visible.insert(id, inst);
            }
        }
        // Both exile zones — public
        for &id in &self.players[player].exile {
            if let Some(inst) = self.objects.get(&id) {
                visible.insert(id, inst);
            }
        }
        for &id in &self.players[opp].exile {
            if let Some(inst) = self.objects.get(&id) {
                visible.insert(id, inst);
            }
        }
        // Viewing player's hand — private to this player
        for &id in &self.players[player].hand {
            if let Some(inst) = self.objects.get(&id) {
                visible.insert(id, inst);
            }
        }
        // Pending trigger sources — visible (they reference battlefield permanents)
        for trigger in &self.pending_triggers {
            if let Some(inst) = self.objects.get(&trigger.source_id) {
                visible.insert(trigger.source_id, inst);
            }
        }
        // Combat participants — attackers and blockers (both keys and values)
        for &id in &self.combat.attackers {
            if let Some(inst) = self.objects.get(&id) {
                visible.insert(id, inst);
            }
        }
        for (&blocker, &attacker) in &self.combat.blockers {
            if let Some(inst) = self.objects.get(&blocker) {
                visible.insert(blocker, inst);
            }
            if let Some(inst) = self.objects.get(&attacker) {
                visible.insert(attacker, inst);
            }
        }

        PlayerView {
            phase: self.phase,
            active_player: self.active_player,
            turn_number: self.turn_number,
            battlefield: &self.battlefield,
            stack: &self.stack,
            combat: &self.combat,
            pending_triggers: &self.pending_triggers,
            priority_player: self.priority_player,

            my_life: self.players[player].life,
            opp_life: self.players[opp].life,
            my_graveyard: &self.players[player].graveyard,
            opp_graveyard: &self.players[opp].graveyard,
            my_exile: &self.players[player].exile,
            opp_exile: &self.players[opp].exile,
            opp_hand_size: self.players[opp].hand.len(),
            opp_library_size: self.players[opp].library.len(),

            my_hand: &self.players[player].hand,

            my_mana_pool: &self.players[player].mana_pool,
            my_land_plays_remaining: self.players[player].land_plays_remaining,

            objects: visible,
            card_db: self.card_db(),
        }
    }
}

impl GameState {
    pub fn new(num_players: usize) -> Self {
        GameState {
            card_db: None,
            objects: HashMap::new(),
            battlefield: Vec::new(),
            stack: Vec::new(),
            players: (0..num_players).map(|_| PlayerState::new()).collect(),
            active_player: 0,
            phase: Phase::Untap,
            priority_player: 0,
            turn_number: 1,
            consecutive_passes: 0,
            combat: CombatState::default(),
            next_object_id: 1,
            next_stack_id: 1,
            pending_triggers: Vec::new(),
            continuous_effects: Vec::new(),
            next_timestamp: 1,
            replacement_effects: Vec::new(),
            game_over: false,
            winner: None,
            pending_events: Vec::new(),
            characteristics_cache: CharacteristicsCache::default(),
        }
    }

    pub fn card_db(&self) -> &CardDatabase {
        self.card_db.as_ref().expect("CardDatabase not set on GameState")
    }

    /// Allocate a new unique ObjectId.
    pub fn new_object_id(&mut self) -> ObjectId {
        let id = self.next_object_id;
        self.next_object_id += 1;
        id
    }

    /// Allocate a new stack entry ID.
    pub fn new_stack_id(&mut self) -> u64 {
        let id = self.next_stack_id;
        self.next_stack_id += 1;
        id
    }

    /// Create a card instance and put it in the specified zone.
    pub fn create_card_in_zone(
        &mut self,
        card_id: CardId,
        owner: PlayerIndex,
        zone: ZoneType,
    ) -> ObjectId {
        let obj_id = self.new_object_id();
        let instance = CardInstance::new(obj_id, card_id, owner);
        self.objects.insert(obj_id, instance);

        match zone {
            ZoneType::Library => self.players[owner].library.push(obj_id),
            ZoneType::Hand => self.players[owner].hand.push(obj_id),
            ZoneType::Battlefield => self.battlefield.push(obj_id),
            ZoneType::Graveyard => self.players[owner].graveyard.push(obj_id),
            ZoneType::Exile => self.players[owner].exile.push(obj_id),
            ZoneType::Stack => {} // handled separately
            ZoneType::Command => {} // not implemented yet
        }
        obj_id
    }

    /// Move a card instance from one zone to another.
    pub fn move_object(
        &mut self,
        obj_id: ObjectId,
        from: ZoneType,
        to: ZoneType,
    ) {
        self.invalidate_characteristics_cache();

        // Emit zone change event
        self.emit_event(GameEvent::ZoneChange {
            object: obj_id,
            from: crate::events::Zone::from(from),
            to: crate::events::Zone::from(to),
        });

        // Remove from all zones (brute force but correct)
        let owner = self.objects[&obj_id].owner;
        let controller = self.objects[&obj_id].controller;

        self.players[owner].library.retain(|&id| id != obj_id);
        self.players[owner].hand.retain(|&id| id != obj_id);
        self.players[owner].graveyard.retain(|&id| id != obj_id);
        self.players[owner].exile.retain(|&id| id != obj_id);
        // Also check controller's zones if different
        if controller != owner {
            self.players[controller].library.retain(|&id| id != obj_id);
            self.players[controller].hand.retain(|&id| id != obj_id);
            self.players[controller].graveyard.retain(|&id| id != obj_id);
            self.players[controller].exile.retain(|&id| id != obj_id);
        }
        self.battlefield.retain(|&id| id != obj_id);
        self.stack.retain(|e| {
            if let StackSource::Spell(spell_id) = e.source {
                spell_id != obj_id
            } else {
                true
            }
        });

        // Add to destination zone
        match to {
            ZoneType::Library => self.players[owner].library.push(obj_id),
            ZoneType::Hand => self.players[owner].hand.push(obj_id),
            ZoneType::Battlefield => {
                // Reset battlefield state when entering
                let enters_tapped = self.card_db()
                    .get(self.objects[&obj_id].card_def_id)
                    .map_or(false, |d| d.enters_tapped);
                if let Some(inst) = self.objects.get_mut(&obj_id) {
                    inst.tapped = enters_tapped;
                    inst.summoning_sick = true;
                    inst.damage_marked = 0;
                    inst.temp_power_mod = 0;
                    inst.temp_toughness_mod = 0;
                    inst.temp_keywords.clear();
                }
                self.battlefield.push(obj_id);
            }
            ZoneType::Graveyard => self.players[owner].graveyard.push(obj_id),
            ZoneType::Exile => self.players[owner].exile.push(obj_id),
            ZoneType::Stack => {} // handled by cast_spell
            ZoneType::Command => {}
        }
    }

    /// Get all permanent ObjectIds on the battlefield controlled by a given player.
    pub fn permanents_controlled_by(&self, player: PlayerIndex) -> Vec<ObjectId> {
        self.battlefield
            .iter()
            .copied()
            .filter(|&id| self.objects[&id].controller == player)
            .collect()
    }

    /// Get all creature ObjectIds on the battlefield controlled by a given player.
    pub fn creatures_controlled_by(&self, player: PlayerIndex) -> Vec<ObjectId> {
        let db = self.card_db();
        self.battlefield
            .iter()
            .copied()
            .filter(|&id| {
                let inst = &self.objects[&id];
                inst.controller == player
                    && db.get(inst.card_def_id).map_or(false, |d| d.is_creature())
            })
            .collect()
    }

    /// Get lands on the battlefield controlled by a given player.
    pub fn lands_controlled_by(&self, player: PlayerIndex) -> Vec<ObjectId> {
        let db = self.card_db();
        self.battlefield
            .iter()
            .copied()
            .filter(|&id| {
                let inst = &self.objects[&id];
                inst.controller == player
                    && db.get(inst.card_def_id).map_or(false, |d| d.is_land())
            })
            .collect()
    }

    /// Get untapped lands controlled by a player.
    pub fn untapped_lands(&self, player: PlayerIndex) -> Vec<ObjectId> {
        self.lands_controlled_by(player)
            .into_iter()
            .filter(|&id| !self.objects[&id].tapped)
            .collect()
    }

    /// The opponent of the given player (two-player only).
    pub fn opponent(&self, player: PlayerIndex) -> PlayerIndex {
        1 - player
    }

    /// Emit a game event to the transient event accumulator.
    /// Events are collected during rule processing and can be drained
    /// by external code (event bus, tests) after each action.
    pub fn emit_event(&mut self, event: GameEvent) {
        self.pending_events.push(event);
    }

    /// Drain all pending events, returning them for processing.
    pub fn drain_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Check if a player has lost.
    pub fn check_player_lost(&self, player: PlayerIndex) -> bool {
        self.players[player].life <= 0
            || self.players[player].has_lost
    }

    /// Allocate a new timestamp for continuous effect ordering.
    pub fn new_timestamp(&mut self) -> u32 {
        let ts = self.next_timestamp;
        self.next_timestamp += 1;
        ts
    }

    /// Refresh continuous effects from static abilities on the battlefield.
    /// This regenerates effects from permanents with static abilities,
    /// preserving any non-static effects (from spells, until-end-of-turn, etc.).
    ///
    /// Uses the two-phase read-write pattern to satisfy the borrow checker:
    /// Phase 1 collects what needs to be added (read-only), Phase 2 mutates.
    pub fn refresh_continuous_effects(&mut self) {
        self.invalidate_characteristics_cache();

        // Remove effects whose source has left the battlefield
        let bf: HashSet<ObjectId> = self.battlefield.iter().copied().collect();
        self.continuous_effects.retain(|e| {
            match e.duration {
                crate::layers::Duration::WhileSourceOnBattlefield => {
                    bf.contains(&e.source_id)
                }
                _ => true, // UntilEndOfTurn and Permanent effects persist
            }
        });

        // Phase 1: Read — collect new effects to add
        let existing_static_sources: std::collections::HashSet<ObjectId> = self
            .continuous_effects
            .iter()
            .filter(|e| e.duration == crate::layers::Duration::WhileSourceOnBattlefield)
            .map(|e| e.source_id)
            .collect();

        let mut new_effects = Vec::new();
        let mut ts = self.next_timestamp;
        {
            let db = self.card_db();
            for &obj_id in &self.battlefield {
                if existing_static_sources.contains(&obj_id) {
                    continue;
                }
                let inst = match self.objects.get(&obj_id) {
                    Some(i) => i,
                    None => continue,
                };
                let def = match db.get(inst.card_def_id) {
                    Some(d) => d,
                    None => continue,
                };
                for sa in &def.static_abilities {
                    let generated = sa.to_continuous_effects(obj_id, inst.controller, ts);
                    ts += 1;
                    new_effects.extend(generated);
                }
            }
        }

        // Phase 2: Write — apply collected effects
        self.next_timestamp = ts;
        self.continuous_effects.extend(new_effects);
    }

    /// Remove all UntilEndOfTurn continuous effects (called during cleanup).
    pub fn cleanup_eot_effects(&mut self) {
        self.invalidate_characteristics_cache();
        self.continuous_effects
            .retain(|e| e.duration != crate::layers::Duration::UntilEndOfTurn);
    }

    /// Get the computed characteristics for an object, using the transient cache
    /// to avoid redundant recomputation within a single game step.
    ///
    /// On a cache miss the battlefield `HashSet` is built once (amortised across
    /// all objects in the same cache epoch) and passed to the layer engine for
    /// O(1) membership tests.
    fn get_characteristics(&self, obj_id: ObjectId) -> Option<ComputedCharacteristics> {
        let mut cache = self.characteristics_cache.0.lock().unwrap();

        // Fast path: cache hit
        if let Some(cached) = cache.entries.get(&obj_id) {
            return Some(cached.clone());
        }

        // Ensure battlefield HashSet is built (once per cache epoch)
        if cache.battlefield_set.is_none() {
            cache.battlefield_set = Some(self.battlefield.iter().copied().collect());
        }

        // Compute characteristics with the cached battlefield set
        let bf_set = cache.battlefield_set.as_ref().unwrap();
        let result = crate::layers::compute_characteristics(
            obj_id,
            &self.continuous_effects,
            &self.objects,
            bf_set,
            self.card_db(),
        );

        if let Some(ref chars) = result {
            cache.entries.insert(obj_id, chars.clone());
        }

        result
    }

    /// Invalidate the characteristics cache.
    ///
    /// Must be called whenever canonical state that affects characteristics
    /// changes: battlefield membership, continuous effects, or object
    /// counters/controller.
    pub fn invalidate_characteristics_cache(&self) {
        let mut cache = self.characteristics_cache.0.lock().unwrap();
        cache.entries.clear();
        cache.battlefield_set = None;
    }

    /// Compute the effective power of a creature using the layer engine.
    pub fn effective_power(&self, obj_id: ObjectId) -> i32 {
        self.get_characteristics(obj_id)
            .map(|c| c.power)
            .unwrap_or(0)
    }

    /// Compute the effective toughness of a creature using the layer engine.
    pub fn effective_toughness(&self, obj_id: ObjectId) -> i32 {
        self.get_characteristics(obj_id)
            .map(|c| c.toughness)
            .unwrap_or(0)
    }

    /// Check if an object has a keyword ability using the layer engine.
    pub fn has_keyword(&self, obj_id: ObjectId, kw: crate::card::KeywordAbility) -> bool {
        self.get_characteristics(obj_id)
            .map(|c| c.keywords.contains(&kw))
            .unwrap_or(false)
    }

    /// Apply damage with replacement effects (CR 614).
    ///
    /// Checks for damage-replacement effects before dealing damage.
    /// Self-replacement effects are applied automatically. If multiple
    /// non-self replacements apply, the affected player chooses the order
    /// (surfaced via Action::ChooseReplacementOrder in a future iteration).
    ///
    /// Returns the actual damage dealt after replacements.
    pub fn deal_damage_with_replacement(&mut self, amount: u32, target: &Target) -> u32 {
        let (self_replacements, _player_choice) =
            crate::replacement::find_applicable_replacements(
                &self.replacement_effects,
                &ReplacementEventKind::DamageDealt,
                match target {
                    Target::Player(p) => *p,
                    Target::Object(id) => self.objects.get(id).map(|i| i.controller).unwrap_or(0),
                },
            );

        let mut effective_amount = amount as i32;

        // Apply self-replacement effects automatically
        for &idx in &self_replacements {
            if let Some(effect) = self.replacement_effects.get(idx) {
                match &effect.action {
                    ReplacementAction::Prevent => {
                        effective_amount = 0;
                    }
                    ReplacementAction::ModifyAmount { delta } => {
                        effective_amount += delta;
                    }
                    _ => {}
                }
            }
        }

        // TODO: handle player-choice replacement effects (ChooseReplacementOrder)
        // For now, apply them in order
        // (CR 614.5: each effect applies only once per event)

        effective_amount.max(0) as u32
    }

    /// Check for death replacement effects (CR 614).
    ///
    /// Returns the zone to send the dying creature to (normally Graveyard,
    /// but can be Exile or other zones if a replacement applies).
    pub fn death_replacement_zone(&self, obj_id: ObjectId) -> ZoneType {
        let controller = self
            .objects
            .get(&obj_id)
            .map(|i| i.controller)
            .unwrap_or(0);

        let (self_replacements, _player_choice) =
            crate::replacement::find_applicable_replacements(
                &self.replacement_effects,
                &ReplacementEventKind::WouldDie,
                controller,
            );

        // Apply self-replacement effects first
        for &idx in &self_replacements {
            if let Some(effect) = self.replacement_effects.get(idx) {
                match &effect.action {
                    ReplacementAction::RedirectToZone(zone) => {
                        return *zone;
                    }
                    ReplacementAction::Prevent => {
                        // Prevented death means the creature stays on the battlefield.
                        // This is a special case — we return Battlefield to signal "don't move".
                        return ZoneType::Battlefield;
                    }
                    _ => {}
                }
            }
        }

        // TODO: handle player-choice replacement effects

        ZoneType::Graveyard
    }

    /// Check for ETB replacement effects and apply them (CR 614).
    ///
    /// Applies modifications like "enters tapped" or "enters with counters".
    pub fn apply_etb_replacements(&mut self, obj_id: ObjectId) {
        let controller = self
            .objects
            .get(&obj_id)
            .map(|i| i.controller)
            .unwrap_or(0);

        let (self_replacements, _player_choice) =
            crate::replacement::find_applicable_replacements(
                &self.replacement_effects,
                &ReplacementEventKind::EntersBattlefield,
                controller,
            );

        let mut counters_changed = false;
        for &idx in &self_replacements {
            if let Some(effect) = self.replacement_effects.get(idx).cloned() {
                match &effect.action {
                    ReplacementAction::EntersModified {
                        enters_tapped,
                        extra_counters,
                    } => {
                        if let Some(inst) = self.objects.get_mut(&obj_id) {
                            if *enters_tapped {
                                inst.tapped = true;
                            }
                            if *extra_counters > 0 {
                                inst.plus_counters += extra_counters;
                                counters_changed = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        if counters_changed {
            self.invalidate_characteristics_cache();
        }
    }

    /// Refresh replacement effects based on the current battlefield.
    /// Removes effects whose source has left the battlefield.
    pub fn refresh_replacement_effects(&mut self) {
        let bf: HashSet<ObjectId> = self.battlefield.iter().copied().collect();
        self.replacement_effects
            .retain(|e| bf.contains(&e.source_id));
    }

    /// Check if an object is a creature using the layer engine.
    pub fn is_creature(&self, obj_id: ObjectId) -> bool {
        self.get_characteristics(obj_id)
            .map(|c| c.card_types.contains(&crate::card::CardType::Creature))
            .unwrap_or(false)
    }
}
