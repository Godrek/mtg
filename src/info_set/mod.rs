//! Phase 1B.1 — Information Set Module
//!
//! An information set represents everything a player can observe about the
//! game state. Two game positions that look identical from a player's
//! perspective belong to the same information set — the player cannot
//! distinguish between them and must play the same mixed strategy in both.
//!
//! This module converts `PlayerView` (the observation API from Phase 0.1)
//! into a hashable `InformationSet` suitable for keying MCCFR regret tables.
//!
//! # Design constraints
//!
//! - Reads **only** from `PlayerView`, never from raw `GameState` fields.
//! - Deterministic: same observable game position always produces the same hash.
//! - The hash is a `u64` computed via a stable, order-independent scheme
//!   for sets (battlefield, graveyards) and order-dependent for sequences
//!   (hand, stack).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::card::CardInstance;
use crate::game::{CardDatabase, PlayerView, StackEntry, StackSource};

/// A compact representation of everything a player can observe.
///
/// Two `InformationSet` values are equal iff the player cannot distinguish
/// the underlying game states. MCCFR regret tables key on `info_set_hash()`.
#[derive(Debug, Clone)]
pub struct InformationSet {
    /// Current game phase.
    pub phase: u8,
    /// Whose turn it is.
    pub active_player: usize,
    /// Current turn number (capped for hashing stability).
    pub turn_number: u32,
    /// Who has priority.
    pub priority_player: usize,

    /// Viewing player's life total.
    pub my_life: i32,
    /// Opponent's life total.
    pub opp_life: i32,

    /// Cards in our hand, represented as sorted CardIds.
    pub my_hand: Vec<u64>,
    /// Opponent's hand size (contents unknown).
    pub opp_hand_size: usize,
    /// Opponent's library size (contents unknown).
    pub opp_library_size: usize,

    /// Permanents on the battlefield, represented as sorted (controller, card_id, tapped, power, toughness) tuples.
    pub battlefield: Vec<PermanentInfo>,

    /// Stack entries, in stack order (top = last element).
    pub stack_entries: Vec<StackInfo>,

    /// Our graveyard as sorted CardIds.
    pub my_graveyard: Vec<u64>,
    /// Opponent's graveyard as sorted CardIds.
    pub opp_graveyard: Vec<u64>,

    /// Our exile zone as sorted CardIds.
    pub my_exile: Vec<u64>,
    /// Opponent's exile zone as sorted CardIds.
    pub opp_exile: Vec<u64>,

    /// Remaining land plays this turn.
    pub my_land_plays_remaining: u32,
    /// Available mana per color (W, U, B, R, G, colorless).
    pub my_mana: [u32; 6],
}

/// Observable information about a permanent on the battlefield.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PermanentInfo {
    pub controller: usize,
    pub card_id: u64,
    pub tapped: bool,
    pub damage_marked: u32,
    pub summoning_sick: bool,
    pub plus_counters: i32,
    pub minus_counters: i32,
}

/// Observable information about a stack entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StackInfo {
    pub controller: usize,
    pub source_card_id: u64,
    pub target_summary: Vec<u64>, // hashed target descriptions
}

