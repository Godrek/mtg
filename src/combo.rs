//! Macro-actions and combo detection for the MCCFR solver.
//!
//! Infinite combos in MTG (e.g., Basalt Monolith + Kinnan = infinite mana)
//! require hundreds of individual actions to execute step-by-step, which
//! exhausts the solver's action budget and results in draws. This module
//! provides:
//!
//! 1. **Macro-actions**: Pre-defined combo sequences offered as single
//!    `Action::ActivateMacro` choices in the legal action list, letting the
//!    solver shortcut known loops without discovering them action-by-action.
//!
//! 2. **Combo proximity scoring**: Partial credit in the heuristic evaluator
//!    for having combo pieces on the battlefield, even before the full combo
//!    is assembled.
//!
//! # Design
//!
//! Combos are registered in a [`ComboRegistry`] attached to `GameState` via
//! `Arc` (zero-cost clone, same pattern as `card_db`). Each [`ComboDef`]
//! specifies:
//! - Required pieces (card IDs that must be on the battlefield)
//! - Preconditions (e.g., a piece must be untapped)
//! - The resulting effect (e.g., add N colorless mana)
//! - A reward-shaping weight for partial assembly
//!
//! When `legal_actions_with()` detects that all pieces of a registered combo
//! are on the battlefield and preconditions are met, it injects an
//! `Action::ActivateMacro { combo_id }` into the legal action list. The
//! rules engine applies the effect in a single state transition.

use serde::{Deserialize, Serialize};

use crate::card::{CardId, ObjectId};
use crate::game::{GameState, PlayerIndex};
use crate::mana::Color;

// =========================================================================
// Combo definitions
// =========================================================================

/// A registered combo that can be activated as a single macro-action.
#[derive(Debug, Clone)]
pub struct ComboDef {
    /// Unique identifier for this combo (index in the registry).
    pub id: usize,
    /// Human-readable name (e.g., "Basalt Monolith + Kinnan Infinite Mana").
    pub name: String,
    /// Card IDs that must all be on the battlefield under the same controller.
    pub required_pieces: Vec<CardId>,
    /// Additional preconditions beyond having pieces on the battlefield.
    pub preconditions: Vec<ComboPrecondition>,
    /// Effect produced by activating the combo.
    pub effect: ComboEffect,
    /// Reward-shaping weight: how much partial credit to give for having
    /// some (but not all) pieces on the battlefield. Higher = more incentive
    /// to assemble the combo. Typical range: 0.05 to 0.3.
    pub reward_weight: f64,
}

/// What executing the combo produces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComboEffect {
    /// Add N colorless mana to the controller's pool.
    AddColorlessMana(u32),
    /// Add N mana of a specific color.
    AddColoredMana(Color, u32),
    /// Deal N damage to target opponent.
    DealDamageToOpponent(u32),
    /// Gain N life.
    GainLife(u32),
    /// Draw N cards.
    DrawCards(u32),
    /// Multiple effects applied in sequence.
    Multiple(Vec<ComboEffect>),
}

/// A precondition that must be satisfied beyond having all pieces on the battlefield.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComboPrecondition {
    /// A specific piece must be untapped (e.g., the mana rock that taps as
    /// part of the combo).
    PieceUntapped(CardId),
}

// =========================================================================
// Combo registry
// =========================================================================

/// Collection of known combos that the solver can activate as macro-actions.
#[derive(Debug, Clone, Default)]
pub struct ComboRegistry {
    pub combos: Vec<ComboDef>,
}

impl ComboRegistry {
    pub fn new() -> Self {
        ComboRegistry { combos: Vec::new() }
    }

    /// Register a combo. The combo's `id` is set to its index in the registry.
    pub fn register(&mut self, mut combo: ComboDef) -> usize {
        let id = self.combos.len();
        combo.id = id;
        self.combos.push(combo);
        id
    }

    /// Get a combo definition by ID.
    pub fn get(&self, id: usize) -> Option<&ComboDef> {
        self.combos.get(id)
    }
}

// =========================================================================
// Default combos for the Kinnan Commander deck
// =========================================================================

