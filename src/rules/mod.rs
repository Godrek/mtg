use rand::seq::SliceRandom;
use rand::Rng;

use crate::action::Action;
use crate::card::{CardType, Effect, KeywordAbility, ManaAbility, ObjectId, TriggerCondition, ZoneType};
use crate::game::{GameState, PendingTrigger, Phase, PlayerIndex, StackEntry, StackSource, Target};

/// Apply an action to the game state, advancing it.
pub fn apply_action(state: &mut GameState, action: &Action) {
    match action {
        Action::PassPriority => {
            if state.phase == Phase::Cleanup
                && state.players[state.active_player].hand.len() > 7
            {
                debug_assert!(
                    false,
                    "PassPriority during cleanup discard is illegal; choose a Discard action."
                );
                return;
            }
            state.consecutive_passes += 1;
            handle_priority_pass(state);
        }

        Action::Discard { object_id } => {
            if state.phase != Phase::Cleanup {
                return;
            }
            if state.priority_player != state.active_player {
                return;
            }
            let player = state.active_player;
            if !state.players[player].hand.contains(object_id) {
                return;
            }
            if state.players[player].hand.len() <= 7 {
                return;
            }

            state.move_object(*object_id, ZoneType::Hand, ZoneType::Graveyard);
            state.consecutive_passes = 0;
            state.priority_player = player;

            if state.players[player].hand.len() <= 7 {
                finalize_cleanup(state);
            }
            // TODO: if discard triggers exist, start another cleanup step (CR 514.3a).
        }

        Action::PlayLand { object_id } => {
            let obj_id = *object_id;
            state.players[state.priority_player].land_plays_remaining -= 1;
            state.move_object(obj_id, ZoneType::Hand, ZoneType::Battlefield);
            // Lands enter untapped by default (we'd check for "enters tapped" later)
            if let Some(inst) = state.objects.get_mut(&obj_id) {
                inst.tapped = false;
                inst.summoning_sick = false; // lands don't have summoning sickness
            }
            state.consecutive_passes = 0;
        }

        Action::CastSpell { object_id, targets } => {
            let obj_id = *object_id;
            let player = state.priority_player;
            let db = state.card_db();
            let inst = &state.objects[&obj_id];
            let def = db.get(inst.card_def_id).unwrap().clone();

            // Pay mana cost
            if let Some(ref cost) = def.mana_cost {
                // First, auto-tap lands to generate mana if pool is insufficient
                auto_tap_lands(state, player, cost);
                // Then pay from pool
                state.players[player].mana_pool.pay(cost);
            }

            // Move to stack
            let stack_id = state.new_stack_id();
            state.stack.push(StackEntry {
                id: stack_id,
                source: StackSource::Spell(obj_id),
                controller: player,
                targets: targets.clone(),
            });
            // Remove from hand (but don't put in a zone yet — it's on the stack)
            state.players[player].hand.retain(|&id| id != obj_id);

            state.consecutive_passes = 0;
        }

        Action::ActivateManaAbility {
            object_id,
            ability_index,
        } => {
            let obj_id = *object_id;
            let idx = *ability_index;
            let player = state.priority_player;

            // Clone the mana ability to avoid borrow conflict
            let ma = {
                let db = state.card_db();
                let inst = &state.objects[&obj_id];
                let def = db.get(inst.card_def_id).unwrap();
                def.mana_abilities.get(idx).cloned()
            };

            if let Some(ma) = ma {
                match &ma {
                    ManaAbility::TapForColor(color) => {
                        state.players[player].mana_pool.add_color(*color, 1);
                    }
                    ManaAbility::TapForColorless => {
                        state.players[player].mana_pool.colorless += 1;
                    }
                    ManaAbility::TapForAny => {
                        state.players[player].mana_pool.colorless += 1;
                    }
                    ManaAbility::TapForChoice(colors) => {
                        if let Some(&color) = colors.first() {
                            state.players[player].mana_pool.add_color(color, 1);
                        }
                    }
                }
            }

            // Tap the permanent
            if let Some(inst) = state.objects.get_mut(&obj_id) {
                inst.tapped = true;
            }
        }

        Action::ActivateAbility {
            object_id,
            ability_index,
            targets,
        } => {
            let obj_id = *object_id;
            let idx = *ability_index;
            let player = state.priority_player;

            // Clone ability info to avoid borrow conflict
            let ability = {
                let db = state.card_db();
                let inst = &state.objects[&obj_id];
                let def = db.get(inst.card_def_id).unwrap();
                def.activated_abilities.get(idx).cloned()
            };

            if let Some(ability) = ability {
                auto_tap_lands(state, player, &ability.cost);
                state.players[player].mana_pool.pay(&ability.cost);

                if ability.requires_tap {
                    if let Some(inst) = state.objects.get_mut(&obj_id) {
                        inst.tapped = true;
                    }
                }

                let stack_id = state.new_stack_id();
                state.stack.push(StackEntry {
                    id: stack_id,
                    source: StackSource::ActivatedAbility {
                        source_id: obj_id,
                        ability_index: idx,
                    },
                    controller: player,
                    targets: targets.clone(),
                });
            }
            state.consecutive_passes = 0;
        }

        Action::DeclareAttackers { attackers } => {
            state.combat.clear();
            state.combat.attackers = attackers.clone();

            // Collect which attackers need tapping (those without vigilance)
            let to_tap: Vec<ObjectId> = {
                let db = state.card_db();
                attackers
                    .iter()
                    .filter(|&&id| {
                        let inst = &state.objects[&id];
                        let def = db.get(inst.card_def_id).unwrap();
                        !inst.has_keyword(def, KeywordAbility::Vigilance)
                    })
                    .copied()
                    .collect()
            };
            for id in to_tap {
                if let Some(inst) = state.objects.get_mut(&id) {
                    inst.tapped = true;
                }
            }

            state.consecutive_passes = 0;
            if attackers.is_empty() {
                // No attackers — skip combat entirely
                state.phase = Phase::EndOfCombat;
                state.priority_player = state.active_player;
            } else {
                // Batch-check attack triggers for all attackers before flushing,
                // so the player gets a single ordering decision for all simultaneous triggers.
                for &attacker_id in attackers {
                    check_triggers(state, TriggerCondition::Attacks, Some(attacker_id));
                }
                let flushed = flush_triggers(state);

                if flushed {
                    // All triggers flushed (0-1 per player) — advance normally
                    state.phase = Phase::DeclareBlockers;
                    state.priority_player = state.opponent(state.active_player);
                }
                // else: triggers need ordering — priority_player already set by
                // flush_triggers. Phase stays DeclareAttackers; legal_actions will
                // offer OrderTriggers. After ordering completes, advance_phase will
                // naturally move to DeclareBlockers.
            }
        }

        Action::DeclareBlockers { blocks } => {
            state.combat.blockers.clear();
            state.combat.attacker_blockers.clear();

            for &(blocker, attacker) in blocks {
                state.combat.blockers.insert(blocker, attacker);
                state
                    .combat
                    .attacker_blockers
                    .entry(attacker)
                    .or_default()
                    .push(blocker);
            }

            // Advance to combat damage (skip first strike if not applicable)
            state.consecutive_passes = 0;
            state.phase = Phase::FirstStrikeDamage;
            // Execute the first strike damage step entry (which may skip to CombatDamage)
            execute_phase_entry_public(state);
        }

        Action::OrderDamageAssignment {
            attacker,
            assignment,
        } => {
            state
                .combat
                .damage_assignment
                .insert(*attacker, assignment.clone());
            state.consecutive_passes = 0;
        }

        Action::OrderTriggers { ordering } => {
            let player = state.priority_player;

            // Place this player's triggers on the stack in the chosen order.
            // The first element goes on the stack first (resolves last due to LIFO).
            for &(source_id, ability_index) in ordering {
                if let Some(pos) = state.pending_triggers.iter().position(|t| {
                    t.controller == player
                        && t.source_id == source_id
                        && t.ability_index == ability_index
                }) {
                    let trigger = state.pending_triggers.remove(pos);
                    push_trigger_to_stack(state, &trigger);
                }
            }

            // Continue flushing remaining triggers (the other player's).
            // This may auto-push them or pause again if that player also has >1.
            // Return value intentionally unused: if the other player also needs to
            // order, the game loop will present OrderTriggers on the next iteration.
            let _ = flush_triggers(state);

            // Don't reset consecutive_passes — this is a pre-priority ordering
            // decision, not a normal game action.
        }

        Action::Concede => {
            let player = state.priority_player;
            state.players[player].has_lost = true;
            state.game_over = true;
            state.winner = Some(state.opponent(player));
        }
    }
}

