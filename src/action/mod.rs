use serde::{Deserialize, Serialize};
use std::fmt;

use crate::card::ObjectId;
use crate::game::{GameState, PlayerIndex, Target};

/// An action a player can take when they have priority.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Pass priority.
    PassPriority,

    /// Discard a card from hand (used in cleanup).
    Discard { object_id: ObjectId },

    /// Play a land from hand.
    PlayLand { object_id: ObjectId },

    /// Cast a spell from hand.
    CastSpell {
        object_id: ObjectId,
        targets: Vec<Target>,
    },

    /// Activate a mana ability (doesn't use the stack).
    ActivateManaAbility {
        object_id: ObjectId,
        ability_index: usize,
    },

    /// Activate a non-mana ability.
    ActivateAbility {
        object_id: ObjectId,
        ability_index: usize,
        targets: Vec<Target>,
    },

    /// Declare attackers (a set of creature ObjectIds).
    DeclareAttackers { attackers: Vec<ObjectId> },

    /// Declare blockers (list of (blocker_id, attacker_id) pairs).
    DeclareBlockers {
        blocks: Vec<(ObjectId, ObjectId)>,
    },

    /// Order damage assignment for a blocked attacker.
    OrderDamageAssignment {
        attacker: ObjectId,
        assignment: Vec<(ObjectId, u32)>,
    },

    /// Concede the game.
    Concede,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::PassPriority => write!(f, "Pass"),
            Action::Discard { object_id } => write!(f, "Discard (obj {})", object_id),
            Action::PlayLand { object_id } => write!(f, "Play land (obj {})", object_id),
            Action::CastSpell { object_id, .. } => write!(f, "Cast spell (obj {})", object_id),
            Action::ActivateManaAbility { object_id, .. } => {
                write!(f, "Tap for mana (obj {})", object_id)
            }
            Action::ActivateAbility { object_id, .. } => {
                write!(f, "Activate ability (obj {})", object_id)
            }
            Action::DeclareAttackers { attackers } => {
                write!(f, "Attack with {} creatures", attackers.len())
            }
            Action::DeclareBlockers { blocks } => {
                write!(f, "Block with {} creatures", blocks.len())
            }
            Action::OrderDamageAssignment { .. } => write!(f, "Assign damage"),
            Action::Concede => write!(f, "Concede"),
        }
    }
}