impl InformationSet {
    /// Construct an `InformationSet` from a `PlayerView`.
    ///
    /// This is the sole entry point — MCCFR never reads raw `GameState`.
    pub fn from_view(view: &PlayerView, _card_db: &CardDatabase) -> Self {
        let phase = phase_to_u8(view.phase);

        // Hand: sorted CardIds for canonical representation
        let mut my_hand: Vec<u64> = view
            .my_hand
            .iter()
            .filter_map(|&id| view.objects.get(&id).map(|inst| inst.card_def_id))
            .collect();
        my_hand.sort();

        // Battlefield: sorted PermanentInfo for canonical representation
        let mut battlefield: Vec<PermanentInfo> = view
            .battlefield
            .iter()
            .filter_map(|&id| {
                view.objects.get(&id).map(|inst| PermanentInfo {
                    controller: inst.controller,
                    card_id: inst.card_def_id,
                    tapped: inst.tapped,
                    damage_marked: inst.damage_marked,
                    summoning_sick: inst.summoning_sick,
                    plus_counters: inst.plus_counters,
                    minus_counters: inst.minus_counters,
                })
            })
            .collect();
        battlefield.sort();

        // Stack: preserve order (LIFO semantics matter)
        let stack_entries: Vec<StackInfo> = view
            .stack
            .iter()
            .map(|entry| stack_entry_to_info(entry, &view.objects))
            .collect();

        // Graveyards: sorted CardIds
        let mut my_graveyard: Vec<u64> = view
            .my_graveyard
            .iter()
            .filter_map(|&id| view.objects.get(&id).map(|inst| inst.card_def_id))
            .collect();
        my_graveyard.sort();

        let mut opp_graveyard: Vec<u64> = view
            .opp_graveyard
            .iter()
            .filter_map(|&id| view.objects.get(&id).map(|inst| inst.card_def_id))
            .collect();
        opp_graveyard.sort();

        // Exile zones: sorted CardIds (public information)
        let mut my_exile: Vec<u64> = view
            .my_exile
            .iter()
            .filter_map(|&id| view.objects.get(&id).map(|inst| inst.card_def_id))
            .collect();
        my_exile.sort();

        let mut opp_exile: Vec<u64> = view
            .opp_exile
            .iter()
            .filter_map(|&id| view.objects.get(&id).map(|inst| inst.card_def_id))
            .collect();
        opp_exile.sort();

        // Mana: per-color breakdown
        let mana = &view.my_mana_pool;
        let my_mana = [
            mana.white,
            mana.blue,
            mana.black,
            mana.red,
            mana.green,
            mana.colorless,
        ];

        InformationSet {
            phase,
            active_player: view.active_player,
            turn_number: view.turn_number,
            priority_player: view.priority_player,
            my_life: view.my_life,
            opp_life: view.opp_life,
            my_hand,
            opp_hand_size: view.opp_hand_size,
            opp_library_size: view.opp_library_size,
            battlefield,
            stack_entries,
            my_graveyard,
            opp_graveyard,
            my_exile,
            opp_exile,
            my_land_plays_remaining: view.my_land_plays_remaining,
            my_mana,
        }
    }

    /// Compute a stable hash for this information set.
    ///
    /// Two identical observable game states always produce the same hash.
    /// Collisions are possible but unlikely for practical info set spaces.
    pub fn hash_value(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.phase.hash(&mut hasher);
        self.active_player.hash(&mut hasher);
        self.turn_number.hash(&mut hasher);
        self.priority_player.hash(&mut hasher);
        self.my_life.hash(&mut hasher);
        self.opp_life.hash(&mut hasher);
        self.my_hand.hash(&mut hasher);
        self.opp_hand_size.hash(&mut hasher);
        self.opp_library_size.hash(&mut hasher);
        self.battlefield.hash(&mut hasher);
        self.stack_entries.hash(&mut hasher);
        self.my_graveyard.hash(&mut hasher);
        self.opp_graveyard.hash(&mut hasher);
        self.my_exile.hash(&mut hasher);
        self.opp_exile.hash(&mut hasher);
        self.my_land_plays_remaining.hash(&mut hasher);
        self.my_mana.hash(&mut hasher);
        hasher.finish()
    }
}

/// Convert a Phase enum to a stable u8 for hashing.
fn phase_to_u8(phase: crate::game::Phase) -> u8 {
    use crate::game::Phase;
    match phase {
        Phase::Untap => 0,
        Phase::Upkeep => 1,
        Phase::Draw => 2,
        Phase::PreCombatMain => 3,
        Phase::BeginningOfCombat => 4,
        Phase::DeclareAttackers => 5,
        Phase::DeclareBlockers => 6,
        Phase::FirstStrikeDamage => 7,
        Phase::CombatDamage => 8,
        Phase::EndOfCombat => 9,
        Phase::PostCombatMain => 10,
        Phase::EndStep => 11,
        Phase::Cleanup => 12,
    }
}