/// Handle when priority is passed (may resolve stack or advance phase).
fn handle_priority_pass(state: &mut GameState) {
    let num_players = state.players.len() as u32;

    if state.consecutive_passes >= num_players {
        // All players passed in succession
        state.consecutive_passes = 0;

        if !state.stack.is_empty() {
            // Resolve top of stack
            resolve_top_of_stack(state);
        } else {
            // Advance to next phase
            advance_phase(state);
        }
    } else {
        // Pass priority to the next player
        state.priority_player = state.opponent(state.priority_player);
    }
}

/// Resolve the top entry on the stack.
fn resolve_top_of_stack(state: &mut GameState) {
    let entry = match state.stack.pop() {
        Some(e) => e,
        None => return,
    };

    match entry.source {
        StackSource::Spell(obj_id) => {
            resolve_spell(state, obj_id, &entry.targets, entry.controller);
        }
        StackSource::ActivatedAbility {
            source_id,
            ability_index,
        } => {
            resolve_activated_ability(state, source_id, ability_index, &entry.targets);
        }
        StackSource::TriggeredAbility {
            source_id,
            ability_index,
        } => {
            resolve_triggered_ability(state, source_id, ability_index, &entry.targets);
        }
    }

    // Check state-based actions after resolution
    check_state_based_actions(state);
}