/// Build the default combo registry with well-known MTG combos.
///
/// Currently registered:
/// - **Basalt Monolith + Kinnan**: Tap Monolith for 3 colorless, Kinnan adds
///   1 (total 4), pay 3 to untap = net +1 per iteration = infinite colorless.
///   Macro produces 100 colorless mana (enough to win via any mana sink).
/// - **Grim Monolith + Kinnan**: Tap Grim for 3, Kinnan adds 1 (total 4),
///   pay 4 to untap = net 0, BUT Kinnan triggers again on the re-tap making
///   it net +1 per full cycle. We model this as producing 100 colorless.
pub fn build_default_combos() -> ComboRegistry {
    use crate::card::sample::ids;

    let mut registry = ComboRegistry::new();

    // Basalt Monolith + Kinnan = infinite colorless mana
    // Monolith taps for 3, Kinnan adds 1 = 4 total; pay 3 to untap = net +1/loop
    registry.register(ComboDef {
        id: 0,
        name: "Basalt Monolith + Kinnan Infinite Mana".into(),
        required_pieces: vec![ids::BASALT_MONOLITH, ids::KINNAN_BONDER_PRODIGY],
        preconditions: vec![ComboPrecondition::PieceUntapped(ids::BASALT_MONOLITH)],
        effect: ComboEffect::AddColorlessMana(100),
        reward_weight: 0.2,
    });

    // Grim Monolith + Kinnan = infinite colorless mana
    // Grim taps for 3, Kinnan adds 1 = 4 total; pay 4 to untap = break-even,
    // but next tap cycle Kinnan triggers again = net +1/full cycle
    registry.register(ComboDef {
        id: 0,
        name: "Grim Monolith + Kinnan Infinite Mana".into(),
        required_pieces: vec![ids::GRIM_MONOLITH, ids::KINNAN_BONDER_PRODIGY],
        preconditions: vec![ComboPrecondition::PieceUntapped(ids::GRIM_MONOLITH)],
        effect: ComboEffect::AddColorlessMana(100),
        reward_weight: 0.2,
    });

    registry
}

// =========================================================================
// Combo detection
// =========================================================================

/// Check which registered combos are currently available for a player.
///
/// Returns a list of combo IDs that can be activated (all pieces present
/// on the battlefield under the player's control and all preconditions met).
pub fn detect_available_combos(
    state: &GameState,
    player: PlayerIndex,
    registry: &ComboRegistry,
) -> Vec<usize> {
    let mut available = Vec::new();

    let controlled: Vec<(ObjectId, CardId)> = state
        .battlefield
        .iter()
        .filter_map(|&obj_id| {
            let inst = state.objects.get(&obj_id)?;
            if inst.controller == player {
                Some((obj_id, inst.card_def_id))
            } else {
                None
            }
        })
        .collect();

    for combo in &registry.combos {
        // Check all required pieces are present
        let mut all_pieces_present = true;
        for &required_card_id in &combo.required_pieces {
            if !controlled.iter().any(|&(_, cid)| cid == required_card_id) {
                all_pieces_present = false;
                break;
            }
        }
        if !all_pieces_present {
            continue;
        }

        // Check preconditions
        let mut preconditions_met = true;
        for precond in &combo.preconditions {
            match precond {
                ComboPrecondition::PieceUntapped(card_id) => {
                    let untapped = controlled.iter().any(|&(obj_id, cid)| {
                        cid == *card_id && !state.objects[&obj_id].tapped
                    });
                    if !untapped {
                        preconditions_met = false;
                        break;
                    }
                }
            }
        }
        if !preconditions_met {
            continue;
        }

        available.push(combo.id);
    }

    available
}

// =========================================================================
// Combo proximity for reward shaping
// =========================================================================

/// Compute a combo proximity bonus for the heuristic evaluator.
///
/// For each registered combo, gives partial credit proportional to the
/// fraction of pieces the player has on the battlefield:
///   `bonus += (pieces_present / total_pieces) * combo.reward_weight`
///
/// When all pieces are assembled (but the macro hasn't been activated yet),
/// the full reward_weight is given. This incentivizes the solver to:
/// 1. Collect combo pieces (play Basalt Monolith, keep Kinnan alive)
/// 2. Activate the macro when available
///
/// Returns a value in [0.0, sum_of_all_weights] — typically [0.0, ~0.5].
pub fn combo_proximity_bonus(
    state: &GameState,
    player: PlayerIndex,
    registry: &ComboRegistry,
) -> f64 {
    let controlled_card_ids: Vec<CardId> = state
        .battlefield
        .iter()
        .filter_map(|&obj_id| {
            let inst = state.objects.get(&obj_id)?;
            if inst.controller == player {
                Some(inst.card_def_id)
            } else {
                None
            }
        })
        .collect();

    let mut bonus = 0.0;
    for combo in &registry.combos {
        if combo.required_pieces.is_empty() {
            continue;
        }
        let pieces_present = combo
            .required_pieces
            .iter()
            .filter(|&&cid| controlled_card_ids.contains(&cid))
            .count();
        let fraction = pieces_present as f64 / combo.required_pieces.len() as f64;
        bonus += fraction * combo.reward_weight;
    }
    bonus
}

