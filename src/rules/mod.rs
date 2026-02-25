mod combat;
mod effects;
mod mana;
mod phases;
mod resolution;
mod sba;
mod setup;
mod tokens;
mod triggers;

use rand::Rng;

use crate::action::Action;
use crate::card::{CardType, KeywordAbility, ManaAbility, ObjectId, TriggerCondition, ZoneType};
use crate::events::{GameEvent, Zone};
use crate::game::{GameState, Phase, PlayerIndex, StackEntry, StackSource};

// Public API re-exports
pub use mana::{total_cost_reduction, apply_cost_reduction, auto_tap_lands};
pub use sba::check_state_based_actions;
pub use triggers::fire_triggers;
pub use setup::{setup_game, setup_commander_game, setup_commander_game_with_partners, set_tutor_targets, reshuffle_opening_hand, validate_commander_deck, validate_commander_deck_with_partner};
pub(crate) use tokens::create_token_from_combo;

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
            phases::handle_priority_pass(state);
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
                phases::finalize_cleanup(state);
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

            // Pay mana cost (with cost reduction from permanents like Jet Medallion)
            if let Some(ref cost) = def.mana_cost {
                let reduction = mana::total_cost_reduction(state, player, is_creature);
                let reduced_cost = mana::apply_cost_reduction(cost, reduction);
                // First, auto-tap lands to generate mana if pool is insufficient
                mana::auto_tap_lands(state, player, &reduced_cost);
                // Then pay from pool — if payment fails, abort the cast
                if !state.players[player].mana_pool.pay(&reduced_cost) {
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
            triggers::fire_spell_cast_triggers(state, player, is_creature);

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

                // Check for ManaFromNonlandBonus (e.g., Kinnan, Bonder Prodigy)
                let source_is_nonland = {
                    let db = state.card_db();
                    let inst = &state.objects[&obj_id];
                    let def = db.get(inst.card_def_id).unwrap();
                    !def.card_types.contains(&CardType::Land)
                };
                if source_is_nonland {
                    let bonus = triggers::mana_from_nonland_bonus_count(state, player);
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
                mana::auto_tap_lands(state, player, &ability.cost);
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
                // Batch-check attack triggers for all attackers before flushing
                for &attacker_id in attackers {
                    triggers::check_triggers(state, TriggerCondition::Attacks, Some(attacker_id));
                }
                let flushed = triggers::flush_triggers(state);

                if flushed {
                    state.phase = Phase::DeclareBlockers;
                    state.priority_player = state.next_player(state.active_player);
                }
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
            phases::execute_phase_entry_public(state);
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
            for &(source_id, ability_index) in ordering {
                if let Some(pos) = state.pending_triggers.iter().position(|t| {
                    t.controller == player
                        && t.source_id == source_id
                        && t.ability_index == ability_index
                }) {
                    let trigger = state.pending_triggers.remove(pos);
                    triggers::push_trigger_to_stack(state, &trigger);
                }
            }

            // Continue flushing remaining triggers (the other player's).
            let _ = triggers::flush_triggers(state);
        }

        Action::CastCommander { object_id, targets } => {
            let obj_id = *object_id;
            let player = state.priority_player;
            let db = state.card_db();
            let inst = &state.objects[&obj_id];
            let def = db.get(inst.card_def_id).unwrap().clone();
            let is_creature = def.is_creature();

            // Determine if this is the partner commander (separate tax tracking)
            let is_partner = state.players[player].partner_commander_object_id == Some(obj_id);

            // Pay mana cost with commander tax (and cost reduction)
            if let Some(ref cost) = def.mana_cost {
                let tax = if is_partner {
                    state.players[player].partner_commander_tax
                } else {
                    state.players[player].commander_tax
                };
                let reduction = mana::total_cost_reduction(state, player, is_creature);
                let mut taxed_cost = cost.clone();
                taxed_cost.generic += tax * 2;
                let final_cost = mana::apply_cost_reduction(&taxed_cost, reduction);
                mana::auto_tap_lands(state, player, &final_cost);
                if !state.players[player].mana_pool.pay(&final_cost) {
                    return;
                }
            }

            // Increment commander tax for next cast (separate tracking per partner)
            if is_partner {
                state.players[player].partner_commander_tax += 1;
            } else {
                state.players[player].commander_tax += 1;
            }

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
            triggers::fire_spell_cast_triggers(state, player, is_creature);

            state.consecutive_passes = 0;
        }

        Action::Equip { equipment_id, target_id } => {
            let eq_id = *equipment_id;
            let tgt_id = *target_id;
            let player = state.priority_player;

            // Pay equip cost
            let equip_cost = {
                let db = state.card_db();
                let inst = &state.objects[&eq_id];
                let def = db.get(inst.card_def_id).unwrap();
                def.equip_cost.clone()
            };
            if let Some(cost) = equip_cost {
                mana::auto_tap_lands(state, player, &cost);
                if !state.players[player].mana_pool.pay(&cost) {
                    return;
                }
            }

            // Detach from previous creature (if any)
            if let Some(old_target) = state.objects.get(&eq_id).and_then(|i| i.attached_to) {
                if let Some(old_inst) = state.objects.get_mut(&old_target) {
                    old_inst.attachments.retain(|&id| id != eq_id);
                }
            }

            // Attach to new creature
            if let Some(eq_inst) = state.objects.get_mut(&eq_id) {
                eq_inst.attached_to = Some(tgt_id);
            }
            if let Some(tgt_inst) = state.objects.get_mut(&tgt_id) {
                if !tgt_inst.attachments.contains(&eq_id) {
                    tgt_inst.attachments.push(eq_id);
                }
            }

            state.refresh_continuous_effects();
            state.consecutive_passes = 0;
        }

        Action::ActivateLoyalty { object_id, ability_index } => {
            let obj_id = *object_id;
            let ab_idx = *ability_index;

            // Read the loyalty ability info
            let (loyalty_cost, _effect, controller) = {
                let db = state.card_db();
                let inst = &state.objects[&obj_id];
                let def = db.get(inst.card_def_id).unwrap();
                let la = &def.loyalty_abilities[ab_idx];
                (la.cost, la.effect.clone(), inst.controller)
            };

            // Adjust loyalty counters
            if loyalty_cost >= 0 {
                // +N: add counters
                if let Some(inst) = state.objects.get_mut(&obj_id) {
                    inst.loyalty_counters += loyalty_cost as u32;
                    inst.loyalty_activated_this_turn = true;
                }
            } else {
                // -N: remove counters
                if let Some(inst) = state.objects.get_mut(&obj_id) {
                    inst.loyalty_counters = inst.loyalty_counters.saturating_sub((-loyalty_cost) as u32);
                    inst.loyalty_activated_this_turn = true;
                }
            }

            // Put the ability on the stack
            let stack_id = state.new_stack_id();
            state.stack.push(crate::game::StackEntry {
                id: stack_id,
                source: crate::game::StackSource::ActivatedAbility {
                    source_id: obj_id,
                    ability_index: ab_idx,
                },
                controller,
                targets: Vec::new(),
            });
            state.consecutive_passes = 0;
        }

        Action::ChooseReplacementOrder { ordering } => {
            let _ = ordering;
            state.consecutive_passes = 0;
        }

        Action::MulliganKeep => {
            let player = state.priority_player;
            state.players[player].mulligan_decided = true;
            setup::advance_mulligan(state);
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
            setup::advance_mulligan(state);
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
            // Check if game ends: only 1 active player left = game over
            let active_count = state.active_player_count();
            if active_count <= 1 {
                state.game_over = true;
                state.winner = (0..state.players.len()).find(|&i| !state.players[i].has_lost);
            }
        }

        Action::ActivateMacro { combo_id } => {
            let player = state.priority_player;
            // Look up the combo from the registry and apply its effect.
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
        triggers::fire_card_draw_triggers(state, player);
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