/// Resolve a spell.
fn resolve_spell(
    state: &mut GameState,
    obj_id: ObjectId,
    targets: &[Target],
    controller: PlayerIndex,
) {
    // Clone the card definition to avoid borrow conflict
    let def = {
        let db = state.card_db();
        let inst = &state.objects[&obj_id];
        db.get(inst.card_def_id).unwrap().clone()
    };

    if def.is_creature() || def.card_types.contains(&CardType::Artifact)
        || def.card_types.contains(&CardType::Enchantment)
        || def.card_types.contains(&CardType::Planeswalker)
    {
        state.move_object(obj_id, ZoneType::Stack, ZoneType::Battlefield);
        if let Some(inst) = state.objects.get_mut(&obj_id) {
            inst.controller = controller;
        }

        // Queue ETB triggered abilities (they go on the stack, not resolve immediately).
        // If flush pauses (controller has >1 ETB trigger), pending_triggers stays
        // populated and the game loop will offer OrderTriggers.
        let _ = fire_triggers(state, TriggerCondition::EntersBattlefield, Some(obj_id));
    } else {
        if let Some(ref effect) = def.spell_effect {
            resolve_effect(state, effect, controller, targets);
        }
        state.move_object(obj_id, ZoneType::Stack, ZoneType::Graveyard);
    }
}

/// Resolve an effect.
fn resolve_effect(
    state: &mut GameState,
    effect: &Effect,
    controller: PlayerIndex,
    targets: &[Target],
) {
    match effect {
        Effect::DealDamage { amount, .. } => {
            for target in targets {
                match target {
                    Target::Player(p) => {
                        state.players[*p].life -= *amount as i32;
                    }
                    Target::Object(id) => {
                        if let Some(inst) = state.objects.get_mut(id) {
                            inst.damage_marked += amount;
                        }
                    }
                }
            }
        }

        Effect::GainLife { amount } => {
            state.players[controller].life += *amount as i32;
        }

        Effect::LoseLife { amount, .. } => {
            for target in targets {
                if let Target::Player(p) = target {
                    state.players[*p].life -= *amount as i32;
                }
            }
        }

        Effect::DrawCards { count } => {
            draw_cards(state, controller, *count as usize);
        }

        Effect::DestroyTarget { .. } => {
            // Collect which objects are destroyable (read phase)
            let destroyable: Vec<ObjectId> = targets
                .iter()
                .filter_map(|target| {
                    if let Target::Object(id) = target {
                        let indestructible = {
                            let db = state.card_db();
                            let inst = &state.objects[id];
                            db.get(inst.card_def_id)
                                .map(|d| inst.has_keyword(d, KeywordAbility::Indestructible))
                                .unwrap_or(false)
                        };
                        if !indestructible { Some(*id) } else { None }
                    } else {
                        None
                    }
                })
                .collect();

            for &id in &destroyable {
                state.move_object(id, ZoneType::Battlefield, ZoneType::Graveyard);
            }
            // Fire dies triggers for destroyed creatures.
            // Batch-check all death triggers before flushing so the controller
            // gets a single ordering decision for simultaneous "when ~ dies" triggers.
            for &id in &destroyable {
                check_triggers(state, TriggerCondition::Dies, Some(id));
                check_triggers(state, TriggerCondition::Dies, None);
            }
            if !destroyable.is_empty() {
                // If flush pauses (player has >1 trigger), pending_triggers will
                // remain populated and legal_actions() will offer OrderTriggers
                // on the next game loop iteration.
                let _ = flush_triggers(state);
            }
        }

        Effect::BounceTo { zone, .. } => {
            for target in targets {
                if let Target::Object(id) = target {
                    state.move_object(*id, ZoneType::Battlefield, *zone);
                }
            }
        }

        Effect::Buff {
            power,
            toughness,
            until_eot,
        } => {
            for target in targets {
                if let Target::Object(id) = target {
                    if let Some(inst) = state.objects.get_mut(id) {
                        if *until_eot {
                            inst.temp_power_mod += power;
                            inst.temp_toughness_mod += toughness;
                        } else {
                            // Permanent buff via +1/+1 counters (simplified: treats any permanent buff as counters)
                            // In MTG, +1/+1 counters give both +1/+1, so we use min(power, toughness) counters
                            // plus temp mods for any asymmetric remainder
                            let counters = (*power).min(*toughness);
                            inst.plus_counters += counters;
                            // Any asymmetric remainder goes as a static modifier
                            // (simplified — real MTG doesn't have this, but handles it per-card)
                            if *power != *toughness {
                                inst.temp_power_mod += power - counters;
                                inst.temp_toughness_mod += toughness - counters;
                            }
                        }
                    }
                }
            }
        }

        Effect::DiscardCards { count, .. } => {
            for target in targets {
                if let Target::Player(p) = target {
                    discard_random(state, *p, *count as usize);
                }
            }
        }

        Effect::CreateToken(_token_def) => {
            // Create a token on the battlefield
            // Tokens need a synthetic CardDef — simplified for now
            // TODO: proper token creation
        }

        Effect::Counter { .. } => {
            // Find the targeted spell on the stack and counter it
            let target_obj_id = targets.iter().find_map(|t| {
                if let Target::Object(id) = t { Some(*id) } else { None }
            });

            if let Some(target_id) = target_obj_id {
                // Find and remove the targeted spell from the stack
                if let Some(idx) = state.stack.iter().position(|entry| {
                    matches!(&entry.source, StackSource::Spell(id) if *id == target_id)
                }) {
                    let countered = state.stack.remove(idx);
                    if let StackSource::Spell(obj_id) = countered.source {
                        state.move_object(obj_id, ZoneType::Stack, ZoneType::Graveyard);
                    }
                }
            } else {
                // Fallback: counter top spell on stack if no target specified
                if let Some(countered) = state.stack.pop() {
                    if let StackSource::Spell(obj_id) = countered.source {
                        state.move_object(obj_id, ZoneType::Stack, ZoneType::Graveyard);
                    }
                }
            }
        }

        Effect::Multiple(effects) => {
            for e in effects {
                resolve_effect(state, e, controller, targets);
            }
        }

        Effect::Unimplemented(_) => {
            // Can't resolve unimplemented effects
        }
    }
}

