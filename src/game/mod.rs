use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::card::{CardDef, CardId, CardInstance, ObjectId, ZoneType};
use crate::mana::ManaPool;

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

    /// Game over flag.
    pub game_over: bool,

    /// Winner (if game is over). None = draw.
    pub winner: Option<PlayerIndex>,
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

    // --- Object lookup (shared, read-only) ---
    /// All card instances in the game, keyed by ObjectId.
    pub objects: &'a HashMap<ObjectId, CardInstance>,
    /// Card definitions database (shared, immutable).
    pub card_db: &'a CardDatabase,
}

impl GameState {
    /// Build a `PlayerView` for the given player, exposing only information
    /// that player is entitled to see under the MTG rules.
    pub fn visible_state(&self, player: PlayerIndex) -> PlayerView<'_> {
        let opp = self.opponent(player);
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

            objects: &self.objects,
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
            game_over: false,
            winner: None,
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
        _from: ZoneType,
        to: ZoneType,
    ) {
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

    /// Check if a player has lost.
    pub fn check_player_lost(&self, player: PlayerIndex) -> bool {
        self.players[player].life <= 0
            || self.players[player].has_lost
    }
}
