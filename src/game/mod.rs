use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Card definitions (shared, immutable).
    /// In practice we'd use an Arc, but for simplicity we store a reference-counted DB.
    #[serde(skip)]
    pub card_db: Option<CardDatabase>,

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
    name_index: HashMap<String, CardId>,
}

impl CardDatabase {
    pub fn new() -> Self {
        CardDatabase {
            cards: HashMap::new(),
            name_index: HashMap::new(),
        }
    }

    pub fn insert(&mut self, card: CardDef) {
        self.name_index.insert(card.name.to_lowercase(), card.id);
        self.cards.insert(card.id, card);
    }

    pub fn get(&self, id: CardId) -> Option<&CardDef> {
        self.cards.get(&id)
    }

    pub fn find_by_name(&self, name: &str) -> Option<CardId> {
        let target = name.trim().to_lowercase();
        self.name_index.get(&target).copied()
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
        self.card_db
            .as_ref()
            .expect("CardDatabase not set on GameState")
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
            ZoneType::Stack => {}   // handled separately
            ZoneType::Command => {} // not implemented yet
        }
        obj_id
    }

    /// Move a card instance from one zone to another.
    pub fn move_object(&mut self, obj_id: ObjectId, _from: ZoneType, to: ZoneType) {
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
            self.players[controller]
                .graveyard
                .retain(|&id| id != obj_id);
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
                let enters_tapped = self
                    .card_db()
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
                inst.controller == player && db.get(inst.card_def_id).map_or(false, |d| d.is_land())
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
        self.players[player].life <= 0 || self.players[player].has_lost
    }
}