// ========================================================================
// Trigger System
// ========================================================================

/// Check all permanents on the battlefield for triggered abilities matching
/// the given condition, and queue any that trigger.
fn check_triggers(state: &mut GameState, condition: TriggerCondition, source_hint: Option<ObjectId>) {
    let triggers: Vec<PendingTrigger> = {
        let db = state.card_db();
        let mut found = Vec::new();

        // Determine which objects to check
        let objects_to_check: Vec<(ObjectId, usize)> = match source_hint {
            // If a specific source is given (e.g., ETB on a specific permanent), only check it
            Some(id) => {
                if let Some(inst) = state.objects.get(&id) {
                    vec![(id, inst.controller)]
                } else {
                    vec![]
                }
            }
            // Otherwise check all permanents on the battlefield
            None => state
                .battlefield
                .iter()
                .map(|&id| (id, state.objects[&id].controller))
                .collect(),
        };

        for (obj_id, controller) in objects_to_check {
            let inst = &state.objects[&obj_id];
            let def = match db.get(inst.card_def_id) {
                Some(d) => d,
                None => continue,
            };

            for (i, trigger) in def.triggered_abilities.iter().enumerate() {
                if trigger.trigger == condition {
                    found.push(PendingTrigger {
                        source_id: obj_id,
                        ability_index: i,
                        controller,
                        targets: vec![], // targets chosen when put on stack (simplified: auto-target)
                    });
                }
            }
        }
        found
    };

    state.pending_triggers.extend(triggers);
}

/// Flush pending triggers onto the stack in APNAP order
/// (Active Player, Non-Active Player). When a player controls multiple
/// simultaneous triggers, they must choose the ordering — this is surfaced
/// as an `Action::OrderTriggers` decision point for MCCFR to observe.
///
/// If a player has >1 trigger, this function pauses (leaves triggers in
/// `pending_triggers` and sets `priority_player`) so the game loop can
/// present the ordering choice. Returns `true` if all triggers were flushed,
/// `false` if paused waiting for a player's ordering decision.
#[must_use]
fn flush_triggers(state: &mut GameState) -> bool {
    if state.pending_triggers.is_empty() {
        return true;
    }

    let active = state.active_player;

    // Count each player's pending triggers
    let ap_count = state
        .pending_triggers
        .iter()
        .filter(|t| t.controller == active)
        .count();
    let nap_count = state
        .pending_triggers
        .iter()
        .filter(|t| t.controller != active)
        .count();

    // APNAP: handle active player's triggers first
    if ap_count > 1 {
        // AP has multiple triggers — pause for ordering decision
        state.priority_player = active;
        return false;
    }

    // AP has 0-1 triggers: auto-push them to the stack
    let ap_triggers: Vec<PendingTrigger> = state
        .pending_triggers
        .iter()
        .filter(|t| t.controller == active)
        .cloned()
        .collect();
    for trigger in ap_triggers {
        push_trigger_to_stack(state, &trigger);
    }
    state.pending_triggers.retain(|t| t.controller != active);

    // Now handle non-active player's triggers
    if nap_count > 1 {
        // NAP has multiple triggers — pause for ordering decision
        let nap = state.opponent(active);
        state.priority_player = nap;
        return false;
    }

    // NAP has 0-1 triggers: auto-push them
    let nap_triggers = std::mem::take(&mut state.pending_triggers);
    for trigger in nap_triggers {
        push_trigger_to_stack(state, &trigger);
    }

    true
}

/// Push a single trigger onto the stack as a TriggeredAbility entry.
fn push_trigger_to_stack(state: &mut GameState, trigger: &PendingTrigger) {
    let stack_id = state.new_stack_id();
    state.stack.push(StackEntry {
        id: stack_id,
        source: StackSource::TriggeredAbility {
            source_id: trigger.source_id,
            ability_index: trigger.ability_index,
        },
        controller: trigger.controller,
        targets: trigger.targets.clone(),
    });
}