/// Convert a `StackEntry` into observable `StackInfo`.
fn stack_entry_to_info(
    entry: &StackEntry,
    objects: &std::collections::HashMap<crate::card::ObjectId, &CardInstance>,
) -> StackInfo {
    let source_card_id = match entry.source {
        StackSource::Spell(obj_id) => objects
            .get(&obj_id)
            .map(|inst| inst.card_def_id)
            .unwrap_or(0),
        StackSource::ActivatedAbility { source_id, .. } => objects
            .get(&source_id)
            .map(|inst| inst.card_def_id)
            .unwrap_or(0),
        StackSource::TriggeredAbility { source_id, .. } => objects
            .get(&source_id)
            .map(|inst| inst.card_def_id)
            .unwrap_or(0),
    };

    let target_summary: Vec<u64> = entry
        .targets
        .iter()
        .map(|t| {
            let mut h = DefaultHasher::new();
            match t {
                crate::game::Target::Player(idx) => {
                    0u8.hash(&mut h);
                    idx.hash(&mut h);
                }
                crate::game::Target::Object(obj_id) => {
                    1u8.hash(&mut h);
                    if let Some(inst) = objects.get(obj_id) {
                        inst.card_def_id.hash(&mut h);
                        inst.controller.hash(&mut h);
                    } else {
                        obj_id.hash(&mut h);
                    }
                }
            }
            h.finish()
        })
        .collect();

    StackInfo {
        controller: entry.controller,
        source_card_id,
        target_summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::sample;
    use crate::card::ZoneType;
    use crate::game::{GameState, Phase};
    use std::sync::Arc;

    fn setup_test_state() -> GameState {
        let db = sample::build_sample_db();
        let mut state = GameState::new(2);
        state.card_db = Some(Arc::new(db));
        for _ in 0..20 {
            state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Library);
            state.create_card_in_zone(sample::ids::FOREST, 1, ZoneType::Library);
        }
        state
    }

    #[test]
    fn test_info_set_from_view_basic() {
        let mut state = setup_test_state();
        state.create_card_in_zone(sample::ids::LIGHTNING_BOLT, 0, ZoneType::Hand);
        state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Battlefield);
        state.active_player = 0;
        state.priority_player = 0;
        state.phase = Phase::PreCombatMain;

        let view = state.visible_state(0);
        let info_set = InformationSet::from_view(&view, state.card_db());

        assert_eq!(info_set.phase, 3); // PreCombatMain
        assert_eq!(info_set.active_player, 0);
        assert_eq!(info_set.my_life, 20);
        assert_eq!(info_set.opp_life, 20);
        assert_eq!(info_set.my_hand.len(), 1);
        assert_eq!(info_set.my_hand[0], sample::ids::LIGHTNING_BOLT);
        assert!(!info_set.battlefield.is_empty());
    }

    #[test]
    fn test_info_set_hash_deterministic() {
        let mut state = setup_test_state();
        state.create_card_in_zone(sample::ids::LIGHTNING_BOLT, 0, ZoneType::Hand);
        state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Battlefield);
        state.active_player = 0;
        state.priority_player = 0;
        state.phase = Phase::PreCombatMain;

        let view1 = state.visible_state(0);
        let info1 = InformationSet::from_view(&view1, state.card_db());

        let view2 = state.visible_state(0);
        let info2 = InformationSet::from_view(&view2, state.card_db());

        assert_eq!(info1.hash_value(), info2.hash_value());
    }

    #[test]
    fn test_info_set_different_hands_different_hash() {
        let mut state1 = setup_test_state();
        state1.create_card_in_zone(sample::ids::LIGHTNING_BOLT, 0, ZoneType::Hand);
        state1.active_player = 0;
        state1.priority_player = 0;
        state1.phase = Phase::PreCombatMain;

        let mut state2 = setup_test_state();
        state2.create_card_in_zone(sample::ids::GREY_OGRE, 0, ZoneType::Hand);
        state2.active_player = 0;
        state2.priority_player = 0;
        state2.phase = Phase::PreCombatMain;

        let view1 = state1.visible_state(0);
        let info1 = InformationSet::from_view(&view1, state1.card_db());

        let view2 = state2.visible_state(0);
        let info2 = InformationSet::from_view(&view2, state2.card_db());

        assert_ne!(info1.hash_value(), info2.hash_value());
    }

    #[test]
    fn test_info_set_same_observable_same_hash() {
        // Two game states that look the same from player 0's perspective
        // should produce the same info set hash, even if opponent's hidden
        // state differs.
        let mut state1 = setup_test_state();
        state1.create_card_in_zone(sample::ids::LIGHTNING_BOLT, 0, ZoneType::Hand);
        state1.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Battlefield);
        // Opponent has bolt in hand (hidden from player 0)
        state1.create_card_in_zone(sample::ids::LIGHTNING_BOLT, 1, ZoneType::Hand);
        state1.active_player = 0;
        state1.priority_player = 0;
        state1.phase = Phase::PreCombatMain;

        let mut state2 = setup_test_state();
        state2.create_card_in_zone(sample::ids::LIGHTNING_BOLT, 0, ZoneType::Hand);
        state2.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Battlefield);
        // Opponent has Grey Ogre in hand instead (hidden from player 0)
        state2.create_card_in_zone(sample::ids::GREY_OGRE, 1, ZoneType::Hand);
        state2.active_player = 0;
        state2.priority_player = 0;
        state2.phase = Phase::PreCombatMain;

        let view1 = state1.visible_state(0);
        let info1 = InformationSet::from_view(&view1, state1.card_db());

        let view2 = state2.visible_state(0);
        let info2 = InformationSet::from_view(&view2, state2.card_db());

        // Same observable state, same hash (opponent hand contents hidden)
        assert_eq!(info1.hash_value(), info2.hash_value());
    }
}