/// Enumerate all legal actions for the player who currently has priority.
pub fn legal_actions(state: &GameState) -> Vec<Action> {
    use crate::game::Phase;

    let player = state.priority_player;
    let mut actions = Vec::new();

    let forced_discard = state.phase == Phase::Cleanup
        && player == state.active_player
        && state.players[player].hand.len() > 7;

    if forced_discard {
        let hand = state.players[player].hand.clone();
        for obj_id in hand {
            actions.push(Action::Discard { object_id: obj_id });
        }
        return actions;
    }

    // Player can always pass priority
    actions.push(Action::PassPriority);

    // Phase-specific action generation
    match state.phase {
        Phase::DeclareAttackers if player == state.active_player => {
            // Enumerate attacker combinations
            let creatures = state.creatures_controlled_by(player);
            let db = state.card_db();
            let eligible: Vec<ObjectId> = creatures
                .into_iter()
                .filter(|&id| {
                    let inst = &state.objects[&id];
                    let def = db.get(inst.card_def_id).unwrap();
                    inst.can_attack(def)
                })
                .collect();

            // Generate all subsets of eligible attackers (including empty = no attack).
            // For large boards, we limit combinations to avoid exponential blowup.
            let subsets = generate_subsets(&eligible, 10); // cap at 10 attackers
            for subset in subsets {
                actions.push(Action::DeclareAttackers { attackers: subset });
            }
        }

        Phase::DeclareBlockers if player != state.active_player => {
            // Enumerate blocking assignments
            let creatures = state.creatures_controlled_by(player);
            let db = state.card_db();
            let eligible_blockers: Vec<ObjectId> = creatures
                .into_iter()
                .filter(|&id| {
                    let inst = &state.objects[&id];
                    !inst.tapped
                        && db.get(inst.card_def_id).is_some()
                })
                .collect();

            let attackers = &state.combat.attackers;
            if attackers.is_empty() {
                // No attackers — just pass
            } else {
                // Generate blocking combinations.
                // Each blocker can block at most one attacker (or not block).
                // For now, enumerate single-blocker assignments and no-block.
                let blocking_combos =
                    generate_blocking_assignments(&eligible_blockers, attackers, state);
                for combo in blocking_combos {
                    actions.push(Action::DeclareBlockers { blocks: combo });
                }
            }
        }

        _ => {
            // Standard priority actions
            let is_main = state.phase.is_main_phase()
                && player == state.active_player
                && state.stack.is_empty();

            let db = state.card_db();
            let hand = &state.players[player].hand;

            for &obj_id in hand {
                let inst = &state.objects[&obj_id];
                let def = match db.get(inst.card_def_id) {
                    Some(d) => d,
                    None => continue,
                };

                // Play lands (sorcery speed, once per turn)
                if def.is_land()
                    && is_main
                    && state.players[player].land_plays_remaining > 0
                {
                    actions.push(Action::PlayLand { object_id: obj_id });
                }

                // Cast spells
                if !def.is_land() {
                    let can_cast_timing = if def.is_instant_speed() {
                        true // instants and flash anytime you have priority
                    } else {
                        is_main // sorcery speed
                    };

                    if can_cast_timing {
                        // Check if player can pay the mana cost
                        if let Some(ref cost) = def.mana_cost {
                            // Check if total available mana (from untapped lands) can cover it.
                            // Simplified: we check the current pool + potential from untapped lands.
                            if can_potentially_pay(state, player, cost) {
                                let targets = enumerate_targets_for_spell(state, player, def);
                                if targets.is_empty() {
                                    actions.push(Action::CastSpell {
                                        object_id: obj_id,
                                        targets: vec![],
                                    });
                                } else {
                                    for target in targets {
                                        actions.push(Action::CastSpell {
                                            object_id: obj_id,
                                            targets: vec![target],
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Activated abilities from permanents
            let permanents = state.permanents_controlled_by(player);
            for &obj_id in &permanents {
                let inst = &state.objects[&obj_id];
                let def = match db.get(inst.card_def_id) {
                    Some(d) => d,
                    None => continue,
                };

                for (i, ability) in def.activated_abilities.iter().enumerate() {
                    if ability.requires_tap && inst.tapped {
                        continue;
                    }
                    if can_potentially_pay(state, player, &ability.cost) {
                        actions.push(Action::ActivateAbility {
                            object_id: obj_id,
                            ability_index: i,
                            targets: vec![], // simplified
                        });
                    }
                }
            }
        }
    }

    actions
}

/// Check if a player can potentially pay a mana cost by tapping untapped lands.
fn can_potentially_pay(
    state: &GameState,
    player: PlayerIndex,
    cost: &crate::mana::ManaCost,
) -> bool {
    use crate::card::ManaAbility;

    // Start with current pool
    let mut pool = state.players[player].mana_pool.clone();

    // Add mana from untapped lands
    let db = state.card_db();
    let untapped = state.untapped_lands(player);
    for &land_id in &untapped {
        let inst = &state.objects[&land_id];
        let def = match db.get(inst.card_def_id) {
            Some(d) => d,
            None => continue,
        };
        // Assume best-case: each land contributes its first ability
        for ma in &def.mana_abilities {
            match ma {
                ManaAbility::TapForColor(color) => {
                    pool.add_color(*color, 1);
                    break; // one mana per land
                }
                ManaAbility::TapForColorless => {
                    pool.colorless += 1;
                    break;
                }
                ManaAbility::TapForAny => {
                    // Optimistically add to whichever color is most needed
                    // For simplicity, add colorless (covers generic)
                    pool.colorless += 1;
                    break;
                }
                ManaAbility::TapForChoice(colors) => {
                    if let Some(&color) = colors.first() {
                        pool.add_color(color, 1);
                    }
                    break;
                }
            }
        }
    }

    pool.can_pay(cost)
}

/// Enumerate valid targets for a spell.
fn enumerate_targets_for_spell(
    state: &GameState,
    caster: PlayerIndex,
    def: &crate::card::CardDef,
) -> Vec<Target> {
    use crate::card::{Effect, TargetSpec};

    let effect = match &def.spell_effect {
        Some(e) => e,
        None => return vec![],
    };

    fn targets_for_spec(state: &GameState, caster: PlayerIndex, spec: &TargetSpec) -> Vec<Target> {
        use crate::card::KeywordAbility;

        let db = state.card_db();
        let mut targets = Vec::new();

        /// Check if a permanent can be targeted by a given player.
        /// Hexproof: can't be targeted by opponents. Shroud: can't be targeted by anyone.
        fn can_target_permanent(
            state: &GameState,
            db: &crate::game::CardDatabase,
            obj_id: crate::card::ObjectId,
            caster: PlayerIndex,
        ) -> bool {
            let inst = &state.objects[&obj_id];
            let def = match db.get(inst.card_def_id) {
                Some(d) => d,
                None => return true,
            };
            if inst.has_keyword(def, KeywordAbility::Shroud) {
                return false;
            }
            if inst.has_keyword(def, KeywordAbility::Hexproof) && inst.controller != caster {
                return false;
            }
            true
        }

        match spec {
            TargetSpec::AnyCreature => {
                for &id in &state.battlefield {
                    let inst = &state.objects[&id];
                    if db.get(inst.card_def_id).map_or(false, |d| d.is_creature())
                        && can_target_permanent(state, db, id, caster)
                    {
                        targets.push(Target::Object(id));
                    }
                }
            }
            TargetSpec::AnyPlayer => {
                for i in 0..state.players.len() {
                    // Hexproof on players (e.g., Leyline of Sanctity) not modeled yet
                    targets.push(Target::Player(i));
                }
            }
            TargetSpec::CreatureOrPlayer => {
                for &id in &state.battlefield {
                    let inst = &state.objects[&id];
                    if db.get(inst.card_def_id).map_or(false, |d| d.is_creature())
                        && can_target_permanent(state, db, id, caster)
                    {
                        targets.push(Target::Object(id));
                    }
                }
                for i in 0..state.players.len() {
                    targets.push(Target::Player(i));
                }
            }
            TargetSpec::Opponent => {
                let opponent = state.opponent(caster);
                targets.push(Target::Player(opponent));
            }
            TargetSpec::AnySpell => {
                for entry in &state.stack {
                    if let crate::game::StackSource::Spell(obj_id) = entry.source {
                        targets.push(Target::Object(obj_id));
                    }
                }
            }
            TargetSpec::AnyNonlandPermanent => {
                for &id in &state.battlefield {
                    let inst = &state.objects[&id];
                    if db.get(inst.card_def_id).map_or(false, |d| !d.is_land())
                        && can_target_permanent(state, db, id, caster)
                    {
                        targets.push(Target::Object(id));
                    }
                }
            }
            TargetSpec::NoTarget | TargetSpec::Controller => {}
            TargetSpec::AnyPermanent => {
                for &id in &state.battlefield {
                    if can_target_permanent(state, db, id, caster) {
                        targets.push(Target::Object(id));
                    }
                }
            }
            TargetSpec::CreatureOrPlaneswalker => {
                for &id in &state.battlefield {
                    let inst = &state.objects[&id];
                    if let Some(d) = db.get(inst.card_def_id) {
                        if (d.is_creature()
                            || d.card_types.contains(&crate::card::CardType::Planeswalker))
                            && can_target_permanent(state, db, id, caster)
                        {
                            targets.push(Target::Object(id));
                        }
                    }
                }
            }
        }
        targets
    }

    match effect {
        Effect::DealDamage { target, .. } => targets_for_spec(state, caster, target),
        Effect::DestroyTarget { target } => targets_for_spec(state, caster, target),
        Effect::BounceTo { target, .. } => targets_for_spec(state, caster, target),
        Effect::Counter { target } => targets_for_spec(state, caster, target),
        Effect::LoseLife { target, .. } => targets_for_spec(state, caster, target),
        Effect::DiscardCards { target, .. } => targets_for_spec(state, caster, target),
        _ => vec![], // non-targeted spells
    }
}

/// Generate subsets of up to `max_size` elements from `items`.
fn generate_subsets(items: &[ObjectId], max_items: usize) -> Vec<Vec<ObjectId>> {
    let items = if items.len() > max_items {
        &items[..max_items]
    } else {
        items
    };

    let n = items.len();
    let mut subsets = Vec::with_capacity(1 << n);

    for mask in 0..(1u32 << n) {
        let mut subset = Vec::new();
        for (i, &item) in items.iter().enumerate() {
            if mask & (1 << i) != 0 {
                subset.push(item);
            }
        }
        subsets.push(subset);
    }
    subsets
}

/// Check if a specific blocker can legally block a specific attacker.
fn can_block(
    state: &GameState,
    blocker_id: ObjectId,
    attacker_id: ObjectId,
) -> bool {
    use crate::card::KeywordAbility;

    let db = state.card_db();
    let blocker_inst = &state.objects[&blocker_id];
    let blocker_def = match db.get(blocker_inst.card_def_id) {
        Some(d) => d,
        None => return false,
    };
    let attacker_inst = &state.objects[&attacker_id];
    let attacker_def = match db.get(attacker_inst.card_def_id) {
        Some(d) => d,
        None => return false,
    };

    // Flying: only flying/reach creatures can block flyers
    if attacker_inst.has_keyword(attacker_def, KeywordAbility::Flying)
        && !blocker_inst.has_keyword(blocker_def, KeywordAbility::Flying)
        && !blocker_inst.has_keyword(blocker_def, KeywordAbility::Reach)
    {
        return false;
    }

    // Fear: can only be blocked by artifact creatures or black creatures
    if attacker_inst.has_keyword(attacker_def, KeywordAbility::Fear) {
        let is_artifact = blocker_def.card_types.contains(&crate::card::CardType::Artifact);
        let is_black = blocker_def.color_identity().contains(&crate::mana::Color::Black);
        if !is_artifact && !is_black {
            return false;
        }
    }

    // Intimidate: can only be blocked by artifact creatures or creatures sharing a color
    if attacker_inst.has_keyword(attacker_def, KeywordAbility::Intimidate) {
        let is_artifact = blocker_def.card_types.contains(&crate::card::CardType::Artifact);
        let attacker_colors = attacker_def.color_identity();
        let blocker_colors = blocker_def.color_identity();
        let shares_color = attacker_colors.iter().any(|c| blocker_colors.contains(c));
        if !is_artifact && !shares_color {
            return false;
        }
    }

    true
}

/// Generate legal blocking assignments.
/// Each eligible blocker can block one attacker or not block at all.
/// Handles menace (requires 2+ blockers) and other blocking restrictions.
fn generate_blocking_assignments(
    blockers: &[ObjectId],
    attackers: &[ObjectId],
    state: &GameState,
) -> Vec<Vec<(ObjectId, ObjectId)>> {
    use crate::card::KeywordAbility;

    let mut assignments = Vec::new();

    // Always include "no blocks"
    assignments.push(vec![]);

    if blockers.is_empty() || attackers.is_empty() {
        return assignments;
    }

    let db = state.card_db();

    // Determine which attackers have menace
    let menace_attackers: Vec<bool> = attackers
        .iter()
        .map(|&id| {
            let inst = &state.objects[&id];
            let def = db.get(inst.card_def_id).unwrap();
            inst.has_keyword(def, KeywordAbility::Menace)
        })
        .collect();

    // Build a legal-block matrix: which blocker can block which attacker
    let can_block_matrix: Vec<Vec<bool>> = blockers
        .iter()
        .map(|&b| {
            attackers.iter().map(|&a| can_block(state, b, a)).collect()
        })
        .collect();

    // Generate single-blocker assignments (only for non-menace attackers)
    for (bi, &blocker) in blockers.iter().enumerate() {
        for (ai, &attacker) in attackers.iter().enumerate() {
            if !can_block_matrix[bi][ai] {
                continue;
            }
            // Menace: can't be single-blocked
            if menace_attackers[ai] {
                continue;
            }
            assignments.push(vec![(blocker, attacker)]);
        }
    }

    // Generate multi-blocker assignments (important for menace and gang-blocking)
    // For each attacker, generate combinations of 2+ blockers that can each block it
    // Limit to 6 eligible blockers per attacker to keep combinatorics manageable
    for (ai, &attacker) in attackers.iter().enumerate() {
        let eligible: Vec<ObjectId> = blockers
            .iter()
            .enumerate()
            .filter(|&(bi, _)| can_block_matrix[bi][ai])
            .map(|(_, &b)| b)
            .take(6)
            .collect();

        let min_blockers = if menace_attackers[ai] { 2 } else { 2 };
        let max_blockers = eligible.len().min(4); // cap at 4 blockers per attacker

        for size in min_blockers..=max_blockers {
            for combo in combinations(&eligible, size) {
                let blocks: Vec<(ObjectId, ObjectId)> =
                    combo.into_iter().map(|b| (b, attacker)).collect();
                assignments.push(blocks);
            }
        }
    }

    // Also allow mixed blocking: one blocker on each of two different attackers
    // (important when facing multiple attackers)
    if attackers.len() >= 2 && blockers.len() >= 2 {
        for (bi1, &b1) in blockers.iter().enumerate() {
            for (ai1, &a1) in attackers.iter().enumerate() {
                if !can_block_matrix[bi1][ai1] || menace_attackers[ai1] {
                    continue;
                }
                for (bi2, &b2) in blockers.iter().enumerate() {
                    if bi2 <= bi1 {
                        continue; // avoid duplicate pairs
                    }
                    for (ai2, &a2) in attackers.iter().enumerate() {
                        if ai2 == ai1 {
                            continue; // already covered by multi-blocker above
                        }
                        if !can_block_matrix[bi2][ai2] || menace_attackers[ai2] {
                            continue;
                        }
                        assignments.push(vec![(b1, a1), (b2, a2)]);
                    }
                }
            }
        }
    }

    assignments
}

/// Generate all combinations of `k` items from `items`.
fn combinations(items: &[ObjectId], k: usize) -> Vec<Vec<ObjectId>> {
    let mut result = Vec::new();
    let mut combo = Vec::with_capacity(k);
    combinations_helper(items, k, 0, &mut combo, &mut result);
    result
}

fn combinations_helper(
    items: &[ObjectId],
    k: usize,
    start: usize,
    current: &mut Vec<ObjectId>,
    result: &mut Vec<Vec<ObjectId>>,
) {
    if current.len() == k {
        result.push(current.clone());
        return;
    }
    for i in start..items.len() {
        current.push(items[i]);
        combinations_helper(items, k, i + 1, current, result);
        current.pop();
    }
}