/// Check triggers for a specific game event and flush them to the stack.
/// Returns `true` if all triggers were flushed, `false` if paused waiting
/// for a player's ordering decision (i.e. pending_triggers is non-empty).
#[must_use]
pub fn fire_triggers(state: &mut GameState, condition: TriggerCondition, source_hint: Option<ObjectId>) -> bool {
    check_triggers(state, condition, source_hint);
    flush_triggers(state)
}

// ========================================================================
// Ability resolution
// ========================================================================

/// Resolve an activated ability.
fn resolve_activated_ability(
    state: &mut GameState,
    source_id: ObjectId,
    ability_index: usize,
    targets: &[Target],
) {
    let (effect, controller) = {
        let db = state.card_db();
        let inst = &state.objects[&source_id];
        let def = db.get(inst.card_def_id).unwrap();
        let effect = def.activated_abilities.get(ability_index).map(|a| a.effect.clone());
        (effect, inst.controller)
    };

    if let Some(effect) = effect {
        resolve_effect(state, &effect, controller, targets);
    }
}

/// Resolve a triggered ability.
fn resolve_triggered_ability(
    state: &mut GameState,
    source_id: ObjectId,
    ability_index: usize,
    targets: &[Target],
) {
    let info = {
        let db = state.card_db();
        state.objects.get(&source_id).and_then(|inst| {
            let def = db.get(inst.card_def_id)?;
            let effect = def.triggered_abilities.get(ability_index).map(|a| a.effect.clone())?;
            Some((effect, inst.controller))
        })
    };

    if let Some((effect, controller)) = info {
        resolve_effect(state, &effect, controller, targets);
    }
}

/// Check and apply state-based actions.
pub fn check_state_based_actions(state: &mut GameState) {
    let mut all_died: Vec<ObjectId> = Vec::new();

    loop {
        let mut any_action = false;

        // Check player life totals
        for i in 0..state.players.len() {
            if state.players[i].life <= 0 && !state.players[i].has_lost {
                state.players[i].has_lost = true;
                any_action = true;
            }
        }

        // Check creature toughness <= 0 or lethal damage
        let to_die: Vec<ObjectId> = {
            let db = state.card_db();
            state
                .battlefield
                .iter()
                .copied()
                .filter(|&id| {
                    let inst = &state.objects[&id];
                    if let Some(def) = db.get(inst.card_def_id) {
                        if def.is_creature() {
                            let toughness = inst.effective_toughness(def);
                            if toughness <= 0 {
                                return true;
                            }
                            if inst.damage_marked as i32 >= toughness {
                                return true;
                            }
                        }
                    }
                    false
                })
                .collect()
        };

        for &obj_id in &to_die {
            state.move_object(obj_id, ZoneType::Battlefield, ZoneType::Graveyard);
            any_action = true;
        }
        all_died.extend(to_die);

        // Check for game end
        let losers: Vec<usize> = (0..state.players.len())
            .filter(|&i| state.players[i].has_lost)
            .collect();

        if losers.len() >= state.players.len() - 1 {
            state.game_over = true;
            // Find the winner (the player who hasn't lost)
            state.winner = (0..state.players.len())
                .find(|&i| !state.players[i].has_lost);
        }

        // Check library empty (lose when trying to draw, not SBA — but we simplify)

        if !any_action {
            break;
        }
    }

    // Fire dies triggers after all SBA are resolved.
    // Batch-check all death triggers before a single flush so the controller
    // gets a combined ordering decision for simultaneous death triggers.
    for &obj_id in &all_died {
        // Check the dying creature's own "when ~ dies" triggers
        check_triggers(state, TriggerCondition::Dies, Some(obj_id));
        // Check battlefield permanents that watch for creature deaths
        check_triggers(state, TriggerCondition::Dies, None);
    }
    if !all_died.is_empty() {
        // If flush pauses (player has >1 trigger), pending_triggers will
        // remain populated and legal_actions() will offer OrderTriggers
        // on the next game loop iteration.
        let _ = flush_triggers(state);
    }
}

/// Advance to the next phase.
fn advance_phase(state: &mut GameState) {
    let current_idx = Phase::TURN_ORDER
        .iter()
        .position(|&p| p == state.phase)
        .unwrap_or(0);

    if current_idx + 1 < Phase::TURN_ORDER.len() {
        state.phase = Phase::TURN_ORDER[current_idx + 1];
    } else {
        // End of turn — go to next turn
        next_turn(state);
        return;
    }

    // Execute phase entry actions
    execute_phase_entry(state);
}

/// Public wrapper for phase entry execution.
fn execute_phase_entry_public(state: &mut GameState) {
    execute_phase_entry(state);
}

