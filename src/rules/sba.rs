use crate::card::{CardDef, CardType, ObjectId, TriggerCondition, ZoneType};
use crate::game::{GameState, PlayerIndex};

/// Find duplicate permanents to remove based on a predicate.
/// Used by both the legendary rule (CR 704.5j) and planeswalker uniqueness
/// rule (CR 704.5i). For each (controller, name) group with >1 match, keeps
/// the newest (highest ObjectId) and returns the rest.
fn find_duplicates_to_remove(
    state: &GameState,
    predicate: impl Fn(&CardDef) -> bool,
) -> Vec<ObjectId> {
    let db = state.card_db();
    let mut name_map: std::collections::HashMap<(usize, String), Vec<ObjectId>> =
        std::collections::HashMap::new();
    for &id in &state.battlefield {
        let inst = &state.objects[&id];
        if let Some(def) = db.get(inst.card_def_id) {
            if predicate(def) {
                name_map
                    .entry((inst.controller, def.name.clone()))
                    .or_default()
                    .push(id);
            }
        }
    }
    let mut to_remove = Vec::new();
    for (_, ids) in name_map {
        if ids.len() > 1 {
            let keep = *ids.iter().max().unwrap();
            for &id in &ids {
                if id != keep {
                    to_remove.push(id);
                }
            }
        }
    }
    to_remove
}

/// Check and apply state-based actions, implementing the CR 704.3 loop.
///
/// The loop structure interleaves SBA checks with trigger checking:
///
/// ```text
/// loop {
///     perform_all_SBAs()         // inner loop until no more SBAs apply
///     check_and_queue_triggers() // queue triggers for events that happened
///     if no_SBAs_performed && no_triggers_queued { break }
///     put_triggers_on_stack()    // may pause for OrderTriggers
/// }
/// // only now grant priority
/// ```
pub fn check_state_based_actions(state: &mut GameState) {
    // Outer CR 704.3 loop: interleave SBA checks with trigger checks
    loop {
        // --- Inner SBA loop: perform all SBAs until stable ---
        let mut died_this_round: Vec<ObjectId> = Vec::new();
        let mut any_sba = false;

        loop {
            let mut any_action = false;

            // CR 704.5a: Player with 0 or less life loses
            for i in 0..state.players.len() {
                if state.players[i].life <= 0 && !state.players[i].has_lost {
                    state.players[i].has_lost = true;
                    any_action = true;
                }
            }

            // CR 903.10a (Commander): Player with 21+ commander damage from
            // a single commander loses the game.
            if state.is_commander_format() {
                for i in 0..state.players.len() {
                    if state.players[i].has_lost {
                        continue;
                    }
                    for dmg in &state.players[i].commander_damage_received {
                        if *dmg >= 21 {
                            state.players[i].has_lost = true;
                            any_action = true;
                            break;
                        }
                    }
                }
            }

            // CR 704.5d: +1/+1 and -1/-1 counter cancellation
            for &obj_id in &state.battlefield.clone() {
                if let Some(inst) = state.objects.get_mut(&obj_id) {
                    if inst.plus_counters > 0 && inst.minus_counters > 0 {
                        let cancel = inst.plus_counters.min(inst.minus_counters);
                        inst.plus_counters -= cancel;
                        inst.minus_counters -= cancel;
                        any_action = true;
                    }
                }
            }
            if any_action {
                state.invalidate_characteristics_cache();
            }

            // CR 704.5j: Legendary rule — if a player controls two or more
            // legendary permanents with the same name, keep the newest.
            let legendary_dupes: Vec<ObjectId> =
                find_duplicates_to_remove(state, |def| {
                    def.supertypes.contains(&crate::card::Supertype::Legendary)
                });
            for &id in &legendary_dupes {
                state.move_object(id, ZoneType::Battlefield, ZoneType::Graveyard);
                any_action = true;
            }
            if !legendary_dupes.is_empty() {
                state.refresh_continuous_effects();
                state.refresh_replacement_effects();
                died_this_round.extend(legendary_dupes);
            }

            // CR 704.5i: Planeswalker uniqueness rule — keep newest per name.
            let pw_dupes: Vec<ObjectId> =
                find_duplicates_to_remove(state, |def| {
                    def.card_types.contains(&CardType::Planeswalker)
                });
            for &id in &pw_dupes {
                state.move_object(id, ZoneType::Battlefield, ZoneType::Graveyard);
                any_action = true;
            }
            if !pw_dupes.is_empty() {
                state.refresh_continuous_effects();
                state.refresh_replacement_effects();
                died_this_round.extend(pw_dupes);
            }

            // CR 704.5f/g: Creature with toughness <= 0 or lethal damage
            let to_die: Vec<ObjectId> = {
                state
                    .battlefield
                    .iter()
                    .copied()
                    .filter(|&id| {
                        if state.is_creature(id) {
                            let toughness = state.effective_toughness(id);
                            let damage = state.objects[&id].damage_marked as i32;
                            if toughness <= 0 {
                                return true;
                            }
                            if damage >= toughness {
                                return true;
                            }
                        }
                        false
                    })
                    .collect()
            };

            for &obj_id in &to_die {
                // Check death replacement effects (CR 614)
                let dest_zone = state.death_replacement_zone(obj_id);
                if dest_zone == ZoneType::Battlefield {
                    // Replacement prevented the death — creature stays
                    continue;
                }
                state.move_object(obj_id, ZoneType::Battlefield, dest_zone);
                any_action = true;
            }
            if !to_die.is_empty() {
                // Refresh continuous effects after permanents leave the battlefield
                state.refresh_continuous_effects();
                state.refresh_replacement_effects();
            }
            died_this_round.extend(to_die);

            // Check for game end
            let losers: Vec<usize> = (0..state.players.len())
                .filter(|&i| state.players[i].has_lost)
                .collect();

            if losers.len() >= state.players.len() - 1 {
                state.game_over = true;
                state.winner = (0..state.players.len())
                    .find(|&i| !state.players[i].has_lost);
            }

            if !any_action {
                break;
            }
            any_sba = true;
        }

        // --- Queue triggers for SBA events ---
        let triggers_before = state.pending_triggers.len();
        for &obj_id in &died_this_round {
            // Check the dying creature's own "when ~ dies" triggers.
            super::triggers::check_triggers(state, TriggerCondition::Dies, Some(obj_id));
        }
        // Check "whenever a creature dies" watcher triggers on surviving permanents.
        if !died_this_round.is_empty() {
            super::triggers::check_triggers(state, TriggerCondition::ACreatureDies, None);
            // Check controller-filtered "whenever a creature you control dies" triggers.
            let dying_controllers: Vec<PlayerIndex> = died_this_round
                .iter()
                .filter_map(|&id| state.objects.get(&id).map(|inst| inst.controller))
                .collect();
            super::triggers::check_your_creature_dies_triggers(state, &dying_controllers);
        }
        let triggers_queued = state.pending_triggers.len() > triggers_before;

        // --- CR 704.3 exit condition ---
        if !any_sba && !triggers_queued {
            break;
        }

        // --- Flush triggers to stack ---
        if !state.pending_triggers.is_empty() {
            let flushed = super::triggers::flush_triggers(state);
            if !flushed {
                // Paused for OrderTriggers — return to game loop
                return;
            }
        }
    }
}