// =========================================================================
// Applying combo effects
// =========================================================================

/// Apply a combo's effect to the game state.
///
/// Called by `apply_action()` when handling `Action::ActivateMacro`.
/// The combo pieces are tapped as a side effect (the Monolith that
/// produces mana ends up tapped after the infinite loop).
pub fn apply_combo_effect(
    state: &mut GameState,
    player: PlayerIndex,
    combo: &ComboDef,
) {
    // Tap any piece that has a PieceUntapped precondition (it was tapped
    // as part of the combo loop).
    for precond in &combo.preconditions {
        match precond {
            ComboPrecondition::PieceUntapped(card_id) => {
                // Find the untapped instance and tap it
                for &obj_id in &state.battlefield {
                    let inst = &state.objects[&obj_id];
                    if inst.controller == player
                        && inst.card_def_id == *card_id
                        && !inst.tapped
                    {
                        if let Some(inst_mut) = state.objects.get_mut(&obj_id) {
                            inst_mut.tapped = true;
                        }
                        break;
                    }
                }
            }
        }
    }

    // Apply the effect
    apply_effect_recursive(state, player, &combo.effect);
}

fn apply_effect_recursive(
    state: &mut GameState,
    player: PlayerIndex,
    effect: &ComboEffect,
) {
    match effect {
        ComboEffect::AddColorlessMana(amount) => {
            state.players[player].mana_pool.colorless += amount;
        }
        ComboEffect::AddColoredMana(color, amount) => {
            state.players[player].mana_pool.add_color(*color, *amount);
        }
        ComboEffect::DealDamageToOpponent(amount) => {
            let opp = state.opponent(player);
            state.players[opp].life -= *amount as i32;
        }
        ComboEffect::GainLife(amount) => {
            state.players[player].life += *amount as i32;
        }
        ComboEffect::DrawCards(count) => {
            for _ in 0..*count {
                if let Some(card_id) = state.players[player].library.pop() {
                    state.players[player].hand.push(card_id);
                }
            }
        }
        ComboEffect::Multiple(effects) => {
            for sub_effect in effects {
                apply_effect_recursive(state, player, sub_effect);
            }
        }
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::sample::{self, ids};
    use crate::card::ZoneType;
    use std::sync::Arc;

    fn setup_state_with_combos() -> (GameState, ComboRegistry) {
        let db = sample::build_sample_db();
        let mut state = GameState::new(2);
        state.card_db = Some(Arc::new(db));

        // Libraries so no one decks out
        for _ in 0..20 {
            state.create_card_in_zone(sample::ids::MOUNTAIN, 0, ZoneType::Library);
            state.create_card_in_zone(sample::ids::FOREST, 1, ZoneType::Library);
        }

        let registry = build_default_combos();
        (state, registry)
    }

    #[test]
    fn test_detect_no_combo_pieces() {
        let (state, registry) = setup_state_with_combos();
        let available = detect_available_combos(&state, 0, &registry);
        assert!(available.is_empty(), "No combo pieces = no combos");
    }

    #[test]
    fn test_detect_partial_combo() {
        let (mut state, registry) = setup_state_with_combos();

        // Only Basalt Monolith, no Kinnan
        state.create_card_in_zone(ids::BASALT_MONOLITH, 0, ZoneType::Battlefield);

        let available = detect_available_combos(&state, 0, &registry);
        assert!(available.is_empty(), "Missing Kinnan = combo not available");
    }

    #[test]
    fn test_detect_full_combo() {
        let (mut state, registry) = setup_state_with_combos();

        // Both pieces on battlefield
        state.create_card_in_zone(ids::BASALT_MONOLITH, 0, ZoneType::Battlefield);
        state.create_card_in_zone(ids::KINNAN_BONDER_PRODIGY, 0, ZoneType::Battlefield);

        // Monolith starts untapped, so combo should be available
        let available = detect_available_combos(&state, 0, &registry);
        assert!(
            available.contains(&0),
            "Basalt Monolith + Kinnan combo should be available"
        );
    }

    #[test]
    fn test_detect_combo_tapped_monolith() {
        let (mut state, registry) = setup_state_with_combos();

        let monolith = state.create_card_in_zone(ids::BASALT_MONOLITH, 0, ZoneType::Battlefield);
        state.create_card_in_zone(ids::KINNAN_BONDER_PRODIGY, 0, ZoneType::Battlefield);

        // Tap the monolith
        state.objects.get_mut(&monolith).unwrap().tapped = true;

        let available = detect_available_combos(&state, 0, &registry);
        assert!(
            !available.contains(&0),
            "Tapped Monolith = combo not available"
        );
    }

    #[test]
    fn test_detect_combo_wrong_controller() {
        let (mut state, registry) = setup_state_with_combos();

        // Monolith controlled by player 0, Kinnan controlled by player 1
        state.create_card_in_zone(ids::BASALT_MONOLITH, 0, ZoneType::Battlefield);
        state.create_card_in_zone(ids::KINNAN_BONDER_PRODIGY, 1, ZoneType::Battlefield);

        let available = detect_available_combos(&state, 0, &registry);
        assert!(
            available.is_empty(),
            "Pieces under different controllers = no combo"
        );
    }

    #[test]
    fn test_combo_proximity_no_pieces() {
        let (state, registry) = setup_state_with_combos();
        let bonus = combo_proximity_bonus(&state, 0, &registry);
        assert!(
            bonus.abs() < 1e-10,
            "No pieces = zero bonus, got {}",
            bonus
        );
    }

    #[test]
    fn test_combo_proximity_partial() {
        let (mut state, registry) = setup_state_with_combos();

        // Only Kinnan (1 of 2 pieces for both combos)
        state.create_card_in_zone(ids::KINNAN_BONDER_PRODIGY, 0, ZoneType::Battlefield);

        let bonus = combo_proximity_bonus(&state, 0, &registry);
        // Kinnan is 1/2 of both combos, each with weight 0.2
        // bonus = (0.5 * 0.2) + (0.5 * 0.2) = 0.2
        assert!(
            (bonus - 0.2).abs() < 1e-10,
            "Half pieces = half weight per combo, got {}",
            bonus
        );
    }

    #[test]
    fn test_combo_proximity_full() {
        let (mut state, registry) = setup_state_with_combos();

        // Both pieces for Basalt combo
        state.create_card_in_zone(ids::BASALT_MONOLITH, 0, ZoneType::Battlefield);
        state.create_card_in_zone(ids::KINNAN_BONDER_PRODIGY, 0, ZoneType::Battlefield);

        let bonus = combo_proximity_bonus(&state, 0, &registry);
        // Basalt combo: 2/2 * 0.2 = 0.2
        // Grim combo: 1/2 * 0.2 = 0.1 (Kinnan present, Grim not)
        assert!(
            (bonus - 0.3).abs() < 1e-10,
            "Full + partial = 0.3, got {}",
            bonus
        );
    }

    #[test]
    fn test_apply_combo_effect_adds_mana() {
        let (mut state, registry) = setup_state_with_combos();

        let monolith = state.create_card_in_zone(ids::BASALT_MONOLITH, 0, ZoneType::Battlefield);
        state.create_card_in_zone(ids::KINNAN_BONDER_PRODIGY, 0, ZoneType::Battlefield);

        let initial_colorless = state.players[0].mana_pool.colorless;
        apply_combo_effect(&mut state, 0, &registry.combos[0]);

        assert_eq!(
            state.players[0].mana_pool.colorless,
            initial_colorless + 100,
            "Combo should add 100 colorless mana"
        );
        assert!(
            state.objects[&monolith].tapped,
            "Monolith should be tapped after combo activation"
        );
    }

    #[test]
    fn test_grim_monolith_combo() {
        let (mut state, registry) = setup_state_with_combos();

        let grim = state.create_card_in_zone(ids::GRIM_MONOLITH, 0, ZoneType::Battlefield);
        state.create_card_in_zone(ids::KINNAN_BONDER_PRODIGY, 0, ZoneType::Battlefield);

        let available = detect_available_combos(&state, 0, &registry);
        assert!(
            available.contains(&1),
            "Grim Monolith + Kinnan combo should be available"
        );

        apply_combo_effect(&mut state, 0, &registry.combos[1]);
        assert_eq!(state.players[0].mana_pool.colorless, 100);
        assert!(state.objects[&grim].tapped);
    }
}