/// Execute actions when entering a new phase.
fn execute_phase_entry(state: &mut GameState) {
    let active = state.active_player;

    match state.phase {
        Phase::Untap => {
            // Untap all permanents controlled by active player
            let permanents = state.permanents_controlled_by(active);
            for obj_id in permanents {
                if let Some(inst) = state.objects.get_mut(&obj_id) {
                    inst.tapped = false;
                    inst.summoning_sick = false;
                }
            }
            // Reset land plays
            state.players[active].land_plays_remaining = 1;
            // Drain mana pools
            for p in &mut state.players {
                p.mana_pool.drain();
            }
            // No priority in untap step — advance immediately
            advance_phase(state);
        }

        Phase::Draw => {
            // Active player draws a card (skip on turn 1 for first player in standard rules)
            if !(state.turn_number == 1 && active == 0) {
                draw_cards(state, active, 1);
            }
            // Priority is given after draw
            state.priority_player = active;
        }

        Phase::Upkeep => {
            // Fire beginning-of-upkeep triggers. Set priority before firing so that
            // if flush_triggers pauses, the priority_player it sets takes precedence.
            // If flush completes, AP keeps priority for normal upkeep actions.
            state.priority_player = active;
            let flushed = fire_triggers(state, TriggerCondition::BeginningOfUpkeep, None);
            if flushed {
                // All triggers auto-flushed; AP keeps priority
                state.priority_player = active;
            }
            // else: flush_triggers set priority_player to whoever needs to order
        }

        Phase::PreCombatMain | Phase::PostCombatMain => {
            state.priority_player = active;
        }

        Phase::BeginningOfCombat => {
            state.priority_player = active;
            state.combat.clear();
        }

        Phase::DeclareAttackers => {
            state.priority_player = active;
        }

        Phase::DeclareBlockers => {
            // Defending player gets priority to declare blockers
            state.priority_player = state.opponent(active);
        }

        Phase::FirstStrikeDamage => {
            // Check if any attacker or blocker has first strike / double strike
            let has_first_strike = has_first_strike_creatures(state);
            if has_first_strike {
                resolve_combat_damage(state, true);
                check_state_based_actions(state);
                state.priority_player = active;
            } else {
                // Skip first strike damage step
                advance_phase(state);
            }
        }

        Phase::CombatDamage => {
            resolve_combat_damage(state, false);
            check_state_based_actions(state);
            state.priority_player = active;
        }

        Phase::EndOfCombat => {
            state.combat.clear();
            state.priority_player = active;
        }

        Phase::EndStep => {
            // Fire end-of-turn triggers. Same pattern as Upkeep:
            // set priority before firing, re-set after if flush completes.
            state.priority_player = active;
            let flushed = fire_triggers(state, TriggerCondition::EndOfTurn, None);
            if flushed {
                state.priority_player = active;
            }
            // else: flush_triggers set priority_player to whoever needs to order
        }

        Phase::Cleanup => {
            // Discard down to max hand size (7)
            let hand_size = state.players[active].hand.len();
            if hand_size > 7 {
                state.priority_player = active;
                state.consecutive_passes = 0;
                return;
            }
            finalize_cleanup(state);
        }
    }
}

fn finalize_cleanup(state: &mut GameState) {
    // Remove EoT effects
    for &obj_id in &state.battlefield.clone() {
        if let Some(inst) = state.objects.get_mut(&obj_id) {
            inst.cleanup_eot();
        }
    }
    // Advance to next turn (no priority in cleanup normally)
    advance_phase(state);
}

/// Move to the next turn.
fn next_turn(state: &mut GameState) {
    state.active_player = state.opponent(state.active_player);
    state.priority_player = state.active_player;
    state.turn_number += 1;
    state.phase = Phase::TURN_ORDER[0]; // Untap
    state.consecutive_passes = 0;

    // Drain mana pools
    for p in &mut state.players {
        p.mana_pool.drain();
    }

    execute_phase_entry(state);
}

/// Draw cards for a player.
pub fn draw_cards(state: &mut GameState, player: PlayerIndex, count: usize) {
    for _ in 0..count {
        if state.players[player].library.is_empty() {
            // Player loses for drawing from empty library
            state.players[player].has_lost = true;
            return;
        }
        let card_id = state.players[player].library.remove(0);
        state.players[player].hand.push(card_id);
    }
}

/// Discard random cards from a player's hand.
fn discard_random(state: &mut GameState, player: PlayerIndex, count: usize) {
    let mut rng = rand::thread_rng();
    for _ in 0..count {
        if state.players[player].hand.is_empty() {
            break;
        }
        let idx = rng.gen_range(0..state.players[player].hand.len());
        let obj_id = state.players[player].hand.remove(idx);
        state.players[player].graveyard.push(obj_id);
    }
}

/// Check if any creature in combat has first strike or double strike.
fn has_first_strike_creatures(state: &GameState) -> bool {
    let db = state.card_db();
    let check = |id: &ObjectId| -> bool {
        let inst = &state.objects[id];
        if let Some(def) = db.get(inst.card_def_id) {
            inst.has_keyword(def, KeywordAbility::FirstStrike)
                || inst.has_keyword(def, KeywordAbility::DoubleStrike)
        } else {
            false
        }
    };

    state.combat.attackers.iter().any(check)
        || state.combat.blockers.keys().any(check)
}

/// A pending damage application collected during combat resolution.
struct DamageEvent {
    target_object: Option<ObjectId>,
    target_player: Option<PlayerIndex>,
    amount: u32,
    lifelink_for: Option<PlayerIndex>,
}

