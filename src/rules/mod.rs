use rand::seq::SliceRandom;
use rand::Rng;

use std::sync::Arc;

use crate::action::Action;
use crate::card::{CardDef, CardType, Effect, KeywordAbility, ManaAbility, ObjectId, TokenDef, TriggerCondition, ZoneType};
use crate::events::{GameEvent, Zone};
use crate::game::{GameState, PendingTrigger, Phase, PlayerIndex, StackEntry, StackSource, Target};

/// Apply an action to the game state, advancing it.
pub fn apply_action(state: &mut GameState, action: &Action) {
    match action {
        Action::PassPriority => {
            // If there's a pending tutor, passing means "fail to find" —
            // clear it and return without advancing priority normally.
            if state.pending_tutor.is_some() {
                state.pending_tutor = None;
                return;
            }

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
            let is_creature = def.is_creature();

            // Pay mana cost
            if let Some(ref cost) = def.mana_cost {
                // First, auto-tap lands to generate mana if pool is insufficient
                auto_tap_lands(state, player, cost);
                // Then pay from pool — if payment fails, abort the cast
                if !state.players[player].mana_pool.pay(cost) {
                    return;
                }
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

            state.emit_event(GameEvent::SpellCast {
                object: obj_id,
                controller: player,
            });
            state.emit_event(GameEvent::ZoneChange {
                object: obj_id,
                from: Zone::Hand,
                to: Zone::Stack,
            });

            // Fire spell-cast triggers (YouCastSpell, OpponentCastsSpell, etc.)
            fire_spell_cast_triggers(state, player, is_creature);

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
                    ManaAbility::TapForColorlessAmount(n) => {
                        state.players[player].mana_pool.colorless += n;
                    }
                }

                // Check for ManaFromNonlandBonus (e.g., Kinnan, Bonder Prodigy):
                // "Whenever you tap a nonland permanent for mana, add one mana
                // of any type that permanent produced."
                // Simplified: add +1 colorless if the source is a nonland permanent
                // and the controller has a permanent with ManaFromNonlandBonus.
                let source_is_nonland = {
                    let db = state.card_db();
                    let inst = &state.objects[&obj_id];
                    let def = db.get(inst.card_def_id).unwrap();
                    !def.card_types.contains(&CardType::Land)
                };
                if source_is_nonland {
                    let bonus = mana_from_nonland_bonus_count(state, player);
                    if bonus > 0 {
                        state.players[player].mana_pool.colorless += bonus;
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
                if !state.players[player].mana_pool.pay(&ability.cost) {
                    return;
                }

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
            let to_tap: Vec<ObjectId> = attackers
                .iter()
                .filter(|&&id| !state.has_keyword(id, KeywordAbility::Vigilance))
                .copied()
                .collect();
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

        Action::CastCommander { object_id, targets } => {
            let obj_id = *object_id;
            let player = state.priority_player;
            let db = state.card_db();
            let inst = &state.objects[&obj_id];
            let def = db.get(inst.card_def_id).unwrap().clone();
            let is_creature = def.is_creature();

            // Pay mana cost with commander tax
            if let Some(ref cost) = def.mana_cost {
                let tax = state.players[player].commander_tax;
                let mut taxed_cost = cost.clone();
                taxed_cost.generic += tax * 2;
                auto_tap_lands(state, player, &taxed_cost);
                if !state.players[player].mana_pool.pay(&taxed_cost) {
                    return;
                }
            }

            // Increment commander tax for next cast
            state.players[player].commander_tax += 1;

            // Move to stack
            let stack_id = state.new_stack_id();
            state.stack.push(StackEntry {
                id: stack_id,
                source: StackSource::Spell(obj_id),
                controller: player,
                targets: targets.clone(),
            });
            // Remove from command zone
            state.players[player].command_zone.retain(|&id| id != obj_id);

            state.emit_event(GameEvent::SpellCast {
                object: obj_id,
                controller: player,
            });
            state.emit_event(GameEvent::ZoneChange {
                object: obj_id,
                from: Zone::Command,
                to: Zone::Stack,
            });

            // Fire spell-cast triggers (YouCastSpell, OpponentCastsSpell, etc.)
            fire_spell_cast_triggers(state, player, is_creature);

            state.consecutive_passes = 0;
        }

        Action::ChooseReplacementOrder { ordering } => {
            // Store the chosen replacement order for the current pending replacement.
            // The replacement engine will apply effects in the chosen order when
            // the event is processed. For now, clear the pending replacement and
            // record the choice.
            //
            // Phase 2A will wire this into the actual replacement application logic.
            // Currently this action variant exists to establish the interface contract
            // between the rules engine and MCCFR — the solver needs to know that
            // replacement ordering is a player decision.
            let _ = ordering;
            state.consecutive_passes = 0;
        }

        Action::MulliganKeep => {
            let player = state.priority_player;
            state.players[player].mulligan_decided = true;
            advance_mulligan(state);
        }

        Action::MulliganMulligan => {
            let player = state.priority_player;
            debug_assert!(
                state.players[player].mulligan_count < crate::action::MAX_MULLIGANS,
                "MulliganMulligan applied but player {} already at max mulligans ({})",
                player,
                state.players[player].mulligan_count,
            );
            // Shuffle hand back into library
            let hand: Vec<crate::card::ObjectId> = state.players[player].hand.drain(..).collect();
            for obj_id in hand {
                state.players[player].library.push(obj_id);
            }
            {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                state.players[player].library.shuffle(&mut rng);
            }
            // Draw 7 new cards
            draw_cards(state, player, 7);
            state.players[player].mulligan_count += 1;
            // Stay on the same player for another keep/mulligan decision
        }

        Action::MulliganBottomCard { object_id } => {
            let player = state.priority_player;
            // Move the card from hand to bottom of library
            if let Some(pos) = state.players[player].hand.iter().position(|&id| id == *object_id) {
                state.players[player].hand.remove(pos);
                state.players[player].library.push(*object_id);
            }
            advance_mulligan(state);
        }

        Action::ChooseTutorTarget { card_id } => {
            if let Some(pending) = state.pending_tutor.take() {
                let player = pending.controller;
                let destination = pending.destination;
                // Find the first instance of this card in the library
                if let Some(pos) = state.players[player]
                    .library
                    .iter()
                    .position(|&obj_id| state.objects[&obj_id].card_def_id == *card_id)
                {
                    let obj_id = state.players[player].library.remove(pos);
                    state.move_object(obj_id, ZoneType::Library, destination);
                }
            }
        }

        Action::Concede => {
            let player = state.priority_player;
            state.players[player].has_lost = true;
            state.game_over = true;
            state.winner = Some(state.opponent(player));
        }

        Action::ActivateMacro { combo_id } => {
            let player = state.priority_player;
            // Look up the combo from the registry and apply its effect.
            // Clone the combo to avoid borrow conflict with state mutation.
            let combo = state
                .combo_registry
                .as_ref()
                .and_then(|reg| reg.get(*combo_id).cloned());
            if let Some(combo) = combo {
                crate::combo::apply_combo_effect(state, player, &combo);
                state.consecutive_passes = 0;
            }
        }
    }
}

/// Advance the mulligan state machine after a keep or bottom-card decision.
///
/// Flow:
/// 1. If current player kept but still has cards to bottom → stay (bottom actions generated)
/// 2. If current player is done → advance to next player who hasn't decided
/// 3. If all players are done deciding and bottoming → transition to Turn 1
fn advance_mulligan(state: &mut GameState) {
    let player = state.priority_player;
    let ps = &state.players[player];

    // If this player kept but still needs to bottom cards, stay on them
    if ps.mulligan_decided {
        let target_hand_size = 7u32.saturating_sub(ps.mulligan_count) as usize;
        if ps.hand.len() > target_hand_size {
            return; // More bottom-card decisions needed
        }
    }

    // This player is fully done — find the next player who needs action
    let num_players = state.players.len();
    for offset in 1..=num_players {
        let next = (player + offset) % num_players;
        let nps = &state.players[next];
        if !nps.mulligan_decided {
            state.priority_player = next;
            return;
        }
        let target = 7u32.saturating_sub(nps.mulligan_count) as usize;
        if nps.hand.len() > target {
            state.priority_player = next;
            return;
        }
    }

    // All players done — transition to Turn 1
    state.phase = Phase::Untap;
    state.active_player = 0;
    state.priority_player = 0;
    state.turn_number = 1;
    execute_phase_entry(state);
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

        // Apply ETB replacement effects (CR 614): enters tapped, enters with counters, etc.
        state.apply_etb_replacements(obj_id);

        // Refresh continuous effects when a permanent enters the battlefield.
        // This picks up any static abilities on the new permanent.
        state.refresh_continuous_effects();

        // Queue ETB triggered abilities (they go on the stack, not resolve immediately).
        // If flush pauses (controller has >1 ETB trigger), pending_triggers stays
        // populated and the game loop will offer OrderTriggers.
        let _ = fire_triggers(state, TriggerCondition::EntersBattlefield, Some(obj_id));
        // Fire "whenever a creature enters" watcher triggers on other permanents
        if def.is_creature() {
            check_triggers(state, TriggerCondition::ACreatureEnters, None);
            let _ = flush_triggers(state);
        }
    } else {
        if let Some(ref effect) = def.spell_effect {
            resolve_effect(state, effect, controller, targets);
        }
        // Commander redirect: non-permanent commander spells go to command zone
        if state.is_commander(obj_id) {
            state.move_object(obj_id, ZoneType::Stack, ZoneType::Command);
        } else {
            state.move_object(obj_id, ZoneType::Stack, ZoneType::Graveyard);
        }
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
        Effect::DealDamage { amount, target: target_spec } => {
            // For untargeted effects, auto-generate targets from the spec.
            let effective_targets: Vec<Target> = if targets.is_empty() {
                match target_spec {
                    crate::card::TargetSpec::NoTarget => {
                        // "Each player" — deal damage to all players
                        (0..state.players.len())
                            .map(|i| Target::Player(i))
                            .collect()
                    }
                    crate::card::TargetSpec::EachCreature => {
                        // "Each creature" — deal damage to all creatures on the battlefield
                        let db = state.card_db();
                        state.battlefield.iter().copied()
                            .filter(|&id| {
                                state.objects.get(&id)
                                    .and_then(|inst| db.get(inst.card_def_id))
                                    .map_or(false, |def| def.is_creature())
                            })
                            .map(Target::Object)
                            .collect()
                    }
                    _ => vec![],
                }
            } else {
                targets.to_vec()
            };

            for target in &effective_targets {
                // Apply replacement effects to damage (CR 614)
                let actual_damage = state.deal_damage_with_replacement(*amount, target);
                if actual_damage == 0 {
                    continue;
                }

                match target {
                    Target::Player(p) => {
                        let old_life = state.players[*p].life;
                        state.players[*p].life -= actual_damage as i32;
                        state.emit_event(GameEvent::LifeChanged {
                            player: *p,
                            old: old_life,
                            new: state.players[*p].life,
                        });
                        state.emit_event(GameEvent::DamageDealt {
                            source: 0,
                            target: target.clone(),
                            amount: actual_damage,
                            is_combat: false,
                        });
                    }
                    Target::Object(id) => {
                        if let Some(inst) = state.objects.get_mut(id) {
                            inst.damage_marked += actual_damage;
                        }
                        state.emit_event(GameEvent::DamageDealt {
                            source: 0,
                            target: target.clone(),
                            amount: actual_damage,
                            is_combat: false,
                        });
                    }
                }
            }
        }

        Effect::GainLife { amount } => {
            let old_life = state.players[controller].life;
            state.players[controller].life += *amount as i32;
            state.emit_event(GameEvent::LifeChanged {
                player: controller,
                old: old_life,
                new: state.players[controller].life,
            });
        }

        Effect::LoseLife { amount, .. } => {
            for target in targets {
                if let Target::Player(p) = target {
                    let old_life = state.players[*p].life;
                    state.players[*p].life -= *amount as i32;
                    state.emit_event(GameEvent::LifeChanged {
                        player: *p,
                        old: old_life,
                        new: state.players[*p].life,
                    });
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
                        let indestructible =
                            state.has_keyword(*id, KeywordAbility::Indestructible);
                        if !indestructible { Some(*id) } else { None }
                    } else {
                        None
                    }
                })
                .collect();

            for &id in &destroyable {
                state.move_object(id, ZoneType::Battlefield, ZoneType::Graveyard);
            }
            if !destroyable.is_empty() {
                state.refresh_continuous_effects();
            }
            // Fire dies triggers for destroyed creatures.
            // Batch-check all death triggers before flushing so the controller
            // gets a single ordering decision for simultaneous "when ~ dies" triggers.
            // Only check the dying creature itself (self-referential "when ~ dies"),
            // not all battlefield permanents (see comment in check_state_based_actions).
            for &id in &destroyable {
                check_triggers(state, TriggerCondition::Dies, Some(id));
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
            use crate::layers::{AffectedObjects, ContinuousEffect, Duration, LayerModification};
            for target in targets {
                if let Target::Object(id) = target {
                    if *until_eot {
                        // Create a continuous effect that lasts until end of turn
                        let ts = state.new_timestamp();
                        state.continuous_effects.push(ContinuousEffect {
                            source_id: *id,
                            controller,
                            timestamp: ts,
                            duration: Duration::UntilEndOfTurn,
                            affected: AffectedObjects::Specific(*id),
                            modification: LayerModification::ModifyPT(*power, *toughness),
                        });
                    } else {
                        // Permanent buff via +1/+1 counters
                        if let Some(inst) = state.objects.get_mut(id) {
                            let counters = (*power).min(*toughness);
                            inst.plus_counters += counters;
                            // Any asymmetric remainder as a permanent continuous effect
                            if *power != *toughness {
                                let ts = state.new_timestamp();
                                state.continuous_effects.push(ContinuousEffect {
                                    source_id: *id,
                                    controller,
                                    timestamp: ts,
                                    duration: Duration::Permanent,
                                    affected: AffectedObjects::Specific(*id),
                                    modification: LayerModification::ModifyPT(
                                        power - counters,
                                        toughness - counters,
                                    ),
                                });
                            }
                        }
                    }
                }
            }
            state.invalidate_characteristics_cache();
        }

        Effect::DiscardCards { count, .. } => {
            for target in targets {
                if let Target::Player(p) = target {
                    discard_random(state, *p, *count as usize);
                }
            }
        }

        Effect::CreateToken(token_def) => {
            create_token(state, token_def, controller);
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

        Effect::ExileTarget { .. } => {
            for target in targets {
                if let Target::Object(id) = target {
                    if state.battlefield.contains(id) {
                        state.move_object(*id, ZoneType::Battlefield, ZoneType::Exile);
                    }
                }
            }
            state.refresh_continuous_effects();
            state.refresh_replacement_effects();
        }

        Effect::DestroyAll => {
            // Destroy all creatures on the battlefield (e.g., Wrath of God)
            let creatures: Vec<ObjectId> = state
                .battlefield
                .iter()
                .copied()
                .filter(|&id| state.is_creature(id))
                .filter(|&id| !state.has_keyword(id, KeywordAbility::Indestructible))
                .collect();

            for &id in &creatures {
                let dest_zone = state.death_replacement_zone(id);
                if dest_zone != ZoneType::Battlefield {
                    state.move_object(id, ZoneType::Battlefield, dest_zone);
                }
            }
            if !creatures.is_empty() {
                state.refresh_continuous_effects();
                state.refresh_replacement_effects();
            }
            // Fire dies triggers
            for &id in &creatures {
                check_triggers(state, TriggerCondition::Dies, Some(id));
            }
            if !creatures.is_empty() {
                let _ = flush_triggers(state);
            }
        }

        Effect::Debuff {
            power,
            toughness,
            until_eot,
        } => {
            use crate::layers::{AffectedObjects, ContinuousEffect, Duration, LayerModification};
            for target in targets {
                if let Target::Object(id) = target {
                    let duration = if *until_eot {
                        Duration::UntilEndOfTurn
                    } else {
                        Duration::Permanent
                    };
                    let ts = state.new_timestamp();
                    state.continuous_effects.push(ContinuousEffect {
                        source_id: *id,
                        controller,
                        timestamp: ts,
                        duration,
                        affected: AffectedObjects::Specific(*id),
                        modification: LayerModification::ModifyPT(-power, -toughness),
                    });
                }
            }
            state.invalidate_characteristics_cache();
        }

        Effect::PutCounters { count, .. } => {
            for target in targets {
                if let Target::Object(id) = target {
                    if let Some(inst) = state.objects.get_mut(id) {
                        if *count > 0 {
                            inst.plus_counters += count;
                        } else {
                            inst.minus_counters += count.abs();
                        }
                    }
                }
            }
            state.invalidate_characteristics_cache();
        }

        Effect::MillCards { count, .. } => {
            for target in targets {
                if let Target::Player(p) = target {
                    for _ in 0..*count {
                        if let Some(card_id) = state.players[*p].library.pop() {
                            state.move_object(card_id, ZoneType::Library, ZoneType::Graveyard);
                        }
                    }
                }
            }
        }

        Effect::SacrificeCreatures { count, .. } => {
            for target in targets {
                if let Target::Player(p) = target {
                    let mut creatures = state.creatures_controlled_by(*p);
                    // Sort by effective power ascending so the weakest are
                    // sacrificed first — a reasonable heuristic standing in
                    // for actual player choice until we surface a UI action.
                    creatures.sort_by_key(|&id| state.effective_power(id));
                    for &id in creatures.iter().take(*count as usize) {
                        state.move_object(id, ZoneType::Battlefield, ZoneType::Graveyard);
                    }
                }
            }
            state.refresh_continuous_effects();
        }

        Effect::PreventCombatDamage => {
            // Not yet implemented — no cards in the current pool use this effect.
            // When added, this should set a flag on GameState that is checked
            // during resolve_combat_damage() to skip damage assignment.
        }

        Effect::AddMana { color, amount } => {
            for _ in 0..*amount {
                match color {
                    Some(c) => state.players[controller].mana_pool.add_color(*c, 1),
                    None => state.players[controller].mana_pool.colorless += 1,
                }
            }
        }

        Effect::ExtraTurn => {
            state.extra_turns.push_back(controller);
        }

        Effect::SkipPhase(phase) => {
            state.skip_phases.insert(*phase);
        }

        Effect::Multiple(effects) => {
            for e in effects {
                resolve_effect(state, e, controller, targets);
            }
        }

        Effect::SearchLibrary { destination } => {
            if state.players[controller].tutor_targets.is_empty() {
                // Legacy behavior: no tutor targets configured, take top card.
                // Maintains backward compatibility with existing tests/configs.
                if !state.players[controller].library.is_empty() {
                    let card_obj = state.players[controller].library.remove(0);
                    state.move_object(card_obj, ZoneType::Library, *destination);
                }
            } else {
                // Set pending tutor — the player will choose via ChooseTutorTarget.
                // legal_actions generates choices restricted to tutor_targets ∩ library.
                // MCCFR learns which target is optimal in each game state.
                state.pending_tutor = Some(crate::game::PendingTutor {
                    controller,
                    destination: *destination,
                });
            }
        }

        Effect::BounceAllNonlandOpponents => {
            // Bounce all nonland permanents opponents control to their owners' hands.
            let db = state.card_db();
            let to_bounce: Vec<ObjectId> = state
                .battlefield
                .iter()
                .copied()
                .filter(|&id| {
                    if let Some(inst) = state.objects.get(&id) {
                        if inst.controller == controller {
                            return false; // skip own permanents
                        }
                        if let Some(def) = db.get(inst.card_def_id) {
                            return !def.is_land();
                        }
                    }
                    false
                })
                .collect();
            for id in to_bounce {
                state.move_object(id, ZoneType::Battlefield, ZoneType::Hand);
            }
        }

        Effect::ReturnToTopOfLibrary { .. } => {
            // Put target card from graveyard on top of owner's library.
            for target in targets {
                match target {
                    Target::Object(obj_id) => {
                        if let Some(inst) = state.objects.get(obj_id) {
                            let owner = inst.owner;
                            state.move_object(*obj_id, ZoneType::Graveyard, ZoneType::Library);
                            // Move to front (top) of library
                            if let Some(pos) = state.players[owner].library.iter().position(|&id| id == *obj_id) {
                                let id = state.players[owner].library.remove(pos);
                                state.players[owner].library.insert(0, id);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Effect::UntapTarget { .. } => {
            for target in targets {
                match target {
                    Target::Object(obj_id) => {
                        if let Some(inst) = state.objects.get_mut(obj_id) {
                            inst.tapped = false;
                        }
                    }
                    _ => {}
                }
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
    state.emit_event(GameEvent::AbilityTriggered {
        source: trigger.source_id,
        ability_index: trigger.ability_index,
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

/// Count how many permanents a player controls that have the
/// `ManaFromNonlandBonus` static ability (e.g., Kinnan, Bonder Prodigy).
/// Returns the total bonus amount (typically 1 per source).
fn mana_from_nonland_bonus_count(state: &GameState, player: PlayerIndex) -> u32 {
    let db = state.card_db();
    let mut count = 0u32;
    for &obj_id in &state.battlefield {
        let inst = match state.objects.get(&obj_id) {
            Some(i) => i,
            None => continue,
        };
        if inst.controller != player {
            continue;
        }
        let def = match db.get(inst.card_def_id) {
            Some(d) => d,
            None => continue,
        };
        for sa in &def.static_abilities {
            if matches!(sa, crate::layers::StaticAbility::ManaFromNonlandBonus) {
                count += 1;
            }
        }
    }
    count
}

/// Fire spell-cast triggers for a spell that was just cast.
/// `caster` is the player who cast the spell. `is_creature` indicates whether
/// the spell is a creature spell (relevant for OpponentCastsNoncreatureSpell).
fn fire_spell_cast_triggers(state: &mut GameState, caster: PlayerIndex, is_creature: bool) {
    let triggers: Vec<PendingTrigger> = {
        let db = state.card_db();
        let mut found = Vec::new();

        for &obj_id in &state.battlefield {
            let inst = &state.objects[&obj_id];
            let controller = inst.controller;
            let def = match db.get(inst.card_def_id) {
                Some(d) => d,
                None => continue,
            };

            for (i, trigger) in def.triggered_abilities.iter().enumerate() {
                let matches = match trigger.trigger {
                    TriggerCondition::YouCastSpell => controller == caster,
                    TriggerCondition::OpponentCastsSpell => controller != caster,
                    TriggerCondition::OpponentCastsNoncreatureSpell => {
                        controller != caster && !is_creature
                    }
                    _ => false,
                };
                if matches {
                    found.push(PendingTrigger {
                        source_id: obj_id,
                        ability_index: i,
                        controller,
                        targets: vec![],
                    });
                }
            }
        }
        found
    };

    state.pending_triggers.extend(triggers);
    let _ = flush_triggers(state);
}

/// Fire card-draw triggers when a player draws a card.
/// `drawing_player` is the player who drew. This fires `OpponentDrawsCard`
/// on permanents controlled by each opponent of the drawing player.
fn fire_card_draw_triggers(state: &mut GameState, drawing_player: PlayerIndex) {
    let triggers: Vec<PendingTrigger> = {
        let db = state.card_db();
        let mut found = Vec::new();

        for &obj_id in &state.battlefield {
            let inst = &state.objects[&obj_id];
            let controller = inst.controller;
            let def = match db.get(inst.card_def_id) {
                Some(d) => d,
                None => continue,
            };

            for (i, trigger) in def.triggered_abilities.iter().enumerate() {
                if trigger.trigger == TriggerCondition::OpponentDrawsCard
                    && controller != drawing_player
                {
                    found.push(PendingTrigger {
                        source_id: obj_id,
                        ability_index: i,
                        controller,
                        targets: vec![],
                    });
                }
            }
        }
        found
    };

    state.pending_triggers.extend(triggers);
    let _ = flush_triggers(state);
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
///
/// This correctly handles cascading scenarios where SBA-caused deaths
/// trigger abilities, whose resolution (after players pass priority)
/// may cause further SBAs. The outer loop ensures that the SBA check
/// runs again after triggers are placed on the stack.
///
/// If `flush_triggers` pauses (because a player has >1 simultaneous
/// trigger and must choose ordering), the function returns. The game
/// loop will present `Action::OrderTriggers`, and after the player
/// orders, the next call to `check_state_based_actions` will resume
/// the outer loop.
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
        // Batch-check all death triggers before a single flush so the controller
        // gets a combined ordering decision for simultaneous death triggers.
        let triggers_before = state.pending_triggers.len();
        for &obj_id in &died_this_round {
            // Check the dying creature's own "when ~ dies" triggers.
            check_triggers(state, TriggerCondition::Dies, Some(obj_id));
        }
        // Check "whenever a creature dies" watcher triggers on surviving permanents.
        if !died_this_round.is_empty() {
            check_triggers(state, TriggerCondition::ACreatureDies, None);
        }
        let triggers_queued = state.pending_triggers.len() > triggers_before;

        // --- CR 704.3 exit condition ---
        // If no SBAs were performed AND no triggers were queued, the loop
        // is stable. Exit and grant priority.
        if !any_sba && !triggers_queued {
            break;
        }

        // --- Flush triggers to stack ---
        // If flush pauses (player has >1 trigger needing ordering), return
        // immediately. The game loop will present OrderTriggers, and after
        // the player orders, check_state_based_actions will be called again
        // to continue the outer loop.
        if !state.pending_triggers.is_empty() {
            let flushed = flush_triggers(state);
            if !flushed {
                // Paused for OrderTriggers — return to game loop
                return;
            }
        }

        // Continue outer loop: re-check SBAs after triggers were placed
        // on the stack (in case the act of putting triggers on the stack
        // caused new state changes).
    }
}

/// Advance to the next phase.
fn advance_phase(state: &mut GameState) {
    let current_idx = Phase::TURN_ORDER
        .iter()
        .position(|&p| p == state.phase)
        .unwrap_or(0);

    // Find the next non-skipped phase
    let mut next_idx = current_idx + 1;
    while next_idx < Phase::TURN_ORDER.len() {
        let candidate = Phase::TURN_ORDER[next_idx];
        if state.skip_phases.contains(&candidate) {
            next_idx += 1;
            continue;
        }
        break;
    }

    if next_idx < Phase::TURN_ORDER.len() {
        state.phase = Phase::TURN_ORDER[next_idx];
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
        Phase::Mulligan => {
            // Mulligan phase is handled by apply_action (MulliganKeep/Mulligan/BottomCard).
            // Priority is set by advance_mulligan.
            state.priority_player = active;
        }

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
    // Remove end-of-turn continuous effects (layer engine)
    state.cleanup_eot_effects();
    // Remove legacy EoT effects on instances
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
    // Clear skip_phases from the ending turn
    state.skip_phases.clear();

    // Check for extra turns (Phase 3A)
    if let Some(extra_turn_player) = state.extra_turns.pop_front() {
        state.active_player = extra_turn_player;
    } else {
        state.active_player = state.opponent(state.active_player);
    }

    state.priority_player = state.active_player;
    state.turn_number += 1;
    state.phase = Phase::TURN_ORDER[0]; // Untap
    state.consecutive_passes = 0;

    state.emit_event(GameEvent::TurnStarted {
        active_player: state.active_player,
        turn_number: state.turn_number,
    });

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
        state.emit_event(GameEvent::CardDrawn {
            player,
            object: card_id,
        });
        state.emit_event(GameEvent::ZoneChange {
            object: card_id,
            from: Zone::Library,
            to: Zone::Hand,
        });

        // Fire OpponentDrawsCard triggers (e.g. Consecrated Sphinx)
        fire_card_draw_triggers(state, player);
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

/// Compute a stable CardId for a token type based on its properties.
/// Uses the high bit to avoid collisions with regular card IDs.
/// Uses FNV-1a hash for stability across Rust versions (DefaultHasher is not
/// guaranteed to be stable).
fn token_card_id(token_def: &TokenDef) -> u64 {
    // FNV-1a 64-bit constants
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in token_def.name.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    for byte in token_def.power.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    for byte in token_def.toughness.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    for color in &token_def.colors {
        let disc = *color as u8;
        hash ^= disc as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    for kw in &token_def.keywords {
        let disc = *kw as u8;
        hash ^= disc as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash | (1u64 << 63)
}

/// Convert a TokenDef into a CardDef suitable for the card database.
fn token_to_card_def(token_def: &TokenDef, card_id: u64) -> CardDef {
    CardDef {
        id: card_id,
        name: token_def.name.clone(),
        card_types: vec![CardType::Creature],
        subtypes: token_def.subtypes.clone(),
        keywords: token_def.keywords.clone(),
        power: Some(token_def.power as i32),
        toughness: Some(token_def.toughness as i32),
        oracle_text: format!("{}/{} {} Token", token_def.power, token_def.toughness, token_def.name),
        ..Default::default()
    }
}

/// Create a token on the battlefield (CR 111.1).
///
/// Registers the token's CardDef in the card database (via Arc::make_mut,
/// which clones only if needed) and creates a CardInstance marked as a token.
/// Fires ETB triggers for the token.
fn create_token(state: &mut GameState, token_def: &TokenDef, controller: PlayerIndex) {
    let card_id = token_card_id(token_def);

    // Register token CardDef in the database if not already present
    let needs_registration = state.card_db().get(card_id).is_none();
    if needs_registration {
        let def = token_to_card_def(token_def, card_id);
        if let Some(ref mut arc) = state.card_db {
            let db = Arc::make_mut(arc);
            if db.get(card_id).is_none() {
                db.insert(def);
            }
        }
    }

    // Create the token instance on the battlefield
    let obj_id = state.create_card_in_zone(card_id, controller, ZoneType::Battlefield);
    if let Some(inst) = state.objects.get_mut(&obj_id) {
        inst.controller = controller;
        inst.is_token = true;
        inst.summoning_sick = true;
    }

    // Refresh continuous effects for any static abilities on the token
    state.refresh_continuous_effects();

    // Fire ETB triggers (self-ETB for the token, and watcher ETBs for other permanents)
    let _ = fire_triggers(state, TriggerCondition::EntersBattlefield, Some(obj_id));
    // Fire "whenever a creature enters" watcher triggers on other permanents
    check_triggers(state, TriggerCondition::ACreatureEnters, None);
    let _ = flush_triggers(state);
}

/// Check if any creature in combat has first strike or double strike.
fn has_first_strike_creatures(state: &GameState) -> bool {
    let check = |id: &ObjectId| -> bool {
        state.has_keyword(*id, KeywordAbility::FirstStrike)
            || state.has_keyword(*id, KeywordAbility::DoubleStrike)
    };

    state.combat.attackers.iter().any(check)
        || state.combat.blockers.keys().any(check)
}

/// A pending damage application collected during combat resolution.
struct DamageEvent {
    /// The attacker/source dealing the damage.
    source_id: ObjectId,
    target_object: Option<ObjectId>,
    target_player: Option<PlayerIndex>,
    amount: u32,
    lifelink_for: Option<PlayerIndex>,
}

/// Resolve combat damage. Collects all damage events first (read phase), then applies them.
fn resolve_combat_damage(state: &mut GameState, first_strike_only: bool) {
    let defending_player = state.opponent(state.active_player);
    let mut damage_events: Vec<DamageEvent> = Vec::new();

    // Phase 1: Collect all damage events (immutable — uses layer engine)
    {
        let attackers = state.combat.attackers.clone();

        for &attacker_id in &attackers {
            if state.objects.get(&attacker_id).is_none() {
                continue;
            }

            let has_fs = state.has_keyword(attacker_id, KeywordAbility::FirstStrike);
            let has_ds = state.has_keyword(attacker_id, KeywordAbility::DoubleStrike);

            let deals_damage = if first_strike_only {
                has_fs || has_ds
            } else {
                !has_fs || has_ds
            };

            if !deals_damage {
                continue;
            }

            let power = state.effective_power(attacker_id).max(0) as u32;
            if power == 0 {
                continue;
            }

            let lifelink = if state.has_keyword(attacker_id, KeywordAbility::Lifelink) {
                Some(state.objects[&attacker_id].controller)
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
                    source_id: attacker_id,
                    target_object: None,
                    target_player: Some(defending_player),
                    amount: power,
                    lifelink_for: lifelink,
                });
            } else {
                let has_trample = state.has_keyword(attacker_id, KeywordAbility::Trample);
                let has_deathtouch = state.has_keyword(attacker_id, KeywordAbility::Deathtouch);

                let mut remaining = power;

                for &blocker_id in &blockers {
                    if remaining == 0 {
                        break;
                    }
                    let blocker_inst = match state.objects.get(&blocker_id) {
                        Some(i) => i,
                        None => continue,
                    };

                    let eff_toughness = state.effective_toughness(blocker_id);
                    let remaining_tough =
                        (eff_toughness - blocker_inst.damage_marked as i32).max(0) as u32;
                    let damage = if has_deathtouch {
                        1.min(remaining)
                    } else {
                        remaining_tough.min(remaining)
                    };

                    damage_events.push(DamageEvent {
                        source_id: attacker_id,
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
                        source_id: attacker_id,
                        target_object: None,
                        target_player: Some(defending_player),
                        amount: remaining,
                        lifelink_for: lifelink,
                    });
                }

                // Blockers deal damage to attacker
                for &blocker_id in &blockers {
                    if state.objects.get(&blocker_id).is_none() {
                        continue;
                    }

                    let blocker_has_fs =
                        state.has_keyword(blocker_id, KeywordAbility::FirstStrike);
                    let blocker_has_ds =
                        state.has_keyword(blocker_id, KeywordAbility::DoubleStrike);

                    let blocker_deals = if first_strike_only {
                        blocker_has_fs || blocker_has_ds
                    } else {
                        !blocker_has_fs || blocker_has_ds
                    };

                    if blocker_deals {
                        let blocker_power =
                            state.effective_power(blocker_id).max(0) as u32;
                        let blocker_lifelink =
                            if state.has_keyword(blocker_id, KeywordAbility::Lifelink) {
                                Some(state.objects[&blocker_id].controller)
                            } else {
                                None
                            };

                        damage_events.push(DamageEvent {
                            source_id: blocker_id,
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
            state.emit_event(GameEvent::DamageDealt {
                source: event.source_id,
                target: Target::Object(obj_id),
                amount: event.amount,
                is_combat: true,
            });
        }
        if let Some(player) = event.target_player {
            let old_life = state.players[player].life;
            state.players[player].life -= event.amount as i32;

            // Commander damage tracking (CR 903.10a)
            if state.is_commander(event.source_id) {
                let source_owner = state.objects.get(&event.source_id)
                    .map(|i| i.owner)
                    .unwrap_or(0);
                if player < state.players.len()
                    && source_owner < state.players[player].commander_damage_received.len()
                {
                    state.players[player].commander_damage_received[source_owner] +=
                        event.amount as i32;
                }
            }

            state.emit_event(GameEvent::DamageDealt {
                source: event.source_id,
                target: Target::Player(player),
                amount: event.amount,
                is_combat: true,
            });
            state.emit_event(GameEvent::LifeChanged {
                player,
                old: old_life,
                new: state.players[player].life,
            });
        }
        if let Some(lifelink_player) = event.lifelink_for {
            let old_life = state.players[lifelink_player].life;
            state.players[lifelink_player].life += event.amount as i32;
            state.emit_event(GameEvent::LifeChanged {
                player: lifelink_player,
                old: old_life,
                new: state.players[lifelink_player].life,
            });
        }
    }
}

/// A tap decision: which land to tap and what mana it produces.
enum TapDecision {
    Color(ObjectId, crate::mana::Color),
    Colorless(ObjectId, u32),
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
                    let produced = match ma {
                        ManaAbility::TapForColor(c) => {
                            decisions.push(TapDecision::Color(land_id, *c));
                            1
                        }
                        ManaAbility::TapForColorless
                        | ManaAbility::TapForAny => {
                            decisions.push(TapDecision::Colorless(land_id, 1));
                            1
                        }
                        ManaAbility::TapForColorlessAmount(n) => {
                            decisions.push(TapDecision::Colorless(land_id, *n));
                            *n
                        }
                        ManaAbility::TapForChoice(colors) => {
                            if let Some(&c) = colors.first() {
                                decisions.push(TapDecision::Color(land_id, c));
                            } else {
                                decisions.push(TapDecision::Colorless(land_id, 1));
                            }
                            1
                        }
                    };
                    tapped_set.insert(land_id);
                    still_need = still_need.saturating_sub(produced);
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
            TapDecision::Colorless(land_id, amount) => {
                state.players[player].mana_pool.colorless += amount;
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

/// Set up a Commander game from two decklists. The commander card ID is
/// specified separately — it starts in the command zone rather than the
/// library.
///
/// `deck0`/`deck1` should contain all 100 cards including the commander.
/// The commander is extracted from the list and placed in the command zone.
pub fn setup_commander_game(
    state: &mut GameState,
    deck0: &[crate::card::CardId],
    deck1: &[crate::card::CardId],
    commander0: crate::card::CardId,
    commander1: crate::card::CardId,
) {
    let mut rng = rand::thread_rng();

    // Record commander designations
    state.players[0].commander_card_id = Some(commander0);
    state.players[1].commander_card_id = Some(commander1);

    // Player 0: create all cards, put commander in command zone, rest in library
    let mut lib0 = Vec::new();
    let mut found_commander0 = false;
    for &card_id in deck0 {
        if card_id == commander0 && !found_commander0 {
            let obj_id = state.create_card_in_zone(card_id, 0, ZoneType::Command);
            state.players[0].commander_object_id = Some(obj_id);
            found_commander0 = true;
        } else {
            lib0.push(state.create_card_in_zone(card_id, 0, ZoneType::Library));
        }
    }
    lib0.shuffle(&mut rng);
    state.players[0].library = lib0;

    // Player 1: same
    let mut lib1 = Vec::new();
    let mut found_commander1 = false;
    for &card_id in deck1 {
        if card_id == commander1 && !found_commander1 {
            let obj_id = state.create_card_in_zone(card_id, 1, ZoneType::Command);
            state.players[1].commander_object_id = Some(obj_id);
            found_commander1 = true;
        } else {
            lib1.push(state.create_card_in_zone(card_id, 1, ZoneType::Library));
        }
    }
    lib1.shuffle(&mut rng);
    state.players[1].library = lib1;

    // Draw opening hands (7 cards each)
    draw_cards(state, 0, 7);
    draw_cards(state, 1, 7);

    // Set starting state — begin in Mulligan phase so players can decide
    // whether to keep or mulligan before the game starts.
    state.active_player = 0;
    state.priority_player = 0;
    state.phase = Phase::Mulligan;
    state.turn_number = 1;
}

/// Configure tutor targets for a player.
///
/// When tutor targets are set, `SearchLibrary` effects present the player with
/// a choice (`ChooseTutorTarget`) restricted to this set instead of automatically
/// taking the top card. This allows MCCFR to learn which card to tutor for in
/// each game state.
///
/// Call this after `setup_game` or `setup_commander_game` but before the game
/// loop begins.
pub fn set_tutor_targets(
    state: &mut GameState,
    player: crate::game::PlayerIndex,
    targets: &[crate::card::CardId],
) {
    state.players[player].tutor_targets = targets.to_vec();
}

/// Reshuffle opening hands for goldfish training iterations.
///
/// Returns all hand cards to each player's library, shuffles both libraries,
/// and redraws 7-card opening hands. Resets turn/phase/mana state so the
/// game starts cleanly from Turn 1.
///
/// This is used by MCCFR goldfish training to vary the starting hand across
/// iterations. Without reshuffling, every iteration trains on the exact same
/// hand — making the learned strategy useless for hands with different cards.
pub fn reshuffle_opening_hand(state: &mut GameState) {
    use rand::seq::SliceRandom;

    let mut rng = rand::thread_rng();

    for player in 0..state.players.len() {
        // Move hand cards back to library
        let hand: Vec<crate::card::ObjectId> = state.players[player].hand.drain(..).collect();
        for obj_id in hand {
            state.players[player].library.push(obj_id);
        }

        // Shuffle library
        state.players[player].library.shuffle(&mut rng);

        // Reset per-player state
        state.players[player].land_plays_remaining = 1;
        state.players[player].mana_pool.drain();
        state.players[player].has_drawn_for_turn = false;
        state.players[player].mulligan_count = 0;
        state.players[player].mulligan_decided = false;
    }

    // Redraw opening hands (7 cards each)
    draw_cards(state, 0, 7);
    draw_cards(state, 1, 7);

    // Reset game state
    state.active_player = 0;
    state.priority_player = 0;
    state.turn_number = 1;

    // Commander games start in Mulligan phase; non-commander skip straight to Turn 1
    if state.is_commander_format() {
        state.phase = Phase::Mulligan;
        // Resolve mulligans immediately using a greedy heuristic so the MCCFR
        // traversal starts from a post-mulligan state. Full-width exploration
        // of the mulligan tree (especially N sequential bottom-card decisions
        // from 7 cards) causes combinatorial explosion in traverse_goldfish.
        // Training still sees mulliganed hands because the heuristic will
        // mulligan bad hands, producing starting positions with 6 or fewer cards.
        resolve_mulligans_with_heuristic(state);
    } else {
        state.phase = Phase::Untap;
        execute_phase_entry(state);
    }
}

/// Resolve the Mulligan phase using a simple land-count heuristic.
///
/// Uses the same logic as `GreedyStrategy`: keep hands with 2-5 lands,
/// mulligan otherwise (up to 2 mulligans), bottom highest-CMC non-lands.
///
/// This is separate from `GreedyStrategy` to avoid a dependency from
/// `rules` on `strategy` and to keep the logic self-contained.
fn resolve_mulligans_with_heuristic(state: &mut GameState) {
    while state.phase == Phase::Mulligan {
        let player = state.priority_player;
        let ps = &state.players[player];

        if !ps.mulligan_decided {
            // Keep/mulligan decision: count lands in hand
            let land_count = ps.hand.iter().filter(|&&obj_id| {
                state.objects.get(&obj_id).map_or(false, |inst| {
                    state.card_db().get(inst.card_def_id).map_or(false, |d| d.is_land())
                })
            }).count();

            let action = if (2..=5).contains(&land_count) || ps.mulligan_count >= 2 {
                Action::MulliganKeep
            } else {
                Action::MulliganMulligan
            };
            apply_action(state, &action);
            continue;
        }

        // Bottom-card decision: bottom the highest-CMC non-land
        let target_hand_size = 7u32.saturating_sub(ps.mulligan_count) as usize;
        if ps.hand.len() > target_hand_size {
            let mut worst_obj = ps.hand[0];
            let mut worst_score = -1i32;
            for &obj_id in &ps.hand {
                if let Some(inst) = state.objects.get(&obj_id) {
                    let def = state.card_db().get(inst.card_def_id);
                    let score = if def.map_or(false, |d| d.is_land()) {
                        0 // keep lands
                    } else {
                        def.map_or(5, |d| d.cmc() as i32)
                    };
                    if score > worst_score {
                        worst_score = score;
                        worst_obj = obj_id;
                    }
                }
            }
            apply_action(state, &Action::MulliganBottomCard { object_id: worst_obj });
        } else {
            // Should not happen — advance_mulligan transitions out
            break;
        }
    }
}

/// Validate a Commander deck:
/// - Exactly 100 cards (including commander)
/// - Singleton (max 1 copy of each non-basic-land card)
/// - Commander must be a legendary creature
///
/// Returns Ok(()) or an error message.
pub fn validate_commander_deck(
    db: &crate::game::CardDatabase,
    deck: &[crate::card::CardId],
    commander: crate::card::CardId,
) -> Result<(), String> {
    if deck.len() != 100 {
        return Err(format!(
            "Commander deck must be exactly 100 cards, got {}",
            deck.len()
        ));
    }

    // Commander must be in the deck
    if !deck.contains(&commander) {
        return Err("Commander card must be included in the deck".to_string());
    }

    // Commander must be a legendary creature
    if let Some(def) = db.get(commander) {
        let is_legendary = def.supertypes.contains(&crate::card::Supertype::Legendary);
        let is_creature = def.is_creature();
        if !is_legendary || !is_creature {
            return Err(format!(
                "Commander '{}' must be a legendary creature",
                def.name
            ));
        }
    } else {
        return Err(format!("Commander card ID {} not found in database", commander));
    }

    // Color identity check (CR 903.4): every card must have a color identity
    // that is a subset of the commander's color identity.
    let commander_identity: std::collections::HashSet<crate::mana::Color> = db
        .get(commander)
        .map(|d| d.color_identity().into_iter().collect())
        .unwrap_or_default();
    for &card_id in deck {
        if let Some(def) = db.get(card_id) {
            let card_identity = def.color_identity();
            for color in &card_identity {
                if !commander_identity.contains(color) {
                    return Err(format!(
                        "'{}' has color {} which is outside the commander's color identity",
                        def.name, color
                    ));
                }
            }
        }
    }

    // Singleton check: no more than 1 copy of non-basic-land cards
    let mut counts: std::collections::HashMap<crate::card::CardId, u32> =
        std::collections::HashMap::new();
    for &card_id in deck {
        *counts.entry(card_id).or_insert(0) += 1;
    }
    for (&card_id, &count) in &counts {
        if count > 1 {
            if let Some(def) = db.get(card_id) {
                if !def.is_basic_land() {
                    return Err(format!(
                        "Commander decks allow only 1 copy of non-basic '{}', found {}",
                        def.name, count
                    ));
                }
            }
        }
    }

    Ok(())
}