/// Resolve combat damage. Collects all damage events first (read phase), then applies them.
fn resolve_combat_damage(state: &mut GameState, first_strike_only: bool) {
    let defending_player = state.opponent(state.active_player);
    let mut damage_events: Vec<DamageEvent> = Vec::new();

    // Phase 1: Collect all damage events (immutable)
    {
        let db = state.card_db();
        let attackers = state.combat.attackers.clone();

        for &attacker_id in &attackers {
            let attacker_inst = match state.objects.get(&attacker_id) {
                Some(i) => i.clone(),
                None => continue,
            };
            let attacker_def = match db.get(attacker_inst.card_def_id) {
                Some(d) => d.clone(),
                None => continue,
            };

            let has_fs = attacker_inst.has_keyword(&attacker_def, KeywordAbility::FirstStrike);
            let has_ds = attacker_inst.has_keyword(&attacker_def, KeywordAbility::DoubleStrike);

            let deals_damage = if first_strike_only {
                has_fs || has_ds
            } else {
                !has_fs || has_ds
            };

            if !deals_damage {
                continue;
            }

            let power = attacker_inst.effective_power(&attacker_def).max(0) as u32;
            if power == 0 {
                continue;
            }

            let lifelink = if attacker_inst.has_keyword(&attacker_def, KeywordAbility::Lifelink) {
                Some(attacker_inst.controller)
            } else {
                None
            };

            let blockers = state
                .combat
                .attacker_blockers
                .get(&attacker_id)
                .cloned()
                .unwrap_or_default();

            if blockers.is_empty() {
                // Unblocked — damage to defending player
                damage_events.push(DamageEvent {
                    target_object: None,
                    target_player: Some(defending_player),
                    amount: power,
                    lifelink_for: lifelink,
                });
            } else {
                let has_trample =
                    attacker_inst.has_keyword(&attacker_def, KeywordAbility::Trample);
                let has_deathtouch =
                    attacker_inst.has_keyword(&attacker_def, KeywordAbility::Deathtouch);

                let mut remaining = power;

                for &blocker_id in &blockers {
                    if remaining == 0 {
                        break;
                    }
                    let blocker_inst = match state.objects.get(&blocker_id) {
                        Some(i) => i.clone(),
                        None => continue,
                    };
                    let blocker_def = match db.get(blocker_inst.card_def_id) {
                        Some(d) => d,
                        None => continue,
                    };

                    let toughness = blocker_inst.remaining_toughness(blocker_def).max(0) as u32;
                    let damage = if has_deathtouch {
                        1.min(remaining)
                    } else {
                        toughness.min(remaining)
                    };

                    damage_events.push(DamageEvent {
                        target_object: Some(blocker_id),
                        target_player: None,
                        amount: damage,
                        lifelink_for: lifelink,
                    });
                    remaining -= damage;
                }

                // Trample: excess to defending player
                if has_trample && remaining > 0 {
                    damage_events.push(DamageEvent {
                        target_object: None,
                        target_player: Some(defending_player),
                        amount: remaining,
                        lifelink_for: lifelink,
                    });
                }

                // Blockers deal damage to attacker
                for &blocker_id in &blockers {
                    let blocker_inst = match state.objects.get(&blocker_id) {
                        Some(i) => i.clone(),
                        None => continue,
                    };
                    let blocker_def = match db.get(blocker_inst.card_def_id) {
                        Some(d) => d.clone(),
                        None => continue,
                    };

                    let blocker_has_fs =
                        blocker_inst.has_keyword(&blocker_def, KeywordAbility::FirstStrike);
                    let blocker_has_ds =
                        blocker_inst.has_keyword(&blocker_def, KeywordAbility::DoubleStrike);

                    let blocker_deals = if first_strike_only {
                        blocker_has_fs || blocker_has_ds
                    } else {
                        !blocker_has_fs || blocker_has_ds
                    };

                    if blocker_deals {
                        let blocker_power =
                            blocker_inst.effective_power(&blocker_def).max(0) as u32;
                        let blocker_lifelink =
                            if blocker_inst.has_keyword(&blocker_def, KeywordAbility::Lifelink) {
                                Some(blocker_inst.controller)
                            } else {
                                None
                            };

                        damage_events.push(DamageEvent {
                            target_object: Some(attacker_id),
                            target_player: None,
                            amount: blocker_power,
                            lifelink_for: blocker_lifelink,
                        });
                    }
                }
            }
        }
    }

    // Phase 2: Apply all damage events (mutable)
    for event in damage_events {
        if let Some(obj_id) = event.target_object {
            if let Some(inst) = state.objects.get_mut(&obj_id) {
                inst.damage_marked += event.amount;
            }
        }
        if let Some(player) = event.target_player {
            state.players[player].life -= event.amount as i32;
        }
        if let Some(lifelink_player) = event.lifelink_for {
            state.players[lifelink_player].life += event.amount as i32;
        }
    }
}

/// A tap decision: which land to tap and what mana it produces.
enum TapDecision {
    Color(ObjectId, crate::mana::Color),
    Colorless(ObjectId),
}

/// Auto-tap lands to pay a mana cost.
/// Simple greedy: tap colored sources first for colored requirements, then any for generic.
/// Uses a two-phase approach: collect tap decisions (read-only), then apply them (mutate).
pub fn auto_tap_lands(
    state: &mut GameState,
    player: PlayerIndex,
    cost: &crate::mana::ManaCost,
) {
    use crate::mana::Color;

    let mut decisions: Vec<TapDecision> = Vec::new();

    // Phase 1: Collect tap decisions (immutable borrow)
    {
        let db = state.card_db();
        let untapped = state.untapped_lands(player);
        let mut tapped_set = std::collections::HashSet::new();

        // Pay colored costs first
        for &color in &Color::ALL {
            let needed = cost.color_amount(color);
            let already_have = state.players[player].mana_pool.get(color);
            if needed <= already_have {
                continue;
            }
            let mut still_need = needed - already_have;

            for &land_id in &untapped {
                if still_need == 0 {
                    break;
                }
                if tapped_set.contains(&land_id) {
                    continue;
                }
                let inst = &state.objects[&land_id];
                if inst.tapped {
                    continue;
                }
                let def = match db.get(inst.card_def_id) {
                    Some(d) => d,
                    None => continue,
                };

                for ma in &def.mana_abilities {
                    let produces_color = match ma {
                        ManaAbility::TapForColor(c) => *c == color,
                        ManaAbility::TapForChoice(colors) => colors.contains(&color),
                        ManaAbility::TapForAny => true,
                        _ => false,
                    };
                    if produces_color {
                        decisions.push(TapDecision::Color(land_id, color));
                        tapped_set.insert(land_id);
                        still_need -= 1;
                        break;
                    }
                }
            }
        }

        // Pay generic costs from remaining untapped lands
        let colored_from_decisions = decisions.len() as u32;
        let pool_total = state.players[player].mana_pool.total() + colored_from_decisions;
        let colored_total: u32 = Color::ALL.iter().map(|&c| cost.color_amount(c)).sum();
        if pool_total < colored_total + cost.generic {
            let mut still_need = (colored_total + cost.generic).saturating_sub(pool_total);

            for &land_id in &untapped {
                if still_need == 0 {
                    break;
                }
                if tapped_set.contains(&land_id) {
                    continue;
                }
                let inst = &state.objects[&land_id];
                if inst.tapped {
                    continue;
                }
                let def = match db.get(inst.card_def_id) {
                    Some(d) => d,
                    None => continue,
                };

                if let Some(ma) = def.mana_abilities.first() {
                    match ma {
                        ManaAbility::TapForColor(c) => {
                            decisions.push(TapDecision::Color(land_id, *c));
                        }
                        ManaAbility::TapForColorless | ManaAbility::TapForAny => {
                            decisions.push(TapDecision::Colorless(land_id));
                        }
                        ManaAbility::TapForChoice(colors) => {
                            if let Some(&c) = colors.first() {
                                decisions.push(TapDecision::Color(land_id, c));
                            } else {
                                decisions.push(TapDecision::Colorless(land_id));
                            }
                        }
                    }
                    tapped_set.insert(land_id);
                    still_need -= 1;
                }
            }
        }
    }

    // Phase 2: Apply decisions (mutable borrow)
    for decision in decisions {
        match decision {
            TapDecision::Color(land_id, color) => {
                state.players[player].mana_pool.add_color(color, 1);
                if let Some(inst) = state.objects.get_mut(&land_id) {
                    inst.tapped = true;
                }
            }
            TapDecision::Colorless(land_id) => {
                state.players[player].mana_pool.colorless += 1;
                if let Some(inst) = state.objects.get_mut(&land_id) {
                    inst.tapped = true;
                }
            }
        }
    }
}

/// Set up a game from two decklists. Shuffles libraries and draws opening hands.
pub fn setup_game(
    state: &mut GameState,
    deck0: &[crate::card::CardId],
    deck1: &[crate::card::CardId],
) {
    let mut rng = rand::thread_rng();

    // Create card instances for player 0
    let mut lib0: Vec<ObjectId> = deck0
        .iter()
        .map(|&card_id| state.create_card_in_zone(card_id, 0, ZoneType::Library))
        .collect();
    lib0.shuffle(&mut rng);
    state.players[0].library = lib0;

    // Create card instances for player 1
    let mut lib1: Vec<ObjectId> = deck1
        .iter()
        .map(|&card_id| state.create_card_in_zone(card_id, 1, ZoneType::Library))
        .collect();
    lib1.shuffle(&mut rng);
    state.players[1].library = lib1;

    // Draw opening hands (7 cards each)
    draw_cards(state, 0, 7);
    draw_cards(state, 1, 7);

    // Set starting state
    state.active_player = 0;
    state.priority_player = 0;
    state.phase = Phase::Untap;
    state.turn_number = 1;

    // Execute first untap step (which auto-advances to upkeep -> draw)
    execute_phase_entry(state);
}
